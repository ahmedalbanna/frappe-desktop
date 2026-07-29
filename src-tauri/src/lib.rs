mod runtime;
mod licensing;
mod backup;
mod setup;

use licensing::LicenseInfo;
use runtime::{ProcessInfo, ServiceConfig};
use setup::{SetupStatus, CompanyData};
use tauri::State;
use std::sync::Mutex;
use std::path::PathBuf;

pub struct AppState {
    pub runtime: Mutex<runtime::RuntimeManager>,
}

fn sites_dir() -> PathBuf {
    runtime::data_dir().join("frappe-bench").join("sites")
}

fn bench_dir() -> PathBuf {
    runtime::data_dir().join("frappe-bench")
}

// ── Service commands ─────────────────────────────────────────

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

// ── Setup commands ────────────────────────────────────────────

#[tauri::command]
fn check_setup_status() -> SetupStatus {
    setup::check_setup(&sites_dir())
}

#[tauri::command]
fn create_site(site_name: String, admin_password: String) -> Result<String, String> {
    setup::create_site(&sites_dir(), &bench_dir(), &site_name, &admin_password)
}

#[tauri::command]
fn install_apps(site_name: String, apps: Vec<String>) -> Result<Vec<String>, String> {
    setup::install_apps(&bench_dir(), &site_name, &apps)
}

#[tauri::command]
fn setup_company(site_name: String, data: CompanyData) -> Result<String, String> {
    setup::setup_company(&bench_dir(), &site_name, &data)
}

// ── Licensing commands ────────────────────────────────────────

#[tauri::command]
fn get_license_status() -> LicenseInfo {
    licensing::validate_license()
}

#[tauri::command]
fn get_hardware_id() -> Result<String, String> {
    licensing::get_machine_id()
}

#[tauri::command]
fn activate_license_online(key: String) -> Result<LicenseInfo, String> {
    licensing::activate_online(key)
}

#[tauri::command]
fn activate_license_offline(key: String, signature: String) -> Result<LicenseInfo, String> {
    licensing::activate_offline(key, signature)
}

#[tauri::command]
fn start_trial() -> Result<LicenseInfo, String> {
    licensing::start_trial()
}

#[tauri::command]
fn deactivate_license() -> Result<(), String> {
    licensing::deactivate()
}

#[tauri::command]
fn check_for_updates() -> Result<serde_json::Value, String> {
    licensing::check_for_updates()
}

// ── Backup commands ───────────────────────────────────────────

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
            check_setup_status,
            create_site,
            install_apps,
            setup_company,
            get_license_status,
            get_hardware_id,
            activate_license_online,
            activate_license_offline,
            start_trial,
            deactivate_license,
            check_for_updates,
            create_backup,
            restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
