pub mod api;
pub mod dns;
pub mod factory;
pub mod manager;
pub mod permissions;
pub mod process;
pub mod profile_updater;
pub mod substore;

#[cfg(test)]
mod tests;

pub use api::{
    ApiClient, ApiClientConfig, ApiError, ConfigPatch, ConnectionItem, ConnectionMetadata,
    ConnectionsResponse, ConnectionsStream, DelayHistory, DelayTestResult, LogItem, LogLevel,
    LogStream, MemoryData, ProviderItem, ProvidersResponse, ProxiesResponse, ProxyHistory,
    ProxyItem, RuleItem, RulesResponse, RuntimeConfig, TrafficData, TrafficStream, Version,
};
pub use dns::{DnsError, DnsManager};
pub use factory::{ConfigFactory, FactoryError, RuntimeConfigPatch, TunPatch};
pub use manager::{CoreManager, CoreManagerConfig, CoreManagerError, CoreState};
pub use permissions::{PermissionError, PermissionManager};
pub use process::{
    find_process_by_name, kill_process, Process, ProcessConfig, ProcessError, ProcessState,
};
pub use profile_updater::{update_profile, ProfileUpdater, ProfileUpdaterError};
pub use substore::{
    SubStoreArtifact, SubStoreClient, SubStoreCollection, SubStoreError, SubStoreSubscription,
};
