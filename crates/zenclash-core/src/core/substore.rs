use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum SubStoreError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Server not running")]
    ServerNotRunning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubStoreSubscription {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub url: String,
    #[serde(rename = "subInfoUrl")]
    pub sub_info_url: Option<String>,
    #[serde(rename = "type")]
    pub sub_type: Option<String>,
    pub tag: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubStoreCollection {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub subscriptions: Vec<String>,
    #[serde(rename = "output")]
    pub output: Option<String>,
    pub tag: Option<Vec<String>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubStoreArtifact {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub source: String,
}

pub struct SubStoreClient {
    base_url: String,
    client: Client,
}

impl SubStoreClient {
    pub fn new(port: u16) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{}", port),
            client: Client::new(),
        }
    }

    pub fn with_url(url: String) -> Self {
        Self {
            base_url: url,
            client: Client::new(),
        }
    }

    pub async fn check_health(&self) -> Result<bool, SubStoreError> {
        let response = self
            .client
            .get(format!("{}/api/health", self.base_url))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    pub async fn get_subscriptions(&self) -> Result<Vec<SubStoreSubscription>, SubStoreError> {
        let response = self
            .client
            .get(format!("{}/api/subscriptions", self.base_url))
            .send()
            .await?;

        let subs = response.json().await?;
        Ok(subs)
    }

    pub async fn get_subscription(
        &self,
        name: &str,
    ) -> Result<SubStoreSubscription, SubStoreError> {
        let response = self
            .client
            .get(format!("{}/api/subscriptions/{}", self.base_url, name))
            .send()
            .await?;

        let sub = response.json().await?;
        Ok(sub)
    }

    pub async fn add_subscription(&self, sub: &SubStoreSubscription) -> Result<(), SubStoreError> {
        let response = self
            .client
            .post(format!("{}/api/subscriptions", self.base_url))
            .json(sub)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(SubStoreError::Parse(text));
        }

        Ok(())
    }

    pub async fn update_subscription(
        &self,
        name: &str,
        sub: &SubStoreSubscription,
    ) -> Result<(), SubStoreError> {
        let response = self
            .client
            .patch(format!("{}/api/subscriptions/{}", self.base_url, name))
            .json(sub)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(SubStoreError::Parse(text));
        }

        Ok(())
    }

    pub async fn delete_subscription(&self, name: &str) -> Result<(), SubStoreError> {
        let response = self
            .client
            .delete(format!("{}/api/subscriptions/{}", self.base_url, name))
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(SubStoreError::Parse(text));
        }

        Ok(())
    }

    pub async fn get_collections(&self) -> Result<Vec<SubStoreCollection>, SubStoreError> {
        let response = self
            .client
            .get(format!("{}/api/collections", self.base_url))
            .send()
            .await?;

        let cols = response.json().await?;
        Ok(cols)
    }

    pub async fn get_collection(&self, name: &str) -> Result<SubStoreCollection, SubStoreError> {
        let response = self
            .client
            .get(format!("{}/api/collections/{}", self.base_url, name))
            .send()
            .await?;

        let col = response.json().await?;
        Ok(col)
    }

    pub async fn add_collection(&self, col: &SubStoreCollection) -> Result<(), SubStoreError> {
        let response = self
            .client
            .post(format!("{}/api/collections", self.base_url))
            .json(col)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(SubStoreError::Parse(text));
        }

        Ok(())
    }

    pub async fn update_collection(
        &self,
        name: &str,
        col: &SubStoreCollection,
    ) -> Result<(), SubStoreError> {
        let response = self
            .client
            .patch(format!("{}/api/collections/{}", self.base_url, name))
            .json(col)
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(SubStoreError::Parse(text));
        }

        Ok(())
    }

    pub async fn delete_collection(&self, name: &str) -> Result<(), SubStoreError> {
        let response = self
            .client
            .delete(format!("{}/api/collections/{}", self.base_url, name))
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(SubStoreError::Parse(text));
        }

        Ok(())
    }

    pub async fn get_artifacts(&self) -> Result<Vec<SubStoreArtifact>, SubStoreError> {
        let response = self
            .client
            .get(format!("{}/api/artifacts", self.base_url))
            .send()
            .await?;

        let artifacts = response.json().await?;
        Ok(artifacts)
    }

    pub async fn download_subscription(&self, name: &str) -> Result<String, SubStoreError> {
        let response = self
            .client
            .get(format!(
                "{}/api/subscriptions/{}/download",
                self.base_url, name
            ))
            .send()
            .await?;

        let content = response.text().await?;
        Ok(content)
    }

    pub async fn download_collection(&self, name: &str) -> Result<String, SubStoreError> {
        let response = self
            .client
            .get(format!(
                "{}/api/collections/{}/download",
                self.base_url, name
            ))
            .send()
            .await?;

        let content = response.text().await?;
        Ok(content)
    }
}
