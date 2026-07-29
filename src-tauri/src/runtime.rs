use serde::Serialize;
use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Clone, Serialize)]
pub struct ProcessInfo {
    pub state: String,
    pub pid: Option<u32>,
    pub healthy: bool,
}

#[derive(Clone, Serialize)]
pub struct ServiceConfig {
    pub mariadb_port: u16,
    pub redis_port: u16,
    pub frappe_port: u16,
    pub data_dir: String,
    pub mariadb_socket: String,
}

pub struct RuntimeManager {
    mariadb_proc: Option<Child>,
    redis_proc: Option<Child>,
    frappe_proc: Option<Child>,
    data_dir: PathBuf,
    log_dir: PathBuf,
    config: ServiceConfig,
}

impl RuntimeManager {
    pub fn new() -> Self {
        let data_dir = dirs_or_default();
        let log_dir = data_dir.join("logs");
        let _ = fs::create_dir_all(&log_dir);
        let sock = data_dir.join("mariadb.sock");

        RuntimeManager {
            mariadb_proc: None,
            redis_proc: None,
            frappe_proc: None,
            config: ServiceConfig {
                mariadb_port: 3307,
                redis_port: 6379,
                frappe_port: 8000,
                data_dir: data_dir.to_string_lossy().to_string(),
                mariadb_socket: sock.to_string_lossy().to_string(),
            },
            data_dir,
            log_dir,
        }
    }

    pub fn get_config(&self) -> ServiceConfig {
        self.config.clone()
    }

    pub fn get_status(&mut self) -> Vec<ProcessInfo> {
        vec![
            proc_info(&mut self.mariadb_proc, self.config.mariadb_port),
            proc_info(&mut self.redis_proc, self.config.redis_port),
            proc_info(&mut self.frappe_proc, self.config.frappe_port),
        ]
    }

    pub fn start_all(&mut self) -> Result<(), String> {
        self.start_mariadb()?;
        self.start_redis()?;
        self.start_frappe()?;
        Ok(())
    }

    pub fn stop_all(&mut self) -> Result<(), String> {
        kill_process(&mut self.frappe_proc);
        kill_process(&mut self.redis_proc);
        kill_process(&mut self.mariadb_proc);
        Ok(())
    }

    pub fn start_service(&mut self, name: &str) -> Result<(), String> {
        match name {
            "mariadb" => self.start_mariadb(),
            "redis" => self.start_redis(),
            "frappe" => self.start_frappe(),
            _ => Err(format!("Unknown service: {}", name)),
        }
    }

    pub fn stop_service(&mut self, name: &str) -> Result<(), String> {
        match name {
            "mariadb" => { kill_process(&mut self.mariadb_proc); Ok(()) }
            "redis" => { kill_process(&mut self.redis_proc); Ok(()) }
            "frappe" => { kill_process(&mut self.frappe_proc); Ok(()) }
            _ => Err(format!("Unknown service: {}", name)),
        }
    }

    pub fn restart_service(&mut self, name: &str) -> Result<(), String> {
        self.stop_service(name)?;
        std::thread::sleep(Duration::from_millis(500));
        self.start_service(name)
    }

    // ── MariaDB ──────────────────────────────────────────────

