use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;
use tracing::{debug, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrafficType {
    Upload,
    Download,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficInfo {
    pub timestamp: DateTime<Utc>,
    pub traffic_type: TrafficType,
    pub bytes: u64,
    pub speed_bps: u64,
    pub connection_id: Option<String>,
    pub process_name: Option<String>,
    pub source: Option<String>,
    pub destination: Option<String>,
}

impl TrafficInfo {
    pub fn new(traffic_type: TrafficType, bytes: u64, speed_bps: u64) -> Self {
        Self {
            timestamp: Utc::now(),
            traffic_type,
            bytes,
            speed_bps,
            connection_id: None,
            process_name: None,
            source: None,
            destination: None,
        }
    }

    pub fn with_connection(mut self, id: impl Into<String>) -> Self {
        self.connection_id = Some(id.into());
        self
    }

    pub fn with_process(mut self, name: impl Into<String>) -> Self {
        self.process_name = Some(name.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_destination(mut self, dest: impl Into<String>) -> Self {
        self.destination = Some(dest.into());
        self
    }

    pub fn format_speed(&self) -> String {
        format_bytes_per_sec(self.speed_bps)
    }

    pub fn format_bytes(&self) -> String {
        format_bytes(self.bytes)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrafficStats {
    pub total_upload: u64,
    pub total_download: u64,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub connections: usize,
    pub start_time: Option<DateTime<Utc>>,
    pub last_update: Option<DateTime<Utc>>,
}

impl TrafficStats {
    pub fn new() -> Self {
        Self {
            start_time: Some(Utc::now()),
            ..Default::default()
        }
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_upload + self.total_download
    }

    pub fn total_speed(&self) -> u64 {
        self.upload_speed + self.download_speed
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        self.start_time.map(|start| Utc::now() - start)
    }

    pub fn update(&mut self, info: &TrafficInfo) {
        self.last_update = Some(info.timestamp);

        match info.traffic_type {
            TrafficType::Upload => {
                self.total_upload += info.bytes;
                self.upload_speed = info.speed_bps;
            },
            TrafficType::Download => {
                self.total_download += info.bytes;
                self.download_speed = info.speed_bps;
            },
        }
    }
}

pub struct TrafficMonitor {
    stats: Arc<RwLock<TrafficStats>>,
    sender: broadcast::Sender<TrafficInfo>,
    running: Arc<RwLock<bool>>,
}

impl TrafficMonitor {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self {
            stats: Arc::new(RwLock::new(TrafficStats::new())),
            sender,
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn subscribe(&self) -> Result<broadcast::Receiver<TrafficInfo>> {
        Ok(self.sender.subscribe())
    }

    pub async fn get_stats(&self) -> Result<TrafficStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    pub async fn record(&self, info: TrafficInfo) -> Result<()> {
        let mut stats = self.stats.write().await;
        stats.update(&info);
        drop(stats);

        let _ = self.sender.send(info);
        Ok(())
    }

    pub async fn record_upload(&self, bytes: u64, speed_bps: u64) -> Result<()> {
        let info = TrafficInfo::new(TrafficType::Upload, bytes, speed_bps);
        self.record(info).await
    }

    pub async fn record_download(&self, bytes: u64, speed_bps: u64) -> Result<()> {
        let info = TrafficInfo::new(TrafficType::Download, bytes, speed_bps);
        self.record(info).await
    }

    pub async fn reset_stats(&self) -> Result<()> {
        let mut stats = self.stats.write().await;
        *stats = TrafficStats::new();
        debug!("Traffic stats reset");
        Ok(())
    }

    pub async fn start_monitoring(&self, interval_ms: u64) -> Result<()> {
        let running = self.running.clone();
        {
            let mut r = running.write().await;
            if *r {
                return Ok(());
            }
            *r = true;
        }

        let stats = self.stats.clone();
        let running_clone = running.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(interval_ms));

            loop {
                ticker.tick().await;

                let is_running = *running_clone.read().await;
                if !is_running {
                    break;
                }

                let stats_guard = stats.read().await;
                debug!(
                    "Traffic: ↑ {} ({}/s) | ↓ {} ({}/s) | Connections: {}",
                    format_bytes(stats_guard.total_upload),
                    format_bytes_per_sec(stats_guard.upload_speed),
                    format_bytes(stats_guard.total_download),
                    format_bytes_per_sec(stats_guard.download_speed),
                    stats_guard.connections
                );
            }

            info!("Traffic monitoring stopped");
        });

        info!("Traffic monitoring started with interval {}ms", interval_ms);
        Ok(())
    }

    pub async fn stop_monitoring(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        debug!("Stopping traffic monitoring");
        Ok(())
    }

    pub async fn is_monitoring(&self) -> bool {
        *self.running.read().await
    }

    pub async fn update_connections(&self, count: usize) -> Result<()> {
        let mut stats = self.stats.write().await;
        stats.connections = count;
        Ok(())
    }

    pub async fn create_snapshot(&self) -> TrafficSnapshot {
        let stats = self.stats.read().await.clone();
        TrafficSnapshot {
            stats,
            timestamp: Utc::now(),
        }
    }
}

impl Default for TrafficMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSnapshot {
    pub stats: TrafficStats,
    pub timestamp: DateTime<Utc>,
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_bytes_per_sec(bps: u64) -> String {
    format!("{}s", format_bytes(bps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_info_new() {
        let info = TrafficInfo::new(TrafficType::Upload, 1000, 500);
        assert_eq!(info.traffic_type, TrafficType::Upload);
        assert_eq!(info.bytes, 1000);
        assert_eq!(info.speed_bps, 500);
        assert!(info.connection_id.is_none());
        assert!(info.process_name.is_none());
    }

    #[test]
    fn test_traffic_info_builder() {
        let info = TrafficInfo::new(TrafficType::Download, 2000, 1000)
            .with_connection("conn-123")
            .with_process("chrome")
            .with_source("127.0.0.1:8080")
            .with_destination("example.com:443");

        assert_eq!(info.connection_id, Some("conn-123".to_string()));
        assert_eq!(info.process_name, Some("chrome".to_string()));
        assert_eq!(info.source, Some("127.0.0.1:8080".to_string()));
        assert_eq!(info.destination, Some("example.com:443".to_string()));
    }

    #[test]
    fn test_traffic_info_format() {
        let info = TrafficInfo::new(TrafficType::Upload, 1024 * 1024, 1024 * 512);
        assert!(info.format_bytes().contains("MB"));
        assert!(info.format_speed().contains("KB"));
    }

    #[test]
    fn test_traffic_stats_new() {
        let stats = TrafficStats::new();
        assert_eq!(stats.total_upload, 0);
        assert_eq!(stats.total_download, 0);
        assert!(stats.start_time.is_some());
    }

    #[test]
    fn test_traffic_stats_update() {
        let mut stats = TrafficStats::new();

        let upload = TrafficInfo::new(TrafficType::Upload, 1000, 500);
        stats.update(&upload);
        assert_eq!(stats.total_upload, 1000);
        assert_eq!(stats.upload_speed, 500);

        let download = TrafficInfo::new(TrafficType::Download, 2000, 1000);
        stats.update(&download);
        assert_eq!(stats.total_download, 2000);
        assert_eq!(stats.download_speed, 1000);

        assert_eq!(stats.total_bytes(), 3000);
        assert_eq!(stats.total_speed(), 1500);
    }

    #[test]
    fn test_traffic_stats_duration() {
        let stats = TrafficStats::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration = stats.duration();
        assert!(duration.is_some());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[tokio::test]
    async fn test_traffic_monitor_new() {
        let monitor = TrafficMonitor::new();
        let stats = monitor.get_stats().await.unwrap();
        assert_eq!(stats.total_upload, 0);
        assert_eq!(stats.total_download, 0);
    }

    #[tokio::test]
    async fn test_traffic_monitor_record() {
        let monitor = TrafficMonitor::new();

        monitor.record_upload(1000, 500).await.unwrap();
        monitor.record_download(2000, 1000).await.unwrap();

        let stats = monitor.get_stats().await.unwrap();
        assert_eq!(stats.total_upload, 1000);
        assert_eq!(stats.total_download, 2000);
        assert_eq!(stats.upload_speed, 500);
        assert_eq!(stats.download_speed, 1000);
    }

    #[tokio::test]
    async fn test_traffic_monitor_subscribe() {
        let monitor = TrafficMonitor::new();
        let mut receiver = monitor.subscribe().await.unwrap();

        monitor.record_upload(1000, 500).await.unwrap();

        let info = receiver.try_recv().unwrap();
        assert_eq!(info.traffic_type, TrafficType::Upload);
        assert_eq!(info.bytes, 1000);
    }

    #[tokio::test]
    async fn test_traffic_monitor_reset() {
        let monitor = TrafficMonitor::new();

        monitor.record_upload(1000, 500).await.unwrap();
        monitor.reset_stats().await.unwrap();

        let stats = monitor.get_stats().await.unwrap();
        assert_eq!(stats.total_upload, 0);
    }

    #[tokio::test]
    async fn test_traffic_monitor_connections() {
        let monitor = TrafficMonitor::new();

        monitor.update_connections(42).await.unwrap();

        let stats = monitor.get_stats().await.unwrap();
        assert_eq!(stats.connections, 42);
    }

    #[tokio::test]
    async fn test_traffic_snapshot() {
        let monitor = TrafficMonitor::new();
        monitor.record_upload(1000, 500).await.unwrap();

        let snapshot = monitor.create_snapshot().await;
        assert_eq!(snapshot.stats.total_upload, 1000);
        assert!(snapshot.timestamp <= Utc::now());
    }
}
