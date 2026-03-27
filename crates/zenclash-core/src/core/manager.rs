use crate::config::{AppConfig, MihomoConfig, ProfileConfig};
use crate::core::api::{ApiClient, ApiClientConfig};
use crate::core::process::{Process, ProcessConfig};
use crate::sysproxy::{ProxyConfig, ProxyMode, SystemProxyManager};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

#[derive(Debug, thiserror::Error)]
pub enum CoreManagerError {
    #[error("Core is already running")]
    AlreadyRunning,

    #[error("Core is not running")]
    NotRunning,

    #[error("Failed to start core: {0}")]
    StartError(String),

    #[error("Failed to stop core: {0}")]
    StopError(String),

    #[error("Failed to reload config: {0}")]
    ReloadError(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("API error: {0}")]
    ApiError(#[from] crate::core::api::ApiError),

    #[error("Process error: {0}")]
    ProcessError(#[from] crate::core::process::ProcessError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    ConfigError(String),
}

pub struct CoreManagerConfig {
    pub core_path: PathBuf,
    pub work_dir: PathBuf,
    pub config_path: PathBuf,
    pub api_url: String,
    pub api_secret: Option<String>,
}

impl CoreManagerConfig {
    pub fn from_app_config(app: &AppConfig) -> Self {
        Self {
            core_path: crate::utils::dirs::mihomo_core_dir().join("mihomo"),
            work_dir: crate::utils::dirs::data_dir(),
            config_path: crate::utils::dirs::config_dir().join("config.yaml"),
            api_url: format!("http://127.0.0.1:{}", app.api_port),
            api_secret: app.api_secret.clone(),
        }
    }
}

pub struct CoreManager {
    config: CoreManagerConfig,
    process: Arc<Process>,
    state: Arc<RwLock<CoreState>>,
    api_client: Arc<RwLock<Option<ApiClient>>>,
}

impl CoreManager {
    pub fn new(config: CoreManagerConfig) -> Self {
        let process_config = ProcessConfig::new(config.core_path.clone())
            .args(vec![
                "-d".to_string(),
                config.work_dir.to_string_lossy().to_string(),
            ])
            .work_dir(config.work_dir.clone());

        let process = Process::new(process_config);

        Self {
            config,
            process: Arc::new(process),
            state: Arc::new(RwLock::new(CoreState::Stopped)),
            api_client: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn state(&self) -> CoreState {
        *self.state.read().await
    }

    pub async fn is_running(&self) -> bool {
        matches!(self.state().await, CoreState::Running)
    }

    pub async fn start(&self) -> Result<(), CoreManagerError> {
        let mut state = self.state.write().await;

        if *state == CoreState::Running {
            return Err(CoreManagerError::AlreadyRunning);
        }

        *state = CoreState::Starting;
        drop(state);

        self.ensure_config_file().await?;

        if let Err(e) = self.process.start().await {
            let mut state = self.state.write().await;
            *state = CoreState::Error;
            return Err(CoreManagerError::StartError(e.to_string()));
        }

        let api_client = ApiClient::new(ApiClientConfig {
            base_url: self.config.api_url.clone(),
            secret: self.config.api_secret.clone(),
            timeout_secs: 10,
        })
        .map_err(|e| CoreManagerError::StartError(e.to_string()))?;

        let mut client = self.api_client.write().await;
        *client = Some(api_client);

        let mut state = self.state.write().await;
        *state = CoreState::Running;

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), CoreManagerError> {
        let mut state = self.state.write().await;

        if *state != CoreState::Running {
            return Err(CoreManagerError::NotRunning);
        }

        *state = CoreState::Stopping;
        drop(state);

        if let Err(e) = self.process.stop().await {
            let mut state = self.state.write().await;
            *state = CoreState::Error;
            return Err(CoreManagerError::StopError(e.to_string()));
        }

        let mut client = self.api_client.write().await;
        *client = None;

        let mut state = self.state.write().await;
        *state = CoreState::Stopped;

        Ok(())
    }

    pub async fn restart(&self) -> Result<(), CoreManagerError> {
        if self.is_running().await {
            self.stop().await?;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        self.start().await
    }

    pub async fn reload(&self) -> Result<(), CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        client
            .reload_config(None)
            .await
            .map_err(|e| CoreManagerError::ReloadError(e.to_string()))?;

        Ok(())
    }

    pub async fn load_profile(&self, profile_id: &str) -> Result<(), CoreManagerError> {
        let profile_config = ProfileConfig::load()?;
        let profile = profile_config
            .get(profile_id)
            .ok_or_else(|| CoreManagerError::ProfileNotFound(profile_id.to_string()))?;

        let profile_path = profile
            .file_path()
            .ok_or_else(|| CoreManagerError::ConfigError("No profile path".to_string()))?;

        let mihomo_config = MihomoConfig::load(&profile_path)?;

        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        let config_yaml = mihomo_config
            .to_yaml()
            .map_err(|e| CoreManagerError::ConfigError(e.to_string()))?;

        client
            .reload_config(Some(&config_yaml))
            .await
            .map_err(|e| CoreManagerError::ReloadError(e.to_string()))?;

        Ok(())
    }

    pub async fn api_client(&self) -> Option<ApiClient> {
        let client = self.api_client.read().await;
        client.clone()
    }

    async fn ensure_config_file(&self) -> std::io::Result<()> {
        if !self.config.config_path.exists() {
            let config = MihomoConfig::default();
            config.save(&self.config.config_path)?;
        }
        Ok(())
    }

    pub async fn get_version(&self) -> Result<String, CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        let version = client.get_version().await?;
        Ok(version.version)
    }

    pub async fn get_traffic(&self) -> Result<crate::core::api::TrafficStream, CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        client.get_traffic().await.map_err(Into::into)
    }

    pub async fn get_connections(
        &self,
    ) -> Result<crate::core::api::ConnectionsResponse, CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        client.get_connections().await.map_err(Into::into)
    }

    pub async fn get_proxies(&self) -> Result<crate::core::api::ProxiesResponse, CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        client.get_proxies().await.map_err(Into::into)
    }

