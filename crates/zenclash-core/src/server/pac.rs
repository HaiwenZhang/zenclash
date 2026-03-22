use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum PacServerError {
    #[error("Failed to start PAC server: {0}")]
    StartFailed(String),

    #[error("Failed to stop PAC server: {0}")]
    StopFailed(String),

    #[error("Port already in use: {0}")]
    PortInUse(u16),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct PacServerConfig {
    pub port: u16,
    pub host: String,
    pub mixed_port: u16,
    pub custom_script: Option<String>,
}

impl Default for PacServerConfig {
    fn default() -> Self {
        Self {
            port: 10000,
            host: "127.0.0.1".into(),
            mixed_port: 7890,
            custom_script: None,
        }
    }
}

pub struct PacServer {
    config: Arc<RwLock<PacServerConfig>>,
    running: Arc<RwLock<bool>>,
    port: Arc<RwLock<u16>>,
}

impl PacServer {
    pub fn new(config: PacServerConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            running: Arc::new(RwLock::new(false)),
            port: Arc::new(RwLock::new(0)),
        }
    }

    pub fn default_pac_script(mixed_port: u16) -> String {
        format!(
            r#"function FindProxyForURL(url, host) {{
    return "PROXY 127.0.0.1:{}; SOCKS5 127.0.0.1:{}; DIRECT;";
}}
"#,
            mixed_port, mixed_port
        )
    }

    pub async fn get_port(&self) -> u16 {
        *self.port.read().await
    }

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    pub async fn update_mixed_port(&self, port: u16) {
        let mut config = self.config.write().await;
        config.mixed_port = port;
    }

    pub async fn start(&self) -> Result<(), PacServerError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }

        let config = self.config.read().await;
        let host = config.host.clone();
        let mixed_port = config.mixed_port;
        let custom_script = config.custom_script.clone();
        let start_port = config.port;
        drop(config);

        let addr: SocketAddr = format!("{}:{}", host, start_port)
            .parse()
            .map_err(|e| PacServerError::StartFailed(e.to_string()))?;

        let port = Arc::clone(&self.port);
        let running_flag = Arc::clone(&self.running);

        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            use tokio::net::TcpListener;

            let listener = match TcpListener::bind(&addr).await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Failed to bind PAC server: {}", e);
                    return;
                },
            };

            let local_port = listener.local_addr().map(|a| a.port()).unwrap_or(0);
            *port.write().await = local_port;
            *running_flag.write().await = true;

            loop {
                if !*running_flag.read().await {
                    break;
                }

                let (mut stream, _) = match listener.accept().await {
                    Ok(conn) => conn,
                    Err(_) => continue,
                };

                let pac_script = if let Some(ref script) = custom_script {
                    script.replace("%mixed-port%", &mixed_port.to_string())
                } else {
                    Self::default_pac_script(mixed_port)
                };

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/x-ns-proxy-autoconfig\r\nContent-Length: {}\r\n\r\n{}",
                    pac_script.len(),
                    pac_script
                );

                let _ = stream.write_all(response.as_bytes()).await;
                let _ = stream.shutdown().await;
            }
        });

        Ok(())
    }

    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        *self.port.write().await = 0;
    }

    pub async fn get_pac_url(&self) -> Option<String> {
        let running = self.running.read().await;
        if !*running {
            return None;
        }

        let config = self.config.read().await;
        let port = *self.port.read().await;

        Some(format!("http://{}:{}/pac", config.host, port))
    }
}

impl Default for PacServer {
    fn default() -> Self {
        Self::new(PacServerConfig::default())
    }
}

pub async fn find_available_port(start_port: u16) -> Result<u16, PacServerError> {
    use tokio::net::TcpListener;

    for port in start_port..=65535 {
        let addr: SocketAddr = format!("127.0.0.1:{}", port)
            .parse()
            .map_err(|e| PacServerError::StartFailed(e.to_string()))?;

        match TcpListener::bind(&addr).await {
            Ok(listener) => {
                drop(listener);
                return Ok(port);
            },
            Err(_) => continue,
        }
    }

    Err(PacServerError::StartFailed(
        "No available port found".into(),
    ))
}
