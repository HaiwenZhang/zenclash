pub mod backup;
pub mod connections;
pub mod dns;
pub mod logs;
pub mod mihomo;
pub mod override_page;
pub mod profiles;
pub mod proxies;
pub mod resources;
pub mod rules;
pub mod settings;
pub mod sniffer;
pub mod substore;
pub mod sysproxy;
pub mod tun;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Proxies,
    Profiles,
    Connections,
    Rules,
    Logs,
    Mihomo,
    Tun,
    Sniffer,
    Resources,
    Dns,
    Backup,
    Override,
    Sysproxy,
    SubStore,
    Settings,
}

impl Page {
    pub fn label(&self) -> &'static str {
        match self {
            Page::Proxies => "Proxies",
            Page::Profiles => "Profiles",
            Page::Connections => "Connections",
            Page::Rules => "Rules",
            Page::Logs => "Logs",
            Page::Mihomo => "Mihomo",
            Page::Tun => "TUN",
            Page::Sniffer => "Sniffer",
            Page::Resources => "Resources",
            Page::Dns => "DNS",
            Page::Backup => "Backup",
            Page::Override => "Override",
            Page::Sysproxy => "System Proxy",
            Page::SubStore => "SubStore",
            Page::Settings => "Settings",
        }
    }

    pub fn icon(&self) -> gpui_component::IconName {
        use gpui_component::IconName;
        match self {
            Page::Proxies => IconName::Globe,
            Page::Profiles => IconName::FileText,
            Page::Connections => IconName::Link,
            Page::Rules => IconName::List,
            Page::Logs => IconName::ScrollText,
            Page::Mihomo => IconName::Cpu,
            Page::Tun => IconName::Route,
            Page::Sniffer => IconName::Search,
            Page::Resources => IconName::Database,
            Page::Dns => IconName::Server,
            Page::Backup => IconName::Archive,
            Page::Override => IconName::FileCode,
            Page::Sysproxy => IconName::Globe,
            Page::SubStore => IconName::Package,
            Page::Settings => IconName::Settings,
        }
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}