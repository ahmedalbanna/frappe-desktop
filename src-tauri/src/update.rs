use tauri::{App, Manager};

pub fn init_updater(app: &App) {
    let _ = app;
}

pub fn check_for_updates() -> Result<Option<String>, String> {
    Ok(None)
}