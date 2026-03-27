use crate::config::{AppConfig, MihomoConfig, ProfileConfig, ProfileItem, ProfileType};

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();
    assert_eq!(config.core, "mihomo");
    assert!(!config.auto_launch);
    assert!(!config.silent_start);
    assert_eq!(config.theme, "system");
}

#[test]
fn test_app_config_custom() {
    let config = AppConfig {
        core: "mihomo-alpha".to_string(),
        auto_launch: true,
        theme: "dark".to_string(),
        ..Default::default()
    };

    assert_eq!(config.core, "mihomo-alpha");
    assert!(config.auto_launch);
    assert_eq!(config.theme, "dark");
}

#[test]
fn test_mihomo_config_default() {
    let config = MihomoConfig::default();
    assert_eq!(config.mixed_port, Some(7890));
    assert!(config.external_controller.is_some());
    assert_eq!(config.mode, "rule");
}

#[test]
fn test_profile_item_creation() {
    let item = ProfileItem::new_remote(
        "Test Profile".to_string(),
        "https://example.com/config.yaml".to_string(),
        None,
    );

    assert!(!item.id.is_empty());
    assert_eq!(item.name, "Test Profile");
    assert_eq!(item.profile_type, ProfileType::Remote);
    assert!(item.url.is_some());
}

#[test]
fn test_profile_type_variants() {
    let remote = ProfileType::Remote;
    let local = ProfileType::Local;

    assert_ne!(remote, local);
}

#[test]
fn test_profile_config_default() {
    let config = ProfileConfig::default();
    assert!(config.items.is_empty());
    assert!(config.current.is_none());
}

#[test]
fn test_profile_config_with_items() {
    let item1 = ProfileItem::new_remote("Profile 1".to_string(), "url1".to_string(), None);
    let item2 = ProfileItem::new_local("Profile 2".to_string(), std::path::PathBuf::from("/path"));

    let mut config = ProfileConfig::default();
    config.add(item1.clone());
    config.add(item2.clone());
    config.set_current(&item1.id);

    assert_eq!(config.items.len(), 2);
    assert_eq!(config.current.as_ref().unwrap(), &item1.id);
}
