use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

pub const APP_DIRECTORY_NAME: &str = "M32";
pub const CONFIG_FILE_NAME: &str = "config.json";
pub const CACHE_DIRECTORY_NAME: &str = "cache";
pub const LOG_DIRECTORY_NAME: &str = "logs";
pub const CRASH_DIRECTORY_NAME: &str = "crashes";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub root: PathBuf,
    pub config_file: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
    pub crash_dir: PathBuf,
}

impl AppPaths {
    pub fn from_local_app_data(local_app_data: &Path) -> Self {
        let root = local_app_data.join(APP_DIRECTORY_NAME);

        Self {
            config_file: root.join(CONFIG_FILE_NAME),
            cache_dir: root.join(CACHE_DIRECTORY_NAME),
            log_dir: root.join(LOG_DIRECTORY_NAME),
            crash_dir: root.join(CRASH_DIRECTORY_NAME),
            root,
        }
    }

    pub fn discover() -> Result<Self, PathError> {
        let local_app_data = env::var_os("LOCALAPPDATA").ok_or(PathError::LocalAppDataUnavailable)?;

        if local_app_data.is_empty() {
            return Err(PathError::LocalAppDataUnavailable);
        }

        Ok(Self::from_local_app_data(Path::new(&local_app_data)))
    }

    pub fn ensure_directories(&self) -> Result<(), PathError> {
        for directory in [&self.root, &self.cache_dir, &self.log_dir, &self.crash_dir] {
            fs::create_dir_all(directory).map_err(|source| PathError::CreateDirectory {
                path: directory.clone(),
                source,
            })?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum PathError {
    LocalAppDataUnavailable,
    CreateDirectory { path: PathBuf, source: io::Error },
}

impl std::fmt::Display for PathError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LocalAppDataUnavailable => {
                write!(formatter, "LOCALAPPDATA is unavailable or empty")
            }
            Self::CreateDirectory { path, source } => {
                write!(
                    formatter,
                    "failed to create M32 directory '{}': {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for PathError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LocalAppDataUnavailable => None,
            Self::CreateDirectory { source, .. } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn windows_local_app_data_layout_is_stable() {
        let paths = AppPaths::from_local_app_data(Path::new(r"C:\Users\M32Test\AppData\Local"));

        assert_eq!(paths.root, PathBuf::from(r"C:\Users\M32Test\AppData\Local").join("M32"));
        assert_eq!(paths.config_file, paths.root.join("config.json"));
        assert_eq!(paths.cache_dir, paths.root.join("cache"));
        assert_eq!(paths.log_dir, paths.root.join("logs"));
        assert_eq!(paths.crash_dir, paths.root.join("crashes"));
    }

    #[test]
    fn directory_names_match_locked_contract() {
        assert_eq!(APP_DIRECTORY_NAME, "M32");
        assert_eq!(CONFIG_FILE_NAME, "config.json");
        assert_eq!(CACHE_DIRECTORY_NAME, "cache");
        assert_eq!(LOG_DIRECTORY_NAME, "logs");
        assert_eq!(CRASH_DIRECTORY_NAME, "crashes");
    }
}
