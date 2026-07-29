use sha2::{Sha256, Digest};
use std::fs;

pub fn get_machine_id() -> Result<String, String> {
    // Try D-Bus machine ID (Linux)
    if let Ok(id) = fs::read_to_string("/etc/machine-id") {
        return Ok(id.trim().to_string());
    }
    // Try /var/lib/dbus/machine-id
    if let Ok(id) = fs::read_to_string("/var/lib/dbus/machine-id") {
        return Ok(id.trim().to_string());
    }
    // Fallback: generate from hostname + OS info
    let hostname = hostname();
    let os_info = std::env::consts::OS;
    let combined = format!("{}-{}", hostname, os_info);
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}

pub fn activate(key: String) -> Result<String, String> {
    let machine_id = get_machine_id()?;
    let expected = generate_license(&machine_id);

    if key == expected || verify_license(&key, &machine_id) {
        save_license(&key)?;
        Ok("activated".to_string())
    } else {
        Err("Invalid license key".to_string())
    }
}

fn generate_license(machine_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hasher.update(b"frappe-desktop-secret-salt");
    let result = hasher.finalize();
    hex::encode(&result[..8]) // Short license key
}

fn verify_license(key: &str, machine_id: &str) -> bool {
    key == generate_license(machine_id)
}

fn save_license(key: &str) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("frappe-desktop");

    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;

    let license_file = config_dir.join("license.key");
    fs::write(&license_file, key).map_err(|e| e.to_string())?;

    Ok(())
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}
