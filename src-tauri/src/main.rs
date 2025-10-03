// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Manager, State};

mod db;
use db::DbState;

// Global monitor state
struct MonitorState {
    is_running: Arc<AtomicBool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AppInfo {
    name: String,
    path: Option<String>,
    icon: Option<String>,
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
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
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
                        if display_name.contains("Update") || 
                           display_name.contains("Hotfix") ||
                           display_name.starts_with("KB") {
                            continue;
                        }
                        
                        let install_location = app_key.get_value::<String, _>("InstallLocation").ok();
                        let display_icon = app_key.get_value::<String, _>("DisplayIcon").ok();
                        
                        seen_names.insert(display_name.clone());
                        
                        apps.push(AppInfo {
                            name: display_name,
                            path: install_location,
                            icon: display_icon,
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
    use sysinfo::{System, ProcessesToUpdate};
    
    // Run in blocking thread using Tauri's runtime, only refresh processes
    tauri::async_runtime::spawn_blocking(|| {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All);
    
    let mut processes = Vec::new();
    let mut seen_names = HashSet::new();
    
    for (_, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_string();
        
        // Skip system processes and duplicates
        if name.is_empty() || 
           seen_names.contains(&name) ||
           name.starts_with("svchost") ||
           name.starts_with("System") {
            continue;
        }
        
        // Get executable path
        let exe_path = process.exe().and_then(|p| p.to_str().map(String::from));
        
        seen_names.insert(name.clone());
        
        processes.push(AppInfo {
            name: name.clone(),
            path: exe_path,
            icon: None,
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
    use sysinfo::{System, ProcessesToUpdate, Signal};
    
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
            Ok(format!("Killed {} instance(s) of {}", killed_count, process_name))
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
    use std::path::Path;
    use image::ImageEncoder;
    use exeico::get_exe_ico;
    
    // Extract the actual exe path from DisplayIcon format
    // DisplayIcon can be "C:\path\to\app.exe,0" or just "C:\path\to\app.exe"
    let exe_path = app_path.split(',').next().unwrap_or(&app_path).trim_matches('"');
    
    if !Path::new(exe_path).exists() {
        return Err(format!("File not found: {}", exe_path));
    }
    
    // Try to extract the real icon from the exe
    match get_exe_ico(exe_path) {
        Ok(ico_data) => {
            // Convert ICO to PNG
            match image::load_from_memory(&ico_data) {
                Ok(img) => {
                    // Resize to 32x32 for consistency
                    let resized = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);
                    
                    // Encode to PNG
                    let mut png_data = Vec::new();
                    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
                    let rgba = resized.to_rgba8();
                    encoder.write_image(&rgba, 32, 32, image::ExtendedColorType::Rgba8)
                        .map_err(|e| format!("Failed to encode PNG: {}", e))?;
                    
                    let base64_image = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_data);
                    Ok(format!("data:image/png;base64,{}", base64_image))
                }
                Err(_) => {
                    // If ICO loading failed, just return the raw ICO data as base64
                    let base64_image = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &ico_data);
                    Ok(format!("data:image/x-icon;base64,{}", base64_image))
                }
            }
        }
        Err(e) => {
            Err(format!("Failed to extract icon: {}", e))
        }
    }
}

#[cfg(target_os = "linux")]
#[tauri::command]
async fn get_app_icon(icon_hint: String) -> Result<String, String> {
    use std::path::PathBuf;
    use image::ImageEncoder;
    
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
    let img = image::open(&icon_path)
        .map_err(|e| format!("Failed to load icon: {}", e))?;
    
    // Resize to 32x32 for consistency
    let resized = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);
    
    // Encode to PNG
    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    let rgba = resized.to_rgba8();
    encoder.write_image(&rgba, 32, 32, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    
    let base64_image = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_data);
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
                format!("/usr/share/icons/hicolor/{}x{}/apps/{}.{}", size, size, icon_name, fmt),
                format!("{}/.local/share/icons/hicolor/{}x{}/apps/{}.{}", home, size, size, icon_name, fmt),
                format!("/usr/share/icons/gnome/{}x{}/apps/{}.{}", size, size, icon_name, fmt),
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
    use sysinfo::{System, ProcessesToUpdate, Signal};
    use std::time::Duration;
    
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
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");
            
            // Create directory if it doesn't exist
            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory");
            
            let db_path = app_data_dir.join("focuslock.db");
            let db_state = DbState::new(db_path.to_str().unwrap())
                .expect("Failed to initialize database");
            
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
