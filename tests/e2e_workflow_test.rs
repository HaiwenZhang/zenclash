use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use zenclash_core::{
    AppConfig, CoreManager, CoreState, HttpClient, MihomoConfig, ProfileConfig, ProfileItem,
    ProfileType, Proxy, ProxyGroup, ProxySelector, ProxyType, SelectionStrategy, SysProxyConfig,
    TunConfig,
};

mod test_utils {
    use super::*;

    pub fn create_test_app_config() -> AppConfig {
        AppConfig {
            core: "mihomo".to_string(),
            auto_launch: false,
            silent_start: false,
            system_proxy: false,
            tun_mode: false,
            theme: "dark".to_string(),
            language: "en".to_string(),
            proxy_layout: "list".to_string(),
            enable_tray_speed: false,
            log_level: "info".to_string(),
            show_conn: true,
            show_delay: true,
            delay_test_url: "http://www.gstatic.com/generate_204".to_string(),
            current_profile: None,
            current_sub: None,
            auto_start: false,
            silent_mode: false,
            user_agent: None,
            webdav_url: None,
            webdav_username: None,
            webdav_password: None,
        }
    }

    pub fn create_test_proxy(name: &str, delay: u32) -> Proxy {
        Proxy {
            name: name.to_string(),
            proxy_type: ProxyType::Shadowsocks,
            server: "test.server.com".to_string(),
            port: 8388,
            password: Some("password".to_string()),
            uuid: None,
            alter_id: None,
            cipher: Some("aes-256-gcm".to_string()),
            udp: true,
            delay: Some(delay),
        }
    }

    pub fn create_test_proxy_group(name: &str, proxies: Vec<Proxy>) -> ProxyGroup {
        ProxyGroup {
            name: name.to_string(),
            group_type: zenclash_core::proxy::ProxyGroupType::UrlTest,
            proxies,
            selected: None,
        }
    }
}

#[tokio::test]
async fn test_end_to_end_workflow() {
    use test_utils::*;

    // Step 1: Initialize configuration
    let app_config = create_test_app_config();
    assert_eq!(app_config.core, "mihomo");
    assert!(!app_config.auto_launch);

    // Step 2: Create core manager
    let core_manager = Arc::new(RwLock::new(CoreManager::new()));
    {
        let manager = core_manager.read().await;
        assert_eq!(manager.state(), CoreState::Stopped);
    }

    // Step 3: Create HTTP client
    let http_client = HttpClient::new_default().expect("Failed to create HTTP client");
    assert!(http_client.client().timeout().is_some());

    // Step 4: Create proxy selector
    let selector = ProxySelector::new(SelectionStrategy::Auto);
    assert_eq!(selector.strategy(), SelectionStrategy::Auto);

    // Step 5: Create test proxies
    let proxies = vec![
        create_test_proxy("SG-Node-1", 150),
        create_test_proxy("SG-Node-2", 200),
        create_test_proxy("JP-Node-1", 250),
    ];

    let proxy_group = create_test_proxy_group("Auto-Select", proxies);
    assert_eq!(proxy_group.proxies.len(), 3);

    // Step 6: Create profile configuration
    let profile_item = ProfileItem {
        id: "test-profile".to_string(),
        name: "Test Profile".to_string(),
        type_: ProfileType::Remote,
        url: Some("https://example.com/config.yaml".to_string()),
        used: false,
        path: None,
        extra: None,
    };

    let profile_config = ProfileConfig {
        items: vec![profile_item],
        current: None,
    };

    assert_eq!(profile_config.items.len(), 1);

    println!("End-to-end workflow test passed");
}

