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
fn sanitized_registry_value(value: &str) -> String {
    let first = value.split(',').next().unwrap_or(value);
    normalize_windows_path(first)
}

#[cfg(target_os = "windows")]
fn resolve_display_icon(
    display_icon: Option<&str>,
    install_location: Option<&str>,
) -> Option<String> {
    let display_icon = display_icon?;
    let sanitized_icon = sanitized_registry_value(display_icon);

    if sanitized_icon.is_empty() {
        return None;
    }

    let icon_path = PathBuf::from(&sanitized_icon);

    if icon_path.is_absolute() {
        if icon_path.exists() {
            return Some(icon_path.to_string_lossy().into_owned());
        }

        if icon_path.extension().is_none() {
            for ext in ["exe", "lnk", "ico"] {
                let with_ext = icon_path.with_extension(ext);
                if with_ext.exists() {
                    return Some(with_ext.to_string_lossy().into_owned());
                }
            }
        }
    }

    let install_location = install_location.map(normalize_windows_path);
    let Some(install_location) = install_location else {
        return None;
    };

    if install_location.is_empty() {
        return None;
    }

    let base = PathBuf::from(&install_location);
    let mut candidate = base.join(&sanitized_icon);

    if candidate.exists() {
        return Some(candidate.to_string_lossy().into_owned());
    }

    if candidate.extension().is_none() {
        for ext in ["exe", "lnk", "ico"] {
            let with_ext = candidate.with_extension(ext);
            if with_ext.exists() {
                return Some(with_ext.to_string_lossy().into_owned());
            }
        }
    }

    if let Ok(entries) = fs::read_dir(&base) {
        let needle = sanitized_icon.to_lowercase();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(file_name) = path.file_name().map(|n| n.to_string_lossy().to_lowercase()) {
                if !file_name.starts_with(&needle) {
                    continue;
                }

                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_ascii_lowercase();
                    if matches!(ext_lower.as_str(), "exe" | "lnk" | "ico") {
                        return Some(path.to_string_lossy().into_owned());
                    }
                }
            }
        }
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

                            let normalized_install = install_location
                                .as_deref()
                                .map(normalize_windows_path)
                                .filter(|p| !p.is_empty());

                            let resolved_icon = resolve_display_icon(
                                display_icon.as_deref(),
                                normalized_install.as_deref(),
                            );

                            let sanitized_display_icon = display_icon
                                .as_deref()
                                .map(sanitized_registry_value)
                                .filter(|p| !p.is_empty());

                            seen_names.insert(display_name.clone());

                            apps.push(AppInfo {
                                name: display_name,
                                path: resolved_icon.clone().or_else(|| normalized_install.clone()),
                                icon_hint: resolved_icon
                                    .or(sanitized_display_icon)
                                    .or_else(|| normalized_install.clone()),
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

/// Parse the DisplayIcon registry format and return the executable path
/// DisplayIcon can be "C:\path\to\app.exe,0" or just "C:\path\to\app.exe"
#[cfg(target_os = "windows")]
fn parse_display_icon_path(display_icon: &str) -> String {
    display_icon
        .split(',')
        .next()
        .unwrap_or(display_icon)
        .trim_matches('"')
        .trim()
        .to_string()
}

/// Try to find an executable in common Windows installation directories
#[cfg(target_os = "windows")]
fn search_common_install_dirs(app_name: &str) -> Option<PathBuf> {
    let program_files = std::env::var("PROGRAMFILES").unwrap_or_else(|_| r"C:\Program Files".to_string());
    let program_files_x86 = std::env::var("PROGRAMFILES(X86)").unwrap_or_else(|_| r"C:\Program Files (x86)".to_string());
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let app_data = std::env::var("APPDATA").unwrap_or_default();

    let search_roots = vec![
        program_files,
        program_files_x86,
        local_app_data,
        app_data,
    ];

    // Clean the app name for directory matching
    let clean_name = app_name
        .replace(".exe", "")
        .replace(".lnk", "")
        .to_lowercase();

    for root in search_roots {
        if root.is_empty() {
            continue;
        }

        let root_path = PathBuf::from(&root);
        if !root_path.exists() {
            continue;
        }

        // Try direct subdirectory match
        if let Ok(entries) = fs::read_dir(&root_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let dir_name = path.file_name()
                    .map(|n| n.to_string_lossy().to_lowercase())
                    .unwrap_or_default();

                // Check if directory name contains our app name
                if dir_name.contains(&clean_name) || clean_name.contains(&dir_name) {
                    // Search for exe files in this directory
                    if let Some(exe_path) = search_directory_for_exe(&path, 2) {
                        return Some(exe_path);
                    }
                }
            }
        }
    }

    None
}

/// Recursively search a directory for executable files (limited depth)
#[cfg(target_os = "windows")]
fn search_directory_for_exe(dir: &Path, max_depth: u32) -> Option<PathBuf> {
    if max_depth == 0 || !dir.is_dir() {
        return None;
    }

    let entries = fs::read_dir(dir).ok()?;

    // First pass: look for exe files directly
    let mut subdirs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.to_string_lossy().to_lowercase() == "exe" {
                    // Skip uninstallers and updaters
                    let file_name = path.file_name()
                        .map(|n| n.to_string_lossy().to_lowercase())
                        .unwrap_or_default();

                    if !file_name.contains("unins")
                        && !file_name.contains("update")
                        && !file_name.contains("setup")
                        && !file_name.contains("install") {
                        return Some(path);
                    }
                }
            }
        } else if path.is_dir() {
            subdirs.push(path);
        }
    }

    // Second pass: recurse into subdirectories
    for subdir in subdirs {
        if let Some(exe) = search_directory_for_exe(&subdir, max_depth - 1) {
            return Some(exe);
        }
    }

    None
}

