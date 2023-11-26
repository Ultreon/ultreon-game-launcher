use semver::VersionReq;
use std::path::PathBuf;
use std::env::consts::OS;
use tauri::InvokeError;
use std::io;
use crate::game::{GameConfig, GameMetadata};

pub fn get_version_req(cfg: &GameConfig) -> Result<VersionReq, Error> {
    let mut vv = cfg.sdk.versions.clone();
    #[allow(deprecated)]
    if vv.to_string() == VersionReq::default().to_string() {
        let map_err = VersionReq::parse(&cfg.sdk.version.to_string())
            .map_err(|e| Error::Launch(format!("Semantic versioning error: {:?}", e)))?
            .clone();
        vv = map_err.to_owned();
    }
    Ok(vv)
}

pub fn get_classpath(cfg: &GameConfig, meta: GameMetadata, data_dir: &String) -> Vec<String> {
    let mut cp = vec![];
    for entry in cfg.classpath.iter() {
        cp.push(data_dir.to_string() + "/" + entry)
    }

    cp.push(format!("{}/games/{}/versions/{}/{}.jar", data_dir, cfg.game, meta.version, meta.version));
    cp
}

/// Runtime errors that can happen inside a Tauri application.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Generic error.
    #[error("Runtime error: {0}")]
    Generic(#[from] anyhow::Error),
    /// Failed to download a file.
    #[error("Download error: {0}")]
    Download(String),
    /// Failed to launch a game version.
    #[error("Launch error: {0}")]
    Launch(String),
    /// Failed to launch a game version.
    #[error("Fetch error: {0}")]
    Fetch(String),
    /// Failed to serialize/deserialize.
    #[error("JSON error: {0}")]
    Json(serde_json::Error),
    /// Unknown API
    #[error("unknown api: {0:?}")]
    UnknownApi(Option<serde_json::Error>),
    /// IO error.
    #[error("{0}")]
    Io(#[from] io::Error),
    /// Zip error.
    #[error("{0}")]
    Zip(#[from] zip::result::ZipError),
    /// Poisoned error.
    #[error("poisoned state: {0}")]
    Poisoned(String),
    /// Poisoned error.
    #[error("failed to lock: {0}")]
    TryLock(String),
    /// Path not allowed by the scope.
    #[error("path not allowed on the configured scope: {0}")]
    PathNotAllowed(PathBuf),
    /// Path not allowed by the scope.
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Program not allowed by the scope.
    #[error("program not allowed on the configured shell scope: {0}")]
    ProgramNotAllowed(PathBuf),
}

impl Error {
    #[allow(dead_code)]
    pub(crate) fn into_anyhow(self) -> anyhow::Error {
        anyhow::anyhow!(self.to_string())
    }

    pub(crate) fn msg(to_string: &str) -> Error {
        Error::Generic(anyhow::Error::msg(to_string.to_string()))
    }
}

impl From<Error> for InvokeError {
    fn from(value: Error) -> InvokeError {
        InvokeError::from_anyhow(value.into_anyhow())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        if error.to_string().contains("unknown variant") {
            Self::UnknownApi(Some(error))
        } else {
            Self::Json(error)
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(error: std::sync::PoisonError<T>) -> Self {
        Self::Poisoned(error.to_string())
    }
}

impl<T> From<std::sync::TryLockError<T>> for Error {
    fn from(error: std::sync::TryLockError<T>) -> Self {
        Self::TryLock(error.to_string())
    }
}

#[cfg(target_os = "linux")]
pub const PATH_SEPARATOR: &str = ":";

#[cfg(target_os = "macos")]
const PATH_SEPARATOR: &str = ":";

#[cfg(target_os = "windows")]
const PATH_SEPARATOR: &str = ";";

pub fn show_error(x: &str) {
    println!("{}", x);
    panic!("{}", x);
}

pub fn get_data_dir() -> PathBuf {
    match OS {
        "windows" => {
            // Windows-specific code to get the app data directory
            match std::env::var("APPDATA") {
                Ok(appdata) => Some(PathBuf::from(appdata)),
                Err(_) => None,
            }
        }
        "macos" => {
            // macOS-specific code to get the app data directory
            match std::env::var("HOME") {
                Ok(home) => Some(PathBuf::from(home).join("Library/Application Support")), // macOS convention
                Err(_) => None,
            }
        }
        "linux" => {
            // Linux-specific code to get the app data directory
            match std::env::var("XDG_CONFIG_HOME") {
                Ok(config_home) => Some(PathBuf::from(config_home)),
                Err(_) => {
                    match std::env::var("HOME") {
                        Ok(home) => Some(PathBuf::from(home).join(".config")), // Linux convention
                        Err(_) => None,
                    }
                }
            }
        }
        _ => {
            // Panic if the platform couldn't be recognized.
            panic!("Unsupported platform");
        }
    }.map(|it| it.join("UltreonGameLauncher")).unwrap()
}
