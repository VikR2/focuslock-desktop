//! Find installed and running apps. Runs once on a background thread at startup.
//!
//! Sources, most trusted first:
//! 1. Start menu shortcuts (what a person actually launches), resolved to their exe.
//! 2. Store and MSIX apps (Spotify, TradingView, WhatsApp...) via the package manager.
//! 3. Uninstall entries whose DisplayIcon names a real exe. Nothing is guessed from folders.
//! 4. Running processes that own a visible window, outside the Windows folder.

use super::{expand_env, from_wide, procs};
use crate::engine::catalog::{IconSource, Origin, Sighting};
use std::path::{Path, PathBuf};
use windows::core::{Interface, HSTRING, PCWSTR};
use windows::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
use windows::Win32::System::Com::{CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER, STGM_READ};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

/// Scan every source. Call from a thread that ran `init_com_for_thread`.
pub fn scan() -> Vec<Sighting> {
    let mut out = Vec::new();
    start_menu(&mut out);
    store_apps(&mut out);
    uninstall_entries(&mut out);
    running(&mut out);
    for s in &mut out {
        s.exe_path = clean_path(&s.exe_path);
        if let IconSource::Path(p) = &mut s.icon {
            *p = clean_path(p);
        }
    }
    out
}

/// Collapse doubled separators (`C:\App\\app.exe`), keeping a UNC prefix.
fn clean_path(p: &str) -> String {
    let p = p.trim().trim_matches('"').replace('/', "\\");
    let (prefix, rest) = if let Some(r) = p.strip_prefix("\\\\") { ("\\\\", r) } else { ("", p.as_str()) };
    let mut out = String::from(prefix);
    let mut last_sep = false;
    for c in rest.chars() {
        if c == '\\' {
            if !last_sep {
                out.push(c);
            }
            last_sep = true;
        } else {
            out.push(c);
            last_sep = false;
        }
    }
    out
}

fn start_menu(out: &mut Vec<Sighting>) {
    let mut roots = Vec::new();
    if let Ok(pd) = std::env::var("ProgramData") {
        roots.push(PathBuf::from(pd).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    if let Ok(ad) = std::env::var("APPDATA") {
        roots.push(PathBuf::from(ad).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    let mut links = Vec::new();
    for root in roots {
        collect_links(&root, 4, &mut links);
    }
    for lnk in links {
        let Some(name) = lnk.file_stem().map(|s| s.to_string_lossy().into_owned()) else { continue };
        let Some((target, args)) = resolve_link(&lnk) else { continue };
        let Some(exe) = launcher_target(&target, &args) else { continue };
        if in_windows_dir(&exe) {
            continue;
        }
        out.push(Sighting {
            name,
            exe_path: exe,
            icon: IconSource::Path(lnk.to_string_lossy().into_owned()),
            origin: Origin::StartMenu,
        });
    }
}

fn collect_links(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        match entry.file_type() {
            Ok(t) if t.is_dir() && depth > 0 => collect_links(&path, depth - 1, out),
            Ok(t) if t.is_file()
                && path.extension().is_some_and(|e| e.eq_ignore_ascii_case("lnk")) => {
                    out.push(path);
                }
            _ => {}
        }
    }
}

fn resolve_link(lnk: &Path) -> Option<(String, String)> {
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let file: IPersistFile = link.cast().ok()?;
        file.Load(&HSTRING::from(lnk.as_os_str()), STGM_READ).ok()?;
        let mut target = [0u16; 1024];
        link.GetPath(&mut target, std::ptr::null_mut(), 0).ok()?;
        let mut args = [0u16; 2048];
        let _ = link.GetArguments(&mut args);
        let target = expand_env(&from_wide(&target));
        (!target.is_empty()).then(|| (target, from_wide(&args)))
    }
}

/// The exe that really runs. Squirrel installs (Discord, Slack, Teams classic)
/// launch `Update.exe --processStart Discord.exe`; the process to block is Discord.exe.
fn launcher_target(target: &str, args: &str) -> Option<String> {
    if !target.to_ascii_lowercase().ends_with(".exe") {
        return None;
    }
    let file = Path::new(target).file_name()?.to_string_lossy().to_ascii_lowercase();
    if file == "update.exe" {
        let lower = args.to_ascii_lowercase();
        let idx = lower.find("--processstart")?;
        let rest = args[idx + "--processstart".len()..].trim_start();
        let exe = rest.trim_start_matches('"').split(['"', ' ']).next()?.trim();
        if exe.is_empty() {
            return None;
        }
        let dir = Path::new(target).parent()?;
        return Some(dir.join(exe).to_string_lossy().into_owned());
    }
    Some(target.to_string())
}

fn store_apps(out: &mut Vec<Sighting>) {
    use windows::Management::Deployment::PackageManager;
    let Ok(manager) = PackageManager::new() else { return };
    let Ok(packages) = manager.FindPackagesByUserSecurityId(&HSTRING::new()) else { return };
    for pkg in packages {
        let skip = pkg.IsFramework().unwrap_or(true)
            || pkg.IsResourcePackage().unwrap_or(true)
            || pkg.IsBundle().unwrap_or(true);
        if skip {
            continue;
        }
        let Ok(dir) = pkg.InstalledPath() else { continue };
        let dir = PathBuf::from(dir.to_string());
        let Ok(manifest) = std::fs::read_to_string(dir.join("AppxManifest.xml")) else { continue };
        let executables = manifest_executables(&manifest);
        if executables.is_empty() {
            continue;
        }
        let Ok(entries) = pkg.GetAppListEntries() else { continue };
        for entry in entries {
            let Ok(aumid) = entry.AppUserModelId().map(|s| s.to_string()) else { continue };
            let app_id = aumid.rsplit('!').next().unwrap_or_default();
            let Some((_, exe)) = executables.iter().find(|(id, _)| id == app_id) else { continue };
            let name = entry
                .DisplayInfo()
                .and_then(|d| d.DisplayName())
                .map(|s| s.to_string())
                .unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            out.push(Sighting {
                name,
                exe_path: dir.join(exe).to_string_lossy().into_owned(),
                icon: IconSource::AppsFolder(aumid),
                origin: Origin::Store,
            });
        }
    }
}

/// `(Application Id, Executable)` pairs from an AppxManifest.xml.
fn manifest_executables(xml: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<Application ") {
        let tag_end = rest[start..].find('>').map(|e| start + e).unwrap_or(rest.len());
        let tag = &rest[start..tag_end];
        if let (Some(id), Some(exe)) = (xml_attr(tag, "Id"), xml_attr(tag, "Executable")) {
            if exe.to_ascii_lowercase().ends_with(".exe") && !exe.contains('$') {
                out.push((id, exe.replace('/', "\\")));
            }
        }
        rest = &rest[tag_end..];
    }
    out
}

fn xml_attr(tag: &str, name: &str) -> Option<String> {
    let pat = format!(" {name}=\"");
    let start = tag.find(&pat)? + pat.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_string())
}

