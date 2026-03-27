use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Socks5,
    Http,
    Ss,
    Ssr,
    Vmess,
    Trojan,
    Snell,
    Vless,
    Hysteria,
    Hysteria2,
    WireGuard,
    Tuic,
    ShadowsocksR,
    Selector,
    UrlTest,
    Fallback,
    LoadBalance,
    Relay,
    Direct,
    Reject,
}

impl std::fmt::Display for ProxyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyType::Socks5 => write!(f, "socks5"),
            ProxyType::Http => write!(f, "http"),
            ProxyType::Ss => write!(f, "ss"),
            ProxyType::Ssr => write!(f, "ssr"),
            ProxyType::Vmess => write!(f, "vmess"),
            ProxyType::Trojan => write!(f, "trojan"),
            ProxyType::Snell => write!(f, "snell"),
            ProxyType::Vless => write!(f, "vless"),
            ProxyType::Hysteria => write!(f, "hysteria"),
            ProxyType::Hysteria2 => write!(f, "hysteria2"),
            ProxyType::WireGuard => write!(f, "wireguard"),
            ProxyType::Tuic => write!(f, "tuic"),
            ProxyType::ShadowsocksR => write!(f, "shadowsocks-r"),
            ProxyType::Selector => write!(f, "selector"),
            ProxyType::UrlTest => write!(f, "url-test"),
            ProxyType::Fallback => write!(f, "fallback"),
            ProxyType::LoadBalance => write!(f, "load-balance"),
            ProxyType::Relay => write!(f, "relay"),
            ProxyType::Direct => write!(f, "direct"),
            ProxyType::Reject => write!(f, "reject"),
        }
    }
}

