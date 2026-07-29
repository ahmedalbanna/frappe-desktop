use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::PathBuf;

const LICENSE_SERVER: &str = "https://api.frappe-desktop.com/v1";
const TRIAL_DAYS: i64 = 30;

#[derive(Serialize, Deserialize, Clone)]
pub struct LicenseInfo {
    pub key: String,
    pub machine_id: String,
    pub license_type: String,        // "trial", "perpetual", "subscription"
    pub activated_at: String,
    pub expires_at: String,
    pub valid: bool,
}

#[derive(Serialize, Deserialize)]
struct ActivateResponse {
    status: String,
    expires_at: Option<String>,
    license_type: Option<String>,
    error: Option<String>,
}

// ── Public API ────────────────────────────────────────────────

pub fn activate_online(key: String) -> Result<LicenseInfo, String> {
    let machine_id = get_machine_id()?;
    let client = reqwest::blocking::Client::new();
    let resp: ActivateResponse = client
        .post(format!("{}/activate", LICENSE_SERVER))
        .json(&serde_json::json!({
            "machine_id": machine_id,
            "license_key": key,
        }))
        .send()
        .map_err(|e| format!("License server error: {}", e))?
        .json()
        .map_err(|e| format!("Invalid server response: {}", e))?;

    if resp.status != "ok" {
        return Err(resp.error.unwrap_or_else(|| "Activation failed".into()));
    }

    let expires = resp.expires_at.unwrap_or_else(|| future_date(365));
    let license = LicenseInfo {
        key,
        machine_id,
        license_type: resp.license_type.unwrap_or_else(|| "perpetual".into()),
        activated_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        expires_at: expires,
        valid: true,
    };
    save_license(&license)?;
    Ok(license)
}

pub fn activate_offline(key: String, signature: String) -> Result<LicenseInfo, String> {
    let machine_id = get_machine_id()?;
    let expected = generate_offline_key(&machine_id, &key);
    if signature != expected {
        return Err("Invalid offline activation code".into());
    }
    let license = LicenseInfo {
        key,
        machine_id,
        license_type: "perpetual".into(),
        activated_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        expires_at: future_date(3650), // ~10 years for perpetual
        valid: true,
    };
    save_license(&license)?;
    Ok(license)
}

pub fn start_trial() -> Result<LicenseInfo, String> {
    let machine_id = get_machine_id()?;
    let trial_key = format!("trial-{}", &machine_id[..16]);
    let license = LicenseInfo {
        key: trial_key,
        machine_id,
        license_type: "trial".into(),
        activated_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        expires_at: future_date(TRIAL_DAYS),
        valid: true,
    };
    save_license(&license)?;
    Ok(license)
}

pub fn deactivate() -> Result<(), String> {
    let lic = load_license()?;
    let _ = reqwest::blocking::Client::new()
        .post(format!("{}/deactivate", LICENSE_SERVER))
        .json(&serde_json::json!({
            "machine_id": lic.machine_id,
            "license_key": lic.key,
        }))
        .send();
    delete_license()?;
    Ok(())
}

pub fn validate_license() -> LicenseInfo {
    match load_license() {
        Ok(mut lic) => {
            if let Ok(expires) = DateTime::parse_from_rfc3339(&lic.expires_at) {
                lic.valid = expires > Utc::now();
                if !lic.valid {
                    lic.license_type = "expired".into();
                }
            } else {
                lic.valid = false;
            }
            lic
        }
        Err(_) => LicenseInfo {
            key: String::new(),
            machine_id: get_machine_id().unwrap_or_default(),
            license_type: "none".into(),
            activated_at: String::new(),
            expires_at: String::new(),
            valid: false,
        },
    }
}

pub fn get_machine_id() -> Result<String, String> {
    if let Ok(id) = fs::read_to_string("/etc/machine-id") {
        return Ok(id.trim().to_string());
    }
    if let Ok(id) = fs::read_to_string("/var/lib/dbus/machine-id") {
        return Ok(id.trim().to_string());
    }
    let combined = format!("{}-{}", hostname(), std::env::consts::OS);
    Ok(hex::encode(Sha256::digest(combined.as_bytes())))
}

pub fn check_for_updates() -> Result<serde_json::Value, String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(format!("{}/updates/latest", LICENSE_SERVER))
        .send()
        .map_err(|e| format!("Update server error: {}", e))?
        .json::<serde_json::Value>()
        .map_err(|e| format!("Invalid update response: {}", e))?;
    Ok(resp)
}

// ── Internal ─────────────────────────────────────────────────

fn generate_offline_key(machine_id: &str, license_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hasher.update(b":");
    hasher.update(license_key.as_bytes());
    hasher.update(b":frappe-desktop-offline-salt");
    hex::encode(&hasher.finalize()[..12])
}

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("frappe-desktop")
}

fn license_path() -> PathBuf {
    config_dir().join("license.json")
}

fn save_license(lic: &LicenseInfo) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(lic).map_err(|e| e.to_string())?;
    fs::write(license_path(), json).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_license() -> Result<LicenseInfo, String> {
    let data = fs::read_to_string(license_path()).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

fn delete_license() -> Result<(), String> {
    let path = license_path();
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn future_date(days: i64) -> String {
    (Utc::now() + Duration::days(days))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}
