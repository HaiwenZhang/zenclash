use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::process::Command;
use thiserror::Error;
use tracing::{debug, info, warn};

#[derive(Debug, Error)]
pub enum DnsError {
    #[error("Failed to set DNS: {0}")]
    SetFailed(String),

    #[error("Failed to get DNS: {0}")]
    GetFailed(String),

    #[error("Failed to restore DNS: {0}")]
    RestoreFailed(String),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("No network service found")]
    NoNetworkService,

    #[error("Invalid DNS server: {0}")]
    InvalidServer(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub servers: Vec<String>,
    pub interface: Option<String>,
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            interface: None,
        }
    }
}

impl DnsConfig {
    pub fn new(servers: Vec<String>) -> Self {
        Self {
            servers,
            interface: None,
        }
    }

    pub fn with_interface(mut self, interface: impl Into<String>) -> Self {
        self.interface = Some(interface.into());
        self
    }

    pub fn validate(&self) -> Result<()> {
        for server in &self.servers {
            if server.parse::<IpAddr>().is_err() {
                return Err(DnsError::InvalidServer(server.clone()).into());
            }
        }
        Ok(())
    }
}

pub struct DnsManager {
    #[cfg(target_os = "macos")]
    network_services: Vec<String>,
    original_dns: parking_lot::RwLock<Vec<String>>,
}

