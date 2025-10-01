// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::{Child, Command};
use std::sync::Mutex;
use tauri::Manager;

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

struct ServerProcess(Mutex<Option<Child>>);

fn start_backend_server(app_handle: &tauri::AppHandle) -> Result<Child, String> {
    // Check if Node.js is available
    if Command::new("node").arg("--version").output().is_err() {
        return Err("Node.js is not installed or not in PATH. The backend server requires Node.js to run.".to_string());
    }

    let resource_path = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;
    
    let server_script = resource_path.join("dist").join("index.js");
    
    // Verify server script exists
    if !server_script.exists() {
        return Err(format!("Backend server script not found at: {}", server_script.display()));
    }
    
    // Spawn Node.js process to run the server
    let child = Command::new("node")
        .arg(server_script)
        .env("PORT", "5000")
        .env("NODE_ENV", "production")
        .spawn()
        .map_err(|e| format!("Failed to start server: {}. Make sure Node.js is installed.", e))?;
    
    Ok(child)
}

async fn wait_for_backend() -> Result<(), String> {
    use std::time::Duration;
    
    // Wait up to 10 seconds for backend to be ready
    for _i in 0..20 {
        if let Ok(response) = reqwest::get("http://localhost:5000/api/health").await {
            if response.status().is_success() {
                println!("Backend is ready!");
                return Ok(());
            }
        }
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }).await.ok();
    }
    
    Err("Backend failed to start within timeout".to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Start the backend server
            match start_backend_server(app.handle()) {
                Ok(child) => {
                    app.manage(ServerProcess(Mutex::new(Some(child))));
                    println!("Backend server started, waiting for it to be ready...");
                    
                    // Wait for backend to be ready
                    tauri::async_runtime::block_on(async {
                        if let Err(e) = wait_for_backend().await {
                            eprintln!("Warning: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Warning: Failed to start backend server: {}", e);
                    eprintln!("The app may not function correctly without the backend");
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_installed_apps,
            get_running_processes
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // Cleanup on app exit
            if let tauri::RunEvent::Exit = event {
                if let Some(server_process) = app_handle.try_state::<ServerProcess>() {
                    if let Ok(mut child) = server_process.0.lock() {
                        if let Some(mut process) = child.take() {
                            let _ = process.kill();
                            println!("Backend server stopped");
                        }
                    }
                }
            }
        });
}
