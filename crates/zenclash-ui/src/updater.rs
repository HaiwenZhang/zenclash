use gpui::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zenclash_core::prelude::*;
use anyhow::Result;

pub struct UpdaterState {
    pub current_version: String,
    pub latest_version: Option<AppVersion>,
    pub download_progress: Option<DownloadProgress>,
    pub status: UpdateStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppVersion {
    pub version: String,
    pub release_date: String,
    pub release_notes: Option<String>,
    pub files: Vec<UpdateFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFile {
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub status: DownloadStatus,
    pub percent: u8,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Checking,
    Downloading,
    Verifying,
    Ready,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable,
    Downloading,
    ReadyToInstall,
    Error,
}

impl Default for UpdaterState {
    fn default() -> Self {
        Self {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            latest_version: None,
            download_progress: None,
            status: UpdateStatus::UpToDate,
        }
    }
}

impl UpdaterState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn compare_versions(a: &str, b: &str) -> i8 {
    let parse_part = |part: &str| {
        let num_part = part.split('-').next().unwrap_or("0");
        num_part.parse::<u32>().unwrap_or(0)
    };

    let v1: Vec<u32> = a
        .trim_start_matches('v')
        .split('.')
        .map(parse_part)
        .collect();
    let v2: Vec<u32> = b
        .trim_start_matches('v')
        .split('.')
        .map(parse_part)
        .collect();

    for i in 0..v1.len().max(v2.len()) {
        let n1 = v1.get(i).copied().unwrap_or(0);
        let n2 = v2.get(i).copied().unwrap_or(0);
        if n1 > n2 {
            return 1;
        }
        if n1 < n2 {
            return -1;
        }
    }
    0
}

pub fn get_platform_update_filename(version: &str) -> Option<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch) {
        ("windows", "x86_64") => Some(format!("zenclash-windows-{}-x64-setup.exe", version)),
        ("windows", "x86") => Some(format!("zenclash-windows-{}-x86-setup.exe", version)),
        ("windows", "aarch64") => Some(format!("zenclash-windows-{}-arm64-setup.exe", version)),
        ("macos", "x86_64") => Some(format!("zenclash-macos-{}-x64.pkg", version)),
        ("macos", "aarch64") => Some(format!("zenclash-macos-{}-arm64.pkg", version)),
        ("linux", "x86_64") => Some(format!("zenclash-linux-{}-x64.AppImage", version)),
        ("linux", "aarch64") => Some(format!("zenclash-linux-{}-arm64.AppImage", version)),
        _ => None,
    }
}

pub fn updates_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ZenClash")
        .join("updates")
}

pub fn themes_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ZenClash")
        .join("themes")
}

pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ZenClash")
        .join("data")
}

pub struct UpdaterManager {
    state: Entity<UpdaterState>,
}

impl UpdaterManager {
    pub fn new(cx: &mut App) -> Self {
        let state = cx.new(|_| UpdaterState::new());
        Self { state }
    }

    pub fn state(&self) -> Entity<UpdaterState> {
        self.state.clone()
    }

    pub async fn check_update(
        &self,
        cx: &mut AsyncApp,
    ) -> Result<Option<AppVersion>, ZenClashError> {
        let client = HttpClient::new(HttpClientConfig::default())
            .map_err(|e| ZenClashError::Http(e))?;

        let response = client
            .get_text("https://github.com/zenclash/zenclash/releases/latest/download/latest.yml")
            .await
            .map_err(|e| ZenClashError::Network(e.to_string()))?;

        let version: AppVersion =
            serde_yaml::from_str(&response).map_err(|e| ZenClashError::Yaml(e))?;

        let current = env!("CARGO_PKG_VERSION");

        if compare_versions(&version.version, current) > 0 {
            self.state
                .update(cx, |state, _| {
                    state.latest_version = Some(version.clone());
                    state.status = UpdateStatus::UpdateAvailable;
                })
                .ok();
            Ok(Some(version))
        } else {
            self.state
                .update(cx, |state, _| {
                    state.status = UpdateStatus::UpToDate;
                })
                .ok();
            Ok(None)
        }
    }

    pub async fn download_update(
        &self,
        version: &str,
        cx: &mut AsyncApp,
    ) -> Result<PathBuf, ZenClashError> {
        let filename = get_platform_update_filename(version).ok_or_else(|| {
            ZenClashError::Config("Platform not supported for auto-update".into())
        })?;

        let url = format!(
            "https://github.com/zenclash/zenclash/releases/download/v{}/{}",
            version, filename
        );

        let updates_path = updates_dir();
        std::fs::create_dir_all(&updates_path).map_err(|e| ZenClashError::Io(e))?;

        let file_path = updates_path.join(&filename);

        self.state
            .update(cx, |state, _| {
                state.status = UpdateStatus::Downloading;
                state.download_progress = Some(DownloadProgress {
                    status: DownloadStatus::Downloading,
                    percent: 0,
                    downloaded_bytes: 0,
                    total_bytes: 0,
                });
            })
            .ok();

        let client = HttpClient::new(HttpClientConfig::default())
            .map_err(|e| ZenClashError::Http(e))?;
        let data = client
            .get_bytes(&url)
            .await
            .map_err(|e| ZenClashError::Network(e.to_string()))?;

        self.state
            .update(cx, |state, _| {
                state.download_progress = Some(DownloadProgress {
                    status: DownloadStatus::Verifying,
                    percent: 100,
                    downloaded_bytes: data.len() as u64,
                    total_bytes: data.len() as u64,
                });
            })
            .ok();

        std::fs::write(&file_path, &data).map_err(|e| ZenClashError::Io(e))?;

        self.state
            .update(cx, |state, _| {
                state.status = UpdateStatus::ReadyToInstall;
                state.download_progress = Some(DownloadProgress {
                    status: DownloadStatus::Ready,
                    percent: 100,
                    downloaded_bytes: data.len() as u64,
                    total_bytes: data.len() as u64,
                });
            })
            .ok();

        Ok(file_path)
    }

