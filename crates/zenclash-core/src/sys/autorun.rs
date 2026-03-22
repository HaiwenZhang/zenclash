use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AutoRunError {
    #[error("Platform not supported for auto-run")]
    UnsupportedPlatform,

    #[error("Failed to enable auto-run: {0}")]
    EnableFailed(String),

    #[error("Failed to disable auto-run: {0}")]
    DisableFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct AutoRunManager;

impl AutoRunManager {
    pub fn is_enabled() -> Result<bool, AutoRunError> {
        #[cfg(target_os = "windows")]
        {
            Self::is_enabled_windows()
        }

        #[cfg(target_os = "macos")]
        {
            Self::is_enabled_macos()
        }

        #[cfg(target_os = "linux")]
        {
            Self::is_enabled_linux()
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Ok(false)
        }
    }

    pub fn enable() -> Result<(), AutoRunError> {
        #[cfg(target_os = "windows")]
        {
            Self::enable_windows()
        }

        #[cfg(target_os = "macos")]
        {
            Self::enable_macos()
        }

        #[cfg(target_os = "linux")]
        {
            Self::enable_linux()
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(AutoRunError::UnsupportedPlatform)
        }
    }

    pub fn disable() -> Result<(), AutoRunError> {
        #[cfg(target_os = "windows")]
        {
            Self::disable_windows()
        }

        #[cfg(target_os = "macos")]
        {
            Self::disable_macos()
        }

        #[cfg(target_os = "linux")]
        {
            Self::disable_linux()
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(AutoRunError::UnsupportedPlatform)
        }
    }

    #[cfg(target_os = "windows")]
    fn is_enabled_windows() -> Result<bool, AutoRunError> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";

        if let Ok(key) = hkcu.open_subkey(path) {
            if key.get_value::<String, _>("ZenClash").is_ok() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    #[cfg(target_os = "windows")]
    fn enable_windows() -> Result<(), AutoRunError> {
        use winreg::enums::*;
        use winreg::RegKey;

        let exe_path =
            std::env::current_exe().map_err(|e| AutoRunError::EnableFailed(e.to_string()))?;

        let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";

        let (key, _) = hkcu
            .create_subkey(path)
            .map_err(|e| AutoRunError::EnableFailed(e.to_string()))?;

        key.set_value("ZenClash", &exe_path.to_string_lossy().to_string())
            .map_err(|e| AutoRunError::EnableFailed(e.to_string()))?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn disable_windows() -> Result<(), AutoRunError> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";

        if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_WRITE) {
            let _ = key.delete_value("ZenClash");
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn is_enabled_macos() -> Result<bool, AutoRunError> {
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get the name of every login item")
            .output()
            .map_err(|e| AutoRunError::EnableFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains("ZenClash"))
    }

    #[cfg(target_os = "macos")]
    fn enable_macos() -> Result<(), AutoRunError> {
        let exe_path =
            std::env::current_exe().map_err(|e| AutoRunError::EnableFailed(e.to_string()))?;

        let app_path = exe_path
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or(exe_path.clone());

        let script = format!(
            "tell application \"System Events\" to make login item at end with properties {{path:\"{}\", hidden:false}}",
            app_path.to_string_lossy()
        );

        std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| AutoRunError::EnableFailed(e.to_string()))?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn disable_macos() -> Result<(), AutoRunError> {
        let script = "tell application \"System Events\" to delete login item \"ZenClash\"";

        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output();

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn is_enabled_linux() -> Result<bool, AutoRunError> {
        let autostart_path = dirs::config_dir()
            .map(|p| p.join("autostart").join("zenclash.desktop"))
            .unwrap_or_else(|| PathBuf::from(".config/autostart/zenclash.desktop"));

        Ok(autostart_path.exists())
    }

    #[cfg(target_os = "linux")]
    fn enable_linux() -> Result<(), AutoRunError> {
        let exe_path =
            std::env::current_exe().map_err(|e| AutoRunError::EnableFailed(e.to_string()))?;

        let autostart_dir = dirs::config_dir()
            .map(|p| p.join("autostart"))
            .unwrap_or_else(|| PathBuf::from(".config/autostart"));

        std::fs::create_dir_all(&autostart_dir)?;

        let desktop_content = format!(
            "[Desktop Entry]\n\
             Name=ZenClash\n\
             Exec={} %U\n\
             Terminal=false\n\
             Type=Application\n\
             Icon=zenclash\n\
             StartupWMClass=zenclash\n\
             Comment=ZenClash Proxy Manager\n\
             Categories=Utility;\n",
            exe_path.to_string_lossy()
        );

        let desktop_path = autostart_dir.join("zenclash.desktop");
        std::fs::write(&desktop_path, desktop_content)?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn disable_linux() -> Result<(), AutoRunError> {
        let autostart_path = dirs::config_dir()
            .map(|p| p.join("autostart").join("zenclash.desktop"))
            .unwrap_or_else(|| PathBuf::from(".config/autostart/zenclash.desktop"));

        if autostart_path.exists() {
            std::fs::remove_file(&autostart_path)?;
        }

        Ok(())
    }
}
