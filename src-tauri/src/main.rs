// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Manager, State};

#[cfg(target_os = "windows")]
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

mod db;
use db::DbState;

// Global monitor state
struct MonitorState {
    is_running: Arc<AtomicBool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    name: String,
    path: Option<String>,
    icon_hint: Option<String>,
}

#[cfg(target_os = "windows")]
fn expand_env_vars(path: &str) -> String {
    let mut result = String::new();
    let mut remainder = path;

    while let Some(start) = remainder.find('%') {
        result.push_str(&remainder[..start]);
        remainder = &remainder[start + 1..];

        if let Some(end) = remainder.find('%') {
            let var_name = &remainder[..end];
            remainder = &remainder[end + 1..];

            if var_name.is_empty() {
                result.push('%');
                continue;
            }

            if let Ok(value) = std::env::var(var_name) {
                result.push_str(&value);
            } else {
                result.push('%');
                result.push_str(var_name);
                result.push('%');
            }
        } else {
            result.push('%');
            result.push_str(remainder);
            remainder = "";
            break;
        }
    }

    result.push_str(remainder);
    result
}

#[cfg(target_os = "windows")]
fn normalize_windows_path(path: &str) -> String {
    expand_env_vars(path).trim_matches('"').trim().to_string()
}

#[cfg(target_os = "windows")]
fn parse_display_icon_path(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let path_part = if let Some((before, _)) = trimmed.split_once(',') {
        before
    } else {
        trimmed
    };

    parse_command_path(path_part)
}

#[cfg(target_os = "windows")]
fn parse_command_path(command: &str) -> Option<PathBuf> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }

    let candidate = if trimmed.starts_with('"') {
        trimmed
            .trim_start_matches('"')
            .split('"')
            .next()
            .unwrap_or_default()
            .to_string()
    } else {
        trimmed
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string()
    };

    let normalized = normalize_windows_path(&candidate);
    if normalized.is_empty() {
        return None;
    }

    Some(PathBuf::from(normalized))
}

#[cfg(target_os = "windows")]
fn should_descend(dir_name: &str, name_hints: &[String]) -> bool {
    if name_hints.is_empty() {
        return true;
    }

    if dir_name.starts_with("app-")
        || dir_name.starts_with("app")
        || dir_name.starts_with("current")
        || dir_name.starts_with("bin")
        || dir_name.starts_with("client")
        || dir_name.starts_with("package")
        || dir_name
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '_')
    {
        return true;
    }

    name_hints
        .iter()
        .any(|hint| dir_name.contains(hint) || hint.contains(dir_name))
}

