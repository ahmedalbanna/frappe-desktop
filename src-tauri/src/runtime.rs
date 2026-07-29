use crate::ServiceStatus;
use std::process::{Child, Command};
use std::path::PathBuf;

pub struct RuntimeManager {
    mariadb_proc: Option<Child>,
    redis_proc: Option<Child>,
    frappe_proc: Option<Child>,
    data_dir: PathBuf,
}

impl RuntimeManager {
    pub fn new() -> Self {
        let data_dir = dirs_or_default();
        RuntimeManager {
            mariadb_proc: None,
            redis_proc: None,
            frappe_proc: None,
            data_dir,
        }
    }

    pub fn get_status(&mut self) -> ServiceStatus {
        ServiceStatus {
            mariadb: if self.mariadb_proc.as_mut().map_or(false, |p| p.try_wait().ok().flatten().is_none()) {
                "running".into()
            } else {
                "stopped".into()
            },
            redis: if self.redis_proc.as_mut().map_or(false, |p| p.try_wait().ok().flatten().is_none()) {
                "running".into()
            } else {
                "stopped".into()
            },
            frappe: if self.frappe_proc.as_mut().map_or(false, |p| p.try_wait().ok().flatten().is_none()) {
                "running".into()
            } else {
                "stopped".into()
            },
        }
    }

    pub fn start_all(&mut self) -> Result<(), String> {
        self.start_mariadb()?;
        self.start_redis()?;
        self.start_frappe()?;
        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<(), String> {
        self.stop_frappe();
        self.stop_redis();
        self.stop_mariadb();
        Ok(())
    }

    fn start_mariadb(&mut self) -> Result<(), String> {
        let mariadb_bin = find_binary("mariadbd")
            .or_else(|| find_binary("mysqld"))
            .ok_or_else(|| "MariaDB binary not found".to_string())?;

        let data_dir = self.data_dir.join("mariadb-data");
        std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

        let child = Command::new(&mariadb_bin)
            .arg("--datadir").arg(data_dir.to_str().unwrap())
            .arg("--port").arg("3307")
            .arg("--skip-grant-tables")
            .spawn()
            .map_err(|e| format!("Failed to start MariaDB: {}", e))?;

        self.mariadb_proc = Some(child);
        Ok(())
    }

    fn start_redis(&mut self) -> Result<(), String> {
        let redis_bin = find_binary("redis-server")
            .ok_or_else(|| "Redis binary not found".to_string())?;

        let child = Command::new(&redis_bin)
            .arg("--port").arg("6379")
            .arg("--save").arg("")
            .arg("--appendonly").arg("no")
            .spawn()
            .map_err(|e| format!("Failed to start Redis: {}", e))?;

        self.redis_proc = Some(child);
        Ok(())
    }

    fn start_frappe(&mut self) -> Result<(), String> {
        let bench = find_binary("bench")
            .ok_or_else(|| "bench not found".to_string())?;

        let child = Command::new(&bench)
            .arg("start")
            .current_dir(self.data_dir.join("frappe-bench"))
            .spawn()
            .map_err(|e| format!("Failed to start Frappe: {}", e))?;

        self.frappe_proc = Some(child);
        Ok(())
    }

    fn stop_mariadb(&mut self) {
        if let Some(mut child) = self.mariadb_proc.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn stop_redis(&mut self) {
        if let Some(mut child) = self.redis_proc.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn stop_frappe(&mut self) {
        if let Some(mut child) = self.frappe_proc.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn dirs_or_default() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("frappe-desktop")
}

fn find_binary(name: &str) -> Option<PathBuf> {
    // First check bundled resources directory
    let bundled = std::env::current_exe().ok()?
        .parent()?
        .join("resources")
        .join(name);

    if bundled.exists() {
        return Some(bundled);
    }

    // Then check PATH
    which::which(name).ok()
}
