use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_core")]
    pub core: String,

    #[serde(default = "default_auto_launch")]
    pub auto_launch: bool,

    #[serde(default)]
    pub silent_start: bool,

    #[serde(default)]
    pub system_proxy: bool,

    #[serde(default)]
    pub tun_mode: bool,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default)]
    pub language: String,

    #[serde(default)]
    pub proxy_layout: String,

    #[serde(default)]
    pub enable_tray_speed: bool,

    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default)]
    pub show_conn: bool,

    #[serde(default)]
    pub show_delay: bool,

    #[serde(default)]
    pub delay_test_url: String,

    #[serde(default)]
    pub current_profile: Option<String>,

    #[serde(default)]
    pub current_sub: Option<String>,

    #[serde(default = "default_api_port")]
    pub api_port: u16,

    #[serde(default)]
    pub api_secret: Option<String>,

    #[serde(default)]
    pub socks_port: Option<u16>,

    #[serde(default)]
    pub mixed_port: Option<u16>,

    #[serde(default)]
    pub redir_port: Option<u16>,

    #[serde(default)]
    pub tproxy_port: Option<u16>,

    #[serde(default)]
    pub external_controller: Option<String>,

    #[serde(default)]
    pub external_ui: Option<String>,

    #[serde(default)]
    pub secret: Option<String>,

    #[serde(default)]
    pub allow_lan: bool,

    #[serde(default)]
    pub bind_address: Option<String>,

    #[serde(default)]
    pub mode: String,

    #[serde(default)]
    pub ipv6: bool,

    #[serde(default)]
    pub unified_delay: bool,

    #[serde(default)]
    pub tcp_concurrent: bool,

    #[serde(default)]
    pub find_process_mode: Option<String>,

    #[serde(default)]
    pub geodata_mode: bool,

    #[serde(default)]
    pub geo_auto_update: bool,

    #[serde(default)]
    pub geo_update_interval: u64,

    #[serde(default)]
    pub profile_auto_update: bool,

    #[serde(default)]
    pub profile_update_interval: u64,
}