#[tokio::test]
async fn test_configuration_workflow() {
    use test_utils::*;

    // Test AppConfig
    let mut app_config = create_test_app_config();

    // Modify configuration
    app_config.theme = "light".to_string();
    app_config.language = "zh-CN".to_string();
    app_config.system_proxy = true;

    assert_eq!(app_config.theme, "light");
    assert_eq!(app_config.language, "zh-CN");
    assert!(app_config.system_proxy);

    // Test MihomoConfig
    let mihomo_config = MihomoConfig {
        port: 7890,
        socks_port: 7891,
        mixed_port: 7893,
        external_controller: "127.0.0.1:9090".to_string(),
        secret: Some("test-secret".to_string()),
        mode: zenclash_core::config::MihomoMode::Rule,
        log_level: "info".to_string(),
        ipv6: false,
        allow_lan: false,
        bind_address: "*".to_string(),
        unified_delay: true,
        tcp_concurrent: true,
        enable_process: true,
        find_process_mode: "strict".to_string(),
        skip_auth_prefixes: vec![],
        external_ui: None,
        external_ui_name: None,
        external_ui_url: None,
        profile: None,
        geox_url: None,
        geo_auto_update: false,
        geo_update_interval: 24,
        geodata_mode: false,
        geodata_loader: "memconservative".to_string(),
        global_client_fingerprint: None,
        global_ua: None,
        keep_alive_idel: 600,
        keep_alive_interval: 15,
        keep_alive_count: 5,
        sni_sniff: true,
        sniffer: None,
        hosts: None,
        dns: None,
        tun: None,
        authentication: vec![],
        ebpf: None,
        experimental: None,
        proxy_groups: vec![],
        rules: vec![],
        sub_rules: None,
        proxies: vec![],
        proxy_providers: None,
        rule_providers: None,
        listeners: vec![],
    };

    assert_eq!(mihomo_config.port, 7890);
    assert!(mihomo_config.secret.is_some());

    println!("Configuration workflow test passed");
}

#[tokio::test]
async fn test_proxy_selection_workflow() {
    use test_utils::*;

    // Create proxy selector with different strategies
    let strategies = vec![
        SelectionStrategy::Auto,
        SelectionStrategy::Manual,
        SelectionStrategy::RoundRobin,
        SelectionStrategy::Latency,
    ];

    for strategy in strategies {
        let selector = ProxySelector::new(strategy.clone());
        assert_eq!(selector.strategy(), strategy);

        let strategy_display = format!("{}", strategy);
        assert!(!strategy_display.is_empty());
    }

    // Create test proxies with different delays
    let proxies = vec![
        create_test_proxy("Fast-Proxy", 50),
        create_test_proxy("Medium-Proxy", 150),
        create_test_proxy("Slow-Proxy", 500),
    ];

    // Find fastest proxy
    let fastest = proxies.iter().min_by_key(|p| p.delay.unwrap_or(u32::MAX));
    assert!(fastest.is_some());
    assert_eq!(fastest.unwrap().name, "Fast-Proxy");

    println!("Proxy selection workflow test passed");
}

#[tokio::test]
async fn test_system_integration_workflow() {
    // Test SysProxyConfig
    let sysproxy_config = SysProxyConfig {
        enabled: true,
        http_port: 7890,
        socks_port: 7891,
        bypass_domains: vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "*.local".to_string(),
        ],
    };

    assert!(sysproxy_config.enabled);
    assert_eq!(sysproxy_config.bypass_domains.len(), 3);

    // Test TunConfig
    let tun_config = TunConfig {
        enable: false,
        device: "utun".to_string(),
        mode: zenclash_core::system::TunMode::Bypass,
        stack: zenclash_core::system::TunStack::System,
        mtu: 9000,
        gso: false,
        gso_max_size: 0,
        inet4_address: None,
        inet6_address: None,
        auto_route: true,
        auto_detect_interface: true,
        dns_hijack: vec![],
        route_address: vec![],
        route_address_set: vec![],
        route_exclude_address: vec![],
        route_exclude_address_set: vec![],
        include_interface: vec![],
        exclude_interface: vec![],
        include_uid: vec![],
        exclude_uid: vec![],
        include_android_user: vec![],
        exclude_android_user: vec![],
        include_package: vec![],
        exclude_package: vec![],
    };

    assert!(!tun_config.enable);
    assert_eq!(tun_config.mtu, 9000);

    println!("System integration workflow test passed");
}

#[tokio::test]
async fn test_error_handling_workflow() {
    use zenclash_core::ZenClashError;

    // Test various error types
    let io_error = ZenClashError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "File not found",
    ));
    assert!(io_error.to_string().contains("IO error"));

    let config_error = ZenClashError::Config("Invalid configuration".to_string());
    assert!(config_error.to_string().contains("Config error"));

    let profile_error = ZenClashError::ProfileNotFound("test-profile".to_string());
    assert!(profile_error.to_string().contains("Profile not found"));

    let network_error = ZenClashError::Network("Connection failed".to_string());
    assert!(network_error.to_string().contains("Network error"));

    println!("Error handling workflow test passed");
}
