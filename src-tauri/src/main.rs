// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    
    let paths = vec![
        (hklm, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (hklm, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
        (hkcu, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];
    
    for (hkey, path) in paths {
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
    use sysinfo::{System, SystemExt, ProcessExt};
    
    // Run in blocking thread using Tauri's runtime, only refresh processes
    tauri::async_runtime::spawn_blocking(|| {
        let mut sys = System::new();
        sys.refresh_processes();
    
    let mut processes = Vec::new();
    let mut seen_names = HashSet::new();
    
    for (_, process) in sys.processes() {
        let name = process.name().to_string();
        
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
            name: name.trim_end_matches(".exe").to_string(),
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

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_installed_apps,
            get_running_processes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
