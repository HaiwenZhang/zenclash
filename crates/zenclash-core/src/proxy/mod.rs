pub mod delay_test;
pub mod proxy;
pub mod selector;

#[cfg(test)]
mod tests;

pub use delay_test::{
    test_all_proxies, url_test, DelayTestConfig, DelayTestError, DelayTestResult, DelayTestStatus,
    DelayTester,
};
pub use proxy::{
    HealthCheck, HealthCheckConfig, Proxy, ProxyCollection, ProxyGroup, ProxyGroupType,
    ProxyOrGroup, ProxyProvider, ProxyType,
};
pub use selector::{ProxySelector, SelectionError, SelectionStrategy};