fn uninstall_entries(out: &mut Vec<Sighting>) {
    const KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
    const KEY_32: &str = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall";
    let roots = [
        (windows_registry::LOCAL_MACHINE, KEY),
        (windows_registry::LOCAL_MACHINE, KEY_32),
        (windows_registry::CURRENT_USER, KEY),
    ];
    for (hive, path) in roots {
        let Ok(root) = hive.open(path) else { continue };
        let Ok(names) = root.keys() else { continue };
        for sub in names {
            let Ok(k) = root.open(&sub) else { continue };
            let Ok(name) = k.get_string("DisplayName") else { continue };
            if k.get_u32("SystemComponent").unwrap_or(0) == 1 {
                continue;
            }
            let Ok(icon) = k.get_string("DisplayIcon") else { continue };
            let exe = display_icon_path(&icon);
            if !is_real_exe(&exe) || in_windows_dir(&exe) {
                continue;
            }
            out.push(Sighting {
                name,
                icon: IconSource::Path(exe.clone()),
                exe_path: exe,
                origin: Origin::Uninstall,
            });
        }
    }
}

/// `"C:\App\app.exe",0` -> `C:\App\app.exe`
fn display_icon_path(raw: &str) -> String {
    let s = raw.trim();
    let s = match s.strip_prefix('"') {
        Some(rest) => rest.split('"').next().unwrap_or(rest),
        None => s.rsplit_once(',').map(|(p, idx)| if idx.trim().parse::<i32>().is_ok() { p } else { s }).unwrap_or(s),
    };
    expand_env(s.trim())
}

fn running(out: &mut Vec<Sighting>) {
    let windowed = procs::windowed_pids();
    let mut seen = std::collections::HashSet::new();
    for p in procs::list() {
        if !windowed.contains(&p.pid) || !seen.insert(p.key.clone()) {
            continue;
        }
        let Some(path) = procs::image_path(p.pid) else { continue };
        if in_windows_dir(&path) {
            continue;
        }
        out.push(Sighting {
            name: file_description(&path).unwrap_or_else(|| stem(&path)),
            icon: IconSource::Path(path.clone()),
            exe_path: path,
            origin: Origin::Running,
        });
    }
}