fn default_core() -> String {
    "mihomo".to_string()
}
fn default_auto_launch() -> bool {
    false
}
fn default_theme() -> String {
    "system".to_string()
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_api_port() -> u16 {
    9090
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            core: default_core(),
            auto_launch: default_auto_launch(),
            silent_start: false,
            system_proxy: false,
            tun_mode: false,
            theme: default_theme(),
            language: String::new(),
            proxy_layout: String::new(),
            enable_tray_speed: false,
            log_level: default_log_level(),
            show_conn: false,
            show_delay: false,
            delay_test_url: String::new(),
            current_profile: None,
            current_sub: None,
            api_port: default_api_port(),
            api_secret: None,
            socks_port: None,
            mixed_port: None,
            redir_port: None,
            tproxy_port: None,
            external_controller: None,
            external_ui: None,
            secret: None,
            allow_lan: false,
            bind_address: None,
            mode: String::new(),
            ipv6: false,
            unified_delay: false,
            tcp_concurrent: false,
            find_process_mode: None,
            geodata_mode: false,
            geo_auto_update: false,
            geo_update_interval: 24,
            profile_auto_update: false,
            profile_update_interval: 24,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppConfigPatch {
    pub core: Option<String>,
    pub auto_launch: Option<bool>,
    pub silent_start: Option<bool>,
    pub system_proxy: Option<bool>,
    pub tun_mode: Option<bool>,
    pub theme: Option<String>,
    pub language: Option<String>,
    pub proxy_layout: Option<String>,
    pub enable_tray_speed: Option<bool>,
    pub log_level: Option<String>,
    pub show_conn: Option<bool>,
    pub show_delay: Option<bool>,
    pub delay_test_url: Option<String>,
    pub current_profile: Option<String>,
    pub current_sub: Option<String>,
    pub api_port: Option<u16>,
    pub api_secret: Option<String>,
    pub socks_port: Option<u16>,
    pub mixed_port: Option<u16>,
    pub redir_port: Option<u16>,
    pub tproxy_port: Option<u16>,
    pub external_controller: Option<String>,
    pub external_ui: Option<String>,
    pub secret: Option<String>,
    pub allow_lan: Option<bool>,
    pub bind_address: Option<String>,
    pub mode: Option<String>,
    pub ipv6: Option<bool>,
    pub unified_delay: Option<bool>,
    pub tcp_concurrent: Option<bool>,
    pub find_process_mode: Option<String>,
    pub geodata_mode: Option<bool>,
    pub geo_auto_update: Option<bool>,
    pub geo_update_interval: Option<u64>,
    pub profile_auto_update: Option<bool>,
    pub profile_update_interval: Option<u64>,
}

impl AppConfigPatch {
    pub fn apply(&self, config: &mut AppConfig) {
        if let Some(v) = &self.core {
            config.core = v.clone();
        }
        if let Some(v) = self.auto_launch {
            config.auto_launch = v;
        }
        if let Some(v) = self.silent_start {
            config.silent_start = v;
        }
        if let Some(v) = self.system_proxy {
            config.system_proxy = v;
        }
        if let Some(v) = self.tun_mode {
            config.tun_mode = v;
        }
        if let Some(v) = &self.theme {
            config.theme = v.clone();
        }
        if let Some(v) = &self.language {
            config.language = v.clone();
        }
        if let Some(v) = &self.proxy_layout {
            config.proxy_layout = v.clone();
        }
        if let Some(v) = self.enable_tray_speed {
            config.enable_tray_speed = v;
        }
        if let Some(v) = &self.log_level {
            config.log_level = v.clone();
        }
        if let Some(v) = self.show_conn {
            config.show_conn = v;
        }
        if let Some(v) = self.show_delay {
            config.show_delay = v;
        }
        if let Some(v) = &self.delay_test_url {
            config.delay_test_url = v.clone();
        }
        if let Some(v) = &self.current_profile {
            config.current_profile = Some(v.clone());
        }
        if let Some(v) = &self.current_sub {
            config.current_sub = Some(v.clone());
        }
        if let Some(v) = self.api_port {
            config.api_port = v;
        }
        if let Some(v) = &self.api_secret {
            config.api_secret = Some(v.clone());
        }
        if let Some(v) = self.socks_port {
            config.socks_port = Some(v);
        }
        if let Some(v) = self.mixed_port {
            config.mixed_port = Some(v);
        }
        if let Some(v) = self.redir_port {
            config.redir_port = Some(v);
        }
        if let Some(v) = self.tproxy_port {
            config.tproxy_port = Some(v);
        }
        if let Some(v) = &self.external_controller {
            config.external_controller = Some(v.clone());
        }
        if let Some(v) = &self.external_ui {
            config.external_ui = Some(v.clone());
        }
        if let Some(v) = &self.secret {
            config.secret = Some(v.clone());
        }
        if let Some(v) = self.allow_lan {
            config.allow_lan = v;
        }
        if let Some(v) = &self.bind_address {
            config.bind_address = Some(v.clone());
        }
        if let Some(v) = &self.mode {
            config.mode = v.clone();
        }
        if let Some(v) = self.ipv6 {
            config.ipv6 = v;
        }
        if let Some(v) = self.unified_delay {
            config.unified_delay = v;
        }
        if let Some(v) = self.tcp_concurrent {
            config.tcp_concurrent = v;
        }
        if let Some(v) = &self.find_process_mode {
            config.find_process_mode = Some(v.clone());
        }
        if let Some(v) = self.geodata_mode {
            config.geodata_mode = v;
        }
        if let Some(v) = self.geo_auto_update {
            config.geo_auto_update = v;
        }
        if let Some(v) = self.geo_update_interval {
            config.geo_update_interval = v;
        }
        if let Some(v) = self.profile_auto_update {
            config.profile_auto_update = v;
        }
        if let Some(v) = self.profile_update_interval {
            config.profile_update_interval = v;
        }
    }
}

impl AppConfig {
    pub fn load() -> std::io::Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        let content = std::fs::read_to_string(&path)?;
        serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&path, content)
    }

    pub fn config_path() -> PathBuf {
        crate::utils::dirs::config_dir().join("app.yaml")
    }

    pub fn patch(&mut self, patch: AppConfigPatch) {
        patch.apply(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.core, "mihomo");
        assert_eq!(config.api_port, 9090);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_app_config_patch() {
        let mut config = AppConfig::default();
        let patch = AppConfigPatch {
            core: Some("custom".to_string()),
            api_port: Some(8080),
            ..Default::default()
        };
        patch.apply(&mut config);
        assert_eq!(config.core, "custom");
        assert_eq!(config.api_port, 8080);
    }
}
