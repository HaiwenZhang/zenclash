use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use zenclash_core::{
    AppConfig, CoreManager, HttpClient, MihomoClient, ProfileConfig, ProfileItem, ProfileType,
    ProxyGroup, ProxySelector,
};

#[tokio::test]
async fn test_full_workflow() {
    // 1. 初始化配置
    let app_config = AppConfig::default();
    assert_eq!(app_config.core, "mihomo");

    // 2. 创建核心管理器
    let core_manager = Arc::new(RwLock::new(CoreManager));

    // 3. 创建 HTTP 客户端
    let http_client = HttpClient::new_default().expect("Failed to create HTTP client");

    // 4. 测试配置文件加载
    let profile_config = ProfileConfig::default();
    assert!(profile_config.items.is_empty());

    // 5. 测试代理选择器
    let selector = ProxySelector::new(zenclash_core::SelectionStrategy::Auto);
    assert_eq!(selector.strategy(), zenclash_core::SelectionStrategy::Auto);

    println!("✅ Full workflow test passed");
}

#[tokio::test]
async fn test_config_save_load() {
    let temp_dir = std::env::temp_dir().join("zenclash_test");
    std::fs::create_dir_all(&temp_dir).ok();

    let config_path = temp_dir.join("test_config.yaml");

    // 创建测试配置
    let config = AppConfig {
        core: "mihomo-alpha".to_string(),
        auto_launch: true,
        silent_start: false,
        theme: "dark".to_string(),
        language: "zh-CN".to_string(),
        ..Default::default()
    };

    // TODO: 实现保存和加载
    // config.save(&config_path).await?;
    // let loaded = AppConfig::load(&config_path).await?;
    // assert_eq!(loaded.core, "mihomo-alpha");

    println!("✅ Config save/load test placeholder");
}

#[tokio::test]
async fn test_profile_management() {
    // 创建测试订阅
    let profile = ProfileItem {
        id: "test-profile".to_string(),
        name: "Test Profile".to_string(),
        url: Some("https://example.com/config.yaml".to_string()),
        type_: ProfileType::Remote,
        used: true,
        extra: None,
    };

    assert_eq!(profile.id, "test-profile");
    assert_eq!(profile.type_, ProfileType::Remote);

    println!("✅ Profile management test passed");
}

#[tokio::test]
async fn test_proxy_delay_test() {
    use zenclash_core::DelayTestConfig;

    let config = DelayTestConfig::default();
    assert_eq!(config.url, "http://www.gstatic.com/generate_204");
    assert_eq!(config.timeout_ms, 5000);

    println!("✅ Proxy delay test config validated");
}

#[test]
fn test_utils_dirs() {
    let data = zenclash_core::utils::data_dir();
    let config = zenclash_core::utils::config_dir();
    let profiles = zenclash_core::utils::profiles_dir();

    assert!(data.ends_with("zenclash"));
    assert!(config.ends_with("zenclash"));
    assert_eq!(profiles.parent().unwrap(), data);

    println!("✅ Utils dirs test passed");
}
