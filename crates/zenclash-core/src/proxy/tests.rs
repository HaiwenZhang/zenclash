#[cfg(test)]
mod tests {
    use crate::proxy::{DelayTestConfig, DelayTestResult, DelayTestStatus};
    use crate::proxy::{Proxy, ProxyGroup, ProxyType};

    #[test]
    fn test_delay_config_default() {
        let config = DelayTestConfig::default();
        assert_eq!(config.url, "http://www.gstatic.com/generate_204");
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.concurrent, 10);
    }

    #[test]
    fn test_delay_result_creation() {
        let result = DelayTestResult::success("test-proxy", 150);
        assert_eq!(result.name, "test-proxy");
        assert_eq!(result.delay, Some(150));
        assert_eq!(result.status, DelayTestStatus::Success);
    }

    #[test]
    fn test_delay_result_with_error() {
        let result = DelayTestResult::failed("failed-proxy", "Timeout");
        assert_eq!(result.status, DelayTestStatus::Failed);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_proxy_creation() {
        let proxy = Proxy::new("SG-Node-1".to_string(), ProxyType::Vmess)
            .with_server("sg.example.com".to_string(), 443);

        assert_eq!(proxy.name, "SG-Node-1");
        assert_eq!(proxy.server, Some("sg.example.com".to_string()));
        assert_eq!(proxy.port, Some(443));
    }

    #[test]
    fn test_proxy_group_creation() {
        let group = ProxyGroup::new_url_test(
            "Auto-Select".to_string(),
            vec!["proxy-1".to_string()],
            "http://test.com".to_string(),
            300,
        );

        assert_eq!(group.name, "Auto-Select");
        assert_eq!(group.proxies.len(), 1);
        assert_eq!(group.group_type, ProxyType::UrlTest);
    }

    #[test]
    fn test_proxy_type_variants() {
        let types = vec![
            ProxyType::Ss,
            ProxyType::Vmess,
            ProxyType::Vless,
            ProxyType::Trojan,
            ProxyType::Socks5,
            ProxyType::Http,
            ProxyType::Snell,
        ];

        for proxy_type in types {
            let proxy = Proxy::new("test".to_string(), proxy_type);
            assert!(!proxy.name.is_empty());
        }
    }
}
