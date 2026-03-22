use crate::core::{ApiClient, ApiClientConfig, CoreManager, CoreState, Process, ProcessConfig};
use std::time::Duration;

#[test]
fn test_api_client_config_default() {
    let config = ApiClientConfig::default();
    assert_eq!(config.timeout_secs, 30);
    assert_eq!(config.base_url, "http://127.0.0.1:9090");
    assert!(config.secret.is_none());
}

#[test]
fn test_api_client_config_with_secret() {
    let config = ApiClientConfig {
        secret: Some("test-secret".to_string()),
        ..Default::default()
    };
    assert_eq!(config.secret.as_ref().unwrap(), "test-secret");
}

#[test]
fn test_process_config_default() {
    let config = ProcessConfig::default();
    assert!(config.core_path.to_string().contains("mihomo"));
    assert!(config.work_dir.to_string().contains("zenclash"));
}

#[tokio::test]
async fn test_core_manager_state() {
    let manager = CoreManager::new();
    assert_eq!(manager.state(), CoreState::Stopped);
}

#[test]
fn test_core_state_transitions() {
    use CoreState::*;

    let states = vec![Stopped, Starting, Running, Stopping, Error];

    for state in states {
        let display = format!("{}", state);
        assert!(!display.is_empty());
    }
}

#[test]
fn test_traffic_data_creation() {
    let data = crate::core::TrafficData {
        up: 1024,
        down: 2048,
    };

    assert_eq!(data.up, 1024);
    assert_eq!(data.down, 2048);
}

#[test]
fn test_connection_item_creation() {
    let conn = crate::core::ConnectionItem {
        id: "conn-123".to_string(),
        source: "192.168.1.1:54321".to_string(),
        destination: "8.8.8.8:443".to_string(),
        proxy: "SG-Node-1".to_string(),
        rule: "MATCH".to_string(),
        chain: vec!["SG-Node-1".to_string()],
        upload: 1024,
        download: 2048,
        start_time: std::time::SystemTime::now(),
    };

    assert_eq!(conn.id, "conn-123");
    assert_eq!(conn.proxy, "SG-Node-1");
    assert!(conn.chain.contains(&"SG-Node-1".to_string()));
}

#[test]
fn test_log_item_parsing() {
    let log = crate::core::LogItem {
        level: "info".to_string(),
        message: "Core started".to_string(),
        timestamp: std::time::SystemTime::now(),
    };

    assert_eq!(log.level, "info");
    assert_eq!(log.message, "Core started");
}

#[test]
fn test_memory_data() {
    let mem = crate::core::MemoryData {
        inuse: 1024000,
        oslimit: 512000000,
    };

    assert!(mem.inuse > 0);
    assert!(mem.oslimit > mem.inuse);
}