/// Load icon from an ICO file directly
#[cfg(target_os = "windows")]
fn load_ico_file(path: &Path) -> Result<String, String> {
    let ico_data = fs::read(path)
        .map_err(|e| format!("Failed to read ICO file: {}", e))?;

    encode_icon_data(&ico_data)
}

/// Encode icon data (ICO format) to base64 PNG data URL
#[cfg(target_os = "windows")]
fn encode_icon_data(ico_data: &[u8]) -> Result<String, String> {
    use image::ImageEncoder;

    match image::load_from_memory(ico_data) {
        Ok(img) => {
            // Resize to 32x32 for consistency
            let resized = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);

            // Encode to PNG
            let mut png_data = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
            let rgba = resized.to_rgba8();
            encoder
                .write_image(&rgba, 32, 32, image::ExtendedColorType::Rgba8)
                .map_err(|e| format!("Failed to encode PNG: {}", e))?;

            let base64_image = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &png_data,
            );
            Ok(format!("data:image/png;base64,{}", base64_image))
        }
        Err(_) => {
            // If ICO loading failed, just return the raw ICO data as base64
            let base64_image = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                ico_data,
            );
            Ok(format!("data:image/x-icon;base64,{}", base64_image))
        }
    }
}

/// Check if a path is a WindowsApps (UWP/Store app) path
/// These paths have restrictive permissions and can't be read directly
#[cfg(target_os = "windows")]
fn is_windows_store_app(app_path: &str) -> bool {
    app_path.to_lowercase().contains("windowsapps")
}

/// Extract the app name from a UWP app path
/// e.g., "C:\...\TradingView.Desktop_2.13.0.7353_x64__n534cwy3pjxzj\TradingView.exe" -> "tradingview"
#[cfg(target_os = "windows")]
fn extract_uwp_app_name(app_path: &str) -> Option<String> {
    let path = PathBuf::from(app_path);

    // Try to get app name from the exe filename first
    if let Some(file_stem) = path.file_stem() {
        let name = file_stem.to_string_lossy().to_lowercase();
        if !name.is_empty() && name != "app" {
            return Some(name);
        }
    }

    // Try to extract from the parent folder name (e.g., "TradingView.Desktop_2.13...")
    if let Some(parent) = path.parent() {
        if let Some(folder_name) = parent.file_name() {
            let folder = folder_name.to_string_lossy();
            // Extract the part before the underscore or version number
            if let Some(name_part) = folder.split('_').next() {
                // Remove ".Desktop" or similar suffixes
                let clean_name = name_part
                    .split('.')
                    .next()
                    .unwrap_or(name_part)
                    .to_lowercase();
                if !clean_name.is_empty() {
                    return Some(clean_name);
                }
            }
        }
    }

    None
}

