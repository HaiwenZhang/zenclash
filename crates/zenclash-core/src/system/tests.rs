use crate::system::{DnsConfig, DnsMode, SysProxyConfig, TunConfig, TunMode};

#[test]
fn test_sysproxy_config_default() {
    let config = SysProxyConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.http_port, 7890);
    assert_eq!(config.socks_port, 7891);
}

#[test]
fn test_sysproxy_config_enabled() {
    let config = SysProxyConfig {
        enabled: true,
        http_port: 8080,
        socks_port: 1080,
        bypass_domains: vec!["localhost".to_string(), "127.0.0.1".to_string()],
    };

    assert!(config.enabled);
    assert_eq!(config.http_port, 8080);
    assert_eq!(config.bypass_domains.len(), 2);
}

#[test]
fn test_dns_config_default() {
    let config = DnsConfig::default();
    assert!(config.enable);
    assert!(!config.ipv6);
    assert_eq!(config.listen, ":1053");
}

#[test]
fn test_dns_config_custom_servers() {
    let config = DnsConfig {
        nameservers: vec!["223.5.5.5".to_string(), "8.8.8.8".to_string()],
        ..Default::default()
    };

    assert_eq!(config.nameservers.len(), 2);
    assert_eq!(config.nameservers[0], "223.5.5.5");
}

#[test]
fn test_tun_config_default() {
    let config = TunConfig::default();
    assert!(!config.enable);
    assert_eq!(config.device, "utun");
    assert_eq!(config.mode, TunMode::Bypass);
}

#[test]
fn test_tun_config_enabled() {
    let config = TunConfig {
        enable: true,
        device: "tun0".to_string(),
        mode: TunMode::Capture,
        stack: crate::system::TunStack::System,
        mtu: 9000,
    };

    assert!(config.enable);
    assert_eq!(config.device, "tun0");
    assert_eq!(config.mtu, 9000);
}

#[test]
fn test_dns_mode_variants() {
    use DnsMode::*;

    let modes = vec![Normal, FakeIp, RedirHost];

    for mode in modes {
        let config = DnsConfig {
            mode: mode.clone(),
            ..Default::default()
        };
        assert_eq!(config.mode, mode);
    }
}

#[test]
fn test_tun_mode_variants() {
    use TunMode::*;

    let modes = vec![Bypass, Capture];

    for mode in modes {
        let config = TunConfig {
            mode: mode.clone(),
            ..Default::default()
        };
        assert_eq!(config.mode, mode);
    }
}

#[test]
fn test_system_config_equality() {
    let config1 = SysProxyConfig::default();
    let config2 = SysProxyConfig::default();

    assert_eq!(config1.enabled, config2.enabled);
    assert_eq!(config1.http_port, config2.http_port);
}
