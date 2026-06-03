use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::process::is_process_alive;

use super::SchedulerStatus;

#[derive(Debug, Serialize, Deserialize)]
struct PidContent {
    pid: u32,
    started_at: String,
}

fn pid_file_path(config_dir: &Path) -> PathBuf {
    config_dir.join("scheduler.pid")
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".into())
}

/// Write PID file for the current process.
pub fn write_pid_file(config_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(config_dir)?;
    let content = PidContent {
        pid: std::process::id(),
        started_at: now_rfc3339(),
    };
    let path = pid_file_path(config_dir);
    std::fs::write(&path, serde_json::to_vec_pretty(&content)?)?;
    Ok(())
}

/// Remove PID file.
pub fn remove_pid_file(config_dir: &Path) -> Result<()> {
    let path = pid_file_path(config_dir);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// Check whether another scheduler instance is already running.
pub fn is_already_running(config_dir: &Path) -> bool {
    let path = pid_file_path(config_dir);
    if !path.exists() {
        return false;
    }
    match read_pid(&path) {
        Some(content) => is_process_alive(content.pid),
        None => false,
    }
}

/// Read the PID from the file.
fn read_pid(path: &Path) -> Option<PidContent> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Query the current scheduler status.
pub fn status(config_dir: &Path) -> SchedulerStatus {
    let path = pid_file_path(config_dir);
    if !path.exists() {
        return SchedulerStatus {
            running: false,
            pid: None,
            last_run: None,
            next_run: None,
        };
    }

    match read_pid(&path) {
        Some(content) => {
            let alive = is_process_alive(content.pid);
            if !alive {
                let _ = std::fs::remove_file(&path);
            }
            SchedulerStatus {
                running: alive,
                pid: if alive { Some(content.pid) } else { None },
                last_run: None,
                next_run: None,
            }
        }
        None => {
            let _ = std::fs::remove_file(&path);
            SchedulerStatus {
                running: false,
                pid: None,
                last_run: None,
                next_run: None,
            }
        }
    }
}

/// Start the scheduler daemon. On Unix, forks and detaches using `daemonize`.
/// On Windows, spawns a detached child process.
/// The caller should exit after this returns Ok (Unix parent exits from `daemonize`).
pub fn start_daemon(config_dir: &Path) -> Result<()> {
    if is_already_running(config_dir) {
        return Err(crate::error::CoreError::Other(
            "Scheduler is already running".into(),
        ));
    }

    #[cfg(unix)]
    {
        use daemonize::Daemonize;

        let pid_path = pid_file_path(config_dir);
        let daemon = Daemonize::new()
            .pid_file(&pid_path)
            .working_directory(config_dir);

        daemon
            .start()
            .map_err(|e| crate::error::CoreError::Other(format!("Failed to daemonize: {e}")))?;

        // We are now the daemon child process
        run_foreground(config_dir)
    }

    #[cfg(windows)]
    {
        let exe = std::env::current_exe()
            .map_err(|e| crate::error::CoreError::Other(format!("Cannot find executable: {e}")))?;

        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        const DETACHED_PROCESS: u32 = 0x00000008;

        std::process::Command::new(&exe)
            .args(["scheduler", "run-daemon"])
            .creation_flags(CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| crate::error::CoreError::Other(format!("Failed to spawn daemon: {e}")))?;

        std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(())
    }
}

/// Run the scheduler loop in the foreground (daemon child on Unix, `run-daemon` on Windows).
pub fn run_foreground(config_dir: &Path) -> Result<()> {
    write_pid_file(config_dir)?;

    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    super::runner::run_loop(config_dir, 30, stop);
    let _ = remove_pid_file(config_dir);
    Ok(())
}

/// Stop a running scheduler by signaling the process.
pub fn stop(config_dir: &Path) -> Result<bool> {
    let path = pid_file_path(config_dir);
    if !path.exists() {
        return Ok(false);
    }

    let content = match read_pid(&path) {
        Some(c) => c,
        None => {
            let _ = std::fs::remove_file(&path);
            return Ok(false);
        }
    };

    if !is_process_alive(content.pid) {
        let _ = std::fs::remove_file(&path);
        return Ok(false);
    }

    terminate_process(content.pid);
    let _ = std::fs::remove_file(&path);
    Ok(true)
}

#[cfg(windows)]
fn terminate_process(pid: u32) {
    extern "system" {
        fn OpenProcess(desired_access: u32, inherit_handle: i32, process_id: u32) -> isize;
        fn TerminateProcess(handle: isize, exit_code: u32) -> i32;
        fn CloseHandle(handle: isize) -> i32;
    }
    const PROCESS_TERMINATE: u32 = 0x0001;
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle != 0 {
            TerminateProcess(handle, 1);
            CloseHandle(handle);
        }
    }
}

#[cfg(not(windows))]
fn terminate_process(pid: u32) {
    let _ = std::process::Command::new("kill")
        .arg(pid.to_string())
        .status();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_read_pid_file() {
        let dir = tempfile::tempdir().unwrap();
        write_pid_file(dir.path()).unwrap();
        let path = pid_file_path(dir.path());
        assert!(path.exists());
        let content = read_pid(&path).unwrap();
        assert_eq!(content.pid, std::process::id());
    }

    #[test]
    fn remove_pid_file_cleans_up() {
        let dir = tempfile::tempdir().unwrap();
        write_pid_file(dir.path()).unwrap();
        remove_pid_file(dir.path()).unwrap();
        assert!(!pid_file_path(dir.path()).exists());
    }

    #[test]
    fn status_returns_not_running_without_pid_file() {
        let dir = tempfile::tempdir().unwrap();
        let s = status(dir.path());
        assert!(!s.running);
        assert!(s.pid.is_none());
    }

    #[test]
    fn stale_pid_file_detected() {
        let dir = tempfile::tempdir().unwrap();
        let stale = PidContent {
            pid: u32::MAX - 1,
            started_at: "2020-01-01T00:00:00Z".into(),
        };
        let path = pid_file_path(dir.path());
        std::fs::write(&path, serde_json::to_vec(&stale).unwrap()).unwrap();

        let s = status(dir.path());
        assert!(!s.running);
        assert!(!path.exists());
    }

    #[test]
    fn stop_returns_false_without_pid_file() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!stop(dir.path()).unwrap());
    }
}