    fn start_mariadb(&mut self) -> Result<(), String> {
        let mariadb_bin = find_binary("mariadbd")
            .or_else(|| find_binary("mysqld"))
            .ok_or_else(|| "MariaDB binary not found".to_string())?;

        let install_bin = find_binary("mariadb-install-db")
            .or_else(|| find_binary("mysql_install_db"));

        let data_dir = self.data_dir.join("mariadb-data");
        let socket_path = &self.config.mariadb_socket;

        let needs_init = !data_dir.join("mysql").join("user.ibd").exists()
            && !data_dir.join("mysql").join("user.MAD").exists();

        if needs_init {
            fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
            if let Some(installer) = install_bin {
                let output = Command::new(&installer)
                    .arg("--datadir").arg(&data_dir)
                    .arg("--auth-root-authentication-method=normal")
                    .output()
                    .map_err(|e| format!("Failed to init MariaDB data dir: {}", e))?;
                if !output.status.success() {
                    return Err(format!(
                        "mariadb-install-db failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
        }

        let log_file = self.log_dir.join("mariadb.log");
        let log_handle = fs::OpenOptions::new()
            .create(true).append(true).open(&log_file)
            .map_err(|e| e.to_string())?;

        let child = Command::new(&mariadb_bin)
            .arg("--datadir").arg(&data_dir)
            .arg("--port").arg(self.config.mariadb_port.to_string())
            .arg("--socket").arg(socket_path)
            .arg("--skip-grant-tables")
            .arg("--skip-networking=0")
            .arg("--bind-address=127.0.0.1")
            .stdout(Stdio::from(log_handle.try_clone().map_err(|e| e.to_string())?))
            .stderr(Stdio::from(log_handle))
            .spawn()
            .map_err(|e| format!("Failed to start MariaDB: {}", e))?;

        self.mariadb_proc = Some(child);
        wait_for_port(self.config.mariadb_port, 10)
    }

    // ── Redis ────────────────────────────────────────────────

    fn start_redis(&mut self) -> Result<(), String> {
        let redis_bin = find_binary("redis-server")
            .ok_or_else(|| "Redis binary not found".to_string())?;

        let log_file = self.log_dir.join("redis.log");
        let log_handle = fs::OpenOptions::new()
            .create(true).append(true).open(&log_file)
            .map_err(|e| e.to_string())?;

        let child = Command::new(&redis_bin)
            .arg("--port").arg(self.config.redis_port.to_string())
            .arg("--bind").arg("127.0.0.1")
            .arg("--save").arg("")
            .arg("--appendonly").arg("no")
            .arg("--loglevel").arg("notice")
            .stdout(Stdio::from(log_handle.try_clone().map_err(|e| e.to_string())?))
            .stderr(Stdio::from(log_handle))
            .spawn()
            .map_err(|e| format!("Failed to start Redis: {}", e))?;

        self.redis_proc = Some(child);
        wait_for_port(self.config.redis_port, 5)
    }

    // ── Frappe ───────────────────────────────────────────────

    fn start_frappe(&mut self) -> Result<(), String> {
        let bench = find_binary("bench")
            .ok_or_else(|| "bench not found".to_string())?;

        let bench_dir = self.data_dir.join("frappe-bench");
        fs::create_dir_all(&bench_dir).map_err(|e| e.to_string())?;

        let log_file = self.log_dir.join("frappe.log");
        let log_handle = fs::OpenOptions::new()
            .create(true).append(true).open(&log_file)
            .map_err(|e| e.to_string())?;

        let child = Command::new(&bench)
            .arg("start")
            .arg("--port").arg(self.config.frappe_port.to_string())
            .current_dir(&bench_dir)
            .stdout(Stdio::from(log_handle.try_clone().map_err(|e| e.to_string())?))
            .stderr(Stdio::from(log_handle))
            .spawn()
            .map_err(|e| format!("Failed to start Frappe: {}", e))?;

        self.frappe_proc = Some(child);
        wait_for_port(self.config.frappe_port, 30)
    }

    // ── Logs ─────────────────────────────────────────────────

    pub fn get_logs(&self, name: &str) -> Result<Vec<String>, String> {
        let log_file = match name {
            "mariadb" => self.log_dir.join("mariadb.log"),
            "redis" => self.log_dir.join("redis.log"),
            "frappe" => self.log_dir.join("frappe.log"),
            _ => return Err(format!("Unknown service: {}", name)),
        };

        if !log_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&log_file).map_err(|e| e.to_string())?;
        let all_lines: Vec<&str> = content.lines().collect();
        let n = all_lines.len();
        let last_100: Vec<String> = if n > 100 {
            all_lines[n - 100..].iter().map(|s| s.to_string()).collect()
        } else {
            all_lines.iter().map(|s| s.to_string()).collect()
        };
        Ok(last_100)
    }
}

// ── Free functions ───────────────────────────────────────────

fn proc_info(proc: &mut Option<Child>, port: u16) -> ProcessInfo {
    let (state, pid) = match proc.as_mut() {
        Some(child) => match child.try_wait() {
            Ok(Some(_)) => ("stopped".into(), None),
            Ok(None) => ("running".into(), Some(child.id())),
            Err(_) => ("error".into(), None),
        },
        None => ("stopped".into(), None),
    };
    let healthy = if state == "running" {
        tcp_health_check(port)
    } else {
        false
    };
    ProcessInfo { state, pid, healthy }
}

fn kill_process(proc: &mut Option<Child>) {
    if let Some(mut child) = proc.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn tcp_health_check(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    if let Ok(mut addrs) = addr.to_socket_addrs() {
        if let Some(sockaddr) = addrs.next() {
            return TcpStream::connect_timeout(&sockaddr, Duration::from_secs(2)).is_ok();
        }
    }
    false
}

fn wait_for_port(port: u16, max_secs: u64) -> Result<(), String> {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(max_secs) {
        if tcp_health_check(port) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    Err(format!("Timeout waiting for port {} ({}s)", port, max_secs))
}

fn dirs_or_default() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("frappe-desktop")
}

fn find_binary(name: &str) -> Option<PathBuf> {
    let bundled = std::env::current_exe().ok()?
        .parent()?
        .join("resources")
        .join(if cfg!(target_os = "windows") {
            format!("{}.exe", name)
        } else {
            name.to_string()
        });

    if bundled.exists() {
        return Some(bundled);
    }

    which::which(name).ok()
}
