use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverrideConfig {
    #[serde(default)]
    pub items: Vec<OverrideItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideItem {
    #[serde(rename = "uid")]
    pub id: String,
    pub name: String,
    pub rules: Vec<OverrideRule>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub profiles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OverrideRule {
    #[serde(rename = "mixin")]
    Mixin(MixinRule),
    #[serde(rename = "script")]
    Script(ScriptRule),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixinRule {
    #[serde(default)]
    pub prepend: Option<serde_yaml::Value>,
    #[serde(default)]
    pub append: Option<serde_yaml::Value>,
    #[serde(default)]
    pub replace: Option<HashMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptRule {
    pub script: String,
    #[serde(default)]
    pub path: Option<String>,
}

impl OverrideConfig {
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

    pub fn config_path() -> std::path::PathBuf {
        crate::utils::dirs::config_dir().join("overrides.yaml")
    }

    pub fn get(&self, id: &str) -> Option<&OverrideItem> {
        self.items.iter().find(|p| p.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut OverrideItem> {
        self.items.iter_mut().find(|p| p.id == id)
    }

    pub fn add(&mut self, item: OverrideItem) {
        if !self.items.iter().any(|p| p.id == item.id) {
            self.items.push(item);
        }
    }

    pub fn remove(&mut self, id: &str) -> Option<OverrideItem> {
        let pos = self.items.iter().position(|p| p.id == id)?;
        Some(self.items.remove(pos))
    }

    pub fn get_for_profile(&self, profile_id: &str) -> Vec<&OverrideItem> {
        self.items
            .iter()
            .filter(|item| {
                item.enabled
                    && (item.profiles.is_empty() || item.profiles.contains(&profile_id.to_string()))
            })
            .collect()
    }
}

impl OverrideItem {
    pub fn new(name: String, rules: Vec<OverrideRule>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            rules,
            enabled: true,
            profiles: vec![],
        }
    }

    pub fn apply(
        &self,
        config: &mut crate::config::mihomo::MihomoConfig,
    ) -> Result<(), OverrideError> {
        for rule in &self.rules {
            match rule {
                OverrideRule::Mixin(mixin) => self.apply_mixin(config, mixin)?,
                OverrideRule::Script(script) => self.apply_script(config, script)?,
            }
        }
        Ok(())
    }

    fn apply_mixin(
        &self,
        config: &mut crate::config::mihomo::MihomoConfig,
        mixin: &MixinRule,
    ) -> Result<(), OverrideError> {
        if let Some(prepend) = &mixin.prepend {
            self.merge_prepend(config, prepend)?;
        }
        if let Some(append) = &mixin.append {
            self.merge_append(config, append)?;
        }
        if let Some(replace) = &mixin.replace {
            self.merge_replace(config, replace)?;
        }
        Ok(())
    }

    fn merge_prepend(
        &self,
        _config: &mut crate::config::mihomo::MihomoConfig,
        _value: &serde_yaml::Value,
    ) -> Result<(), OverrideError> {
        Ok(())
    }

    fn merge_append(
        &self,
        _config: &mut crate::config::mihomo::MihomoConfig,
        _value: &serde_yaml::Value,
    ) -> Result<(), OverrideError> {
        Ok(())
    }

    fn merge_replace(
        &self,
        config: &mut crate::config::mihomo::MihomoConfig,
        replace: &HashMap<String, serde_yaml::Value>,
    ) -> Result<(), OverrideError> {
        for (key, value) in replace {
            match key.as_str() {
                "mode" => {
                    if let Some(v) = value.as_str() {
                        config.mode = v.to_string();
                    }
                }
                "log_level" => {
                    if let Some(v) = value.as_str() {
                        config.log_level = v.to_string();
                    }
                }
                "allow_lan" => {
                    if let Some(v) = value.as_bool() {
                        config.allow_lan = v;
                    }
                }
                "ipv6" => {
                    if let Some(v) = value.as_bool() {
                        config.ipv6 = v;
                    }
                }
                "unified_delay" => {
                    if let Some(v) = value.as_bool() {
                        config.unified_delay = v;
                    }
                }
                "tcp_concurrent" => {
                    if let Some(v) = value.as_bool() {
                        config.tcp_concurrent = v;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn apply_script(
        &self,
        _config: &mut crate::config::mihomo::MihomoConfig,
        _script: &ScriptRule,
    ) -> Result<(), OverrideError> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OverrideError {
    #[error("Failed to apply mixin: {0}")]
    MixinError(String),

    #[error("Failed to apply script: {0}")]
    ScriptError(String),

    #[error("Invalid YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_override_item_new() {
        let item = OverrideItem::new("test".to_string(), vec![]);
        assert_eq!(item.name, "test");
        assert!(item.enabled);
    }

    #[test]
    fn test_override_config_get_for_profile() {
        let mut config = OverrideConfig::default();
        let item1 = OverrideItem {
            id: "1".to_string(),
            name: "global".to_string(),
            rules: vec![],
            enabled: true,
            profiles: vec![],
        };
        let item2 = OverrideItem {
            id: "2".to_string(),
            name: "specific".to_string(),
            rules: vec![],
            enabled: true,
            profiles: vec!["profile-1".to_string()],
        };
        config.add(item1);
        config.add(item2);

        let result = config.get_for_profile("profile-1");
        assert_eq!(result.len(), 2);

        let result = config.get_for_profile("profile-2");
        assert_eq!(result.len(), 1);
    }
}
