use std::process::Command;
use std::path::Path;
use chrono::Local;

pub fn create_backup(dest: &str) -> Result<String, String> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_name = format!("frappe_backup_{}.sql.gz", timestamp);
    let backup_path = Path::new(dest).join(&backup_name);

    let output = Command::new("mysqldump")
        .arg("--port=3307")
        .arg("--all-databases")
        .arg("--single-transaction")
        .arg("--quick")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start mysqldump: {}", e))?;

    let gzip = Command::new("gzip")
        .arg("-c")
        .stdin(output.stdout.unwrap())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start gzip: {}", e))?;

    let output = gzip.wait_with_output().map_err(|e| e.to_string())?;

    std::fs::write(&backup_path, &output.stdout)
        .map_err(|e| format!("Failed to write backup: {}", e))?;

    Ok(backup_path.to_str().unwrap().to_string())
}

pub fn restore_backup(path: &str) -> Result<String, String> {
    let gunzip = Command::new("gunzip")
        .arg("-c")
        .arg(path)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start gunzip: {}", e))?;

    let output = Command::new("mysql")
        .arg("--port=3307")
        .stdin(gunzip.stdout.unwrap())
        .output()
        .map_err(|e| format!("Failed to start mysql: {}", e))?;

    if output.status.success() {
        Ok("Restore completed".to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("Restore failed: {}", error))
    }
}
