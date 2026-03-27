use crate::config::{MihomoConfig, OverrideItem};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum FactoryError {
    #[error("Failed to parse config: {0}")]
    ParseError(String),

    #[error("Failed to merge config: {0}")]
    MergeError(String),

    #[error("Failed to execute script override: {0}")]
    ScriptExecutionError(String),

    #[error("Failed to process rules: {0}")]
    RuleError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Override error: {0}")]
    OverrideError(#[from] crate::config::OverrideError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFileAction {
    pub prepend: Vec<String>,
    pub append: Vec<String>,
    pub delete: Vec<String>,
}

pub struct ConfigFactory;

impl ConfigFactory {
    pub fn merge_configs(
        base: &MihomoConfig,
        profile: &MihomoConfig,
        overrides: &[OverrideItem],
    ) -> Result<MihomoConfig, FactoryError> {
        let mut result = base.clone();

        result = Self::deep_merge(&result, profile)?;

        for override_item in overrides {
            if !override_item.enabled {
                continue;
            }

            override_item.apply(&mut result)?;
        }

        Ok(result)
    }

    pub fn deep_merge(
        base: &MihomoConfig,
        overlay: &MihomoConfig,
    ) -> Result<MihomoConfig, FactoryError> {
        let base_value = serde_yaml::to_value(base)?;
        let overlay_value = serde_yaml::to_value(overlay)?;

        let merged = Self::merge_yaml_values(base_value, overlay_value);

        serde_yaml::from_value(merged).map_err(FactoryError::from)
    }

    fn merge_yaml_values(base: Value, overlay: Value) -> Value {
        match (base, overlay) {
            (Value::Mapping(mut base_map), Value::Mapping(overlay_map)) => {
                for (key, overlay_val) in overlay_map {
                    if let Some(base_val) = base_map.get(&key) {
                        let merged_val = Self::merge_yaml_values(base_val.clone(), overlay_val);
                        base_map.insert(key, merged_val);
                    } else {
                        base_map.insert(key, overlay_val);
                    }
                }
                Value::Mapping(base_map)
            }
            (_, overlay) => overlay,
        }
    }

    pub fn apply_yaml_override(
        config: &MihomoConfig,
        yaml_content: &str,
    ) -> Result<MihomoConfig, FactoryError> {
        let override_value: Value = serde_yaml::from_str(yaml_content)?;
        let config_value = serde_yaml::to_value(config)?;

        let merged = Self::merge_yaml_values(config_value, override_value);

        serde_yaml::from_value(merged).map_err(FactoryError::from)
    }

    pub fn process_rule_files(
        config: &mut MihomoConfig,
        rule_actions: &HashMap<String, RuleFileAction>,
    ) -> Result<(), FactoryError> {
        let rules = config
            .rules
            .as_mut()
            .ok_or_else(|| FactoryError::RuleError("No rules in config".into()))?;

        let mut final_rules = Vec::new();

        for action in rule_actions.values() {
            final_rules.extend(action.prepend.iter().cloned());
        }

        final_rules.extend(rules.clone());

        for action in rule_actions.values() {
            for pattern in &action.delete {
                final_rules.retain(|r| !r.contains(pattern));
            }
        }

        for action in rule_actions.values() {
            final_rules.extend(action.append.iter().cloned());
        }

        *rules = final_rules;
        Ok(())
    }

    pub fn generate_runtime_config(
        base_config: &MihomoConfig,
        profile_config: &MihomoConfig,
        overrides: &[OverrideItem],
        mode: Option<&str>,
    ) -> Result<String, FactoryError> {
        let mut config = Self::merge_configs(base_config, profile_config, overrides)?;

        if let Some(m) = mode {
            config.mode = m.to_string();
        }

        serde_yaml::to_string(&config).map_err(FactoryError::from)
    }

    pub fn copy_geo_files(work_dir: &Path) -> Result<(), FactoryError> {
        let geo_files = ["Country.mmdb", "GeoSite.dat", "geosite.dat", "geoip.dat"];

        let data_dir = crate::utils::dirs::data_dir();

        for file in &geo_files {
            let src = data_dir.join(file);
            let dst = work_dir.join(file);

            if src.exists() && !dst.exists() {
                std::fs::copy(&src, &dst)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tun: Option<TunPatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<crate::config::DnsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
}

pub fn generate_pac_script(mixed_port: u16) -> String {
    format!(
        r#"function FindProxyForURL(url, host) {{
    return "PROXY 127.0.0.1:{}; SOCKS5 127.0.0.1:{}; DIRECT;";
}}
"#,
        mixed_port, mixed_port
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pac_script() {
        let pac = generate_pac_script(7890);
        assert!(pac.contains("7890"));
        assert!(pac.contains("PROXY"));
        assert!(pac.contains("SOCKS5"));
    }

    #[test]
    fn test_rule_file_action() {
        let action = RuleFileAction {
            prepend: vec![],
            append: vec![],
            delete: vec!["test".to_string()],
        };
        assert_eq!(action.delete.len(), 1);
    }
}
