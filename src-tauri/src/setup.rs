use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize)]
pub struct SetupStatus {
    pub configured: bool,
    pub site_name: Option<String>,
    pub apps_installed: Vec<String>,
}

#[derive(Deserialize)]
pub struct CompanyData {
    pub company_name: String,
    pub abbreviation: String,
    pub country: String,
    pub currency: String,
}

#[derive(Serialize)]
pub struct SetupProgress {
    pub step: String,
    pub message: String,
    pub completed: bool,
}

pub fn check_setup(sites_dir: &PathBuf) -> SetupStatus {
    let sites = list_sites(sites_dir);
    if sites.is_empty() {
        return SetupStatus {
            configured: false,
            site_name: None,
            apps_installed: vec![],
        };
    }
    let site = sites[0].clone();
    let apps = list_installed_apps(sites_dir, &site);
    SetupStatus {
        configured: true,
        site_name: Some(site),
        apps_installed: apps,
    }
}

pub fn find_resources_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("resources")
}

pub fn find_bundled_binary(name: &str) -> Option<PathBuf> {
    let resources_dir = find_resources_dir();
    let binary_name = if cfg!(target_os = "windows") {
        format!("{}.exe", name)
    } else {
        name.to_string()
    };

    let candidate = resources_dir.join(name).join(&binary_name);
    if candidate.exists() {
        return Some(candidate);
    }

    let candidate = resources_dir.join(&binary_name);
    if candidate.exists() {
        return Some(candidate);
    }

    None
}

pub fn create_site(
    sites_dir: &PathBuf,
    bench_dir: &PathBuf,
    site_name: &str,
    admin_password: &str,
) -> Result<String, String> {
    let bench = find_bench()?;

    if sites_dir.join(site_name).join("site_config.json").exists() {
        return Err(format!("Site '{}' already exists", site_name));
    }

    let output = Command::new(&bench)
        .arg("new-site")
        .arg(site_name)
        .arg("--admin-password")
        .arg(admin_password)
        .arg("--db-port")
        .arg("3307")
        .arg("--mariadb-root-password")
        .arg(admin_password)
        .current_dir(bench_dir)
        .output()
        .map_err(|e| format!("Failed to run bench new-site: {}", e))?;

    if output.status.success() {
        Ok(format!("Site '{}' created", site_name))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already exists") {
            Ok(format!("Site '{}' already exists, continuing", site_name))
        } else {
            Err(format!("Failed to create site: {}", stderr))
        }
    }
}

pub fn install_apps(
    bench_dir: &PathBuf,
    site_name: &str,
    apps: &[String],
) -> Result<Vec<String>, String> {
    let bench = find_bench()?;
    let mut installed = vec![];

    for app in apps {
        let output = Command::new(&bench)
            .arg("--site")
            .arg(site_name)
            .arg("install-app")
            .arg(app)
            .current_dir(bench_dir)
            .output()
            .map_err(|e| format!("Failed to install app '{}': {}", e, app))?;

        if output.status.success() {
            installed.push(app.clone());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("already installed") {
                installed.push(app.clone());
            } else {
                return Err(format!("Failed to install '{}': {}", app, stderr));
            }
        }
    }
    Ok(installed)
}

pub fn setup_company(
    bench_dir: &PathBuf,
    site_name: &str,
    data: &CompanyData,
) -> Result<String, String> {
    let bench = find_bench()?;

    let python_script = format!(
        r#"
import frappe
frappe.init(site='{site}')
frappe.connect()
frappe.get_doc({{
    "doctype": "Company",
    "company_name": "{name}",
    "abbreviation": "{abbr}",
    "country": "{country}",
    "default_currency": "{currency}",
    "enable_perpetual_inventory": 0
}}).insert(ignore_if_duplicate=True)
frappe.db.commit()
frappe.destroy()
"#,
        site = site_name,
        name = data.company_name,
        abbr = data.abbreviation,
        country = data.country,
        currency = data.currency,
    );

    let mut output = Command::new(&bench)
        .arg("--site")
        .arg(site_name)
        .arg("console")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(bench_dir)
        .spawn()
        .map_err(|e| format!("Failed to start bench console: {}", e))?;

    {
        use std::io::Write;
        if let Some(mut stdin) = output.stdin.take() {
            let _ = stdin.write_all(python_script.as_bytes());
        }
    }

    let result = output.wait_with_output().map_err(|e| e.to_string())?;
    if result.status.success() {
        Ok("Company setup completed".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        if stderr.contains("Error") || stderr.contains("Traceback") {
            Err(format!("Company setup failed: {}", stderr))
        } else {
            Ok("Company setup completed (with warnings)".to_string())
        }
    }
}

fn list_sites(sites_dir: &PathBuf) -> Vec<String> {
    let mut sites = vec![];
    if let Ok(entries) = fs::read_dir(sites_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let site_config = entry.path().join("site_config.json");
                if site_config.exists() {
                    if let Some(name) = entry.file_name().to_str() {
                        sites.push(name.to_string());
                    }
                }
            }
        }
    }
    sites
}

fn list_installed_apps(sites_dir: &PathBuf, site: &str) -> Vec<String> {
    let apps_file = sites_dir.join(site).join("apps.txt");
    if let Ok(content) = fs::read_to_string(&apps_file) {
        return content.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    }
    vec![]
}

fn find_bench() -> Result<String, String> {
    let candidates = if cfg!(target_os = "windows") {
        vec!["bench.exe", "bench"]
    } else {
        vec!["bench"]
    };

    for name in &candidates {
        if let Ok(path) = which::which(name) {
            return Ok(path.to_string_lossy().to_string());
        }
    }

    let bundled = find_bundled_binary("bench");
    if let Some(bench_path) = bundled {
        return Ok(bench_path.to_string_lossy().to_string());
    }

    let fallbacks = vec!["/usr/local/bin/bench", "/usr/bin/bench", "/home/frappe/.local/bin/bench"];
    for path in &fallbacks {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }

    Err("bench command not found. Please install Frappe Bench or bundled binaries.".to_string())
}
