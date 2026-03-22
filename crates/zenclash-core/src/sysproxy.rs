use std::fmt;

pub use sysproxy::{Autoproxy, Sysproxy};

#[derive(Debug, thiserror::Error)]
pub enum SysproxyError {
    #[error("Failed to set system proxy: {0}")]
    SetFailed(String),

    #[error("Failed to disable system proxy: {0}")]
    DisableFailed(String),

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyMode {
    Manual,
    Auto,
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub mode: ProxyMode,
    pub host: String,
    pub port: u16,
    pub bypass: Vec<String>,
    pub pac_url: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            mode: ProxyMode::Manual,
            host: "127.0.0.1".into(),
            port: 7890,
            bypass: default_bypass(),
            pac_url: None,
        }
    }
}

pub fn default_bypass() -> Vec<String> {
    #[cfg(target_os = "linux")]
    {
        vec![
            "localhost".into(),
            "127.0.0.1".into(),
            "192.168.0.0/16".into(),
            "10.0.0.0/8".into(),
            "172.16.0.0/12".into(),
            "::1".into(),
        ]
    }

    #[cfg(target_os = "macos")]
    {
        vec![
            "127.0.0.1".into(),
            "192.168.0.0/16".into(),
            "10.0.0.0/8".into(),
            "172.16.0.0/12".into(),
            "localhost".into(),
            "*.local".into(),
            "*.crashlytics.com".into(),
            "<local>".into(),
        ]
    }

    #[cfg(target_os = "windows")]
    {
        vec![
            "localhost".into(),
            "127.*".into(),
            "192.168.*".into(),
            "10.*".into(),
            "172.16.*".into(),
            "172.17.*".into(),
            "172.18.*".into(),
            "172.19.*".into(),
            "172.20.*".into(),
            "172.21.*".into(),
            "172.22.*".into(),
            "172.23.*".into(),
            "172.24.*".into(),
            "172.25.*".into(),
            "172.26.*".into(),
            "172.27.*".into(),
            "172.28.*".into(),
            "172.29.*".into(),
            "172.30.*".into(),
            "172.31.*".into(),
            "<local>".into(),
        ]
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        vec!["localhost".into(), "127.0.0.1".into()]
    }
}

pub struct SystemProxyManager;

impl SystemProxyManager {
    pub fn enable(config: &ProxyConfig) -> Result<(), SysproxyError> {
        match config.mode {
            ProxyMode::Manual => {
                let bypass = config.bypass.join(if cfg!(target_os = "windows") {
                    ";"
                } else {
                    ","
                });

                #[cfg(target_os = "macos")]
                {
                    Self::enable_macos_manual(&config.host, config.port, &bypass)?;
                }

                #[cfg(not(target_os = "macos"))]
                {
                    let proxy = Sysproxy {
                        host: config.host.clone(),
                        port: config.port,
                        bypass: bypass.clone(),
                    };
                    proxy
                        .set_system_proxy()
                        .map_err(|e| SysproxyError::SetFailed(e.to_string()))?;
                }
            },
            ProxyMode::Auto => {
                let pac_url = config.pac_url.as_ref().ok_or_else(|| {
                    SysproxyError::SetFailed("PAC URL required for auto mode".into())
                })?;

                #[cfg(target_os = "macos")]
                {
                    Self::enable_macos_auto(pac_url)?;
                }

                #[cfg(not(target_os = "macos"))]
                {
                    let auto = Autoproxy {
                        url: pac_url.clone(),
                    };
                    auto.set_auto_proxy()
                        .map_err(|e| SysproxyError::SetFailed(e.to_string()))?;
                }
            },
        }

        Ok(())
    }

    pub fn disable() -> Result<(), SysproxyError> {
        #[cfg(target_os = "macos")]
        {
            Self::disable_macos()?;
        }

        #[cfg(not(target_os = "macos"))]
        {
            Sysproxy::disable_system_proxy()
                .map_err(|e| SysproxyError::DisableFailed(e.to_string()))?;
            Autoproxy::disable_auto_proxy()
                .map_err(|e| SysproxyError::DisableFailed(e.to_string()))?;
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn enable_macos_manual(host: &str, port: u16, bypass: &str) -> Result<(), SysproxyError> {
        let script = format!(
            r#"do shell script "networksetup -setwebproxy Wi-Fi {} {} && networksetup -setsecurewebproxy Wi-Fi {} {} && networksetup -setproxybypassdomains Wi-Fi {}" with administrator privileges"#,
            host,
            port,
            host,
            port,
            bypass.replace(",", "\" \"")
        );

        std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| SysproxyError::SetFailed(e.to_string()))?;

        let enable_script = r#"do shell script "networksetup -setwebproxystate Wi-Fi on && networksetup -setsecurewebproxystate Wi-Fi on" with administrator privileges"#;

        std::process::Command::new("osascript")
            .arg("-e")
            .arg(enable_script)
            .output()
            .map_err(|e| SysproxyError::SetFailed(e.to_string()))?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn enable_macos_auto(pac_url: &str) -> Result<(), SysproxyError> {
        let script = format!(
            r#"do shell script "networksetup -setautoproxyurl Wi-fi '{}'" with administrator privileges"#,
            pac_url
        );

        std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| SysproxyError::SetFailed(e.to_string()))?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn disable_macos() -> Result<(), SysproxyError> {
        let script = r#"do shell script "networksetup -setwebproxystate Wi-Fi off && networksetup -setsecurewebproxystate Wi-Fi off && networksetup -setautoproxystate Wi-Fi off" with administrator privileges"#;

        std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| SysproxyError::DisableFailed(e.to_string()))?;

        Ok(())
    }

    pub fn is_enabled() -> bool {
        #[cfg(target_os = "macos")]
        {
            Self::is_enabled_macos()
        }

        #[cfg(target_os = "windows")]
        {
            Sysproxy::get_system_proxy()
                .map(|p| p.port > 0)
                .unwrap_or(false)
        }

        #[cfg(target_os = "linux")]
        {
            Sysproxy::get_system_proxy()
                .map(|p| p.port > 0)
                .unwrap_or(false)
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            false
        }
    }

    #[cfg(target_os = "macos")]
    fn is_enabled_macos() -> bool {
        std::process::Command::new("networksetup")
            .args(["-getwebproxy", "Wi-Fi"])
            .output()
            .map(|o| {
                let s = String::from_utf8_lossy(&o.stdout);
                s.contains("Enabled: Yes")
            })
            .unwrap_or(false)
    }
}
