use crate::core::api::ApiClient;
use crate::proxy::proxy::ProxyCollection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayTestStatus {
    Pending,
    Testing,
    Success,
    Failed,
    Timeout,
}

#[derive(Debug, Clone)]
pub struct DelayTestResult {
    pub name: String,
    pub delay: Option<u32>,
    pub status: DelayTestStatus,
    pub error: Option<String>,
}

impl DelayTestResult {
    pub fn pending(name: &str) -> Self {
        Self {
            name: name.to_string(),
            delay: None,
            status: DelayTestStatus::Pending,
            error: None,
        }
    }

    pub fn success(name: &str, delay: u32) -> Self {
        Self {
            name: name.to_string(),
            delay: Some(delay),
            status: DelayTestStatus::Success,
            error: None,
        }
    }

    pub fn failed(name: &str, error: &str) -> Self {
        Self {
            name: name.to_string(),
            delay: None,
            status: DelayTestStatus::Failed,
            error: Some(error.to_string()),
        }
    }

    pub fn timeout(name: &str) -> Self {
        Self {
            name: name.to_string(),
            delay: None,
            status: DelayTestStatus::Timeout,
            error: Some("Timeout".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DelayTestConfig {
    pub url: String,
    pub timeout_ms: u32,
    pub concurrent: usize,
}

impl Default for DelayTestConfig {
    fn default() -> Self {
        Self {
            url: "http://www.gstatic.com/generate_204".to_string(),
            timeout_ms: 5000,
            concurrent: 10,
        }
    }
}

#[derive(Clone)]
pub struct DelayTester {
    client: ApiClient,
    config: DelayTestConfig,
    results: Arc<RwLock<HashMap<String, DelayTestResult>>>,
}

impl DelayTester {
    pub fn new(client: ApiClient, config: DelayTestConfig) -> Self {
        Self {
            client,
            config,
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn new_default(client: ApiClient) -> Self {
        Self::new(client, DelayTestConfig::default())
    }

    pub async fn test_single(&self, name: &str) -> DelayTestResult {
        {
            let mut results = self.results.write().await;
            results.insert(
                name.to_string(),
                DelayTestResult {
                    name: name.to_string(),
                    delay: None,
                    status: DelayTestStatus::Testing,
                    error: None,
                },
            );
        }

        match self
            .client
            .delay_test(name, Some(&self.config.url), Some(self.config.timeout_ms))
            .await
        {
            Ok(result) => {
                let res = DelayTestResult::success(name, result.delay);
                let mut results = self.results.write().await;
                results.insert(name.to_string(), res.clone());
                res
            },
            Err(e) => {
                let status = if e.to_string().contains("timeout") {
                    DelayTestStatus::Timeout
                } else {
                    DelayTestStatus::Failed
                };
                let res = DelayTestResult {
                    name: name.to_string(),
                    delay: None,
                    status,
                    error: Some(e.to_string()),
                };
                let mut results = self.results.write().await;
                results.insert(name.to_string(), res.clone());
                res
            },
        }
    }

    pub async fn test_group(&self, group_name: &str) -> HashMap<String, DelayTestResult> {
        let proxies = match self.client.get_proxy(group_name).await {
            Ok(group) => group.all.unwrap_or_default(),
            Err(_) => return HashMap::new(),
        };

        let mut results = HashMap::new();
        let mut tasks = vec![];

        for proxy in proxies {
            let tester = self.clone();
            let name = proxy.clone();
            tasks.push(tokio::spawn(async move { tester.test_single(&name).await }));
        }

        for task in tasks {
            if let Ok(result) = task.await {
                results.insert(result.name.clone(), result);
            }
        }

        let mut stored = self.results.write().await;
        for (name, result) in &results {
            stored.insert(name.clone(), result.clone());
        }

        results
    }

    pub async fn test_all(&self, collection: &ProxyCollection) -> HashMap<String, DelayTestResult> {
        let mut results = HashMap::new();
        let names: Vec<String> = collection.proxies.keys().cloned().collect();

        let chunks: Vec<Vec<String>> = names
            .chunks(self.config.concurrent)
            .map(|c| c.to_vec())
            .collect();

        for chunk in chunks {
            let mut tasks = vec![];

            for name in chunk {
                let tester = self.clone();
                let n = name.clone();
                tasks.push(tokio::spawn(async move { tester.test_single(&n).await }));
            }

            for task in tasks {
                if let Ok(result) = task.await {
                    results.insert(result.name.clone(), result);
                }
            }
        }

        let mut stored = self.results.write().await;
        for (name, result) in &results {
            stored.insert(name.clone(), result.clone());
        }

        results
    }

    pub async fn get_result(&self, name: &str) -> Option<DelayTestResult> {
        let results = self.results.read().await;
        results.get(name).cloned()
    }

    pub async fn get_all_results(&self) -> HashMap<String, DelayTestResult> {
        let results = self.results.read().await;
        results.clone()
    }

    pub async fn clear_results(&self) {
        let mut results = self.results.write().await;
        results.clear();
    }
}

pub async fn url_test(
    client: &ApiClient,
    proxy_name: &str,
    url: &str,
    timeout_ms: u32,
) -> Result<u32, DelayTestError> {
    let result = client
        .delay_test(proxy_name, Some(url), Some(timeout_ms))
        .await
        .map_err(DelayTestError::Api)?;

    if result.delay > 0 {
        Ok(result.delay)
    } else {
        Err(DelayTestError::Timeout)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DelayTestError {
    #[error("API error: {0}")]
    Api(#[from] crate::core::api::ApiError),

    #[error("Timeout")]
    Timeout,

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_test_result() {
        let result = DelayTestResult::success("test", 100);
        assert_eq!(result.delay, Some(100));
        assert_eq!(result.status, DelayTestStatus::Success);

        let failed = DelayTestResult::failed("test", "error");
        assert_eq!(failed.status, DelayTestStatus::Failed);
    }

    #[test]
    fn test_delay_test_config_default() {
        let config = DelayTestConfig::default();
        assert_eq!(config.timeout_ms, 5000);
        assert!(config.url.starts_with("http"));
    }
}
