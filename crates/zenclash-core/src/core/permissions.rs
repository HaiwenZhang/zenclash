use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PermissionError {
    #[error("Failed to grant TUN permission: {0}")]
    GrantFailed(String),

    #[error("Admin privileges required")]
    AdminRequired,

    #[error("Platform not supported")]
    UnsupportedPlatform,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct PermissionManager;

impl PermissionManager {
    pub fn check_admin() -> bool {
        #[cfg(target_os = "windows")]
        {
            Self::check_admin_windows()
        }

        #[cfg(unix)]
        {
            Self::check_admin_unix()
        }

        #[cfg(not(any(target_os = "windows", unix)))]
        {
            false
        }
    }

    #[cfg(target_os = "windows")]
    fn check_admin_windows() -> bool {
        std::process::Command::new("net")
            .args(["session"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(unix)]
    fn check_admin_unix() -> bool {
        unsafe { libc::getuid() == 0 }
    }

    pub fn grant_tun_permission() -> Result<(), PermissionError> {
        #[cfg(target_os = "macos")]
        {
            Self::grant_tun_macos()
        }

        #[cfg(target_os = "linux")]
        {
            Self::grant_tun_linux()
        }

        #[cfg(target_os = "windows")]
        {
            Err(PermissionError::UnsupportedPlatform)
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(PermissionError::UnsupportedPlatform)
        }
    }

    #[cfg(target_os = "macos")]
    fn grant_tun_macos() -> Result<(), PermissionError> {
        let exe_path =
            std::env::current_exe().map_err(|e| PermissionError::GrantFailed(e.to_string()))?;

        let script = format!(
            r#"do shell script "chown root:admin '{}' && chmod +sx '{}'" with administrator privileges"#,
            exe_path.display(),
            exe_path.display()
        );

        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| PermissionError::GrantFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PermissionError::GrantFailed(stderr.to_string()));
        }

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn grant_tun_linux() -> Result<(), PermissionError> {
        let exe_path =
            std::env::current_exe().map_err(|e| PermissionError::GrantFailed(e.to_string()))?;

        let output = std::process::Command::new("pkexec")
            .args(["chmod", "+sx", &exe_path.to_string_lossy()])
            .output()
            .map_err(|e| PermissionError::GrantFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PermissionError::GrantFailed(stderr.to_string()));
        }

        Ok(())
    }

    pub fn check_tun_permission() -> bool {
        #[cfg(unix)]
        {
            if let Ok(exe_path) = std::env::current_exe() {
                if let Ok(metadata) = std::fs::metadata(&exe_path) {
                    use std::os::unix::fs::MetadataExt;
                    let mode = metadata.mode();
                    let setuid = (mode & 0o4000) != 0;
                    let exec = (mode & 0o111) != 0;
                    return setuid && exec;
                }
            }
            false
        }

        #[cfg(target_os = "windows")]
        {
            Self::check_admin()
        }

        #[cfg(not(any(target_os = "windows", unix)))]
        {
            true
        }
    }

    #[cfg(target_os = "windows")]
    pub fn setup_firewall() -> Result<(), PermissionError> {
        let exe_path =
            std::env::current_exe().map_err(|e| PermissionError::GrantFailed(e.to_string()))?;

        let rules = ["ZenClash", "mihomo", "mihomo-alpha"];

        for rule in &rules {
            let _ = std::process::Command::new("netsh")
                .args([
                    "advfirewall",
                    "firewall",
                    "delete",
                    "rule",
                    &format!("name={}", rule),
                ])
                .output();
        }

        let output = std::process::Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "add",
                "rule",
                &format!("name=ZenClash"),
                "dir=in",
                "action=allow",
                &format!("program={}", exe_path.display()),
                "enable=yes",
            ])
            .output()
            .map_err(|e| PermissionError::GrantFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PermissionError::GrantFailed(stderr.to_string()));
        }

        Ok(())
    }
}
