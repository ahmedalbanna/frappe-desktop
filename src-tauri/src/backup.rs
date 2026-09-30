use std::process::Command;
use std::path::Path;
use std::fs;
use std::io::Write;
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

pub fn create_full_backup(dest: &str, site_name: &str) -> Result<String, String> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_name = format!("frappe_backup_{}.tar.gz", timestamp);
    let backup_path = Path::new(dest).join(&backup_name);

    let mutable_output = Command::new("mysqldump")
        .arg("--port=3307")
        .arg("--single-transaction")
        .arg("--quick")
        .arg(site_name)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start mysqldump: {}", e))?;

    let mut sql_data = Vec::new();
    if let Some(mut stdout) = mutable_output.stdout.take() {
        use std::io::Read;
        std::io::copy(&mut stdout, &mut sql_data).map_err(|e| format!("Failed to read SQL dump: {}", e))?;
    }

    let mut tar_builder = Command::new("tar")
        .arg("-czf")
        .arg(&backup_path)
        .arg("-C")
        .arg(dest)
        .arg(".")
        .spawn()
        .map_err(|e| format!("Failed to create tar archive: {}", e))?;

    let status = tar_builder.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("Failed to create backup archive".to_string());
    }

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

pub fn backup_files(site_name: &str, dest: &str) -> Result<String, String> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_name = format!("files_backup_{}.tar.gz", timestamp);
    let backup_path = Path::new(dest).join(&backup_name);

    let site_path = Path::new("sites").join(site_name);
    let public_files = site_path.join("public").join("files");
    let private_files = site_path.join("private").join("files");

    let mut cmd = Command::new("tar");
    cmd.arg("-czf")
        .arg(&backup_path)
        .arg("-C");

    if public_files.exists() {
        cmd.arg(&public_files);
    }

    let status = cmd.status().map_err(|e| format!("Failed to create files backup: {}", e))?;
    if !status.success() {
        return Err("Failed to backup files".to_string());
    }

    Ok(backup_path.to_str().unwrap().to_string())
}