impl std::str::FromStr for ProxyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "socks5" => Ok(ProxyType::Socks5),
            "http" => Ok(ProxyType::Http),
            "ss" | "shadowsocks" => Ok(ProxyType::Ss),
            "ssr" => Ok(ProxyType::Ssr),
            "vmess" => Ok(ProxyType::Vmess),
            "trojan" => Ok(ProxyType::Trojan),
            "snell" => Ok(ProxyType::Snell),
            "vless" => Ok(ProxyType::Vless),
            "hysteria" => Ok(ProxyType::Hysteria),
            "hysteria2" => Ok(ProxyType::Hysteria2),
            "wireguard" => Ok(ProxyType::WireGuard),
            "tuic" => Ok(ProxyType::Tuic),
            "shadowsocks-r" => Ok(ProxyType::ShadowsocksR),
            "selector" => Ok(ProxyType::Selector),
            "url-test" => Ok(ProxyType::UrlTest),
            "fallback" => Ok(ProxyType::Fallback),
            "load-balance" => Ok(ProxyType::LoadBalance),
            "relay" => Ok(ProxyType::Relay),
            "direct" => Ok(ProxyType::Direct),
            "reject" => Ok(ProxyType::Reject),
            _ => Err(format!("Unknown proxy type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proxy {
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: ProxyType,
    pub server: Option<String>,
    pub port: Option<u16>,
    #[serde(default)]
    pub alive: bool,
    #[serde(default)]
    pub delay: Option<u32>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Proxy {
    pub fn new(name: String, proxy_type: ProxyType) -> Self {
        Self {
            name,
            proxy_type,
            server: None,
            port: None,
            alive: false,
            delay: None,
            extra: HashMap::new(),
        }
    }

    pub fn with_server(mut self, server: String, port: u16) -> Self {
        self.server = Some(server);
        self.port = Some(port);
        self
    }

    pub fn is_group(&self) -> bool {
        matches!(
            self.proxy_type,
            ProxyType::Selector
                | ProxyType::UrlTest
                | ProxyType::Fallback
                | ProxyType::LoadBalance
                | ProxyType::Relay
        )
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self.proxy_type, ProxyType::Direct | ProxyType::Reject)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    #[serde(rename = "type")]
    pub group_type: ProxyType,
    #[serde(default)]
    pub proxies: Vec<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub interval: Option<u32>,
    #[serde(default)]
    pub tolerance: Option<u32>,
    #[serde(default)]
    pub lazy: Option<bool>,
    #[serde(default)]
    pub timeout: Option<u32>,
    #[serde(default)]
    pub use_count: Option<u32>,
    #[serde(rename = "use", default)]
    pub use_providers: Vec<String>,
    #[serde(default)]
    pub current: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl ProxyGroup {
    pub fn new_selector(name: String, proxies: Vec<String>) -> Self {
        Self {
            name,
            group_type: ProxyType::Selector,
            proxies,
            url: None,
            interval: None,
            tolerance: None,
            lazy: None,
            timeout: None,
            use_count: None,
            use_providers: vec![],
            current: None,
            extra: HashMap::new(),
        }
    }

    pub fn new_url_test(name: String, proxies: Vec<String>, url: String, interval: u32) -> Self {
        Self {
            name,
            group_type: ProxyType::UrlTest,
            proxies,
            url: Some(url),
            interval: Some(interval),
            tolerance: None,
            lazy: None,
            timeout: None,
            use_count: None,
            use_providers: vec![],
            current: None,
            extra: HashMap::new(),
        }
    }

    pub fn new_fallback(name: String, proxies: Vec<String>, url: String, interval: u32) -> Self {
        Self {
            name,
            group_type: ProxyType::Fallback,
            proxies,
            url: Some(url),
            interval: Some(interval),
            tolerance: None,
            lazy: None,
            timeout: None,
            use_count: None,
            use_providers: vec![],
            current: None,
            extra: HashMap::new(),
        }
    }

    pub fn new_load_balance(
        name: String,
        proxies: Vec<String>,
        url: String,
        interval: u32,
    ) -> Self {
        Self {
            name,
            group_type: ProxyType::LoadBalance,
            proxies,
            url: Some(url),
            interval: Some(interval),
            tolerance: None,
            lazy: None,
            timeout: None,
            use_count: None,
            use_providers: vec![],
            current: None,
            extra: HashMap::new(),
        }
    }

    pub fn select(&mut self, proxy_name: &str) -> bool {
        if self.group_type != ProxyType::Selector {
            return false;
        }
        if self.proxies.contains(&proxy_name.to_string())
            || self.use_providers.iter().any(|p| p == proxy_name)
        {
            self.current = Some(proxy_name.to_string());
            return true;
        }
        false
    }

    pub fn get_current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    pub fn all_proxies(&self) -> &[String] {
        &self.proxies
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyProvider {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub path: Option<String>,
    pub url: Option<String>,
    pub interval: Option<u32>,
    #[serde(default)]
    pub health_check: Option<HealthCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub enable: bool,
    pub url: Option<String>,
    pub interval: Option<u32>,
    pub lazy: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxyCollection {
    pub proxies: HashMap<String, Proxy>,
    pub groups: HashMap<String, ProxyGroup>,
    pub providers: HashMap<String, ProxyProvider>,
}

impl ProxyCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_proxy(&mut self, proxy: Proxy) {
        self.proxies.insert(proxy.name.clone(), proxy);
    }

    pub fn add_group(&mut self, group: ProxyGroup) {
        self.groups.insert(group.name.clone(), group);
    }

    pub fn add_provider(&mut self, provider: ProxyProvider) {
        self.providers.insert(provider.name.clone(), provider);
    }

    pub fn get_proxy(&self, name: &str) -> Option<&Proxy> {
        self.proxies.get(name)
    }

    pub fn get_group(&self, name: &str) -> Option<&ProxyGroup> {
        self.groups.get(name)
    }

    pub fn get_provider(&self, name: &str) -> Option<&ProxyProvider> {
        self.providers.get(name)
    }

    pub fn get_proxy_or_group(&self, name: &str) -> Option<ProxyOrGroup<'_>> {
        if let Some(proxy) = self.proxies.get(name) {
            return Some(ProxyOrGroup::Proxy(proxy));
        }
        if let Some(group) = self.groups.get(name) {
            return Some(ProxyOrGroup::Group(group));
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum ProxyOrGroup<'a> {
    Proxy(&'a Proxy),
    Group(&'a ProxyGroup),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_proxy_type_from_str() {
        assert_eq!(ProxyType::from_str("socks5").unwrap(), ProxyType::Socks5);
        assert_eq!(ProxyType::from_str("vmess").unwrap(), ProxyType::Vmess);
        assert_eq!(ProxyType::from_str("url-test").unwrap(), ProxyType::UrlTest);
    }

    #[test]
    fn test_proxy_is_group() {
        let proxy = Proxy::new("test".to_string(), ProxyType::Ss);
        assert!(!proxy.is_group());

        let group = Proxy::new("group".to_string(), ProxyType::Selector);
        assert!(group.is_group());
    }

    #[test]
    fn test_proxy_group_selector() {
        let mut group = ProxyGroup::new_selector(
            "test".to_string(),
            vec!["proxy1".to_string(), "proxy2".to_string()],
        );

        assert!(group.select("proxy1"));
        assert_eq!(group.get_current(), Some("proxy1"));
        assert!(!group.select("unknown"));
    }
}
