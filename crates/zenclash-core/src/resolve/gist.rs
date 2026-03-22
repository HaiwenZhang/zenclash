use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum GistError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("GitHub API error: {0}")]
    ApiError(String),

    #[error("Gist not found")]
    NotFound,

    #[error("Unauthorized - check your GitHub token")]
    Unauthorized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistFile {
    pub filename: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub language: Option<String>,
    pub raw_url: String,
    pub size: u64,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gist {
    pub id: String,
    pub url: String,
    pub html_url: String,
    pub description: Option<String>,
    pub public: bool,
    pub created_at: String,
    pub updated_at: String,
    pub files: std::collections::HashMap<String, GistFile>,
}

#[derive(Debug, Clone, Serialize)]
struct CreateGistRequest {
    pub description: String,
    pub public: bool,
    pub files: std::collections::HashMap<String, GistFileContent>,
}

#[derive(Debug, Clone, Serialize)]
struct GistFileContent {
    pub content: String,
}

pub struct GistClient {
    client: Client,
    token: Option<String>,
}

impl GistClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            token: None,
        }
    }

    pub fn with_token(token: String) -> Self {
        let mut client = Self::new();
        client.token = Some(token);
        client
    }

    fn apply_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.token {
            builder.bearer_auth(token)
        } else {
            builder
        }
    }

    pub async fn get_gist(&self, gist_id: &str) -> Result<Gist, GistError> {
        let response = self
            .apply_auth(
                self.client
                    .get(format!("https://api.github.com/gists/{}", gist_id)),
            )
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "ZenClash")
            .send()
            .await?;

        match response.status().as_u16() {
            200 => Ok(response.json().await?),
            404 => Err(GistError::NotFound),
            401 | 403 => Err(GistError::Unauthorized),
            _ => Err(GistError::ApiError(
                response.text().await.unwrap_or_default(),
            )),
        }
    }

    pub async fn create_gist(
        &self,
        description: &str,
        public: bool,
        files: std::collections::HashMap<String, String>,
    ) -> Result<Gist, GistError> {
        let gist_files: std::collections::HashMap<String, GistFileContent> = files
            .into_iter()
            .map(|(name, content)| (name, GistFileContent { content }))
            .collect();

        let request = CreateGistRequest {
            description: description.to_string(),
            public,
            files: gist_files,
        };

        let response = self
            .apply_auth(self.client.post("https://api.github.com/gists"))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "ZenClash")
            .json(&request)
            .send()
            .await?;

        match response.status().as_u16() {
            201 => Ok(response.json().await?),
            401 | 403 => Err(GistError::Unauthorized),
            _ => Err(GistError::ApiError(
                response.text().await.unwrap_or_default(),
            )),
        }
    }

    pub async fn update_gist(
        &self,
        gist_id: &str,
        description: Option<&str>,
        files: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Gist, GistError> {
        let mut body = serde_json::json!({});

        if let Some(desc) = description {
            body["description"] = serde_json::json!(desc);
        }

        if let Some(files) = files {
            let gist_files: std::collections::HashMap<String, GistFileContent> = files
                .into_iter()
                .map(|(name, content)| (name, GistFileContent { content }))
                .collect();
            body["files"] = serde_json::to_value(gist_files).unwrap_or(serde_json::json!({}));
        }

        let response = self
            .apply_auth(
                self.client
                    .patch(format!("https://api.github.com/gists/{}", gist_id)),
            )
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "ZenClash")
            .json(&body)
            .send()
            .await?;

        match response.status().as_u16() {
            200 => Ok(response.json().await?),
            404 => Err(GistError::NotFound),
            401 | 403 => Err(GistError::Unauthorized),
            _ => Err(GistError::ApiError(
                response.text().await.unwrap_or_default(),
            )),
        }
    }

    pub async fn delete_gist(&self, gist_id: &str) -> Result<(), GistError> {
        let response = self
            .apply_auth(
                self.client
                    .delete(format!("https://api.github.com/gists/{}", gist_id)),
            )
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "ZenClash")
            .send()
            .await?;

        match response.status().as_u16() {
            204 => Ok(()),
            404 => Err(GistError::NotFound),
            401 | 403 => Err(GistError::Unauthorized),
            _ => Err(GistError::ApiError(
                response.text().await.unwrap_or_default(),
            )),
        }
    }

    pub async fn list_gists(&self) -> Result<Vec<Gist>, GistError> {
        let response = self
            .apply_auth(self.client.get("https://api.github.com/gists"))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "ZenClash")
            .send()
            .await?;

        match response.status().as_u16() {
            200 => Ok(response.json().await?),
            401 | 403 => Err(GistError::Unauthorized),
            _ => Err(GistError::ApiError(
                response.text().await.unwrap_or_default(),
            )),
        }
    }

    pub async fn get_gist_file_content(
        &self,
        gist_id: &str,
        filename: &str,
    ) -> Result<String, GistError> {
        let gist = self.get_gist(gist_id).await?;

        if let Some(file) = gist.files.get(filename) {
            if let Some(ref content) = file.content {
                return Ok(content.clone());
            }

            let response = self
                .client
                .get(&file.raw_url)
                .header("User-Agent", "ZenClash")
                .send()
                .await?;

            return Ok(response.text().await?);
        }

        Err(GistError::NotFound)
    }

    pub async fn sync_config(
        &self,
        gist_id: Option<&str>,
        config: &str,
    ) -> Result<String, GistError> {
        let mut files = std::collections::HashMap::new();
        files.insert("config.yaml".to_string(), config.to_string());

        if let Some(id) = gist_id {
            self.update_gist(id, Some("ZenClash Runtime Config"), Some(files))
                .await?;
            Ok(id.to_string())
        } else {
            let gist = self
                .create_gist("ZenClash Runtime Config", false, files)
                .await?;
            Ok(gist.id)
        }
    }
}

impl Default for GistClient {
    fn default() -> Self {
        Self::new()
    }
}
