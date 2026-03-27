use crate::system::TunConfig;

#[test]
fn test_tun_config_default() {
    let config = TunConfig::default();
    assert!(!config.enable);
}

#[test]
fn test_tun_config_enabled() {
    let config = TunConfig {
        enable: true,
        stack: "system".to_string(),
        dns_hijack: vec!["any:53".to_string()],
        auto_route: true,
        auto_detect_interface: true,
        device_name: None,
        mtu: None,
        strict_route: None,
        endpoint_distance_nat: None,
    };

    assert!(config.enable);
    assert!(config.auto_route);
}
