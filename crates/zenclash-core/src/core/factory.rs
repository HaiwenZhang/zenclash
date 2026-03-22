use crate::config::{MihomoConfig, OverrideItem, OverrideRule};
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
    pub prepend: Vec<Rule>,
    pub append: Vec<Rule>,
    pub delete: Vec<String>,
}

use crate::config::Rule;

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
            },
            (_, overlay) => overlay,
        }
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

        for (_file_name, action) in rule_actions {
            for rule in &action.prepend {
                final_rules.push(rule.clone());
            }
        }

        final_rules.extend(rules.clone());

        for (_file_name, action) in rule_actions {
            for pattern in &action.delete {
                final_rules.retain(|r| !r.to_clash_string().contains(pattern));
            }
        }

        for (_file_name, action) in rule_actions {
            for rule in &action.append {
                final_rules.push(rule.clone());
            }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleFileAction {
    pub prepend: Vec<Rule>,
    pub append: Vec<Rule>,
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

            match override_item.item_type {
                OverrideType::Yaml => {
                    if let Some(content) = &override_item.file {
                        result = Self::apply_yaml_override(&result, content)?;
                    }
                },
                OverrideType::Js => {
                    if let Some(content) = &override_item.file {
                        result = Self::apply_js_override(&result, content)?;
                    }
                },
            }
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
            },
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

    pub fn apply_js_override(
        config: &MihomoConfig,
        js_content: &str,
    ) -> Result<MihomoConfig, FactoryError> {
        let config_json = serde_json::to_string_pretty(&config)
            .map_err(|e| FactoryError::JsExecutionError(e.to_string()))?;

        let script = format!(
            r#"
            const config = {};
            const main = {};
            let result;
            try {{
                result = main(config);
            }} catch (e) {{
                result = config;
            }}
            JSON.stringify(result);
            "#,
            config_json, js_content
        );

        let result = Self::execute_js(&script)?;

        let merged: MihomoConfig = serde_json::from_str(&result).map_err(|e| {
            FactoryError::JsExecutionError(format!("Failed to parse JS result: {}", e))
        })?;

        Ok(merged)
    }

    #[cfg(feature = "js-override")]
    fn execute_js(script: &str) -> Result<String, FactoryError> {
        use deno_core::{JsRuntime, RuntimeOptions};

        let mut runtime = JsRuntime::new(RuntimeOptions::default());

        let result = runtime
            .execute_script("<override>", script.into())
            .map_err(|e| FactoryError::JsExecutionError(e.to_string()))?;

        let result_str = runtime
            .resolve_value(result)
            .map_err(|e| FactoryError::JsExecutionError(e.to_string()))?;

        Ok(result_str.as_string().unwrap_or_default().to_string())
    }

    #[cfg(not(feature = "js-override"))]
    fn execute_js(_script: &str) -> Result<String, FactoryError> {
        Err(FactoryError::JsExecutionError(
            "JS override feature not enabled. Enable 'js-override' feature flag.".into(),
        ))
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

        for (file_name, action) in rule_actions {
            for rule in &action.prepend {
                final_rules.push(rule.clone());
            }
        }

        final_rules.extend(rules.clone());

        for (file_name, action) in rule_actions {
            for pattern in &action.delete {
                final_rules.retain(|r| !r.to_clash_string().contains(pattern));
            }
        }

        for (file_name, action) in rule_actions {
            for rule in &action.append {
                final_rules.push(rule.clone());
            }
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
            config.mode = Some(m.to_string());
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
