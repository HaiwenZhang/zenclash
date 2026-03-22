use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FloatingWindowPosition {
    #[default]
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    Custom(i32, i32),
}

#[derive(Debug, Clone)]
pub struct FloatingWindowState {
    pub visible: bool,
    pub position: FloatingWindowPosition,
    pub width: u32,
    pub height: u32,
    pub show_upload: bool,
    pub show_download: bool,
    pub show_connections: bool,
    pub upload_speed: u64,
    pub download_speed: u64,
    pub connection_count: usize,
    pub proxy_name: Option<String>,
}

impl Default for FloatingWindowState {
    fn default() -> Self {
        Self {
            visible: false,
            position: FloatingWindowPosition::default(),
            width: 120,
            height: 42,
            show_upload: true,
            show_download: true,
            show_connections: false,
            upload_speed: 0,
            download_speed: 0,
            connection_count: 0,
            proxy_name: None,
        }
    }
}

pub struct FloatingWindowManager {
    state: Arc<RwLock<FloatingWindowState>>,
}

impl FloatingWindowManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(FloatingWindowState::default())),
        }
    }

    pub async fn show(&self) {
        let mut state = self.state.write().await;
        state.visible = true;
    }

    pub async fn hide(&self) {
        let mut state = self.state.write().await;
        state.visible = false;
    }

    pub async fn toggle(&self) {
        let mut state = self.state.write().await;
        state.visible = !state.visible;
    }

    pub async fn is_visible(&self) -> bool {
        self.state.read().await.visible
    }

    pub async fn update_traffic(&self, upload: u64, download: u64) {
        let mut state = self.state.write().await;
        state.upload_speed = upload;
        state.download_speed = download;
    }

    pub async fn update_connections(&self, count: usize) {
        let mut state = self.state.write().await;
        state.connection_count = count;
    }

    pub async fn update_proxy(&self, name: Option<String>) {
        let mut state = self.state.write().await;
        state.proxy_name = name;
    }

    pub async fn set_position(&self, position: FloatingWindowPosition) {
        let mut state = self.state.write().await;
        state.position = position;
    }

    pub async fn set_size(&self, width: u32, height: u32) {
        let mut state = self.state.write().await;
        state.width = width;
        state.height = height;
    }

    pub async fn get_state(&self) -> FloatingWindowState {
        self.state.read().await.clone()
    }

    pub fn spawn_window(&self) {
        #[cfg(target_os = "macos")]
        {
            self.spawn_macos();
        }

        #[cfg(target_os = "windows")]
        {
            self.spawn_windows();
        }

        #[cfg(target_os = "linux")]
        {
            self.spawn_linux();
        }
    }

    #[cfg(target_os = "macos")]
    fn spawn_macos(&self) {
        // On macOS, the floating window is handled by the GPUI application layer
        // This would integrate with the main GPUI window system
    }

    #[cfg(target_os = "windows")]
    fn spawn_windows(&self) {
        // On Windows, use a separate topmost window
    }

    #[cfg(target_os = "linux")]
    fn spawn_linux(&self) {
        // On Linux, depends on the window manager
    }
}

impl Default for FloatingWindowManager {
    fn default() -> Self {
        Self::new()
    }
}
