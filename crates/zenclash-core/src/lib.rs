pub mod config;
pub mod core;
pub mod error;
pub mod proxy;
pub mod resolve;
pub mod server;
pub mod sys;
pub mod sysproxy;
pub mod utils;

pub use error::{Result, ZenClashError};

pub const APP_NAME: &str = "ZenClash";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod prelude {
    pub use crate::config::{
        AppConfig, AppConfigPatch, DnsConfig, MihomoConfig, OverrideConfig, OverrideItem,
        ProfileConfig, ProfileItem, ProfileType, Rule, RuleProvider, RuleType, SubscriptionInfo,
        TunConfig,
    };
    pub use crate::core::{
        ApiClient, ApiClientConfig, ConfigFactory, CoreManager, CoreManagerConfig, CoreState,
        DnsManager, FactoryError, LogStream, Process, ProcessConfig, ProcessState, ProfileUpdater,
        SubStoreClient, TrafficData, TrafficStream,
    };
    pub use crate::error::{Result, ZenClashError};
    pub use crate::proxy::{
        DelayTestConfig, DelayTestResult, DelayTester, Proxy, ProxyCollection, ProxyGroup,
        ProxySelector, ProxyType, SelectionStrategy,
    };
    pub use crate::resolve::{FloatingWindowManager, GistClient};
    pub use crate::server::{find_available_port, PacServer, PacServerConfig};
    pub use crate::sys::{AutoRunManager, SsidMonitor};
    pub use crate::sysproxy::{
        default_bypass, ProxyConfig, ProxyMode, SysproxyError, SystemProxyManager,
    };
    pub use crate::utils::{
        cache_dir, config_dir, core_log_path, data_dir, format_speed, format_traffic, profiles_dir,
        setup_default_logger, setup_logger, HttpClient, HttpClientConfig, LogLevel, LoggerConfig,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prelude_exports() {
        let _ = utils::data_dir();
        let _ = utils::config_dir();
    }
}
