use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;
use thiserror::Error;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ProxyType {
    #[default]
    Http,
    Https,
    Socks,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub enabled: bool,
}

impl ProxyConfig {
    pub fn new(proxy_type: ProxyType, host: impl Into<String>, port: u16) -> Self {
        Self {
            proxy_type,
            host: host.into(),
            port,
            enabled: true,
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Error)]
pub enum SysProxyError {
    #[error("Failed to execute command: {0}")]
    CommandFailed(#[from] std::io::Error),

    #[error("Command returned non-zero exit code: {0}")]
    ExitCode(String),

    #[error("Failed to parse output: {0}")]
    ParseError(String),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("No network service found")]
    NoNetworkService,

    #[error("Proxy not configured")]
    NotConfigured,
}

pub struct SysProxyManager {
    #[cfg(target_os = "macos")]
    network_services: Vec<String>,
    #[cfg(target_os = "linux")]
    original_proxy: Option<ProxyConfig>,
}

impl SysProxyManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            #[cfg(target_os = "macos")]
            network_services: Self::get_network_services()?,
            #[cfg(target_os = "linux")]
            original_proxy: None,
        })
    }

    #[cfg(target_os = "macos")]
    fn get_network_services() -> Result<Vec<String>> {
        let output = Command::new("networksetup")
            .args(["-listallnetworkservices"])
            .output()?;

        if !output.status.success() {
            return Err(SysProxyError::ExitCode(
                String::from_utf8_lossy(&output.stderr).to_string(),
            )
            .into());
        }

        let services: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(1)
            .filter(|line| !line.is_empty() && !line.starts_with('*'))
            .map(|s| s.to_string())
            .collect();

        if services.is_empty() {
            return Err(SysProxyError::NoNetworkService.into());
        }

        debug!("Found network services: {:?}", services);
        Ok(services)
    }

    pub async fn enable_http_proxy(&self, host: &str, port: u16) -> Result<()> {
        info!("Enabling HTTP proxy: {}:{}", host, port);

        #[cfg(target_os = "macos")]
        {
            self.enable_proxy_macos("webproxy", host, port).await
        }

        #[cfg(target_os = "linux")]
        {
            self.enable_proxy_linux("http", host, port).await
        }

        #[cfg(target_os = "windows")]
        {
            self.enable_proxy_windows("http", host, port).await
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(SysProxyError::PlatformNotSupported(std::env::consts::OS.to_string()).into())
        }
    }

    pub async fn enable_https_proxy(&self, host: &str, port: u16) -> Result<()> {
        info!("Enabling HTTPS proxy: {}:{}", host, port);

        #[cfg(target_os = "macos")]
        {
            self.enable_proxy_macos("securewebproxy", host, port).await
        }

        #[cfg(target_os = "linux")]
        {
            self.enable_proxy_linux("https", host, port).await
        }

        #[cfg(target_os = "windows")]
        {
            self.enable_proxy_windows("https", host, port).await
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(SysProxyError::PlatformNotSupported(std::env::consts::OS.to_string()).into())
        }
    }

    pub async fn enable_socks_proxy(&self, host: &str, port: u16) -> Result<()> {
        info!("Enabling SOCKS proxy: {}:{}", host, port);

        #[cfg(target_os = "macos")]
        {
            self.enable_proxy_macos("socksfirewallproxy", host, port)
                .await
        }

        #[cfg(target_os = "linux")]
        {
            self.enable_proxy_linux("socks", host, port).await
        }

        #[cfg(target_os = "windows")]
        {
            self.enable_proxy_windows("socks", host, port).await
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(SysProxyError::PlatformNotSupported(std::env::consts::OS.to_string()).into())
        }
    }

    pub async fn disable_proxy(&self) -> Result<()> {
        info!("Disabling all system proxies");

        #[cfg(target_os = "macos")]
        {
            self.disable_proxy_macos().await
        }

        #[cfg(target_os = "linux")]
        {
            self.disable_proxy_linux().await
        }

        #[cfg(target_os = "windows")]
        {
            self.disable_proxy_windows().await
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(SysProxyError::PlatformNotSupported(std::env::consts::OS.to_string()).into())
        }
    }

    pub async fn get_current_proxy(&self) -> Result<Option<ProxyConfig>> {
        #[cfg(target_os = "macos")]
        {
            self.get_proxy_macos().await
        }

        #[cfg(target_os = "linux")]
        {
            self.get_proxy_linux().await
        }

        #[cfg(target_os = "windows")]
        {
            self.get_proxy_windows().await
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(SysProxyError::PlatformNotSupported(std::env::consts::OS.to_string()).into())
        }
    }

    #[cfg(target_os = "macos")]
    async fn enable_proxy_macos(&self, proxy_type: &str, host: &str, port: u16) -> Result<()> {
        let host = host.to_string();
        let port_str = port.to_string();
        let proxy_type_owned = proxy_type.to_string();
        let services = self.network_services.clone();

        for service in services {
            let host_clone = host.clone();
            let port_clone = port_str.clone();
            let proxy_type_clone = proxy_type_owned.clone();
            let service_for_log = service.clone();

            let output = tokio::task::spawn_blocking(move || {
                Command::new("networksetup")
                    .args([
                        &format!("-set{}", proxy_type_clone),
                        &service_for_log,
                        &host_clone,
                        &port_clone,
                    ])
                    .output()
            })
            .await??;

            if !output.status.success() {
                warn!(
                    "Failed to set {} for service {}: {}",
                    proxy_type,
                    service,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn disable_proxy_macos(&self) -> Result<()> {
        let services = self.network_services.clone();

        for service in services {
            for proxy_type in ["webproxy", "securewebproxy", "socksfirewallproxy"] {
                let service_clone = service.clone();
                let proxy_type_owned = proxy_type.to_string();

                let output = tokio::task::spawn_blocking(move || {
                    Command::new("networksetup")
                        .args([
                            &format!("-set{}state", proxy_type_owned),
                            &service_clone,
                            "off",
                        ])
                        .output()
                })
                .await??;

                if !output.status.success() {
                    warn!("Failed to disable {} for service {}", proxy_type, service);
                }
            }
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn get_proxy_macos(&self) -> Result<Option<ProxyConfig>> {
        if self.network_services.is_empty() {
            return Ok(None);
        }

        let service = self.network_services.first().unwrap().clone();

        let output = tokio::task::spawn_blocking(move || {
            Command::new("networksetup")
                .args(["-getwebproxy", &service])
                .output()
        })
        .await??;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        if lines.len() >= 3 {
            let enabled = lines[0].contains("Yes");
            let host = lines[1].split(':').nth(1).map(|s| s.trim().to_string());
            let port = lines[2]
                .split(':')
                .nth(1)
                .and_then(|s| s.trim().parse::<u16>().ok());

            if let (Some(host), Some(port)) = (host, port) {
                return Ok(Some(
                    ProxyConfig::new(ProxyType::Http, host, port).with_enabled(enabled),
                ));
            }
        }

        Ok(None)
    }

    #[cfg(target_os = "linux")]
    async fn enable_proxy_linux(&self, proxy_type: &str, host: &str, port: u16) -> Result<()> {
        let proxy_url = format!("http://{}:{}", host, port);
        let key = match proxy_type {
            "http" => "org.gnome.system.proxy.http",
            "https" => "org.gnome.system.proxy.https",
            "socks" => "org.gnome.system.proxy.socks",
            _ => "org.gnome.system.proxy.http",
        };

        let proxy_url_clone = proxy_url.clone();
        let key_clone = key.to_string();

        tokio::task::spawn_blocking(move || {
            let is_gnome = Command::new("gsettings")
                .args(["get", "org.gnome.system.proxy", "mode"])
                .output()
                .is_ok();

            if is_gnome {
                Command::new("gsettings")
                    .args(["set", "org.gnome.system.proxy", "mode", "manual"])
                    .status()?;

                Command::new("gsettings")
                    .args(["set", key_clone, "host", host])
                    .status()?;

                Command::new("gsettings")
                    .args(["set", key_clone, "port", &port.to_string()])
                    .status()?;
            } else {
                std::env::set_var(
                    format!("{}_proxy", proxy_type.to_uppercase()),
                    &proxy_url_clone,
                );
                std::env::set_var(
                    format!("{}_PROXY", proxy_type.to_uppercase()),
                    &proxy_url_clone,
                );
            }

            Ok::<_, std::io::Error>(())
        })
        .await??;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn disable_proxy_linux(&self) -> Result<()> {
        tokio::task::spawn_blocking(|| {
            let is_gnome = Command::new("gsettings")
                .args(["get", "org.gnome.system.proxy", "mode"])
                .output()
                .is_ok();

            if is_gnome {
                Command::new("gsettings")
                    .args(["set", "org.gnome.system.proxy", "mode", "none"])
                    .status()?;
            }

            for var in [
                "http_proxy",
                "HTTP_PROXY",
                "https_proxy",
                "HTTPS_PROXY",
                "all_proxy",
                "ALL_PROXY",
            ] {
                std::env::remove_var(var);
            }

            Ok::<_, std::io::Error>(())
        })
        .await??;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn get_proxy_linux(&self) -> Result<Option<ProxyConfig>> {
        tokio::task::spawn_blocking(|| {
            let is_gnome = Command::new("gsettings")
                .args(["get", "org.gnome.system.proxy", "mode"])
                .output()
                .is_ok();

            if is_gnome {
                let output = Command::new("gsettings")
                    .args(["get", "org.gnome.system.proxy.http", "host"])
                    .output()?;

                let host = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .trim_matches('\'')
                    .to_string();

                if host.is_empty() || host == "''" {
                    return Ok(None);
                }

                let output = Command::new("gsettings")
                    .args(["get", "org.gnome.system.proxy.http", "port"])
                    .output()?;

                let port: u16 = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse()
                    .unwrap_or(0);

                if port > 0 {
                    return Ok(Some(ProxyConfig::new(ProxyType::Http, host, port)));
                }
            }

            for var in ["http_proxy", "HTTP_PROXY"] {
                if let Ok(proxy) = std::env::var(var) {
                    if let Some(config) = parse_proxy_url(&proxy) {
                        return Ok(Some(config));
                    }
                }
            }

            Ok(None)
        })
        .await?
    }

    #[cfg(target_os = "windows")]
    async fn enable_proxy_windows(&self, proxy_type: &str, host: &str, port: u16) -> Result<()> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let settings = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            KEY_WRITE,
        )?;

        let proxy_server = if proxy_type == "socks" {
            format!("socks={}:{}", host, port)
        } else {
            format!("{}={}:{};http={}:{}", proxy_type, host, port, host, port)
        };

        settings.set_value("ProxyEnable", &1u32)?;
        settings.set_value("ProxyServer", &proxy_server)?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn disable_proxy_windows(&self) -> Result<()> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let settings = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            KEY_WRITE,
        )?;

        settings.set_value("ProxyEnable", &0u32)?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn get_proxy_windows(&self) -> Result<Option<ProxyConfig>> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let settings = hkcu.open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            KEY_READ,
        )?;

        let enabled: u32 = settings.get_value("ProxyEnable").unwrap_or(0);

        if enabled == 0 {
            return Ok(None);
        }

        let proxy_server: String = settings.get_value("ProxyServer").unwrap_or_default();

        if proxy_server.is_empty() {
            return Ok(None);
        }

        if let Some(config) = parse_proxy_url(&proxy_server) {
            return Ok(Some(config));
        }

        Ok(None)
    }
}

impl Default for SysProxyManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            #[cfg(target_os = "macos")]
            network_services: vec![],
            #[cfg(target_os = "linux")]
            original_proxy: None,
        })
    }
}

