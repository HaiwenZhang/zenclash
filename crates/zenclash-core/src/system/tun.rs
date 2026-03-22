use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::process::Command;
use thiserror::Error;
use tracing::{debug, info, warn};

#[derive(Debug, Error)]
pub enum TunError {
    #[error("TUN device not supported on this platform")]
    NotSupported,

    #[error("Failed to create TUN device: {0}")]
    CreationFailed(String),

    #[error("Failed to configure TUN device: {0}")]
    ConfigurationFailed(String),

    #[error("Failed to destroy TUN device: {0}")]
    DestructionFailed(String),

    #[error("TUN device already exists: {0}")]
    AlreadyExists(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunConfig {
    pub name: String,
    pub address: IpAddr,
    pub netmask: String,
    pub mtu: u16,
    pub gateway: Option<IpAddr>,
    pub dns: Vec<IpAddr>,
}

impl Default for TunConfig {
    fn default() -> Self {
        Self {
            name: "utun0".to_string(),
            address: "198.18.0.1".parse().unwrap(),
            netmask: "255.255.255.0".to_string(),
            mtu: 1500,
            gateway: None,
            dns: vec!["198.18.0.2".parse().unwrap()],
        }
    }
}

impl TunConfig {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn with_address(mut self, addr: IpAddr) -> Self {
        self.address = addr;
        self
    }

    pub fn with_netmask(mut self, mask: impl Into<String>) -> Self {
        self.netmask = mask.into();
        self
    }

    pub fn with_mtu(mut self, mtu: u16) -> Self {
        self.mtu = mtu;
        self
    }

    pub fn with_gateway(mut self, gateway: IpAddr) -> Self {
        self.gateway = Some(gateway);
        self
    }

    pub fn with_dns(mut self, dns: Vec<IpAddr>) -> Self {
        self.dns = dns;
        self
    }
}

#[derive(Debug, Clone)]
pub struct TunDevice {
    pub name: String,
    pub fd: Option<i32>,
    pub config: TunConfig,
}

impl TunDevice {
    fn new(name: String, config: TunConfig) -> Self {
        Self {
            name,
            fd: None,
            config,
        }
    }
}

pub struct TunManager {
    current_device: parking_lot::RwLock<Option<TunDevice>>,
}

impl TunManager {
    pub fn new() -> Self {
        Self {
            current_device: parking_lot::RwLock::new(None),
        }
    }

    pub fn is_supported(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.is_supported_macos()
        }

        #[cfg(target_os = "linux")]
        {
            self.is_supported_linux()
        }

        #[cfg(target_os = "windows")]
        {
            self.is_supported_windows()
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            false
        }
    }

    pub async fn create_tun(&self, config: TunConfig) -> Result<TunDevice> {
        info!("Creating TUN device: {}", config.name);

        if !self.is_supported() {
            return Err(TunError::NotSupported.into());
        }

        {
            let current = self.current_device.read();
            if current.is_some() {
                return Err(TunError::AlreadyExists(config.name).into());
            }
        }

        #[cfg(target_os = "macos")]
        {
            self.create_tun_macos(config).await
        }

        #[cfg(target_os = "linux")]
        {
            self.create_tun_linux(config).await
        }

        #[cfg(target_os = "windows")]
        {
            self.create_tun_windows(config).await
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(TunError::NotSupported.into())
        }
    }

    pub async fn destroy_tun(&self) -> Result<()> {
        let device_name = {
            let current = self.current_device.read();
            if let Some(device) = current.as_ref() {
                device.name.clone()
            } else {
                return Ok(());
            }
        };

        info!("Destroying TUN device: {}", device_name);

        #[cfg(target_os = "macos")]
        {
            self.destroy_tun_macos(&device_name).await?;
        }

        #[cfg(target_os = "linux")]
        {
            self.destroy_tun_linux(&device_name).await?;
        }

        #[cfg(target_os = "windows")]
        {
            self.destroy_tun_windows(&device_name).await?;
        }

        let mut current = self.current_device.write();
        *current = None;

        Ok(())
    }

    pub fn get_device(&self) -> Option<TunDevice> {
        self.current_device.read().clone()
    }

    #[cfg(target_os = "macos")]
    fn is_supported_macos(&self) -> bool {
        Command::new("ifconfig").output().is_ok()
    }

    #[cfg(target_os = "macos")]
    async fn create_tun_macos(&self, config: TunConfig) -> Result<TunDevice> {
        let name = config.name.clone();
        let addr = config.address.to_string();
        let netmask = config.netmask.clone();
        let mtu = config.mtu.to_string();
        let device_name = config.name.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let utun_name = if name.starts_with("utun") {
                name.clone()
            } else {
                format!(
                    "utun{}",
                    name.trim_start_matches("utun").parse::<u32>().unwrap_or(0)
                )
            };

            let check = Command::new("ifconfig").arg(&utun_name).output();

            if check.is_ok() && check?.status.success() {
                return Err(TunError::AlreadyExists(utun_name).into());
            }

            let output = Command::new("ifconfig")
                .args([&utun_name, &addr, &addr, "netmask", &netmask])
                .output()
                .map_err(|e| TunError::CreationFailed(e.to_string()))?;

            if !output.status.success() {
                return Err(TunError::CreationFailed(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                )
                .into());
            }

            let output = Command::new("ifconfig")
                .args([&utun_name, "mtu", &mtu])
                .output()
                .map_err(|e| TunError::ConfigurationFailed(e.to_string()))?;

            if !output.status.success() {
                warn!(
                    "Failed to set MTU: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            let output = Command::new("ifconfig")
                .args([&utun_name, "up"])
                .output()
                .map_err(|e| TunError::ConfigurationFailed(e.to_string()))?;

            if !output.status.success() {
                return Err(TunError::ConfigurationFailed(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                )
                .into());
            }

            Ok(())
        })
        .await??;

        let device = TunDevice::new(device_name, config);
        *self.current_device.write() = Some(device.clone());

        Ok(device)
    }

    #[cfg(target_os = "macos")]
    async fn destroy_tun_macos(&self, name: &str) -> Result<()> {
        let name = name.to_string();
        tokio::task::spawn_blocking(move || {
            let output = Command::new("ifconfig")
                .args([&name, "down"])
                .output()
                .map_err(|e| TunError::DestructionFailed(e.to_string()))?;

            if !output.status.success() {
                warn!(
                    "Failed to bring down TUN device: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            debug!("TUN device {} destroyed", name);
            Ok::<_, TunError>(())
        })
        .await??;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn is_supported_linux(&self) -> bool {
        std::path::Path::new("/dev/net/tun").exists()
    }

    #[cfg(target_os = "linux")]
    async fn create_tun_linux(&self, config: TunConfig) -> Result<TunDevice> {
        use libc::{c_char, c_short, IFF_NO_PI, IFF_TUN, TUNSETIFF};
        use std::fs::OpenOptions;
        use std::os::unix::io::AsRawFd;
        use std::ptr;

        let name = config.name.clone();
        let addr = config.address.to_string();
        let netmask = config.netmask.clone();
        let mtu = config.mtu.to_string();

        let result = tokio::task::spawn_blocking(move || -> Result<TunDevice> {
            let tun_file = OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/net/tun")
                .map_err(|e| TunError::CreationFailed(e.to_string()))?;

            let fd = tun_file.as_raw_fd();

            #[repr(C)]
            struct IfReq {
                name: [c_char; 16],
                flags: c_short,
            }

            let mut ifr = IfReq {
                name: [0; 16],
                flags: (IFF_TUN | IFF_NO_PI) as c_short,
            };

            for (i, c) in name.bytes().take(15).enumerate() {
                ifr.name[i] = c as c_char;
            }

            unsafe {
                if libc::ioctl(fd, TUNSETIFF, &mut ifr) < 0 {
                    return Err(
                        TunError::CreationFailed("ioctl TUNSETIFF failed".to_string()).into(),
                    );
                }
            }

            std::mem::forget(tun_file);

            let output = Command::new("ip")
                .args(["addr", "add", &format!("{}/{}", addr, 24), "dev", &name])
                .output()
                .map_err(|e| TunError::ConfigurationFailed(e.to_string()))?;

            if !output.status.success() {
                return Err(TunError::ConfigurationFailed(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                )
                .into());
            }

            let output = Command::new("ip")
                .args(["link", "set", "dev", &name, "mtu", &mtu])
                .output()
                .map_err(|e| TunError::ConfigurationFailed(e.to_string()))?;

            if !output.status.success() {
                warn!(
                    "Failed to set MTU: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            let output = Command::new("ip")
                .args(["link", "set", "dev", &name, "up"])
                .output()
                .map_err(|e| TunError::ConfigurationFailed(e.to_string()))?;

            if !output.status.success() {
                return Err(TunError::ConfigurationFailed(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                )
                .into());
            }

            let mut device = TunDevice::new(name, config);
            device.fd = Some(fd);
            Ok(device)
        })
        .await??;

        *self.current_device.write() = Some(result.clone());
        Ok(result)
    }

    #[cfg(target_os = "linux")]
    async fn destroy_tun_linux(&self, name: &str) -> Result<()> {
        let name = name.to_string();
        tokio::task::spawn_blocking(move || {
            let output = Command::new("ip")
                .args(["link", "set", "dev", &name, "down"])
                .output()
                .map_err(|e| TunError::DestructionFailed(e.to_string()))?;

            if !output.status.success() {
                warn!(
                    "Failed to bring down TUN device: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            debug!("TUN device {} destroyed", name);
            Ok::<_, TunError>(())
        })
        .await??;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn is_supported_windows(&self) -> bool {
        false
    }

    #[cfg(target_os = "windows")]
    async fn create_tun_windows(&self, _config: TunConfig) -> Result<TunDevice> {
        Err(TunError::NotSupported.into())
    }

    #[cfg(target_os = "windows")]
    async fn destroy_tun_windows(&self, _name: &str) -> Result<()> {
        Err(TunError::NotSupported.into())
    }
}

impl Default for TunManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tun_config_default() {
        let config = TunConfig::default();
        assert_eq!(config.name, "utun0");
        assert_eq!(config.mtu, 1500);
    }

    #[test]
    fn test_tun_config_builder() {
        let config = TunConfig::new("mytun")
            .with_address("10.0.0.1".parse().unwrap())
            .with_netmask("255.255.0.0")
            .with_mtu(1400)
            .with_dns(vec!["8.8.8.8".parse().unwrap()]);

        assert_eq!(config.name, "mytun");
        assert_eq!(config.address, "10.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(config.netmask, "255.255.0.0");
        assert_eq!(config.mtu, 1400);
        assert_eq!(config.dns.len(), 1);
    }

    #[test]
    fn test_tun_device_new() {
        let config = TunConfig::new("test0");
        let device = TunDevice::new("test0".to_string(), config.clone());
        assert_eq!(device.name, "test0");
        assert!(device.fd.is_none());
    }

    #[test]
    fn test_tun_manager_new() {
        let manager = TunManager::new();
        assert!(manager.get_device().is_none());
    }

    #[test]
    fn test_tun_error_display() {
        let err = TunError::NotSupported;
        assert!(err.to_string().contains("not supported"));

        let err = TunError::CreationFailed("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }
}