    pub async fn select_proxy(&self, group: &str, proxy: &str) -> Result<(), CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        client.select_proxy(group, proxy).await.map_err(Into::into)
    }

    pub async fn delay_test(
        &self,
        name: &str,
        url: Option<&str>,
        timeout: Option<u32>,
    ) -> Result<crate::core::api::DelayTestResult, CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        client
            .delay_test(name, url, timeout)
            .await
            .map_err(Into::into)
    }

    pub async fn enable_sysproxy(&self) -> Result<(), CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        let runtime_config = client.get_config().await?;
        let port = runtime_config.mixed_port.unwrap_or(7890);

        let proxy_config = ProxyConfig {
            mode: ProxyMode::Manual,
            host: "127.0.0.1".into(),
            port,
            bypass: crate::sysproxy::default_bypass(),
            pac_url: None,
        };

        SystemProxyManager::enable(&proxy_config)
            .map_err(|e| CoreManagerError::ConfigError(e.to_string()))?;

        Ok(())
    }

    pub async fn disable_sysproxy(&self) -> Result<(), CoreManagerError> {
        SystemProxyManager::disable().map_err(|e| CoreManagerError::ConfigError(e.to_string()))?;

        Ok(())
    }

    pub async fn enable_tun(&self) -> Result<(), CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        let config = serde_json::json!({
            "tun": {
                "enable": true
            }
        });

        client
            .patch_config(&config)
            .await
            .map_err(|e| CoreManagerError::ApiError(e))?;

        Ok(())
    }

    pub async fn disable_tun(&self) -> Result<(), CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        let config = serde_json::json!({
            "tun": {
                "enable": false
            }
        });

        client
            .patch_config(&config)
            .await
            .map_err(|e| CoreManagerError::ApiError(e))?;

        Ok(())
    }

    pub async fn set_mode(&self, mode: &str) -> Result<(), CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        let config = serde_json::json!({
            "mode": mode
        });

        client
            .patch_config(&config)
            .await
            .map_err(|e| CoreManagerError::ApiError(e))?;

        Ok(())
    }

    pub async fn close_connection(&self, id: &str) -> Result<(), CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        client.close_connection(id).await.map_err(Into::into)
    }

    pub async fn close_all_connections(&self) -> Result<(), CoreManagerError> {
        let client = self.api_client.read().await;
        let client = client.as_ref().ok_or(CoreManagerError::NotRunning)?;

        client.close_all_connections().await.map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_state() {
        assert_eq!(CoreState::Stopped, CoreState::Stopped);
        assert_ne!(CoreState::Stopped, CoreState::Running);
    }

    #[test]
    fn test_core_manager_config_from_app() {
        let app = AppConfig::default();
        let config = CoreManagerConfig::from_app_config(&app);
        assert!(config.api_url.starts_with("http://"));
    }
}
