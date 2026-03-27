use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zenclash")
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zenclash")
}

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zenclash")
}

pub fn profiles_dir() -> PathBuf {
    data_dir().join("profiles")
}

pub fn core_log_path() -> PathBuf {
    data_dir().join("logs").join("core.log")
}

pub fn app_config_path() -> PathBuf {
    config_dir().join("config.yaml")
}

pub fn mihomo_config_path() -> PathBuf {
    config_dir().join("mihomo.yaml")
}

pub fn profile_config_path() -> PathBuf {
    data_dir().join("profile.yaml")
}

pub fn override_config_path() -> PathBuf {
    data_dir().join("override.yaml")
}

pub fn mihomo_work_dir() -> PathBuf {
    data_dir().join("work")
}

pub fn mihomo_work_config_path() -> PathBuf {
    mihomo_work_dir().join("config.yaml")
}

pub fn mihomo_core_dir() -> PathBuf {
    data_dir().join("core")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_dir_structure() {
        let data = data_dir();
        assert!(data.ends_with("zenclash"));
        assert_eq!(data.file_name().unwrap(), "zenclash");
    }

    #[test]
    fn test_config_dir_structure() {
        let config = config_dir();
        assert!(config.ends_with("zenclash"));
    }

    #[test]
    fn test_profiles_dir_is_subdir() {
        let profiles = profiles_dir();
        let data = data_dir();
        assert_eq!(profiles.parent().unwrap(), data);
        assert!(profiles.to_string_lossy().contains("profiles"));
    }

    #[test]
    fn test_core_log_path_structure() {
        let log_path = core_log_path();
        assert!(log_path.ends_with("core.log"));
        assert!(log_path.parent().unwrap().ends_with("logs"));
    }

    #[test]
    fn test_app_config_path() {
        let path = app_config_path();
        assert!(path.ends_with("config.yaml"));
    }

    #[test]
    fn test_mihomo_paths() {
        let work_dir = mihomo_work_dir();
        let work_config = mihomo_work_config_path();
        let core_dir = mihomo_core_dir();

        assert!(work_dir.ends_with("work"));
        assert!(work_config.ends_with("config.yaml"));
        assert!(core_dir.ends_with("core"));
    }

    #[test]
    fn test_path_consistency() {
        let data = data_dir();
        let config = config_dir();
        let cache = cache_dir();

        assert!(!data.as_os_str().is_empty());
        assert!(!config.as_os_str().is_empty());
        assert!(!cache.as_os_str().is_empty());
        assert!(data.ends_with("zenclash"));
        assert!(config.ends_with("zenclash"));
        assert!(cache.ends_with("zenclash"));
    }
}
