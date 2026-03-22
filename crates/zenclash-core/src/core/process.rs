use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Process not running")]
    NotRunning,

    #[error("Process already running")]
    AlreadyRunning,

    #[error("Failed to start process: {0}")]
    StartError(String),

    #[error("Failed to stop process: {0}")]
    StopError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct ProcessConfig {
    pub path: PathBuf,
    pub args: Vec<String>,
    pub work_dir: PathBuf,
    pub envs: Vec<(String, String)>,
}

impl ProcessConfig {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            args: vec![],
            work_dir: std::env::current_dir().unwrap_or_default(),
            envs: vec![],
        }
    }

    pub fn args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn work_dir(mut self, dir: PathBuf) -> Self {
        self.work_dir = dir;
        self
    }

    pub fn env(mut self, key: String, value: String) -> Self {
        self.envs.push((key, value));
        self
    }
}

pub struct Process {
    config: ProcessConfig,
    child: Arc<Mutex<Option<Child>>>,
    state: Arc<Mutex<ProcessState>>,
}

impl Process {
    pub fn new(config: ProcessConfig) -> Self {
        Self {
            config,
            child: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(ProcessState::Stopped)),
        }
    }

    pub async fn state(&self) -> ProcessState {
        *self.state.lock().await
    }

    pub async fn is_running(&self) -> bool {
        let child = self.child.lock().await;
        child.is_some()
    }

    pub async fn pid(&self) -> Option<u32> {
        let child = self.child.lock().await;
        child.as_ref().and_then(|c| c.id())
    }

    pub async fn start(&self) -> Result<(), ProcessError> {
        let mut child_guard = self.child.lock().await;
        let mut state_guard = self.state.lock().await;

        if child_guard.is_some() {
            return Err(ProcessError::AlreadyRunning);
        }

        *state_guard = ProcessState::Starting;
        drop(state_guard);

        let mut cmd = Command::new(&self.config.path);
        cmd.args(&self.config.args)
            .current_dir(&self.config.work_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        for (key, value) in &self.config.envs {
            cmd.env(key, value);
        }

        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let mut state = self.state.lock().await;
                *state = ProcessState::Error;
                return Err(ProcessError::StartError(e.to_string()));
            },
        };

        *child_guard = Some(child);

        let mut state = self.state.lock().await;
        *state = ProcessState::Running;

        Ok(())
    }

    pub async fn start_with_output<F>(&self, on_output: F) -> Result<(), ProcessError>
    where
        F: FnMut(&str) + Send + 'static + Clone,
    {
        let mut child_guard = self.child.lock().await;
        let mut state_guard = self.state.lock().await;

        if child_guard.is_some() {
            return Err(ProcessError::AlreadyRunning);
        }

        *state_guard = ProcessState::Starting;
        drop(state_guard);

        let mut cmd = Command::new(&self.config.path);
        cmd.args(&self.config.args)
            .current_dir(&self.config.work_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        for (key, value) in &self.config.envs {
            cmd.env(key, value);
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let mut state = self.state.lock().await;
                *state = ProcessState::Error;
                return Err(ProcessError::StartError(e.to_string()));
            },
        };

        let mut on_output_stdout = on_output.clone();
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    on_output_stdout(&line);
                }
            });
        }

        let mut on_output_stderr = on_output;
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    on_output_stderr(&line);
                }
            });
        }

        *child_guard = Some(child);

        let mut state = self.state.lock().await;
        *state = ProcessState::Running;

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), ProcessError> {
        let mut child_guard = self.child.lock().await;
        let mut state_guard = self.state.lock().await;

        if child_guard.is_none() {
            return Err(ProcessError::NotRunning);
        }

        *state_guard = ProcessState::Stopping;
        drop(state_guard);

        if let Some(mut child) = child_guard.take() {
            if let Err(e) = child.kill().await {
                let mut state = self.state.lock().await;
                *state = ProcessState::Error;
                return Err(ProcessError::StopError(e.to_string()));
            }
        }

        let mut state = self.state.lock().await;
        *state = ProcessState::Stopped;

        Ok(())
    }

    pub async fn restart(&self) -> Result<(), ProcessError> {
        if self.is_running().await {
            self.stop().await?;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        self.start().await
    }

    pub async fn wait(&self) -> Result<Option<std::process::ExitStatus>, ProcessError> {
        let mut child_guard = self.child.lock().await;

        if let Some(child) = child_guard.as_mut() {
            let status = child.wait().await?;
            let mut state = self.state.lock().await;
            *state = ProcessState::Stopped;
            *child_guard = None;
            return Ok(Some(status));
        }

        Ok(None)
    }
}

pub fn find_process_by_name(name: &str) -> Option<u32> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("pgrep").arg("-x").arg(name).output().ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .next()
                .and_then(|s| s.trim().parse::<u32>().ok())
        } else {
            None
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let output = Command::new("pgrep").arg("-x").arg(name).output().ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .next()
                .and_then(|s| s.trim().parse::<u32>().ok())
        } else {
            None
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("tasklist")
            .args([
                "/FI",
                &format!("IMAGENAME eq {}", name),
                "/FO",
                "CSV",
                "/NH",
            ])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains(name) {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    let pid = parts[1].trim_matches('"').parse::<u32>().ok()?;
                    return Some(pid);
                }
            }
        }
        None
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

pub fn kill_process(pid: u32) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill").arg(pid.to_string()).status()?;
        Ok(())
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_config() {
        let config = ProcessConfig::new(PathBuf::from("/usr/bin/echo"))
            .args(vec!["hello".to_string()])
            .env("KEY".to_string(), "value".to_string());

        assert_eq!(config.path, PathBuf::from("/usr/bin/echo"));
        assert_eq!(config.args, vec!["hello"]);
        assert_eq!(config.envs.len(), 1);
    }

    #[tokio::test]
    async fn test_process_state_initial() {
        let config = ProcessConfig::new(PathBuf::from("/bin/echo"));
        let process = Process::new(config);
        assert_eq!(process.state().await, ProcessState::Stopped);
        assert!(!process.is_running().await);
    }
}
