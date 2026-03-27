use serde::{Deserialize, Serialize};

pub mod app;
pub mod mihomo;
pub mod r#override;
pub mod profile;

#[cfg(test)]
mod tests;

pub use app::{AppConfig, AppConfigPatch};
pub use mihomo::{
    DnsConfig, FallbackFilter, GeoXUrl, HealthCheckConfig, ListenerConfig, MihomoConfig,
    ProfileSettings, ProxyConfig, ProxyGroupConfig, ProxyProviderConfig, RuleProviderConfig,
    ScriptConfig, TunConfig,
};
pub use profile::{ProfileConfig, ProfileExtra, ProfileItem, ProfileType, SubscriptionInfo};
pub use r#override::{
    MixinRule, OverrideConfig, OverrideError, OverrideItem, OverrideRule, ScriptRule,
};

/// Rule type enumeration for Clash routing rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RuleType {
    Domain,
    DomainSuffix,
    DomainKeyword,
    GeoIP,
    Geosite,
    IPCIDR,
    SrcIPCIDR,
    DstPort,
    SrcPort,
    ProcessName,
    ProcessPath,
    Match,
}

impl RuleType {
    /// Convert to string representation for Clash config
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleType::Domain => "DOMAIN",
            RuleType::DomainSuffix => "DOMAIN-SUFFIX",
            RuleType::DomainKeyword => "DOMAIN-KEYWORD",
            RuleType::GeoIP => "GEOIP",
            RuleType::Geosite => "GEOSITE",
            RuleType::IPCIDR => "IP-CIDR",
            RuleType::SrcIPCIDR => "SRC-IP-CIDR",
            RuleType::DstPort => "DST-PORT",
            RuleType::SrcPort => "SRC-PORT",
            RuleType::ProcessName => "PROCESS-NAME",
            RuleType::ProcessPath => "PROCESS-PATH",
            RuleType::Match => "MATCH",
        }
    }

    /// Parse from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "DOMAIN" => Some(RuleType::Domain),
            "DOMAIN-SUFFIX" => Some(RuleType::DomainSuffix),
            "DOMAIN-KEYWORD" => Some(RuleType::DomainKeyword),
            "GEOIP" => Some(RuleType::GeoIP),
            "GEOSITE" => Some(RuleType::Geosite),
            "IP-CIDR" => Some(RuleType::IPCIDR),
            "SRC-IP-CIDR" => Some(RuleType::SrcIPCIDR),
            "DST-PORT" => Some(RuleType::DstPort),
            "SRC-PORT" => Some(RuleType::SrcPort),
            "PROCESS-NAME" => Some(RuleType::ProcessName),
            "PROCESS-PATH" => Some(RuleType::ProcessPath),
            "MATCH" => Some(RuleType::Match),
            _ => None,
        }
    }
}

impl std::fmt::Display for RuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "type")]
    pub rule_type: RuleType,
    pub payload: String,
    pub proxy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_resolve: Option<bool>,
}

impl Rule {
    /// Create a new rule
    pub fn new(rule_type: RuleType, payload: String, proxy: String) -> Self {
        Self {
            rule_type,
            payload,
            proxy,
            no_resolve: None,
        }
    }

    /// Parse from Clash config format: "TYPE,PAYLOAD,PROXY"
    pub fn parse(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.splitn(3, ',').collect();
        if parts.len() < 3 {
            return None;
        }

        let rule_type = RuleType::from_str(parts[0])?;
        let payload = parts[1].to_string();
        let proxy = parts[2].to_string();

        Some(Self {
            rule_type,
            payload,
            proxy,
            no_resolve: None,
        })
    }

    /// Convert to Clash config format
    pub fn to_clash_string(&self) -> String {
        if let Some(true) = self.no_resolve {
            format!(
                "{},{},{},no-resolve",
                self.rule_type.as_str(),
                self.payload,
                self.proxy
            )
        } else {
            format!(
                "{},{},{}",
                self.rule_type.as_str(),
                self.payload,
                self.proxy
            )
        }
    }
}

/// Rule provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleProvider {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub behavior: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u64>,
}
