use crate::core::{ApiClient, ApiClientConfig, CoreManager, CoreManagerConfig, CoreState, Process, ProcessConfig};
use std::path::PathBuf;

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
fn test_process_config() {
    let config = ProcessConfig::new(PathBuf::from("/usr/bin/mihomo"))
        .args(vec!["-d".to_string(), "/tmp".to_string()])
        .work_dir(PathBuf::from("/tmp"));

    assert_eq!(config.path, PathBuf::from("/usr/bin/mihomo"));
    assert_eq!(config.args.len(), 2);
}

#[tokio::test]
async fn test_process_state_initial() {
    let config = ProcessConfig::new(PathBuf::from("/bin/echo"));
    let process = Process::new(config);
    use crate::core::ProcessState;
    assert_eq!(process.state().await, ProcessState::Stopped);
    assert!(!process.is_running().await);
}

#[test]
fn test_core_state_transitions() {
    use CoreState::*;

    let states = vec![Stopped, Starting, Running, Stopping, Error];

    for state in states {
        let display = format!("{:?}", state);
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
        metadata: crate::core::ConnectionMetadata {
            network: "tcp".to_string(),
            conn_type: "HTTP".to_string(),
            host: Some("example.com".to_string()),
            source_ip: "192.168.1.1".to_string(),
            source_port: "54321".to_string(),
            destination_ip: Some("8.8.8.8".to_string()),
            destination_port: "443".to_string(),
            process: None,
            process_path: None,
        },
        upload: 1024,
        download: 2048,
        start: "2024-01-01T00:00:00Z".to_string(),
        chains: vec!["SG-Node-1".to_string()],
        rule: Some("MATCH".to_string()),
    };

    assert_eq!(conn.id, "conn-123");
    assert!(conn.chains.contains(&"SG-Node-1".to_string()));
}

#[test]
fn test_log_item_parsing() {
    let log = crate::core::LogItem {
        level: "info".to_string(),
        payload: "Core started".to_string(),
    };

    assert_eq!(log.level, "info");
    assert_eq!(log.payload, "Core started");
}
