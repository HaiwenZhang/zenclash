use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Debug, thiserror::Error)]
pub enum SsidError {
    #[error("Failed to get SSID: {0}")]
    GetFailed(String),

    #[error("Platform not supported")]
    UnsupportedPlatform,
}

pub struct SsidMonitor {
    current_ssid: Arc<RwLock<Option<String>>>,
    pause_ssids: Arc<RwLock<Vec<String>>>,
    running: Arc<RwLock<bool>>,
}

impl SsidMonitor {
    pub fn new() -> Self {
        Self {
            current_ssid: Arc::new(RwLock::new(None)),
            pause_ssids: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn get_current_ssid() -> Result<Option<String>, SsidError> {
        #[cfg(target_os = "windows")]
        {
            Self::get_ssid_windows()
        }

        #[cfg(target_os = "macos")]
        {
            Self::get_ssid_macos()
        }

        #[cfg(target_os = "linux")]
        {
            Self::get_ssid_linux()
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(SsidError::UnsupportedPlatform)
        }
    }

    #[cfg(target_os = "windows")]
    fn get_ssid_windows() -> Result<Option<String>, SsidError> {
        let output = std::process::Command::new("netsh")
            .args(["wlan", "show", "interfaces"])
            .output()
            .map_err(|e| SsidError::GetFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.trim().starts_with("SSID") {
                if let Some(pos) = line.find(':') {
                    return Ok(Some(line[pos + 1..].trim().to_string()));
                }
            }
        }

        Ok(None)
    }

    #[cfg(target_os = "macos")]
    fn get_ssid_macos() -> Result<Option<String>, SsidError> {
        let output = std::process::Command::new(
            "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport"
        )
        .arg("-I")
        .output()
        .map_err(|e| SsidError::GetFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.trim().starts_with("WARNING") {
            return Self::get_ssid_macos_fallback();
        }

        for line in stdout.lines() {
            if line.trim().starts_with("SSID") {
                if let Some(pos) = line.find(':') {
                    return Ok(Some(line[pos + 1..].trim().to_string()));
                }
            }
        }

        Ok(None)
    }

    #[cfg(target_os = "macos")]
    fn get_ssid_macos_fallback() -> Result<Option<String>, SsidError> {
        let output = std::process::Command::new("networksetup")
            .args(["-listpreferredwirelessnetworks", "en0"])
            .output()
            .map_err(|e| SsidError::GetFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.len() > 1 {
            Ok(Some(lines[1].trim().to_string()))
        } else {
            Ok(None)
        }
    }

    #[cfg(target_os = "linux")]
    fn get_ssid_linux() -> Result<Option<String>, SsidError> {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg("iwconfig 2>/dev/null | grep 'ESSID' | awk -F'\"' '{print $2}'")
            .output()
            .map_err(|e| SsidError::GetFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let ssid = stdout.trim();

        if ssid.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ssid.to_string()))
        }
    }

    pub async fn set_pause_ssids(&self, ssids: Vec<String>) {
        let mut pause = self.pause_ssids.write().await;
        *pause = ssids;
    }

    pub async fn get_current(&self) -> Option<String> {
        self.current_ssid.read().await.clone()
    }

    pub async fn check_and_update(&self) -> Option<bool> {
        let ssid = Self::get_current_ssid().await.ok().flatten();
        let mut current = self.current_ssid.write().await;

        if ssid.as_ref() == current.as_ref() {
            return None;
        }

        let should_pause = if let Some(ref s) = ssid {
            let pause = self.pause_ssids.read().await;
            pause.contains(s)
        } else {
            false
        };

        *current = ssid;
        Some(should_pause)
    }

    pub async fn start_monitoring<F>(&self, mut on_change: F)
    where
        F: FnMut(bool) + Send + 'static,
    {
        let mut running = self.running.write().await;
        *running = true;
        drop(running);

        let current_ssid = self.current_ssid.clone();
        let pause_ssids = self.pause_ssids.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            loop {
                interval.tick().await;

                if !*running.read().await {
                    break;
                }

                if let Ok(Some(ssid)) = Self::get_current_ssid().await {
                    let mut current = current_ssid.write().await;

                    if Some(&ssid) != current.as_ref() {
                        let pause = pause_ssids.read().await;
                        let should_pause = pause.contains(&ssid);
                        *current = Some(ssid);
                        drop(current);
                        drop(pause);

                        on_change(should_pause);
                    }
                }
            }
        });
    }

    pub async fn stop_monitoring(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }
}

impl Default for SsidMonitor {
    fn default() -> Self {
        Self::new()
    }
}
