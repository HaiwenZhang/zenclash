use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum DnsError {
    #[error("Failed to set DNS: {0}")]
    SetFailed(String),

    #[error("Failed to restore DNS: {0}")]
    RestoreFailed(String),

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
struct DnsBackup {
    original_dns: Vec<String>,
    interface: String,
}

pub struct DnsManager {
    backup: Arc<RwLock<Option<DnsBackup>>>,
    primary_dns: String,
    secondary_dns: String,
}

impl DnsManager {
    pub fn new() -> Self {
        Self {
            backup: Arc::new(RwLock::new(None)),
            primary_dns: "223.5.5.5".into(),
            secondary_dns: "119.29.29.29".into(),
        }
    }

    pub fn with_dns(mut self, primary: String, secondary: String) -> Self {
        self.primary_dns = primary;
        self.secondary_dns = secondary;
        self
    }

    pub async fn set_dns(&self) -> Result<(), DnsError> {
        #[cfg(target_os = "macos")]
        {
            self.set_dns_macos().await
        }

        #[cfg(target_os = "windows")]
        {
            self.set_dns_windows().await
        }

        #[cfg(target_os = "linux")]
        {
            self.set_dns_linux().await
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err(DnsError::UnsupportedPlatform(
                "Current platform is not supported".into(),
            ))
        }
    }

    pub async fn restore_dns(&self) -> Result<(), DnsError> {
        #[cfg(target_os = "macos")]
        {
            self.restore_dns_macos().await
        }

        #[cfg(target_os = "windows")]
        {
            self.restore_dns_windows().await
        }

        #[cfg(target_os = "linux")]
        {
            self.restore_dns_linux().await
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Err(DnsError::UnsupportedPlatform(
                "Current platform is not supported".into(),
            ))
        }
    }

    #[cfg(target_os = "macos")]
    async fn set_dns_macos(&self) -> Result<(), DnsError> {
        let interface = self.get_primary_interface_macos()?;

        let current_dns = self.get_current_dns_macos(&interface)?;

        {
            let mut backup = self.backup.write().await;
            *backup = Some(DnsBackup {
                original_dns: current_dns,
                interface: interface.clone(),
            });
        }

        let script = format!(
            r#"do shell script "networksetup -setdnsservers '{}' {} {}" with administrator privileges"#,
            interface, self.primary_dns, self.secondary_dns
        );

        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| DnsError::SetFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DnsError::SetFailed(stderr.to_string()));
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn restore_dns_macos(&self) -> Result<(), DnsError> {
        let backup = {
            let b = self.backup.read().await;
            b.clone()
        };

        if let Some(backup) = backup {
            let dns_args = if backup.original_dns.is_empty() {
                "empty".to_string()
            } else {
                backup.original_dns.join(" ")
            };

            let script = format!(
                r#"do shell script "networksetup -setdnsservers '{}' {}" with administrator privileges"#,
                backup.interface, dns_args
            );

            let output = std::process::Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output()
                .map_err(|e| DnsError::RestoreFailed(e.to_string()))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(DnsError::RestoreFailed(stderr.to_string()));
            }

            let mut b = self.backup.write().await;
            *b = None;
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn get_primary_interface_macos(&self) -> Result<String, DnsError> {
        let output = std::process::Command::new("networksetup")
            .args(["-listallnetworkservices"])
            .output()
            .map_err(|e| DnsError::SetFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines().skip(1) {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('*') {
                if trimmed.contains("Wi-Fi") || trimmed.contains("Ethernet") {
                    return Ok(trimmed.to_string());
                }
            }
        }

        for line in stdout.lines().skip(1) {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('*') {
                return Ok(trimmed.to_string());
            }
        }

        Err(DnsError::SetFailed("No network interface found".into()))
    }

    #[cfg(target_os = "macos")]
    fn get_current_dns_macos(&self, interface: &str) -> Result<Vec<String>, DnsError> {
        let output = std::process::Command::new("networksetup")
            .args(["-getdnsservers", interface])
            .output()
            .map_err(|e| DnsError::SetFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let dns_servers: Vec<String> = stdout
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty()
                    && trimmed != "There aren't any DNS servers set on Wi-Fi."
                    && trimmed != "There aren't any DNS servers set on Ethernet."
            })
            .map(|s| s.trim().to_string())
            .collect();

        Ok(dns_servers)
    }

    #[cfg(target_os = "windows")]
    async fn set_dns_windows(&self) -> Result<(), DnsError> {
        let output = std::process::Command::new("netsh")
            .args([
                "interface",
                "ip",
                "set",
                "dns",
                "Ethernet",
                "static",
                &self.primary_dns,
            ])
            .output()
            .map_err(|e| DnsError::SetFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DnsError::SetFailed(stderr.to_string()));
        }

        let _ = std::process::Command::new("netsh")
            .args([
                "interface",
                "ip",
                "add",
                "dns",
                "Ethernet",
                &self.secondary_dns,
                "index=2",
            ])
            .output();

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn restore_dns_windows(&self) -> Result<(), DnsError> {
        let output = std::process::Command::new("netsh")
            .args(["interface", "ip", "set", "dns", "Ethernet", "dhcp"])
            .output()
            .map_err(|e| DnsError::RestoreFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DnsError::RestoreFailed(stderr.to_string()));
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn set_dns_linux(&self) -> Result<(), DnsError> {
        Err(DnsError::UnsupportedPlatform(
            "Linux DNS management requires NetworkManager or systemd-resolved".into(),
        ))
    }

    #[cfg(target_os = "linux")]
    async fn restore_dns_linux(&self) -> Result<(), DnsError> {
        Err(DnsError::UnsupportedPlatform(
            "Linux DNS management requires NetworkManager or systemd-resolved".into(),
        ))
    }

    pub async fn is_configured(&self) -> bool {
        self.backup.read().await.is_some()
    }
}

impl Default for DnsManager {
    fn default() -> Self {
        Self::new()
    }
}