/// Search Start Menu for shortcuts matching app name and extract icon
#[cfg(target_os = "windows")]
fn find_start_menu_icon(app_name: &str) -> Option<PathBuf> {
    let app_name_lower = app_name.to_lowercase();

    // Start Menu locations to search
    let start_menu_paths = vec![
        std::env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("Microsoft").join("Windows").join("Start Menu").join("Programs"))
            .ok(),
        std::env::var("PROGRAMDATA")
            .map(|p| PathBuf::from(p).join("Microsoft").join("Windows").join("Start Menu").join("Programs"))
            .ok(),
        // Also check Desktop for shortcuts
        std::env::var("USERPROFILE")
            .map(|p| PathBuf::from(p).join("Desktop"))
            .ok(),
    ];

    for start_menu in start_menu_paths.into_iter().flatten() {
        if !start_menu.exists() {
            continue;
        }

        // Search recursively (limited depth)
        if let Some(icon) = search_start_menu_for_app(&start_menu, &app_name_lower, 3) {
            return Some(icon);
        }
    }

    None
}

/// Recursively search Start Menu for matching shortcuts
#[cfg(target_os = "windows")]
fn search_start_menu_for_app(dir: &Path, app_name: &str, depth: u32) -> Option<PathBuf> {
    if depth == 0 || !dir.is_dir() {
        return None;
    }

    let entries = fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() {
            let file_name = path.file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            // Check if this shortcut matches our app name
            if file_name.contains(app_name) {
                // Found a matching shortcut - look for associated icon files nearby
                if let Some(parent) = path.parent() {
                    // Check for .ico files in the same directory
                    if let Ok(siblings) = fs::read_dir(parent) {
                        for sibling in siblings.flatten() {
                            let sibling_path = sibling.path();
                            if sibling_path.is_file() {
                                let sibling_name = sibling_path.file_name()
                                    .map(|n| n.to_string_lossy().to_lowercase())
                                    .unwrap_or_default();

                                if sibling_name.contains(app_name) &&
                                   (sibling_name.ends_with(".ico") || sibling_name.ends_with(".png")) {
                                    return Some(sibling_path);
                                }
                            }
                        }
                    }
                }

                // If it's an .lnk file, we can try to extract icon from it
                if file_name.ends_with(".lnk") {
                    return Some(path);
                }
            }
        } else if path.is_dir() {
            subdirs.push(path);
        }
    }

    // Recurse into subdirectories
    for subdir in subdirs {
        if let Some(icon) = search_start_menu_for_app(&subdir, app_name, depth - 1) {
            return Some(icon);
        }
    }

    None
}

/// Try to extract icon from a .lnk shortcut file
#[cfg(target_os = "windows")]
fn extract_icon_from_lnk(lnk_path: &Path) -> Result<String, String> {
    use exeico::get_exe_ico;

    // Read the .lnk file to find the target
    // .lnk files have a complex binary format, but we can try to extract the target path
    let lnk_data = fs::read(lnk_path)
        .map_err(|e| format!("Failed to read shortcut: {}", e))?;

    // Simple heuristic: look for .exe path in the file
    // This is a simplified approach - proper parsing would require a dedicated library
    let content = String::from_utf8_lossy(&lnk_data);

    // Look for paths ending in .exe
    for segment in content.split('\0') {
        let trimmed = segment.trim();
        if trimmed.len() > 4 &&
           trimmed.to_lowercase().ends_with(".exe") &&
           !trimmed.to_lowercase().contains("windowsapps") {
            // Found a potential exe path, try to extract icon from it
            if Path::new(trimmed).exists() {
                if let Ok(ico_data) = get_exe_ico(trimmed) {
                    return encode_icon_data(&ico_data);
                }
            }
        }
    }

    Err("Could not extract icon from shortcut".to_string())
}

/// Load a PNG/image file and return as base64 data URL
#[cfg(target_os = "windows")]
fn load_image_file(path: &Path) -> Result<String, String> {
    use image::ImageEncoder;

    match image::open(path) {
        Ok(img) => {
            let resized = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);
            let mut png_data = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
            let rgba = resized.to_rgba8();
            encoder
                .write_image(&rgba, 32, 32, image::ExtendedColorType::Rgba8)
                .map_err(|e| format!("Failed to encode PNG: {}", e))?;
            let base64_image = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &png_data,
            );
            Ok(format!("data:image/png;base64,{}", base64_image))
        }
        Err(e) => Err(format!("Failed to load image: {}", e))
    }
}

