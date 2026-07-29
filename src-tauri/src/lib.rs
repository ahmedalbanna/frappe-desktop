mod runtime;
mod licensing;
mod backup;

use runtime::{ProcessInfo, ServiceConfig};
use tauri::State;
use std::sync::Mutex;

pub struct AppState {
    pub runtime: Mutex<runtime::RuntimeManager>,
}

#[tauri::command]
fn get_service_status(state: State<AppState>) -> Vec<ProcessInfo> {
    state.runtime.lock().unwrap().get_status()
}

#[tauri::command]
fn get_service_config(state: State<AppState>) -> ServiceConfig {
    state.runtime.lock().unwrap().get_config()
}

#[tauri::command]
fn start_all_services(state: State<AppState>) -> Result<(), String> {
    state.runtime.lock().unwrap().start_all()
}

#[tauri::command]
fn stop_all_services(state: State<AppState>) -> Result<(), String> {
    state.runtime.lock().unwrap().stop_all()
}

#[tauri::command]
fn start_service(state: State<AppState>, name: String) -> Result<(), String> {
    state.runtime.lock().unwrap().start_service(&name)
}

#[tauri::command]
fn stop_service(state: State<AppState>, name: String) -> Result<(), String> {
    state.runtime.lock().unwrap().stop_service(&name)
}

#[tauri::command]
fn restart_service(state: State<AppState>, name: String) -> Result<(), String> {
    state.runtime.lock().unwrap().restart_service(&name)
}

#[tauri::command]
fn get_service_logs(state: State<AppState>, name: String) -> Result<Vec<String>, String> {
    state.runtime.lock().unwrap().get_logs(&name)
}

#[tauri::command]
fn get_frappe_url() -> String {
    "http://localhost:8000".to_string()
}

#[tauri::command]
fn get_hardware_id() -> Result<String, String> {
    licensing::get_machine_id()
}

#[tauri::command]
fn activate_license(key: String) -> Result<String, String> {
    licensing::activate(key)
}

#[tauri::command]
fn create_backup(path: String) -> Result<String, String> {
    backup::create_backup(&path)
}

#[tauri::command]
fn restore_backup(path: String) -> Result<String, String> {
    backup::restore_backup(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState {
        runtime: Mutex::new(runtime::RuntimeManager::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_service_status,
            get_service_config,
            start_all_services,
            stop_all_services,
            start_service,
            stop_service,
            restart_service,
            get_service_logs,
            get_frappe_url,
            get_hardware_id,
            activate_license,
            create_backup,
            restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