    pub fn install_update(&self, _file_path: &PathBuf) -> Result<(), ZenClashError> {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new(_file_path)
                .args(&["/S", "--force-run"])
                .spawn()
                .map_err(|e| ZenClashError::Io(e))?;
            std::process::exit(0);
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(_file_path)
                .spawn()
                .map_err(|e| ZenClashError::Io(e))?;
        }

        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("chmod")
                .args(&["+x", _file_path.to_str().unwrap()])
                .spawn()
                .map_err(|e| ZenClashError::Io(e))?;

            std::process::Command::new(_file_path)
                .spawn()
                .map_err(|e| ZenClashError::Io(e))?;
            std::process::exit(0);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub key: String,
    pub label: String,
}

pub struct ThemeManager;

impl ThemeManager {
    pub fn themes_dir() -> PathBuf {
        themes_dir()
    }

    pub fn list_themes() -> Result<Vec<ThemeInfo>, ZenClashError> {
        let themes_path = Self::themes_dir();
        if !themes_path.exists() {
            std::fs::create_dir_all(&themes_path).map_err(|e| ZenClashError::Io(e))?;
            return Ok(vec![ThemeInfo {
                key: "default.css".into(),
                label: "Default".into(),
            }]);
        }

        let mut themes = vec![ThemeInfo {
            key: "default.css".into(),
            label: "Default".into(),
        }];

        for entry in std::fs::read_dir(&themes_path).map_err(|e| ZenClashError::Io(e))? {
            let entry = entry.map_err(|e| ZenClashError::Io(e))?;
            let path = entry.path();

            if path.extension().map(|e| e == "css").unwrap_or(false) {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown.css")
                    .to_string();

                let label = if let Ok(content) = std::fs::read_to_string(&path) {
                    content
                        .lines()
                        .next()
                        .and_then(|line| {
                            if line.starts_with("/*") && line.ends_with("*/") {
                                Some(line[2..line.len() - 2].trim().to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| filename.clone())
                } else {
                    filename.clone()
                };

                themes.push(ThemeInfo {
                    key: filename,
                    label,
                });
            }
        }

        Ok(themes)
    }

    pub fn read_theme(name: &str) -> Result<String, ZenClashError> {
        let path = Self::themes_dir().join(name);
        if !path.exists() {
            return Ok(String::new());
        }
        std::fs::read_to_string(&path).map_err(|e| ZenClashError::Io(e))
    }

    pub fn write_theme(name: &str, css: &str) -> Result<(), ZenClashError> {
        let path = Self::themes_dir().join(name);
        std::fs::write(&path, css).map_err(|e| ZenClashError::Io(e))
    }

    pub async fn fetch_themes(_proxy_port: u16) -> Result<(), ZenClashError> {
        let url = "https://github.com/zenclash/theme-hub/releases/download/latest/themes.zip";

        let client = HttpClient::new(HttpClientConfig::default())
            .map_err(|e| ZenClashError::Http(e))?;

        let data = client
            .get_bytes(url)
            .await
            .map_err(|e| ZenClashError::Network(e.to_string()))?;

        let themes_path = Self::themes_dir();
        std::fs::create_dir_all(&themes_path).map_err(|e| ZenClashError::Io(e))?;

        let reader = std::io::Cursor::new(&data);
        let mut archive =
            zip::ZipArchive::new(reader).map_err(|e| ZenClashError::Unknown(e.to_string()))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| ZenClashError::Io(std::io::Error::other(e)))?;
            let outpath = match file.enclosed_name() {
                Some(path) => themes_path.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath).map_err(|e| ZenClashError::Io(e))?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p).map_err(|e| ZenClashError::Io(e))?;
                    }
                }
                let mut outfile =
                    std::fs::File::create(&outpath).map_err(|e| ZenClashError::Io(e))?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| ZenClashError::Io(e))?;
            }
        }

        Ok(())
    }

    pub fn import_theme(source_path: &str) -> Result<(), ZenClashError> {
        let source = std::path::Path::new(source_path);
        if !source.exists() {
            return Err(ZenClashError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Source file not found",
            )));
        }

        let filename = source
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| ZenClashError::Config("Invalid filename".into()))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| ZenClashError::Unknown(e.to_string()))?
            .as_secs();

        let dest_name = format!("{:x}-{}", timestamp, filename);
        let dest_path = Self::themes_dir().join(&dest_name);

        std::fs::copy(source, dest_path).map_err(|e| ZenClashError::Io(e))?;

        Ok(())
    }
}
