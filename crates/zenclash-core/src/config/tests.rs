use crate::config::{AppConfig, MihomoConfig, ProfileConfig, ProfileItem, ProfileType};

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();
    assert_eq!(config.core, "mihomo");
    assert!(!config.auto_launch);
    assert!(!config.silent_start);
    assert_eq!(config.theme, "system");
    assert_eq!(config.language, "en");
}

#[test]
fn test_app_config_custom() {
    let config = AppConfig {
        core: "mihomo-alpha".to_string(),
        auto_launch: true,
        theme: "dark".to_string(),
        language: "zh-CN".to_string(),
        ..Default::default()
    };

    assert_eq!(config.core, "mihomo-alpha");
    assert!(config.auto_launch);
    assert_eq!(config.theme, "dark");
}

#[test]
fn test_mihomo_config_default() {
    let config = MihomoConfig::default();
    assert_eq!(config.port, 7890);
    assert_eq!(config.socks_port, 7891);
    assert_eq!(config.redir_port, 7892);
    assert_eq!(config.mixed_port, 7893);
    assert_eq!(config.external_controller, "127.0.0.1:9090");
}

#[test]
fn test_mihomo_config_mode() {
    let config = MihomoConfig {
        mode: crate::config::MihomoMode::Global,
        ..Default::default()
    };

    assert_eq!(config.mode, crate::config::MihomoMode::Global);
}

#[test]
fn test_profile_item_creation() {
    let item = ProfileItem {
        id: "test-id".to_string(),
        name: "Test Profile".to_string(),
        type_: ProfileType::Remote,
        url: Some("https://example.com/config.yaml".to_string()),
        used: true,
        path: Some("/path/to/config.yaml".to_string()),
        extra: None,
    };

    assert_eq!(item.id, "test-id");
    assert_eq!(item.name, "Test Profile");
    assert_eq!(item.type_, ProfileType::Remote);
    assert!(item.url.is_some());
    assert!(item.used);
}

#[test]
fn test_profile_type_variants() {
    let remote = ProfileType::Remote;
    let local = ProfileType::Local;

    assert_ne!(remote, local);

    let remote_item = ProfileItem {
        id: "1".to_string(),
        name: "Remote".to_string(),
        type_: remote,
        url: Some("http://test.com".to_string()),
        used: false,
        path: None,
        extra: None,
    };

    let local_item = ProfileItem {
        id: "2".to_string(),
        name: "Local".to_string(),
        type_: local,
        url: None,
        used: false,
        path: Some("/local.yaml".to_string()),
        extra: None,
    };

    assert!(remote_item.url.is_some());
    assert!(local_item.url.is_none());
}

#[test]
fn test_profile_config_default() {
    let config = ProfileConfig::default();
    assert!(config.items.is_empty());
    assert!(config.current.is_none());
}

#[test]
fn test_profile_config_with_items() {
    let items = vec![
        ProfileItem {
            id: "1".to_string(),
            name: "Profile 1".to_string(),
            type_: ProfileType::Remote,
            url: Some("url1".to_string()),
            used: true,
            path: None,
            extra: None,
        },
        ProfileItem {
            id: "2".to_string(),
            name: "Profile 2".to_string(),
            type_: ProfileType::Local,
            url: None,
            used: false,
            path: Some("path".to_string()),
            extra: None,
        },
    ];

    let config = ProfileConfig {
        items,
        current: Some("1".to_string()),
    };

    assert_eq!(config.items.len(), 2);
    assert_eq!(config.current.as_ref().unwrap(), "1");
}
