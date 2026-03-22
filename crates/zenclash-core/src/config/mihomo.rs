use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MihomoConfig {
    pub mixed_port: Option<u16>,
    pub socks_port: Option<u16>,
    pub port: Option<u16>,
    pub redir_port: Option<u16>,
    pub tproxy_port: Option<u16>,
    pub authentication: Vec<String>,
    pub allow_lan: bool,
    pub bind_address: Option<String>,
    pub mode: String,
    pub log_level: String,
    pub ipv6: bool,
    pub external_controller: Option<String>,
    pub external_ui: Option<String>,
    pub external_ui_download_url: Option<String>,
    pub external_ui_download_detour: Option<String>,
    pub secret: Option<String>,
    pub interface_name: Option<String>,
    pub so_mark: Option<u32>,
    pub tun: Option<TunConfig>,
    pub dns: Option<DnsConfig>,
    pub hosts: Option<HashMap<String, String>>,
    pub geodata_mode: bool,
    pub geo_auto_update: bool,
    pub geodata_loader: Option<String>,
    pub unified_delay: bool,
    pub tcp_concurrent: bool,
    pub find_process_mode: Option<String>,
    pub global_client_fingerprint: Option<String>,
    pub sniff: Option<bool>,
    pub sniff_override_destination: Option<bool>,
    pub skip_cert_verify: bool,
    pub proxies: Option<Vec<ProxyConfig>>,
    pub proxy_groups: Option<Vec<ProxyGroupConfig>>,
    pub rules: Option<Vec<String>>,
    pub rule_providers: Option<HashMap<String, RuleProviderConfig>>,
    pub proxy_providers: Option<HashMap<String, ProxyProviderConfig>>,
    pub script: Option<ScriptConfig>,
    pub profile: Option<ProfileSettings>,
    pub geox_url: Option<GeoXUrl>,
    pub listeners: Option<Vec<ListenerConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TunConfig {
    pub enable: bool,
    pub stack: String,
    pub dns_hijack: Vec<String>,
    pub auto_route: bool,
    pub auto_detect_interface: bool,
    #[serde(rename = "device")]
    pub device_name: Option<String>,
    pub mtu: Option<u16>,
    pub strict_route: Option<bool>,
    pub endpoint_distance_nat: Option<bool>,
}

impl Default for TunConfig {
    fn default() -> Self {
        Self {
            enable: false,
            stack: "system".to_string(),
            dns_hijack: vec!["any:53".to_string()],
            auto_route: true,
            auto_detect_interface: true,
            device_name: None,
            mtu: None,
            strict_route: None,
            endpoint_distance_nat: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DnsConfig {
    pub enable: bool,
    pub ipv6: bool,
    pub enhanced_mode: String,
    pub fake_ip_range: String,
    pub fake_ip_filter: Vec<String>,
    pub default_nameserver: Vec<String>,
    pub nameserver: Vec<String>,
    pub fallback: Option<Vec<String>>,
    pub fallback_filter: Option<FallbackFilter>,
    pub listen: Option<String>,
    pub use_hosts: bool,
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            enable: true,
            ipv6: false,
            enhanced_mode: "fake-ip".to_string(),
            fake_ip_range: "198.18.0.1/16".to_string(),
            fake_ip_filter: vec!["*.lan".to_string(), "*.localhost".to_string()],
            default_nameserver: vec!["223.5.5.5".to_string(), "119.29.29.29".to_string()],
            nameserver: vec![
                "https://doh.pub/dns-query".to_string(),
                "https://dns.alidns.com/dns-query".to_string(),
            ],
            fallback: None,
            fallback_filter: None,
            listen: None,
            use_hosts: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackFilter {
    pub geoip: Option<bool>,
    pub geoip_code: Option<String>,
    pub ipcidr: Option<Vec<String>>,
    pub domain: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub server: String,
    pub port: u16,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroupConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub proxies: Option<Vec<String>>,
    #[serde(rename = "use")]
    pub use_providers: Option<Vec<String>>,
    pub url: Option<String>,
    pub interval: Option<u32>,
    pub tolerance: Option<u32>,
    pub lazy: Option<bool>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleProviderConfig {
    #[serde(rename = "type")]
    pub provider_type: String,
    pub behavior: String,
    pub path: Option<String>,
    pub url: Option<String>,
    pub interval: Option<u32>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyProviderConfig {
    #[serde(rename = "type")]
    pub provider_type: String,
    pub path: Option<String>,
    pub url: Option<String>,
    pub interval: Option<u32>,
    pub health_check: Option<HealthCheckConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enable: bool,
    pub url: Option<String>,
    pub interval: Option<u32>,
    pub lazy: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptConfig {
    pub shortcuts: Option<HashMap<String, String>>,
    pub code: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileSettings {
    pub store_selected: Option<bool>,
    pub store_fake_ip: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoXUrl {
    pub geoip: Option<String>,
    pub geosite: Option<String>,
    pub mmdb: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub listener_type: String,
    pub port: u16,
    pub listen: Option<String>,
    pub proxy: Option<String>,
}

impl MihomoConfig {
    pub fn load(path: &PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn save(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, content)
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

impl Default for MihomoConfig {
    fn default() -> Self {
        Self {
            mixed_port: Some(7890),
            socks_port: None,
            port: None,
            redir_port: None,
            tproxy_port: None,
            authentication: vec![],
            allow_lan: false,
            bind_address: None,
            mode: "rule".to_string(),
            log_level: "info".to_string(),
            ipv6: false,
            external_controller: Some("127.0.0.1:9090".to_string()),
            external_ui: None,
            external_ui_download_url: None,
            external_ui_download_detour: None,
            secret: None,
            interface_name: None,
            so_mark: None,
            tun: None,
            dns: Some(DnsConfig::default()),
            hosts: None,
            geodata_mode: false,
            geo_auto_update: false,
            geodata_loader: None,
            unified_delay: false,
            tcp_concurrent: false,
            find_process_mode: None,
            global_client_fingerprint: None,
            sniff: Some(true),
            sniff_override_destination: None,
            skip_cert_verify: false,
            proxies: None,
            proxy_groups: None,
            rules: Some(vec![
                "GEOIP,CN,DIRECT".to_string(),
                "MATCH,PROXY".to_string(),
            ]),
            rule_providers: None,
            proxy_providers: None,
            script: None,
            profile: None,
            geox_url: None,
            listeners: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mihomo_config_default() {
        let config = MihomoConfig::default();
        assert_eq!(config.mixed_port, Some(7890));
        assert_eq!(config.mode, "rule");
        assert!(config.dns.is_some());
    }

    #[test]
    fn test_mihomo_config_to_yaml() {
        let config = MihomoConfig::default();
        let yaml = config.to_yaml().unwrap();
        assert!(yaml.contains("mixed_port"));
    }
}
