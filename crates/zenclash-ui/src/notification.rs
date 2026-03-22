use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub on_proxy_change: bool,
    pub on_profile_update: bool,
    pub on_connection_error: bool,
    pub on_update_available: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            on_proxy_change: true,
            on_profile_update: true,
            on_connection_error: true,
            on_update_available: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub notification_type: NotificationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
}

pub struct NotificationManager {
    config: NotificationConfig,
}

impl NotificationManager {
    pub fn new(config: NotificationConfig) -> Self {
        Self { config }
    }

    pub fn show(&self, notification: Notification) {
        if !self.config.enabled {
            return;
        }

        #[cfg(target_os = "macos")]
        {
            self.show_macos(&notification);
        }

        #[cfg(target_os = "windows")]
        {
            self.show_windows(&notification);
        }

        #[cfg(target_os = "linux")]
        {
            self.show_linux(&notification);
        }
    }

    #[cfg(target_os = "macos")]
    fn show_macos(&self, notification: &Notification) {
        let script = format!(
            r#"display notification "{}" with title "{}""#,
            notification.body.replace('"', r#"\"#),
            notification.title.replace('"', r#"\"#)
        );

        std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "windows")]
    fn show_windows(&self, notification: &Notification) {
        let _ = notify_rust::Notification::new()
            .summary(&notification.title)
            .body(&notification.body)
            .show();
    }

    #[cfg(target_os = "linux")]
    fn show_linux(&self, notification: &Notification) {
        let _ = notify_rust::Notification::new()
            .summary(&notification.title)
            .body(&notification.body)
            .show();
    }

    pub fn notify_proxy_changed(&self, proxy_name: &str) {
        if self.config.on_proxy_change {
            self.show(Notification {
                title: "Proxy Changed".into(),
                body: format!("Switched to {}", proxy_name),
                notification_type: NotificationType::Info,
            });
        }
    }

    pub fn notify_profile_updated(&self, profile_name: &str) {
        if self.config.on_profile_update {
            self.show(Notification {
                title: "Profile Updated".into(),
                body: format!("{} has been updated", profile_name),
                notification_type: NotificationType::Success,
            });
        }
    }

    pub fn notify_connection_error(&self, error: &str) {
        if self.config.on_connection_error {
            self.show(Notification {
                title: "Connection Error".into(),
                body: error.to_string(),
                notification_type: NotificationType::Error,
            });
        }
    }

    pub fn notify_update_available(&self, version: &str) {
        if self.config.on_update_available {
            self.show(Notification {
                title: "Update Available".into(),
                body: format!("Version {} is available", version),
                notification_type: NotificationType::Info,
            });
        }
    }

    pub fn notify_core_started(&self) {
        self.show(Notification {
            title: "ZenClash".into(),
            body: "Core started successfully".into(),
            notification_type: NotificationType::Success,
        });
    }

    pub fn notify_core_stopped(&self) {
        self.show(Notification {
            title: "ZenClash".into(),
            body: "Core stopped".into(),
            notification_type: NotificationType::Info,
        });
    }

    pub fn notify_tun_enabled(&self) {
        self.show(Notification {
            title: "TUN Mode".into(),
            body: "TUN mode enabled".into(),
            notification_type: NotificationType::Success,
        });
    }

    pub fn notify_tun_disabled(&self) {
        self.show(Notification {
            title: "TUN Mode".into(),
            body: "TUN mode disabled".into(),
            notification_type: NotificationType::Info,
        });
    }

    pub fn notify_sysproxy_enabled(&self) {
        self.show(Notification {
            title: "System Proxy".into(),
            body: "System proxy enabled".into(),
            notification_type: NotificationType::Success,
        });
    }

    pub fn notify_sysproxy_disabled(&self) {
        self.show(Notification {
            title: "System Proxy".into(),
            body: "System proxy disabled".into(),
            notification_type: NotificationType::Info,
        });
    }
}