fn parse_proxy_url(url: &str) -> Option<ProxyConfig> {
    let url = url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_start_matches("socks5://")
        .trim_start_matches("socks://");

    let parts: Vec<&str> = url.split(':').collect();
    if parts.len() == 2 {
        let host = parts[0].to_string();
        let port = parts[1].parse().ok()?;
        return Some(ProxyConfig::new(ProxyType::Http, host, port));
    }
    None
}

impl ProxyConfig {
    fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_config_new() {
        let config = ProxyConfig::new(ProxyType::Http, "127.0.0.1", 7890);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 7890);
        assert!(config.enabled);
        assert_eq!(config.address(), "127.0.0.1:7890");
    }

    #[test]
    fn test_proxy_config_address() {
        let config = ProxyConfig::new(ProxyType::Socks, "localhost", 1080);
        assert_eq!(config.address(), "localhost:1080");
    }

    #[test]
    fn test_parse_proxy_url() {
        let config = parse_proxy_url("127.0.0.1:7890").unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 7890);

        let config = parse_proxy_url("http://127.0.0.1:7890").unwrap();
        assert_eq!(config.host, "127.0.0.1");

        let config = parse_proxy_url("localhost:8080").unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);

        assert!(parse_proxy_url("invalid").is_none());
        assert!(parse_proxy_url("host:invalid_port").is_none());
    }

    #[test]
    fn test_proxy_type_equality() {
        assert_eq!(ProxyType::Http, ProxyType::Http);
        assert_ne!(ProxyType::Http, ProxyType::Https);
    }
}