impl DnsManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            #[cfg(target_os = "macos")]
            network_services: Self::get_network_services()?,
            original_dns: parking_lot::RwLock::new(Vec::new()),
        })
    }

    #[cfg(target_os = "macos")]
    fn get_network_services() -> Result<Vec<String>> {
        let output = Command::new("networksetup")
            .args(["-listallnetworkservices"])
            .output()?;

        if !output.status.success() {
            return Err(
                DnsError::GetFailed(String::from_utf8_lossy(&output.stderr).to_string()).into(),
            );
        }

        let services: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(1)
            .filter(|line| !line.is_empty() && !line.starts_with('*'))
            .map(|s| s.to_string())
            .collect();

        if services.is_empty() {
            return Err(DnsError::NoNetworkService.into());
        }

        debug!("Found network services: {:?}", services);
        Ok(services)
    }

    pub async fn set_dns(&self, servers: Vec<String>) -> Result<()> {
        info!("Setting DNS servers: {:?}", servers);

        let config = DnsConfig::new(servers);
        config.validate()?;

        #[cfg(target_os = "macos")]
        {
            self.set_dns_macos(&config).await
        }

        #[cfg(target_os = "linux")]
        {
            self.set_dns_linux(&config).await
        }

        #[cfg(target_os = "windows")]
        {
            self.set_dns_windows(&config).await
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(DnsError::PlatformNotSupported(std::env::consts::OS.to_string()).into())
        }
    }

    pub async fn restore_dns(&self) -> Result<()> {
        info!("Restoring original DNS settings");

        let original = {
            let original = self.original_dns.read();
            if original.is_empty() {
                return Ok(());
            }
            original.clone()
        };

        #[cfg(target_os = "macos")]
        {
            self.restore_dns_macos(&original).await
        }

        #[cfg(target_os = "linux")]
        {
            self.restore_dns_linux(&original).await
        }

        #[cfg(target_os = "windows")]
        {
            self.restore_dns_windows(&original).await
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(DnsError::PlatformNotSupported(std::env::consts::OS.to_string()).into())
        }
    }

    pub async fn get_current_dns(&self) -> Result<Vec<String>> {
        #[cfg(target_os = "macos")]
        {
            self.get_dns_macos().await
        }

        #[cfg(target_os = "linux")]
        {
            self.get_dns_linux().await
        }

        #[cfg(target_os = "windows")]
        {
            self.get_dns_windows().await
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(DnsError::PlatformNotSupported(std::env::consts::OS.to_string()).into())
        }
    }

    #[cfg(target_os = "macos")]
    async fn set_dns_macos(&self, config: &DnsConfig) -> Result<()> {
        let servers = config.servers.clone();
        let services = self.network_services.clone();

        let current = self.get_current_dns().await?;
        *self.original_dns.write() = current;

        for service in services {
            let servers_clone = servers.clone();
            let service_clone = service.clone();

            let output = tokio::task::spawn_blocking(move || {
                let mut args = vec!["-setdnsservers", &service_clone];
                args.extend(servers_clone.iter().map(|s| s.as_str()));

                Command::new("networksetup").args(&args).output()
            })
            .await??;

            if !output.status.success() {
                warn!(
                    "Failed to set DNS for service {}: {}",
                    service,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn restore_dns_macos(&self, original: &[String]) -> Result<()> {
        let services = self.network_services.clone();
        let original = original.to_vec();

        for service in services {
            let original_clone = original.clone();
            let service_clone = service.clone();

            let output = tokio::task::spawn_blocking(move || {
                let mut args = vec!["-setdnsservers", &service_clone];
                if original_clone.is_empty() {
                    args.push("empty");
                } else {
                    args.extend(original_clone.iter().map(|s| s.as_str()));
                }

                Command::new("networksetup").args(&args).output()
            })
            .await??;

            if !output.status.success() {
                warn!(
                    "Failed to restore DNS for service {}: {}",
                    service,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        self.original_dns.write().clear();
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn get_dns_macos(&self) -> Result<Vec<String>> {
        if self.network_services.is_empty() {
            return Ok(Vec::new());
        }

        let service = self.network_services.first().unwrap().clone();

        let output = tokio::task::spawn_blocking(move || {
            Command::new("networksetup")
                .args(["-getdnsservers", &service])
                .output()
        })
        .await??;

        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.contains("There aren't any DNS servers") {
            return Ok(Vec::new());
        }

        let servers: Vec<String> = stdout
            .lines()
            .filter(|line| line.parse::<IpAddr>().is_ok())
            .map(|s| s.to_string())
            .collect();

        Ok(servers)
    }

    #[cfg(target_os = "linux")]
    async fn set_dns_linux(&self, config: &DnsConfig) -> Result<()> {
        let servers = config.servers.clone();

        let current = self.get_current_dns().await?;
        *self.original_dns.write() = current;

        tokio::task::spawn_blocking(move || {
            let is_gnome = Command::new("gsettings")
                .args(["get", "org.gnome.system.proxy", "mode"])
                .output()
                .is_ok();

            if is_gnome {
                for (i, server) in servers.iter().enumerate() {
                    Command::new("gsettings")
                        .args([
                            "set",
                            "org.gnome.system.dns",
                            &format!("servers[{}]", i),
                            server,
                        ])
                        .status()?;
                }
            } else {
                let resolv_conf = "/etc/resolv.conf";
                let content = servers
                    .iter()
                    .map(|s| format!("nameserver {}", s))
                    .collect::<Vec<_>>()
                    .join("\n");

                std::fs::write(resolv_conf, content)?;
            }

            Ok::<_, std::io::Error>(())
        })
        .await??;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn restore_dns_linux(&self, original: &[String]) -> Result<()> {
        let original = original.to_vec();

        tokio::task::spawn_blocking(move || {
            let is_gnome = Command::new("gsettings")
                .args(["get", "org.gnome.system.proxy", "mode"])
                .output()
                .is_ok();

            if is_gnome {
                Command::new("gsettings")
                    .args(["set", "org.gnome.system.dns", "servers", "[]"])
                    .status()?;
            } else {
                let resolv_conf = "/etc/resolv.conf";
                let content = original
                    .iter()
                    .map(|s| format!("nameserver {}", s))
                    .collect::<Vec<_>>()
                    .join("\n");

                std::fs::write(resolv_conf, content)?;
            }

            Ok::<_, std::io::Error>(())
        })
        .await??;

        self.original_dns.write().clear();
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn get_dns_linux(&self) -> Result<Vec<String>> {
        tokio::task::spawn_blocking(|| {
            let resolv_conf = std::fs::read_to_string("/etc/resolv.conf")?;

            let servers: Vec<String> = resolv_conf
                .lines()
                .filter(|line| line.starts_with("nameserver"))
                .filter_map(|line| line.split_whitespace().nth(1))
                .map(|s| s.to_string())
                .collect();

            Ok(servers)
        })
        .await?
    }

    #[cfg(target_os = "windows")]
    async fn set_dns_windows(&self, config: &DnsConfig) -> Result<()> {
        use winreg::enums::*;
        use winreg::RegKey;

        let servers = config.servers.clone();

        let current = self.get_current_dns().await?;
        *self.original_dns.write() = current;

        tokio::task::spawn_blocking(move || {
            let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
            let interfaces = hkcu.open_subkey(
                "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces",
            )?;

            for iface in interfaces.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(key) = interfaces.open_subkey_with_flags(&iface, KEY_WRITE) {
                    let dhcp: u32 = key.get_value("EnableDHCP").unwrap_or(1);

                    if dhcp == 1 {
                        let _: Result<(), _> = key.set_value("NameServer", &servers.join(","));
                    }
                }
            }

            Ok::<_, std::io::Error>(())
        })
        .await??;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn restore_dns_windows(&self, original: &[String]) -> Result<()> {
        use winreg::enums::*;
        use winreg::RegKey;

        let original = original.to_vec();

        tokio::task::spawn_blocking(move || {
            let hkcu = RegKey::predef(HKEY_LOCAL_MACHINE);
            let interfaces = hkcu.open_subkey(
                "SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\\Interfaces",
            )?;

            for iface in interfaces.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(key) = interfaces.open_subkey_with_flags(&iface, KEY_WRITE) {
                    let _: Result<(), _> = key.set_value("NameServer", "");
                }
            }

            Ok::<_, std::io::Error>(())
        })
        .await??;

        self.original_dns.write().clear();
        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn get_dns_windows(&self) -> Result<Vec<String>> {
        use winreg::enums::*;
        use winreg::RegKey;

        tokio::task::spawn_blocking(|| {
            let output = Command::new("powershell")
                .args(["-Command", "Get-DnsClientServerAddress -AddressFamily IPv4 | Select-Object -ExpandProperty ServerAddresses | Select-Object -First 2"])
                .output()?;

            if !output.status.success() {
                return Ok(Vec::new());
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let servers: Vec<String> = stdout
                .lines()
                .filter(|line| line.parse::<IpAddr>().is_ok())
                .map(|s| s.trim().to_string())
                .collect();

            Ok(servers)
        })
        .await?
    }
}

impl Default for DnsManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            #[cfg(target_os = "macos")]
            network_services: vec![],
            original_dns: parking_lot::RwLock::new(Vec::new()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_config_default() {
        let config = DnsConfig::default();
        assert_eq!(config.servers.len(), 2);
        assert!(config.interface.is_none());
    }

    #[test]
    fn test_dns_config_new() {
        let config = DnsConfig::new(vec!["1.1.1.1".to_string(), "1.0.0.1".to_string()]);
        assert_eq!(config.servers.len(), 2);
        assert!(config.interface.is_none());
    }

    #[test]
    fn test_dns_config_with_interface() {
        let config = DnsConfig::new(vec!["1.1.1.1".to_string()]).with_interface("Wi-Fi");
        assert_eq!(config.interface, Some("Wi-Fi".to_string()));
    }

    #[test]
    fn test_dns_config_validate() {
        let config = DnsConfig::new(vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()]);
        assert!(config.validate().is_ok());

        let config = DnsConfig::new(vec!["invalid".to_string()]);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_dns_error_display() {
        let err = DnsError::SetFailed("test error".to_string());
        assert!(err.to_string().contains("test error"));

        let err = DnsError::InvalidServer("bad-dns".to_string());
        assert!(err.to_string().contains("bad-dns"));
    }

    #[test]
    fn test_dns_manager_new() {
        let manager = DnsManager::new();
        assert!(manager.is_ok());
    }
}