#[cfg(target_os = "windows")]
fn search_directory_for_icon(
    dir: &Path,
    hint_path: Option<&Path>,
    name_hints: &[String],
    depth: usize,
    allow_direct_join: bool,
) -> Option<PathBuf> {
    if depth == 0 || !dir.exists() {
        return None;
    }

    if allow_direct_join {
        if let Some(hint) = hint_path {
            if !hint.is_absolute() {
                let candidate = dir.join(hint);

                if candidate.is_file() {
                    return Some(candidate);
                }

                if candidate.extension().is_none() {
                    for ext in ["exe", "lnk", "ico", "appref-ms"] {
                        let with_ext = candidate.with_extension(ext);
                        if with_ext.is_file() {
                            return Some(with_ext);
                        }
                    }
                }

                if candidate.is_dir() && depth > 1 {
                    if let Some(found) = search_directory_for_icon(
                        &candidate,
                        hint_path,
                        name_hints,
                        depth - 1,
                        false,
                    ) {
                        return Some(found);
                    }
                }
            }
        }

        if hint_path.is_none() {
            for hint in name_hints {
                if hint.len() < 3 {
                    continue;
                }

                let candidate = dir.join(hint);

                if candidate.is_file() {
                    return Some(candidate);
                }

                if candidate.extension().is_none() {
                    for ext in ["exe", "lnk", "ico", "appref-ms"] {
                        let with_ext = candidate.with_extension(ext);
                        if with_ext.is_file() {
                            return Some(with_ext);
                        }
                    }
                }

                if candidate.is_dir() && depth > 1 {
                    if let Some(found) =
                        search_directory_for_icon(&candidate, None, name_hints, depth - 1, false)
                    {
                        return Some(found);
                    }
                }
            }
        }
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return None,
    };

    for (index, entry) in entries.flatten().enumerate() {
        if index > 2000 {
            break;
        }

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        let path = entry.path();
        let name_lower = entry.file_name().to_string_lossy().to_ascii_lowercase();

        if file_type.is_file() {
            let ext_lower = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_ascii_lowercase());

            let Some(ext) = ext_lower.as_deref() else {
                continue;
            };

            if !matches!(ext, "exe" | "lnk" | "ico" | "appref-ms") {
                continue;
            }

            let stem_lower = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_ascii_lowercase());

            let matches_hint = if name_hints.is_empty() {
                true
            } else {
                name_hints.iter().any(|hint| {
                    if name_lower == *hint {
                        true
                    } else if let Some(stem) = stem_lower.as_ref() {
                        stem == hint
                            || stem.starts_with(hint)
                            || hint.starts_with(stem)
                            || name_lower.contains(hint)
                    } else {
                        name_lower.contains(hint)
                    }
                })
            };

            if matches_hint {
                return Some(path);
            }
        } else if file_type.is_dir() && depth > 1 {
            if should_descend(&name_lower, name_hints) {
                if let Some(found) =
                    search_directory_for_icon(&path, hint_path, name_hints, depth - 1, false)
                {
                    return Some(found);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn search_common_install_dirs(hint_path: Option<&Path>, name_hints: &[String]) -> Option<PathBuf> {
    let mut roots = Vec::new();
    for var in [
        "LOCALAPPDATA",
        "PROGRAMFILES",
        "PROGRAMFILES(X86)",
        "PROGRAMDATA",
    ] {
        if let Ok(value) = std::env::var(var) {
            if !value.is_empty() {
                let path = PathBuf::from(value);
                if path.is_dir() {
                    roots.push(path);
                }
            }
        }
    }

    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let programs = PathBuf::from(user_profile)
            .join("AppData")
            .join("Local")
            .join("Programs");
        if programs.is_dir() {
            roots.push(programs);
        }
    }

    for root in roots {
        if let Some(found) = search_directory_for_icon(&root, hint_path, name_hints, 3, true) {
            return Some(found);
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn encode_dynamic_image_as_data_url(
    image: image::DynamicImage,
    size: u32,
) -> Result<String, String> {
    use image::ImageEncoder;

    let resized = image.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    let rgba = resized.to_rgba8();
    encoder
        .write_image(&rgba, size, size, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    Ok(format!(
        "data:image/png;base64,{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_data)
    ))
}

#[cfg(target_os = "windows")]
fn encode_bytes_as_data_url(mime_type: &str, bytes: &[u8]) -> String {
    format!(
        "data:{};base64,{}",
        mime_type,
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, bytes)
    )
}

#[cfg(target_os = "windows")]
fn load_icon_from_existing_path(path: &Path) -> Result<String, String> {
    use exeico::get_exe_ico;

    if !path.exists() {
        return Err(format!("Icon path does not exist: {}", path.display()));
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    match extension.as_deref() {
        Some("exe") | Some("dll") => {
            let icon_bytes =
                get_exe_ico(path).map_err(|e| format!("Failed to extract icon: {}", e))?;

            match image::load_from_memory(&icon_bytes) {
                Ok(img) => encode_dynamic_image_as_data_url(img, 64),
                Err(_) => Ok(encode_bytes_as_data_url("image/x-icon", &icon_bytes)),
            }
        }
        _ => {
            let image = image::open(path)
                .map_err(|e| format!("Failed to load icon image {}: {}", path.display(), e))?;
            encode_dynamic_image_as_data_url(image, 64)
        }
    }
}

#[cfg(target_os = "windows")]
fn build_icon_name_hints_from_input(input: &str) -> Vec<String> {
    let mut hints = Vec::new();
    let trimmed = input
        .trim_matches(|c| matches!(c, '\"' | '\''))
        .trim()
        .to_string();

    if trimmed.is_empty() {
        return hints;
    }

    let normalized = normalize_windows_path(&trimmed);
    let path = Path::new(&normalized);

    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        hints.push(file_name.to_string());
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if stem.len() >= 3 {
                hints.push(stem.to_string());
            }
        }
    }

    for token in trimmed.split(|c: char| {
        matches!(
            c,
            '\\' | '/' | ' ' | '.' | '-' | '_' | ':' | ';' | ',' | '(' | ')' | '[' | ']'
        )
    }) {
        let clean = token.trim();
        if clean.len() >= 3 {
            hints.push(clean.to_string());
        }
    }

    let mut unique: Vec<String> = hints
        .into_iter()
        .map(|hint| hint.to_ascii_lowercase())
        .filter(|hint| !hint.is_empty())
        .collect();
    unique.sort();
    unique.dedup();
    unique
}

#[cfg(target_os = "windows")]
fn build_domain_candidates(name_hints: &[String]) -> Vec<String> {
    let mut candidates = Vec::new();

    for hint in name_hints {
        if hint.len() < 3 {
            continue;
        }

        if hint.contains('.') {
            candidates.push(hint.clone());
            continue;
        }

        let parts: Vec<&str> = hint
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|segment| !segment.is_empty())
            .collect();

        if parts.is_empty() {
            continue;
        }

        let collapsed = parts.join("");
        let hyphenated = parts.join("-");

        for base in [collapsed.as_str(), hyphenated.as_str(), hint.as_str()] {
            if base.is_empty() {
                continue;
            }

            for tld in [".com", ".net", ".io", ".app"] {
                candidates.push(format!("{}{}", base, tld));
            }
        }
    }

    candidates.sort();
    candidates.dedup();
    candidates.truncate(20);
    candidates
}

#[cfg(target_os = "windows")]
fn try_resolve_icon_path_from_input(input: &str, name_hints: &[String]) -> Option<PathBuf> {
    if input.is_empty() {
        return None;
    }

    let normalized = normalize_windows_path(input);
    if normalized.is_empty() {
        return None;
    }

    let candidate = PathBuf::from(&normalized);

    if candidate.is_file() {
        return Some(candidate);
    }

    if candidate.extension().is_none() {
        for ext in ["exe", "lnk", "ico", "png", "appref-ms"] {
            let with_ext = candidate.with_extension(ext);
            if with_ext.is_file() {
                return Some(with_ext);
            }
        }
    }

    if candidate.is_dir() {
        return search_directory_for_icon(&candidate, None, name_hints, 4, true)
            .filter(|path| path.is_file());
    }

    if let Some(parent) = candidate.parent() {
        if parent.is_dir() {
            return search_directory_for_icon(
                parent,
                Some(candidate.as_path()),
                name_hints,
                3,
                true,
            )
            .filter(|path| path.is_file());
        }
    }

    None
}

#[cfg(target_os = "windows")]
async fn fetch_icon_from_web(input: &str, name_hints: &[String]) -> Result<Option<String>, String> {
    use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

    let domain_candidates = build_domain_candidates(name_hints);
    if domain_candidates.is_empty() {
        return Ok(None);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) FocusLockIconFetcher/1.0",
        ),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    for domain in domain_candidates {
        for provider in [
            format!("https://icon.horse/icon/{}", domain),
            format!("https://logo.clearbit.com/{}", domain),
            format!(
                "https://www.google.com/s2/favicons?sz=128&domain={}",
                domain
            ),
        ] {
            if let Ok(response) = client.get(&provider).send().await {
                if !response.status().is_success() {
                    continue;
                }

                if let Ok(bytes) = response.bytes().await {
                    if bytes.is_empty() {
                        continue;
                    }

                    let content_type = response
                        .headers()
                        .get(reqwest::header::CONTENT_TYPE)
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or("image/png");

                    if content_type.contains("svg") {
                        return Ok(Some(encode_bytes_as_data_url("image/svg+xml", &bytes)));
                    }

                    if let Ok(image) = image::load_from_memory(&bytes) {
                        return Ok(Some(encode_dynamic_image_as_data_url(image, 64)?));
                    }

                    return Ok(Some(encode_bytes_as_data_url(content_type, &bytes)));
                }
            }
        }
    }

    eprintln!(
        "Web icon lookup failed for {} using hints {:?}",
        input, name_hints
    );

    Ok(None)
}

#[cfg(target_os = "windows")]
fn resolve_display_icon(
    display_name: &str,
    display_icon: Option<&str>,
    install_location: Option<&Path>,
    additional_paths: &[PathBuf],
) -> Option<PathBuf> {
    let parsed_display_icon = display_icon.and_then(parse_display_icon_path);

    let mut name_hints: Vec<String> = Vec::new();

    if let Some(path) = parsed_display_icon.as_ref() {
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            name_hints.push(file_name.to_string());
            if let Some(stem) = Path::new(file_name).file_stem().and_then(|s| s.to_str()) {
                if stem.len() >= 2 {
                    name_hints.push(stem.to_string());
                }
            }
        }
    }

    if let Some(raw_icon) = display_icon {
        for token in raw_icon.split(['\\', '/', ',', ' ']) {
            let trimmed = token.trim_matches(|c| matches!(c, '"' | '\'' | '[' | ']'));
            if trimmed.len() < 3 {
                continue;
            }

            name_hints.push(trimmed.to_string());

            for sub in trimmed.split(['.', ':']).filter(|sub| sub.len() >= 3) {
                name_hints.push(sub.to_string());
            }
        }
    }

    if !display_name.is_empty() {
        name_hints.push(display_name.to_string());
        let collapsed = display_name.replace([' ', '-', '_'], "");
        if collapsed.len() >= 3 {
            name_hints.push(collapsed);
        }
    }

    for extra in additional_paths {
        if let Some(file_name) = extra.file_name().and_then(|n| n.to_str()) {
            name_hints.push(file_name.to_string());
            if let Some(stem) = extra.file_stem().and_then(|s| s.to_str()) {
                if stem.len() >= 3 {
                    name_hints.push(stem.to_string());
                }
            }
        }
    }

    if let Some(base) = install_location {
        if let Some(folder) = base.file_name().and_then(|n| n.to_str()) {
            name_hints.push(folder.to_string());
        }
    }

    let mut name_hints: Vec<String> = name_hints
        .into_iter()
        .map(|name| name.to_ascii_lowercase())
        .filter(|name| !name.is_empty())
        .collect();
    name_hints.sort();
    name_hints.dedup();

    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(path) = parsed_display_icon.clone() {
        candidates.push(path);
    }

    if let (Some(base), Some(icon_path)) = (install_location, parsed_display_icon.as_ref()) {
        if !icon_path.is_absolute() {
            candidates.push(base.join(icon_path));
        }
    }

    candidates.extend(additional_paths.iter().cloned());

    let try_extensions = ["exe", "lnk", "ico", "appref-ms"];

    for candidate in candidates {
        if candidate.is_file() {
            return Some(candidate);
        }

        if candidate.extension().is_none() {
            for ext in try_extensions {
                let with_ext = candidate.with_extension(ext);
                if with_ext.is_file() {
                    return Some(with_ext);
                }
            }
        }

        if candidate.is_dir() {
            if let Some(found) = search_directory_for_icon(&candidate, None, &name_hints, 3, true) {
                return Some(found);
            }
        }
    }

    if let Some(path) = parsed_display_icon.as_ref() {
        if path.is_absolute() {
            if let Some(parent) = path.parent() {
                if let Some(found) = search_directory_for_icon(
                    parent,
                    parsed_display_icon.as_deref(),
                    &name_hints,
                    3,
                    false,
                ) {
                    return Some(found);
                }
            }
        }
    }

    if let Some(base) = install_location {
        if let Some(found) =
            search_directory_for_icon(base, parsed_display_icon.as_deref(), &name_hints, 4, true)
        {
            return Some(found);
        }
    }

    for extra in additional_paths {
        if let Some(parent) = extra.parent() {
            if let Some(found) = search_directory_for_icon(
                parent,
                parsed_display_icon.as_deref(),
                &name_hints,
                3,
                true,
            ) {
                return Some(found);
            }
        }
    }

    if let Some(found) = search_common_install_dirs(parsed_display_icon.as_deref(), &name_hints) {
        return Some(found);
    }

    None
}

#[cfg(target_os = "windows")]
#[tauri::command]
async fn get_installed_apps() -> Result<Vec<AppInfo>, String> {
    use winreg::enums::*;
    use winreg::RegKey;

    // Run heavy registry work in blocking thread using Tauri's runtime
    tauri::async_runtime::spawn_blocking(|| {
        let mut apps = Vec::new();
        let mut seen_names = HashSet::new();

        // Check both HKLM and HKCU for installed applications
        let paths = vec![
            (
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
            (
                HKEY_LOCAL_MACHINE,
                r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
            (
                HKEY_CURRENT_USER,
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            ),
        ];

        for (hkey_type, path) in paths {
            let hkey = RegKey::predef(hkey_type);
            if let Ok(uninstall_key) = hkey.open_subkey(path) {
                for key_name in uninstall_key.enum_keys().filter_map(|k| k.ok()) {
                    if let Ok(app_key) = uninstall_key.open_subkey(&key_name) {
                        if let Ok(display_name) = app_key.get_value::<String, _>("DisplayName") {
                            // Skip if we've already seen this app
                            if seen_names.contains(&display_name) {
                                continue;
                            }

                            // Skip system components and updates
                            if display_name.contains("Update")
                                || display_name.contains("Hotfix")
                                || display_name.starts_with("KB")
                            {
                                continue;
                            }

                            let install_location =
                                app_key.get_value::<String, _>("InstallLocation").ok();
                            let display_icon = app_key.get_value::<String, _>("DisplayIcon").ok();
                            let uninstall_string =
                                app_key.get_value::<String, _>("UninstallString").ok();
                            let quiet_uninstall_string =
                                app_key.get_value::<String, _>("QuietUninstallString").ok();

                            let normalized_install = install_location
                                .as_deref()
                                .map(normalize_windows_path)
                                .filter(|p| !p.is_empty());

                            let install_path =
                                normalized_install.as_ref().map(|p| PathBuf::from(p));

                            let uninstall_path = uninstall_string
                                .as_ref()
                                .and_then(|s| parse_command_path(s));
                            let quiet_uninstall_path = quiet_uninstall_string
                                .as_ref()
                                .and_then(|s| parse_command_path(s));

                            let mut extra_paths = Vec::new();
                            if let Some(path) = uninstall_path.clone() {
                                extra_paths.push(path);
                            }
                            if let Some(path) = quiet_uninstall_path.clone() {
                                extra_paths.push(path);
                            }

                            let resolved_icon = resolve_display_icon(
                                &display_name,
                                display_icon.as_deref(),
                                install_path.as_deref(),
                                &extra_paths,
                            );

                            let resolved_icon_str = resolved_icon
                                .as_ref()
                                .map(|p| p.to_string_lossy().into_owned());

                            let parsed_display_icon =
                                display_icon.as_deref().and_then(parse_display_icon_path);

                            let display_icon_hint = parsed_display_icon
                                .as_ref()
                                .filter(|p| p.is_absolute() && p.is_file())
                                .map(|p| p.to_string_lossy().into_owned());

                            let uninstall_hint = uninstall_path
                                .as_ref()
                                .filter(|p| p.is_file())
                                .map(|p| p.to_string_lossy().into_owned());
                            let quiet_uninstall_hint = quiet_uninstall_path
                                .as_ref()
                                .filter(|p| p.is_file())
                                .map(|p| p.to_string_lossy().into_owned());

                            let install_file_hint = install_path
                                .as_ref()
                                .filter(|p| p.is_file())
                                .map(|p| p.to_string_lossy().into_owned());

                            seen_names.insert(display_name.clone());

                            let path_value = resolved_icon_str
                                .clone()
                                .or_else(|| display_icon_hint.clone())
                                .or_else(|| normalized_install.clone());

                            let icon_hint_value = resolved_icon_str
                                .clone()
                                .or_else(|| display_icon_hint.clone())
                                .or_else(|| uninstall_hint.clone())
                                .or_else(|| quiet_uninstall_hint.clone())
                                .or_else(|| install_file_hint.clone());

                            apps.push(AppInfo {
                                name: display_name,
                                path: path_value,
                                icon_hint: icon_hint_value,
                            });
                        }
                    }
                }
            }
        }

        // Sort alphabetically
        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(apps)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[cfg(target_os = "windows")]
#[tauri::command]
async fn get_running_processes() -> Result<Vec<AppInfo>, String> {
    use sysinfo::{ProcessesToUpdate, System};

    // Run in blocking thread using Tauri's runtime, only refresh processes
    tauri::async_runtime::spawn_blocking(|| {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All);

        let mut processes = Vec::new();
        let mut seen_names = HashSet::new();

        for (_, process) in sys.processes() {
            let name = process.name().to_string_lossy().to_string();

            // Skip system processes and duplicates
            if name.is_empty()
                || seen_names.contains(&name)
                || name.starts_with("svchost")
                || name.starts_with("System")
            {
                continue;
            }

            // Get executable path
            let exe_path = process.exe().and_then(|p| p.to_str().map(String::from));

            seen_names.insert(name.clone());

            processes.push(AppInfo {
                name: name.clone(),
                path: exe_path,
                icon_hint: None,
            });
        }

        // Sort alphabetically
        processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(processes)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn get_installed_apps() -> Result<Vec<AppInfo>, String> {
    Err("This feature is only available on Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn get_running_processes() -> Result<Vec<AppInfo>, String> {
    Err("This feature is only available on Windows".to_string())
}

#[cfg(target_os = "windows")]
#[tauri::command]
async fn kill_process(process_name: String) -> Result<String, String> {
    use sysinfo::{ProcessesToUpdate, Signal, System};

    tauri::async_runtime::spawn_blocking(move || {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All);

        let mut killed_count = 0;

        for (_pid, process) in sys.processes() {
            let name = process.name().to_string_lossy().to_string();

            // Match process name (case-insensitive)
            if name.eq_ignore_ascii_case(&process_name) {
                if process.kill_with(Signal::Kill).is_some() {
                    killed_count += 1;
                }
            }
        }

        if killed_count > 0 {
            Ok(format!(
                "Killed {} instance(s) of {}",
                killed_count, process_name
            ))
        } else {
            Err(format!("Process '{}' not found", process_name))
        }
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn kill_process(_process_name: String) -> Result<String, String> {
    Err("This feature is only available on Windows".to_string())
}

// Extract icon from application path and return as base64 PNG
#[cfg(target_os = "windows")]
#[tauri::command]
async fn get_app_icon(app_path: String) -> Result<String, String> {
    let raw_input = app_path.split(',').next().unwrap_or(&app_path).trim();

    let normalized = normalize_windows_path(raw_input);

    let mut name_hints = build_icon_name_hints_from_input(raw_input);
    name_hints.extend(build_icon_name_hints_from_input(&app_path));
    name_hints.sort();
    name_hints.dedup();

    let mut path_inputs = Vec::new();
    if !normalized.is_empty() {
        path_inputs.push(normalized.clone());
    }
    if raw_input != normalized {
        path_inputs.push(raw_input.to_string());
    }
    if app_path != raw_input {
        path_inputs.push(app_path.clone());
    }

    for input in path_inputs.iter() {
        if input.is_empty() {
            continue;
        }

        if let Some(path) = try_resolve_icon_path_from_input(input, &name_hints) {
            let icon_result = tauri::async_runtime::spawn_blocking({
                let path_clone = path.clone();
                move || load_icon_from_existing_path(&path_clone)
            })
            .await
            .map_err(|e| format!("Task error: {}", e))??;

            return Ok(icon_result);
        }
    }

    if let Some(web_icon) = fetch_icon_from_web(&app_path, &name_hints).await? {
        return Ok(web_icon);
    }

    Err(format!("Unable to locate icon for {}", app_path))
}

#[cfg(target_os = "linux")]
#[tauri::command]
async fn get_app_icon(icon_hint: String) -> Result<String, String> {
    use image::ImageEncoder;
    use std::path::PathBuf;

    // icon_hint could be an icon name or path
    let icon_path = if icon_hint.starts_with('/') {
        // Absolute path
        PathBuf::from(&icon_hint)
    } else {
        // Icon name - search standard locations
        find_linux_icon(&icon_hint).ok_or("Icon not found")?
    };

    if !icon_path.exists() {
        return Err(format!("Icon file not found: {:?}", icon_path));
    }

    // Load the icon image
    let img = image::open(&icon_path).map_err(|e| format!("Failed to load icon: {}", e))?;

    // Resize to 32x32 for consistency
    let resized = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);

    // Encode to PNG
    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    let rgba = resized.to_rgba8();
    encoder
        .write_image(&rgba, 32, 32, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    let base64_image =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_data);
    Ok(format!("data:image/png;base64,{}", base64_image))
}

#[cfg(target_os = "linux")]
fn find_linux_icon(icon_name: &str) -> Option<PathBuf> {
    let sizes = vec![48, 64, 128, 256, 32];
    let formats = vec!["png", "svg", "xpm"];
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());

    // Search icon theme paths in order of priority
    for size in &sizes {
        for fmt in &formats {
            let paths = vec![
                format!(
                    "/usr/share/icons/hicolor/{}x{}/apps/{}.{}",
                    size, size, icon_name, fmt
                ),
                format!(
                    "{}/.local/share/icons/hicolor/{}x{}/apps/{}.{}",
                    home, size, size, icon_name, fmt
                ),
                format!(
                    "/usr/share/icons/gnome/{}x{}/apps/{}.{}",
                    size, size, icon_name, fmt
                ),
            ];

            for path in paths {
                let p = PathBuf::from(&path);
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }

    // Check pixmaps as fallback
    for fmt in &formats {
        let paths = vec![
            format!("/usr/share/pixmaps/{}.{}", icon_name, fmt),
            format!("{}/.local/share/pixmaps/{}.{}", home, icon_name, fmt),
        ];

        for path in paths {
            let p = PathBuf::from(&path);
            if p.exists() {
                return Some(p);
            }
        }
    }

    None
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
#[tauri::command]
async fn get_app_icon(_app_path: String) -> Result<String, String> {
    Err("Icon extraction not implemented on this platform".to_string())
}

#[cfg(target_os = "windows")]
#[tauri::command]
async fn start_session_monitor(
    app: tauri::AppHandle,
    monitor: State<'_, MonitorState>,
) -> Result<String, String> {
    use std::time::Duration;
    use sysinfo::{ProcessesToUpdate, Signal, System};

    // If already running, don't start again
    if monitor.is_running.load(Ordering::Relaxed) {
        return Ok("Monitor already running".to_string());
    }

    monitor.is_running.store(true, Ordering::Relaxed);
    let is_running = monitor.is_running.clone();

    // Spawn background monitoring task
    tauri::async_runtime::spawn(async move {
        println!("[Monitor] Starting session monitor loop");

        while is_running.load(Ordering::Relaxed) {
            // Get database state from app handle
            let db: State<DbState> = app.state();

            // Get active sessions
            if let Ok(sessions) = db::get_sessions(db.clone()) {
                let has_active_session = sessions.iter().any(|s| s.status == "running");

                if !has_active_session {
                    // No active sessions - stop monitoring
                    println!("[Monitor] No active sessions, stopping monitor");
                    is_running.store(false, Ordering::Relaxed);
                    break;
                }

                // Get block rules
                let db_for_rules: State<DbState> = app.state();
                if let Ok(rules) = db::get_block_rules(db_for_rules) {
                    let rules_clone = rules.clone();

                    // Get running processes in blocking thread
                    if let Ok(()) = tauri::async_runtime::spawn_blocking(move || {
                        let mut sys = System::new();
                        sys.refresh_processes(ProcessesToUpdate::All);

                        for (_, process) in sys.processes() {
                            let process_name = process.name().to_string_lossy().to_string();
                            let process_exe_path = process.exe()
                                .and_then(|p| p.to_str())
                                .unwrap_or("");

                            // Check if process matches any block rule
                            for rule in &rules_clone {
                                // Extract exe name from rule's app_id (could be path or exe name)
                                let rule_exe_name = if rule.app_id.contains('\\') || rule.app_id.contains('/') {
                                    // It's a path - extract the last component and add .exe if missing
                                    let path_parts: Vec<&str> = rule.app_id.split(&['\\', '/'][..]).collect();
                                    let last_part = path_parts.last().unwrap_or(&"");
                                    if last_part.to_lowercase().ends_with(".exe") {
                                        last_part.to_string()
                                    } else {
                                        format!("{}.exe", last_part)
                                    }
                                } else {
                                    // Already an exe name
                                    if rule.app_id.to_lowercase().ends_with(".exe") {
                                        rule.app_id.clone()
                                    } else {
                                        format!("{}.exe", rule.app_id)
                                    }
                                };

                                let matches = match rule.match_kind.as_str() {
                                    "exe" => process_name.eq_ignore_ascii_case(&rule_exe_name),
                                    "path" => process_exe_path.to_lowercase().contains(&rule.app_id.to_lowercase()),
                                    _ => process_name.to_lowercase().contains(&rule_exe_name.to_lowercase()),
                                };

                                if matches {
                                    if rule.mode == "hard" {
                                        // Hard mode: Kill the process
                                        if process.kill_with(Signal::Kill).is_some() {
                                            println!("[Monitor] HARD BLOCK - Killed: {} (rule: {})", process_name, rule.app_id);
                                        }
                                    } else if rule.mode == "soft" {
                                        // Soft mode: Log warning only
                                        println!("[Monitor] SOFT BLOCK - Warning: {} is running but not blocked (rule: {})", process_name, rule.app_id);
                                    }
                                }
                            }
                        }
                    }).await {
                        // Successfully checked processes
                    }
                }
            } else {
                // Database error - stop monitoring
                println!("[Monitor] Database error, stopping monitor");
                is_running.store(false, Ordering::Relaxed);
                break;
            }

            // Check every 2 seconds
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        // Ensure flag is reset when loop exits
        is_running.store(false, Ordering::Relaxed);
        println!("[Monitor] Session monitor loop stopped");
    });

    Ok("Session monitor started".to_string())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn start_session_monitor() -> Result<String, String> {
    Err("This feature is only available on Windows".to_string())
}

#[tauri::command]
fn stop_session_monitor(monitor: State<MonitorState>) -> Result<String, String> {
    monitor.is_running.store(false, Ordering::Relaxed);
    Ok("Session monitor stopped".to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize SQLite database in app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // Create directory if it doesn't exist
            std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

            let db_path = app_data_dir.join("focuslock.db");
            let db_state =
                DbState::new(db_path.to_str().unwrap()).expect("Failed to initialize database");

            // Initialize monitor state
            let monitor_state = MonitorState {
                is_running: Arc::new(AtomicBool::new(false)),
            };

            app.manage(db_state);
            app.manage(monitor_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_installed_apps,
            get_running_processes,
            kill_process,
            get_app_icon,
            start_session_monitor,
            stop_session_monitor,
            // Database commands
            db::get_favorites,
            db::create_favorite,
            db::delete_favorite,
            db::get_block_rules,
            db::create_block_rule,
            db::update_block_rule,
            db::delete_block_rule,
            db::get_sessions,
            db::create_session,
            db::update_session,
            db::get_settings,
            db::upsert_setting,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, _event| {
            // No cleanup needed - SQLite handles it
        });
}
