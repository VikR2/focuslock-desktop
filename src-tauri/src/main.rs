// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::Manager;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};

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

fn get_log_path() -> std::path::PathBuf {
    std::env::temp_dir().join("focuslock_debug.log")
}

fn log_to_file(msg: &str) {
    let log_path = get_log_path();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(file, "[{}] {}", timestamp, msg);
    }
}

fn start_backend_server(app_handle: &tauri::AppHandle) -> Result<Child, String> {
    log_to_file("=== Starting backend server ===");
    
    // Check if Node.js is available
    log_to_file("Checking for Node.js...");
    match Command::new("node").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            log_to_file(&format!("Node.js found: {}", version.trim()));
        }
        Err(e) => {
            let err_msg = format!("Node.js check failed: {}", e);
            log_to_file(&err_msg);
            return Err("Node.js is not installed or not in PATH. The backend server requires Node.js to run.".to_string());
        }
    }

    let resource_path = app_handle
        .path()
        .resource_dir()
        .map_err(|e| {
            let err_msg = format!("Failed to get resource dir: {}", e);
            log_to_file(&err_msg);
            err_msg
        })?;
    
    log_to_file(&format!("Resource path: {}", resource_path.display()));
    
    let server_script = resource_path.join("dist").join("index.js");
    log_to_file(&format!("Looking for server script at: {}", server_script.display()));
    
    // Verify server script exists
    if !server_script.exists() {
        let err_msg = format!("Backend server script not found at: {}", server_script.display());
        log_to_file(&err_msg);
        
        // List contents of resource directory for debugging
        if let Ok(entries) = std::fs::read_dir(&resource_path) {
            log_to_file("Resource directory contents:");
            for entry in entries.flatten() {
                log_to_file(&format!("  - {}", entry.path().display()));
            }
        }
        
        return Err(err_msg);
    }
    
    log_to_file("Server script found, starting Node.js process...");
    
    // Spawn Node.js process to run the server with stdout/stderr capture
    let mut child = Command::new("node")
        .arg(&server_script)
        .env("PORT", "5000")
        .env("NODE_ENV", "production")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            let err_msg = format!("Failed to start server: {}. Make sure Node.js is installed.", e);
            log_to_file(&err_msg);
            err_msg
        })?;
    
    log_to_file(&format!("Node.js process started with PID: {:?}", child.id()));
    
    // Capture stdout in a separate thread
    if let Some(stdout) = child.stdout.take() {
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().flatten() {
                log_to_file(&format!("[BACKEND STDOUT] {}", line));
            }
        });
    }
    
    // Capture stderr in a separate thread
    if let Some(stderr) = child.stderr.take() {
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                log_to_file(&format!("[BACKEND STDERR] {}", line));
            }
        });
    }
    
    Ok(child)
}

async fn wait_for_backend() -> Result<(), String> {
    use std::time::Duration;
    
    log_to_file("Waiting for backend health check...");
    
    // Wait up to 10 seconds for backend to be ready
    for i in 0..20 {
        log_to_file(&format!("Health check attempt {}/20", i + 1));
        
        match reqwest::get("http://localhost:5000/api/health").await {
            Ok(response) => {
                let status = response.status();
                log_to_file(&format!("Health check response status: {}", status));
                
                if status.is_success() {
                    log_to_file("Backend is ready!");
                    return Ok(());
                }
            }
            Err(e) => {
                log_to_file(&format!("Health check failed: {}", e));
            }
        }
        
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    let err_msg = "Backend failed to start within timeout";
    log_to_file(err_msg);
    Err(err_msg.to_string())
}

#[tauri::command]
fn get_debug_log_path() -> String {
    get_log_path()
        .to_string_lossy()
        .to_string()
}

fn main() {
    // Log the app startup
    log_to_file(&format!("App starting... Log file: {}", get_log_path().display()));
    
    tauri::Builder::default()
        .setup(|app| {
            // Start the backend server
            match start_backend_server(app.handle()) {
                Ok(child) => {
                    app.manage(ServerProcess(Mutex::new(Some(child))));
                    log_to_file("Backend server process created, waiting for health check...");
                    
                    // Wait for backend to be ready
                    tauri::async_runtime::block_on(async {
                        if let Err(e) = wait_for_backend().await {
                            log_to_file(&format!("Backend startup error: {}", e));
                        }
                    });
                }
                Err(e) => {
                    log_to_file(&format!("Critical: Failed to start backend server: {}", e));
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_installed_apps,
            get_running_processes,
            get_debug_log_path
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
