pub mod backup;
pub mod connections;
pub mod dashboard;
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
    Dashboard,
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
            Page::Dashboard => "Dashboard",
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
            Page::Dashboard => IconName::LayoutDashboard,
            Page::Proxies => IconName::Globe,
            Page::Profiles => IconName::File,
            Page::Connections => IconName::ExternalLink,
            Page::Rules => IconName::Menu,
            Page::Logs => IconName::BookOpen,
            Page::Mihomo => IconName::Settings,
            Page::Tun => IconName::Map,
            Page::Sniffer => IconName::Search,
            Page::Resources => IconName::Inbox,
            Page::Dns => IconName::Building2,
            Page::Backup => IconName::Folder,
            Page::Override => IconName::File,
            Page::Sysproxy => IconName::Globe,
            Page::SubStore => IconName::Folder,
            Page::Settings => IconName::Settings,
        }
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Trait for page components
pub trait PageTrait: Sized {
    fn title() -> &'static str;
    fn icon() -> gpui_component::IconName;
}

// Re-export for backward compatibility
pub use PageTrait as PageApi;