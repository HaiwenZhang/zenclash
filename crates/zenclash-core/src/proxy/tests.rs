#[cfg(test)]
mod tests {
    use crate::proxy::{DelayTestConfig, DelayTestResult, DelayTester};
    use crate::proxy::{Proxy, ProxyGroup, ProxySelector, ProxyType, SelectionStrategy};

    #[test]
    fn test_delay_config_default() {
        let config = DelayTestConfig::default();
        assert_eq!(config.url, "http://www.gstatic.com/generate_204");
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.concurrent, 10);
    }

    #[test]
    fn test_delay_result_creation() {
        let result = DelayTestResult {
            proxy_name: "test-proxy".to_string(),
            delay_ms: 150,
            error: None,
        };
        assert_eq!(result.proxy_name, "test-proxy");
        assert_eq!(result.delay_ms, 150);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_delay_result_with_error() {
        let result = DelayTestResult {
            proxy_name: "failed-proxy".to_string(),
            delay_ms: u32::MAX,
            error: Some("Timeout".to_string()),
        };
        assert_eq!(result.delay_ms, u32::MAX);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_proxy_creation() {
        let proxy = Proxy {
            name: "SG-Node-1".to_string(),
            proxy_type: ProxyType::Vmess,
            server: "sg.example.com".to_string(),
            port: 443,
            password: None,
            uuid: Some("uuid-123".to_string()),
            alter_id: Some(0),
            cipher: None,
            udp: true,
        };

        assert_eq!(proxy.name, "SG-Node-1");
        assert_eq!(proxy.server, "sg.example.com");
        assert_eq!(proxy.port, 443);
        assert!(proxy.udp);
    }

    #[test]
    fn test_proxy_group_creation() {
        let proxies = vec![Proxy {
            name: "proxy-1".to_string(),
            proxy_type: ProxyType::Shadowsocks,
            server: "server1.com".to_string(),
            port: 8388,
            password: Some("pass".to_string()),
            uuid: None,
            alter_id: None,
            cipher: Some("aes-256-gcm".to_string()),
            udp: true,
        }];

        let group = ProxyGroup {
            name: "Auto-Select".to_string(),
            group_type: crate::proxy::ProxyGroupType::UrlTest,
            proxies,
            selected: Some("proxy-1".to_string()),
        };

        assert_eq!(group.name, "Auto-Select");
        assert_eq!(group.proxies.len(), 1);
        assert!(group.selected.is_some());
    }

    #[test]
    fn test_proxy_selector_strategies() {
        let auto_selector = ProxySelector::new(SelectionStrategy::Auto);
        assert_eq!(auto_selector.strategy(), SelectionStrategy::Auto);

        let manual_selector = ProxySelector::new(SelectionStrategy::Manual);
        assert_eq!(manual_selector.strategy(), SelectionStrategy::Manual);

        let round_robin_selector = ProxySelector::new(SelectionStrategy::RoundRobin);
        assert_eq!(
            round_robin_selector.strategy(),
            SelectionStrategy::RoundRobin
        );
    }

    #[test]
    fn test_selection_strategy_display() {
        assert_eq!(format!("{}", SelectionStrategy::Auto), "Auto");
        assert_eq!(format!("{}", SelectionStrategy::Manual), "Manual");
        assert_eq!(format!("{}", SelectionStrategy::RoundRobin), "Round Robin");
        assert_eq!(format!("{}", SelectionStrategy::Latency), "Latency");
    }

    #[test]
    fn test_proxy_type_variants() {
        let types = vec![
            ProxyType::Shadowsocks,
            ProxyType::Vmess,
            ProxyType::Vless,
            ProxyType::Trojan,
            ProxyType::Socks5,
            ProxyType::Http,
            ProxyType::Snell,
        ];

        for proxy_type in types {
            let proxy = Proxy {
                name: "test".to_string(),
                proxy_type,
                server: "test.com".to_string(),
                port: 443,
                password: None,
                uuid: None,
                alter_id: None,
                cipher: None,
                udp: true,
            };
            assert!(!proxy.name.is_empty());
        }
    }

    #[tokio::test]
    async fn test_delay_tester_creation() {
        let tester = DelayTester::new(DelayTestConfig::default());
        assert_eq!(tester.config().timeout_ms, 5000);
    }
}
