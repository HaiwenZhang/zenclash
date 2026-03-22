use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    #[serde(default)]
    pub items: Vec<ProfileItem>,
    #[serde(default)]
    pub current: Option<String>,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            items: vec![],
            current: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileItem {
    #[serde(rename = "uid")]
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub profile_type: ProfileType,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub interval: Option<u64>,
    #[serde(default)]
    pub last_update: Option<DateTime<Utc>>,
    #[serde(default)]
    pub sub_info: Option<SubscriptionInfo>,
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default)]
    pub updated: Option<DateTime<Utc>>,
    #[serde(default)]
    pub extra: ProfileExtra,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProfileType {
    Local,
    Remote,
}

impl std::fmt::Display for ProfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileType::Local => write!(f, "local"),
            ProfileType::Remote => write!(f, "remote"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SubscriptionInfo {
    #[serde(default)]
    pub upload: u64,
    #[serde(default)]
    pub download: u64,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub expire: Option<DateTime<Utc>>,
}

impl SubscriptionInfo {
    pub fn used(&self) -> u64 {
        self.upload + self.download
    }

    pub fn remaining(&self) -> u64 {
        self.total.saturating_sub(self.used())
    }

    pub fn usage_percent(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.used() as f64 / self.total as f64) * 100.0
    }

    pub fn days_remaining(&self) -> Option<i64> {
        self.expire.map(|exp| {
            let now = Utc::now();
            (exp - now).num_days()
        })
    }

    pub fn is_expired(&self) -> bool {
        self.expire.map(|exp| exp < Utc::now()).unwrap_or(false)
    }

    pub fn is_exhausted(&self) -> bool {
        self.total > 0 && self.used() >= self.total
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileExtra {
    #[serde(default)]
    pub proxy_count: Option<usize>,
    #[serde(default)]
    pub rule_count: Option<usize>,
    #[serde(default)]
    pub rule_provider_count: Option<usize>,
    #[serde(default)]
    pub proxy_provider_count: Option<usize>,
}

impl ProfileConfig {
    pub fn load() -> std::io::Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        let content = std::fs::read_to_string(&path)?;
        serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&path, content)
    }

    pub fn config_path() -> PathBuf {
        crate::utils::dirs::config_dir().join("profiles.yaml")
    }

    pub fn get(&self, id: &str) -> Option<&ProfileItem> {
        self.items.iter().find(|p| p.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut ProfileItem> {
        self.items.iter_mut().find(|p| p.id == id)
    }

    pub fn add(&mut self, item: ProfileItem) {
        if !self.items.iter().any(|p| p.id == item.id) {
            self.items.push(item);
        }
    }

    pub fn remove(&mut self, id: &str) -> Option<ProfileItem> {
        let pos = self.items.iter().position(|p| p.id == id)?;
        Some(self.items.remove(pos))
    }

    pub fn set_current(&mut self, id: &str) -> bool {
        if self.items.iter().any(|p| p.id == id) {
            self.current = Some(id.to_string());
            true
        } else {
            false
        }
    }

    pub fn current(&self) -> Option<&ProfileItem> {
        self.current.as_ref().and_then(|id| self.get(id))
    }
}

impl ProfileItem {
    pub fn new_local(name: String, path: PathBuf) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            profile_type: ProfileType::Local,
            url: None,
            path: Some(path),
            interval: None,
            last_update: None,
            sub_info: None,
            auto_update: false,
            updated: None,
            extra: ProfileExtra::default(),
        }
    }

    pub fn new_remote(name: String, url: String, interval: Option<u64>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            profile_type: ProfileType::Remote,
            url: Some(url),
            path: None,
            interval,
            last_update: None,
            sub_info: None,
            auto_update: interval.is_some(),
            updated: None,
            extra: ProfileExtra::default(),
        }
    }

    pub fn file_path(&self) -> Option<PathBuf> {
        if let Some(p) = &self.path {
            return Some(p.clone());
        }
        Some(
            crate::utils::dirs::profiles_dir()
                .join(&self.id)
                .with_extension("yaml"),
        )
    }

    pub fn needs_update(&self) -> bool {
        if !self.auto_update {
            return false;
        }
        let interval = match self.interval {
            Some(i) => i,
            None => return false,
        };
        let last = match self.last_update {
            Some(t) => t,
            None => return true,
        };
        let now = Utc::now();
        (now - last).num_seconds() as u64 >= interval * 3600
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_info_usage() {
        let info = SubscriptionInfo {
            upload: 100,
            download: 900,
            total: 2000,
            expire: None,
        };
        assert_eq!(info.used(), 1000);
        assert_eq!(info.remaining(), 1000);
        assert_eq!(info.usage_percent(), 50.0);
    }

    #[test]
    fn test_profile_item_new_local() {
        let item = ProfileItem::new_local("test".to_string(), PathBuf::from("/tmp/test.yaml"));
        assert_eq!(item.name, "test");
        assert_eq!(item.profile_type, ProfileType::Local);
        assert!(item.url.is_none());
    }

    #[test]
    fn test_profile_item_new_remote() {
        let item = ProfileItem::new_remote(
            "test".to_string(),
            "http://example.com".to_string(),
            Some(24),
        );
        assert_eq!(item.name, "test");
        assert_eq!(item.profile_type, ProfileType::Remote);
        assert!(item.url.is_some());
        assert!(item.auto_update);
    }
}
