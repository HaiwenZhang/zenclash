use futures_util::StreamExt;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    pub base_url: String,
    pub secret: Option<String>,
    pub timeout_secs: u64,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:9090".to_string(),
            secret: None,
            timeout_secs: 30,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Timeout")]
    Timeout,
}

#[derive(Clone)]
pub struct ApiClient {
    config: ApiClientConfig,
    client: Client,
}

impl ApiClient {
    pub fn new(config: ApiClientConfig) -> Result<Self, ApiError> {
        let mut builder = Client::builder().timeout(Duration::from_secs(config.timeout_secs));

        if let Some(secret) = &config.secret {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", secret))
                    .map_err(|e| ApiError::Parse(e.to_string()))?,
            );
            builder = builder.default_headers(headers);
        }

        let client = builder.build()?;

        Ok(Self { config, client })
    }

    pub fn new_default() -> Result<Self, ApiError> {
        Self::new(ApiClientConfig::default())
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.config.base_url.trim_end_matches('/'), path)
    }

    async fn request<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> Result<T, ApiError> {
        let url = self.url(path);
        let response = self.client.request(method, &url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::Api(format!("Status {}: {}", status, text)));
        }

        response.json().await.map_err(ApiError::Http)
    }

    async fn request_with_body<T: DeserializeOwned, B: Serialize>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: B,
    ) -> Result<T, ApiError> {
        let url = self.url(path);
        let response = self.client.request(method, &url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(ApiError::Api(format!("Status {}: {}", status, text)));
        }

        response.json().await.map_err(ApiError::Http)
    }

    pub async fn get_version(&self) -> Result<Version, ApiError> {
        self.request(reqwest::Method::GET, "/version").await
    }

    pub async fn get_config(&self) -> Result<RuntimeConfig, ApiError> {
        self.request(reqwest::Method::GET, "/configs").await
    }

    pub async fn patch_config(&self, config: &serde_json::Value) -> Result<(), ApiError> {
        self.request_with_body(reqwest::Method::PATCH, "/configs", config)
            .await
    }

    pub async fn patch_config_struct(&self, config: ConfigPatch) -> Result<(), ApiError> {
        self.request_with_body(reqwest::Method::PATCH, "/configs", config)
            .await
    }

    pub async fn reload_config(&self, path: Option<&str>) -> Result<(), ApiError> {
        let body = path
            .map(|p| serde_json::json!({ "path": p }))
            .unwrap_or_default();
        self.request_with_body(reqwest::Method::PUT, "/configs", body)
            .await
    }

    pub async fn get_proxies(&self) -> Result<ProxiesResponse, ApiError> {
        self.request(reqwest::Method::GET, "/proxies").await
    }

    pub async fn get_proxy(&self, name: &str) -> Result<ProxyItem, ApiError> {
        let encoded = urlencoding::encode(name);
        self.request(reqwest::Method::GET, &format!("/proxies/{}", encoded))
            .await
    }

    pub async fn select_proxy(&self, group: &str, proxy: &str) -> Result<(), ApiError> {
        let encoded = urlencoding::encode(group);
        let body = serde_json::json!({ "name": proxy });
        self.request_with_body(reqwest::Method::PUT, &format!("/proxies/{}", encoded), body)
            .await
    }

    pub async fn delay_test(
        &self,
        name: &str,
        url: Option<&str>,
        timeout: Option<u32>,
    ) -> Result<DelayTestResult, ApiError> {
        let encoded = urlencoding::encode(name);
        let mut query = String::new();
        if let Some(u) = url {
            query.push_str(&format!("url={}", urlencoding::encode(u)));
        }
        if let Some(t) = timeout {
            if !query.is_empty() {
                query.push('&');
            }
            query.push_str(&format!("timeout={}", t));
        }
        let path = if query.is_empty() {
            format!("/proxies/{}/delay", encoded)
        } else {
            format!("/proxies/{}/delay?{}", encoded, query)
        };
        self.request(reqwest::Method::GET, &path).await
    }

    pub async fn delay_test_group(
        &self,
        group: &str,
        url: Option<&str>,
        timeout: Option<u32>,
    ) -> Result<HashMap<String, DelayTestResult>, ApiError> {
        let encoded = urlencoding::encode(group);
        let mut query = String::new();
        if let Some(u) = url {
            query.push_str(&format!("url={}", urlencoding::encode(u)));
        }
        if let Some(t) = timeout {
            if !query.is_empty() {
                query.push('&');
            }
            query.push_str(&format!("timeout={}", t));
        }
        let path = if query.is_empty() {
            format!("/group/{}/delay", encoded)
        } else {
            format!("/group/{}/delay?{}", encoded, query)
        };
        self.request(reqwest::Method::GET, &path).await
    }

    pub async fn get_connections(&self) -> Result<ConnectionsResponse, ApiError> {
        self.request(reqwest::Method::GET, "/connections").await
    }

    pub async fn close_connection(&self, id: &str) -> Result<(), ApiError> {
        let url = self.url(&format!("/connections/{}", id));
        self.client.delete(&url).send().await?;
        Ok(())
    }

    pub async fn close_all_connections(&self) -> Result<(), ApiError> {
        let url = self.url("/connections");
        self.client.delete(&url).send().await?;
        Ok(())
    }

    pub async fn get_providers_proxies(&self) -> Result<ProvidersResponse, ApiError> {
        self.request(reqwest::Method::GET, "/providers/proxies")
            .await
    }

    pub async fn health_check_provider(&self, name: &str) -> Result<(), ApiError> {
        let encoded = urlencoding::encode(name);
        let url = self.url(&format!("/providers/proxies/{}/healthcheck", encoded));
        self.client.post(&url).send().await?;
        Ok(())
    }

    pub async fn upgrade_geo(&self) -> Result<(), ApiError> {
        let url = self.url("/configs/geo");
        self.client.post(&url).send().await?;
        Ok(())
    }

    pub async fn get_rules(&self) -> Result<RulesResponse, ApiError> {
        self.request(reqwest::Method::GET, "/rules").await
    }

    pub async fn get_traffic(&self) -> Result<TrafficStream, ApiError> {
        let ws_url = self
            .config
            .base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let url = format!("{}/traffic", ws_url);

        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| ApiError::WebSocket(e.to_string()))?;

        Ok(TrafficStream {
            inner: Arc::new(Mutex::new(ws_stream)),
        })
    }

    pub async fn get_connections_stream(&self) -> Result<ConnectionsStream, ApiError> {
        let ws_url = self
            .config
            .base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let url = format!("{}/connections", ws_url);

        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| ApiError::WebSocket(e.to_string()))?;

        Ok(ConnectionsStream {
            inner: Arc::new(Mutex::new(ws_stream)),
        })
    }

    pub async fn get_logs(&self, level: Option<&str>) -> Result<LogStream, ApiError> {
        let ws_url = self
            .config
            .base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let url = if let Some(l) = level {
            format!("{}/logs?level={}", ws_url, l)
        } else {
            format!("{}/logs", ws_url)
        };

        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| ApiError::WebSocket(e.to_string()))?;

        Ok(LogStream {
            inner: Arc::new(Mutex::new(ws_stream)),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub version: String,
    #[serde(default)]
    pub premium: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub port: Option<u16>,
    pub socks_port: Option<u16>,
    pub mixed_port: Option<u16>,
    pub redir_port: Option<u16>,
    pub tproxy_port: Option<u16>,
    pub allow_lan: bool,
    pub bind_address: Option<String>,
    pub mode: String,
    pub log_level: String,
    pub ipv6: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_lan: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxiesResponse {
    pub proxies: HashMap<String, ProxyItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyItem {
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub name: String,
    #[serde(default)]
    pub now: Option<String>,
    #[serde(default)]
    pub alive: Option<bool>,
    #[serde(default)]
    pub history: Vec<DelayHistory>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub all: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayHistory {
    pub time: Option<String>,
    pub delay: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayTestResult {
    pub delay: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionsResponse {
    pub download_total: u64,
    pub upload_total: u64,
    pub connections: Vec<ConnectionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionItem {
    pub id: String,
    pub metadata: ConnectionMetadata,
    pub upload: u64,
    pub download: u64,
    pub start: String,
    #[serde(default)]
    pub chains: Vec<String>,
    #[serde(default)]
    pub rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetadata {
    pub network: String,
    #[serde(rename = "type")]
    pub conn_type: String,
    pub host: Option<String>,
    pub source_ip: String,
    pub source_port: String,
    pub destination_ip: Option<String>,
    pub destination_port: String,
    pub process: Option<String>,
    pub process_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersResponse {
    pub providers: HashMap<String, ProviderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderItem {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: String,
    pub proxies: Vec<ProxyItem>,
    #[serde(default)]
    pub vehicle_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesResponse {
    pub rules: Vec<RuleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleItem {
    #[serde(rename = "type")]
    pub rule_type: String,
    pub payload: String,
    pub proxy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficData {
    pub up: u64,
    pub down: u64,
}

pub struct TrafficStream {
    inner: Arc<
        Mutex<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    >,
}

impl TrafficStream {
    pub async fn next(&self) -> Option<TrafficData> {
        let mut stream = self.inner.lock().await;
        while let Some(msg) = stream.next().await {
            if let Ok(WsMessage::Text(text)) = msg {
                if let Ok(data) = serde_json::from_str::<TrafficData>(&text) {
                    return Some(data);
                }
            }
        }
        None
    }
}

pub struct ConnectionsStream {
    inner: Arc<
        Mutex<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    >,
}

impl ConnectionsStream {
    pub async fn next(&self) -> Option<ConnectionsResponse> {
        let mut stream = self.inner.lock().await;
        while let Some(msg) = stream.next().await {
            if let Ok(WsMessage::Text(text)) = msg {
                if let Ok(data) = serde_json::from_str::<ConnectionsResponse>(&text) {
                    return Some(data);
                }
            }
        }
        None
    }
}

pub struct LogStream {
    inner: Arc<
        Mutex<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    >,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogItem {
    #[serde(rename = "type")]
    pub level: String,
    pub payload: String,
}

impl LogStream {
    pub async fn next(&self) -> Option<LogItem> {
        let mut stream = self.inner.lock().await;
        while let Some(msg) = stream.next().await {
            if let Ok(WsMessage::Text(text)) = msg {
                if let Ok(data) = serde_json::from_str::<LogItem>(&text) {
                    return Some(data);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_config_default() {
        let config = ApiClientConfig::default();
        assert_eq!(config.base_url, "http://127.0.0.1:9090");
        assert!(config.secret.is_none());
    }

    #[test]
    fn test_api_client_new() {
        let client = ApiClient::new_default();
        assert!(client.is_ok());
    }
}