/// Try to resolve an icon path using multiple strategies
#[cfg(target_os = "windows")]
fn try_resolve_icon_path(input: &str) -> Option<PathBuf> {
    let parsed = parse_display_icon_path(input);
    let expanded = expand_env_vars(&parsed);
    let path = PathBuf::from(&expanded);

    // Strategy 1: Direct path exists
    if path.exists() && path.is_file() {
        return Some(path);
    }

    // Strategy 2: Try adding common extensions
    if path.extension().is_none() {
        for ext in &["exe", "ico", "lnk"] {
            let with_ext = path.with_extension(ext);
            if with_ext.exists() && with_ext.is_file() {
                return Some(with_ext);
            }
        }
    }

    // Strategy 3: Check if it's in the same directory with different extension
    if let Some(parent) = path.parent() {
        if let Some(stem) = path.file_stem() {
            for ext in &["exe", "ico"] {
                let alt_path = parent.join(stem).with_extension(ext);
                if alt_path.exists() && alt_path.is_file() {
                    return Some(alt_path);
                }
            }
        }
    }

    // Strategy 4: Search in the parent directory for any matching exe
    if let Some(parent) = path.parent() {
        if parent.exists() && parent.is_dir() {
            if let Some(exe) = search_directory_for_exe(parent, 1) {
                return Some(exe);
            }
        }
    }

    None
}

// Extract icon from application path and return as base64 PNG
#[cfg(target_os = "windows")]
#[tauri::command]
async fn get_app_icon(app_path: String) -> Result<String, String> {
    use exeico::get_exe_ico;

    // Strategy 0: Check for UWP/Windows Store apps first
    // These apps are in WindowsApps folder which has restrictive permissions
    // We cannot read from it directly, so we search Start Menu for shortcuts instead
    if is_windows_store_app(&app_path) {
        // Extract app name from the UWP path
        if let Some(app_name) = extract_uwp_app_name(&app_path) {
            // Search Start Menu for a matching shortcut
            if let Some(shortcut_path) = find_start_menu_icon(&app_name) {
                let ext = shortcut_path.extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_default();

                match ext.as_str() {
                    "ico" => {
                        return load_ico_file(&shortcut_path);
                    }
                    "png" | "jpg" | "jpeg" | "bmp" => {
                        return load_image_file(&shortcut_path);
                    }
                    "lnk" => {
                        // Try to extract icon from the shortcut's target
                        if let Ok(icon) = extract_icon_from_lnk(&shortcut_path) {
                            return Ok(icon);
                        }
                    }
                    _ => {}
                }
            }
        }

        // For UWP apps, return a specific error so frontend can use fallback
        return Err(format!("UWP_APP:{}", app_path));
    }

    // Try to resolve the icon path using multiple strategies
    let resolved_path = try_resolve_icon_path(&app_path);

    // If we have a resolved path, try to extract icon based on file type
    if let Some(path) = resolved_path {
        let ext = path.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "ico" => {
                // Load ICO file directly
                return load_ico_file(&path);
            }
            "exe" | "dll" => {
                // Extract icon from executable
                let path_str = path.to_string_lossy().to_string();
                match get_exe_ico(&path_str) {
                    Ok(ico_data) => {
                        return encode_icon_data(&ico_data);
                    }
                    Err(e) => {
                        // Log but continue to fallback
                        eprintln!("Failed to extract icon from {}: {}", path_str, e);
                    }
                }
            }
            "png" | "jpg" | "jpeg" | "bmp" => {
                // Load image directly
                return load_image_file(&path);
            }
            _ => {
                // Try as executable anyway
                let path_str = path.to_string_lossy().to_string();
                if let Ok(ico_data) = get_exe_ico(&path_str) {
                    return encode_icon_data(&ico_data);
                }
            }
        }
    }

    // Fallback: Try searching common install directories
    // Extract a reasonable app name from the path
    let app_name = app_path
        .split(&['\\', '/'][..])
        .filter(|s| !s.is_empty())
        .last()
        .unwrap_or(&app_path);

    if let Some(found_path) = search_common_install_dirs(app_name) {
        let path_str = found_path.to_string_lossy().to_string();
        if let Ok(ico_data) = get_exe_ico(&path_str) {
            return encode_icon_data(&ico_data);
        }
    }

    // Last fallback: search Start Menu for any app
    let clean_app_name = app_name.replace(".exe", "").to_lowercase();
    if let Some(shortcut_path) = find_start_menu_icon(&clean_app_name) {
        let ext = shortcut_path.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "ico" => {
                return load_ico_file(&shortcut_path);
            }
            "png" | "jpg" | "jpeg" | "bmp" => {
                return load_image_file(&shortcut_path);
            }
            "lnk" => {
                if let Ok(icon) = extract_icon_from_lnk(&shortcut_path) {
                    return Ok(icon);
                }
            }
            _ => {}
        }
    }

    Err(format!("Unable to locate icon for: {}", app_path))
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