/// Keys of the apps that have a window open right now. Cheap enough to poll.
pub fn running_keys() -> std::collections::HashSet<String> {
    let windowed = procs::windowed_pids();
    procs::list()
        .into_iter()
        .filter(|p| windowed.contains(&p.pid))
        .filter_map(|p| procs::image_path(p.pid))
        .filter(|path| !in_windows_dir(path))
        .map(|path| crate::engine::app_key(&clean_path(&path)))
        .collect()
}

fn is_real_exe(path: &str) -> bool {
    path.to_ascii_lowercase().ends_with(".exe") && Path::new(path).is_file()
}

fn in_windows_dir(path: &str) -> bool {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into()).to_ascii_lowercase();
    path.to_ascii_lowercase().starts_with(&(windir + "\\"))
}

fn stem(path: &str) -> String {
    Path::new(path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

/// The friendly name an exe carries, e.g. `Discord` for Discord.exe.
fn file_description(path: &str) -> Option<String> {
    unsafe {
        let wide = HSTRING::from(path);
        let size = GetFileVersionInfoSizeW(&wide, None);
        if size == 0 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        GetFileVersionInfoW(&wide, None, size, data.as_mut_ptr() as *mut _).ok()?;

        let mut ptr = std::ptr::null_mut();
        let mut len = 0u32;
        let translation = HSTRING::from(r"\VarFileInfo\Translation");
        let mut langs = vec![(0x0409u16, 0x04b0u16)];
        if VerQueryValueW(data.as_ptr() as *const _, PCWSTR(translation.as_ptr()), &mut ptr, &mut len).as_bool() && len >= 4 {
            let pairs = std::slice::from_raw_parts(ptr as *const u16, (len / 2) as usize);
            langs = pairs.as_chunks::<2>().0.iter().map(|c| (c[0], c[1])).chain(langs).collect();
        }
        for (lang, cp) in langs {
            let key = HSTRING::from(format!(r"\StringFileInfo\{lang:04x}{cp:04x}\FileDescription"));
            if VerQueryValueW(data.as_ptr() as *const _, PCWSTR(key.as_ptr()), &mut ptr, &mut len).as_bool() && len > 1 {
                let text = from_wide(std::slice::from_raw_parts(ptr as *const u16, len as usize));
                let text = text.trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn squirrel_launcher_resolves_to_real_exe() {
        let exe = launcher_target(r"C:\Users\me\AppData\Local\Discord\Update.exe", r#"--processStart "Discord.exe""#).unwrap();
        assert_eq!(crate::engine::exe_key(&exe), "discord.exe");
        let exe = launcher_target(r"C:\Users\me\AppData\Local\slack\Update.exe", "--processStart slack.exe --process-start-args x").unwrap();
        assert_eq!(crate::engine::exe_key(&exe), "slack.exe");
    }

    #[test]
    fn non_exe_shortcut_targets_are_skipped() {
        assert!(launcher_target(r"C:\docs\manual.pdf", "").is_none());
    }

    #[test]
    fn parses_manifest_applications() {
        let xml = r#"<Applications><Application Id="Spotify" Executable="Spotify.exe" EntryPoint="x"><uap:VisualElements/></Application>
            <Application Id="App" Executable="$targetnametoken$.exe"/></Applications>"#;
        assert_eq!(manifest_executables(xml), vec![("Spotify".to_string(), "Spotify.exe".to_string())]);
    }

    #[test]
    fn doubled_separators_collapse() {
        assert_eq!(clean_path(r"C:\Program Files (x86)\Overwolf\\Overwolf.exe"), r"C:\Program Files (x86)\Overwolf\Overwolf.exe");
        assert_eq!(clean_path(r"\\server\share\\a.exe"), r"\\server\share\a.exe");
        assert_eq!(clean_path("C:/Apps/x.exe"), r"C:\Apps\x.exe");
    }

    #[test]
    fn display_icon_forms() {
        assert_eq!(display_icon_path(r#""C:\App\app.exe",0"#), r"C:\App\app.exe");
        assert_eq!(display_icon_path(r"C:\App\app.exe,-101"), r"C:\App\app.exe");
        assert_eq!(display_icon_path(r"C:\App, Inc\app.exe"), r"C:\App, Inc\app.exe");
    }

    #[test]
    fn real_scan_finds_apps_with_names() {
        super::super::init_com_for_thread();
        let found = crate::engine::catalog::merge(scan());
        assert!(found.len() > 10, "only {} apps found", found.len());
        assert!(found
            .iter()
            .all(|a| !a.name.is_empty() && (a.key.ends_with(".exe") || a.key.starts_with("pkg:"))));
    }
}
