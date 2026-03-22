use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout_secs: u64,
    pub connect_timeout_secs: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub user_agent: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            connect_timeout_secs: 10,
            max_retries: 3,
            retry_delay_ms: 1000,
            user_agent: format!("zenclash/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("HTTP error status: {0}")]
    StatusError(StatusCode),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Max retries exceeded")]
    MaxRetriesExceeded,

    #[error("Request timeout")]
    Timeout,
}

pub struct HttpClient {
    client: Client,
    config: HttpClientConfig,
}

impl HttpClient {
    pub fn new(config: HttpClientConfig) -> Result<Self, HttpError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .connect_timeout(Duration::from_secs(config.connect_timeout_secs))
            .user_agent(&config.user_agent)
            .danger_accept_invalid_certs(false)
            .build()?;

        Ok(Self { client, config })
    }

    pub fn new_default() -> Result<Self, HttpError> {
        Self::new(HttpClientConfig::default())
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    async fn execute_with_retry(&self, request: reqwest::Request) -> Result<Response, HttpError> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
            }

            let request_clone = request.try_clone().ok_or_else(|| {
                HttpError::ParseError("Request body cannot be cloned for retry".to_string())
            })?;

            match self.client.execute(request_clone).await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if response.status().is_server_error() {
                        last_error = Some(HttpError::StatusError(response.status()));
                        continue;
                    } else {
                        return Err(HttpError::StatusError(response.status()));
                    }
                },
                Err(e) => {
                    if e.is_timeout() {
                        last_error = Some(HttpError::Timeout);
                    } else if e.is_connect() {
                        last_error = Some(HttpError::RequestError(e));
                        continue;
                    } else {
                        return Err(HttpError::RequestError(e));
                    }
                },
            }
        }

        Err(last_error.unwrap_or(HttpError::MaxRetriesExceeded))
    }

    pub async fn get(&self, url: &str) -> Result<Response, HttpError> {
        let request = self.client.get(url).build()?;
        self.execute_with_retry(request).await
    }

    pub async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, HttpError> {
        let response = self.get(url).await?;
        response
            .json()
            .await
            .map_err(|e| HttpError::ParseError(e.to_string()))
    }

    pub async fn get_text(&self, url: &str) -> Result<String, HttpError> {
        let response = self.get(url).await?;
        response.text().await.map_err(HttpError::RequestError)
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, HttpError> {
        let response = self.get(url).await?;
        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(HttpError::RequestError)
    }

    pub async fn post(
        &self,
        url: &str,
        body: impl serde::Serialize,
    ) -> Result<Response, HttpError> {
        let request = self.client.post(url).json(&body).build()?;
        self.execute_with_retry(request).await
    }

    pub async fn post_json<T: DeserializeOwned>(
        &self,
        url: &str,
        body: impl serde::Serialize,
    ) -> Result<T, HttpError> {
        let response = self.post(url, body).await?;
        response
            .json()
            .await
            .map_err(|e| HttpError::ParseError(e.to_string()))
    }

    pub async fn put(&self, url: &str, body: impl serde::Serialize) -> Result<Response, HttpError> {
        let request = self.client.put(url).json(&body).build()?;
        self.execute_with_retry(request).await
    }

    pub async fn delete(&self, url: &str) -> Result<Response, HttpError> {
        let request = self.client.delete(url).build()?;
        self.execute_with_retry(request).await
    }

    pub async fn patch(
        &self,
        url: &str,
        body: impl serde::Serialize,
    ) -> Result<Response, HttpError> {
        let request = self.client.patch(url).json(&body).build()?;
        self.execute_with_retry(request).await
    }

    pub async fn head(&self, url: &str) -> Result<Response, HttpError> {
        let request = self.client.head(url).build()?;
        self.execute_with_retry(request).await
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Result<Self, HttpError> {
        self.config.timeout_secs = timeout_secs;
        self.client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .connect_timeout(Duration::from_secs(self.config.connect_timeout_secs))
            .user_agent(&self.config.user_agent)
            .build()?;
        Ok(self)
    }

    pub fn with_auth(mut self, token: &str) -> Result<Self, HttpError> {
        let header_value = format!("Bearer {}", token);
        self.client = Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .connect_timeout(Duration::from_secs(self.config.connect_timeout_secs))
            .user_agent(&self.config.user_agent)
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&header_value)
                        .map_err(|e| HttpError::ParseError(e.to_string()))?,
                );
                headers
            })
            .build()?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_config_default() {
        let config = HttpClientConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.connect_timeout_secs, 10);
        assert_eq!(config.max_retries, 3);
        assert!(config.user_agent.starts_with("zenclash/"));
    }

    #[test]
    fn test_http_client_new() {
        let client = HttpClient::new_default();
        assert!(client.is_ok());
    }
}
