// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate msgbox;

use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::exit;
use std::sync::Mutex;
use tauri::api::dialog::blocking::FileDialogBuilder;
use tauri::InvokeError;
use tauri::State;
use zip::ZipArchive; // Note the updated import

/// Runtime errors that can happen inside a Tauri application.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Failed to serialize/deserialize.
    #[error("Runtime error: {0}")]
    Runtime(anyhow::Error),
    /// Failed to serialize/deserialize.
    #[error("JSON error: {0}")]
    Json(serde_json::Error),
    /// Failed to serialize/deserialize.
    #[error("unknown api: {0:?}")]
    UnknownApi(Option<serde_json::Error>),
    /// IO error.
    #[error("{0}")]
    Io(#[from] std::io::Error),
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
    /// Program not allowed by the scope.
    #[error("program not allowed on the configured shell scope: {0}")]
    ProgramNotAllowed(PathBuf),
}

impl Error {
    #[allow(dead_code)]
    pub(crate) fn into_anyhow(self) -> anyhow::Error {
        anyhow::anyhow!(self.to_string())
    }

    fn msg(to_string: &str) -> Error {
        return Error::Runtime(anyhow::Error::msg(to_string.to_string()));
    }
}

impl Into<InvokeError> for Error {
    fn into(self) -> InvokeError {
        return InvokeError::from_anyhow(self.into_anyhow());
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

#[derive(Default)]
struct Profiles(Mutex<Vec<Profile>>);

#[cfg(target_os = "linux")]
const PATH_SEPARATOR: &str = ":";

#[cfg(target_os = "macos")]
const PATH_SEPARATOR: &str = ":";

#[cfg(target_os = "windows")]
const PATH_SEPARATOR: &str = ";";

#[derive(Deserialize, Serialize)]
pub struct GameMetadata {
    version: String,
}

#[derive(Deserialize, Serialize)]
pub struct SDK {
    version: String,
    r#type: String,
}

#[derive(Deserialize, Serialize)]
pub struct GameConfig {
    classpath: Vec<String>,
    sdk: SDK,
    main_class: String,
    game: String,
}

#[derive(Deserialize, Serialize)]
pub struct Profile {
    game: String,
    name: String,
    version: String,
}
impl Profile {
    fn clone(&self) -> Profile {
        Profile {
            game: self.game.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
        }
    }
}

#[tauri::command]
fn close() -> () {
    exit(0);
}

#[tauri::command]
fn launch(profile: Profile) -> () {
    let game: String = profile.game;
    let version: String = profile.version;

    let version_dir = "games/".to_string() + "/" + &game + "/versions/" + &version + "/";

    fn read_cfg(dir: &String) -> Result<GameConfig, io::Error> {
        let file = File::open(dir.to_string() + "config.json")?;
        let cfg = from_reader::<&File, GameConfig>(&file)?;
        drop(file);
        return Ok(cfg);
    }

    fn read_meta(dir: &String) -> Result<GameMetadata, io::Error> {
        let file = File::open(dir.to_string() + "metadata.json")?;
        let meta = from_reader::<&File, GameMetadata>(&file)?;
        drop(file);
        return Ok(meta);
    }

    let cfg = match read_cfg(&version_dir) {
        Ok(v) => v,
        Err(err) => return show_error(&err.to_string()),
    };
    let meta = match read_meta(&version_dir) {
        Ok(v) => v,
        Err(err) => return show_error(&err.to_string()),
    };

    let mut cp = vec![];
    for entry in cfg.classpath.iter() {
        cp.push(entry.to_owned())
    }

    cp.push(
        "games/".to_string()
            + &cfg.game
            + "/versions/"
            + &meta.version
            + "/"
            + &meta.version
            + ".jar",
    );

    let cp = cp.join(PATH_SEPARATOR) + PATH_SEPARATOR + &meta.version;
    process::Command::new("java").args(["-cp", &cp, &cfg.main_class]);

    return;
}

#[tauri::command(async)]
fn load_profiles(profile_state: State<'_, Profiles>) -> Result<Vec<Profile>, Error> {
    println!("Loading profiles.");
    let mutex_profiles = &mut profile_state.inner().0.lock()?;
    if !mutex_profiles.is_empty() {
        let mut profiles = vec![];
        for profile in mutex_profiles.iter() {
            profiles.push(profile.clone())
        }
        println!("Reusing old state.");
        return Ok(profiles);
    }

    let binding = get_data_dir().join("profiles.json");
    let path = binding.as_path();
    if !Path::exists(path) {
        println!("Profiles data doesn't exist, returning empty vec.");
        return Ok(vec![]);
    }

    let open = OpenOptions::new().read(true).open(&path.to_path_buf())?;

    let mut profiles: Vec<Profile> = serde_json::from_reader(open)?;
    mutex_profiles.append(&mut profiles);
    println!("Returning profile data.");
    return Ok(profiles);
}

#[tauri::command(async)]
fn import(profile_state: State<'_, Profiles>, name: String) -> Result<Profile, Error> {
    let path_buf = FileDialogBuilder::new().pick_file();
    if path_buf.is_none() {
        return Ok(Profile {
            game: "error".to_string(),
            name: "ERROR".to_string(),
            version: "error".to_string(),
        });
    };

    let path = &path_buf
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
        .as_mut()
        .to_owned();

    let file = File::open(path)?;
    let profile = list_zip_contents(&file, &name)?;
    drop(file);

    let mut profile_mutex = profile_state.inner().0.try_lock()?;
    profile_mutex.push(profile.clone());

    let path = &Path::new(&get_data_dir())
        .to_path_buf()
        .join("profiles.json");
    let mut options = &mut OpenOptions::new();
    if !Path::exists(path) {
        options = options.create_new(true);
    }

    let open = options.write(true).open(&path.to_path_buf())?;

    let mut profiles = vec![];
    let binding = profile_mutex;
    for profile in binding.iter() {
        profiles.push(profile)
    }
    serde_json::to_writer(open, &profiles)?;

    return Ok(profile);
}

fn show_error(x: &str) -> () {
    println!("{}", x);

    panic!("{}", x);
}

fn list_zip_contents(reader: &File, name: &String) -> Result<Profile, Error> {
    let data_dir = get_data_dir();
    let mut zip = zip::ZipArchive::new(reader)?;

    let metadata = read_metadata(&mut zip)?;
    let config = read_config(&mut zip)?;

    let version: &str = &metadata.version;
    println!("Version: {}", version);

    let game_name = config.game.as_str();

    extract_single_file(
        &mut zip,
        &data_dir
            .join("games/".to_string() + game_name + "/versions/" + version)
            .to_str()
            .ok_or_else(|| {
                Error::msg(&("Failed to extract '".to_string() + &metadata.version + ".jar)"))
            })?,
        &(metadata.version.to_string() + ".jar"),
    )?;

    extract_single_file(
        &mut zip,
        &data_dir
            .join("games/".to_string() + game_name + "/versions/" + version)
            .to_str()
            .unwrap(),
        "config.json",
    )?;

    extract_single_file(
        &mut zip,
        &data_dir
            .join("games/".to_string() + game_name + "/versions/" + version)
            .to_str()
            .unwrap(),
        "metadata.json",
    )?;

    extract_zip(&mut zip, &data_dir, config.classpath).unwrap_or_else(|error| {
        return show_error(error.to_string().as_str());
    });

    let profile = Profile {
        game: game_name.to_owned(),
        name: (name).to_string(),
        version: version.to_owned(),
    };

    return Ok(profile);
}

fn get_data_dir() -> PathBuf {
    let data_dir = match std::env::consts::OS {
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
            // Default case for unsupported platforms
            None
        }
    }
    .map(|it| return it.join("UltreonGameLauncher"))
    .unwrap();
    data_dir
}

// Function to extract a specific file from a zip archive to a specified folder
fn extract_single_file(
    archive: &mut ZipArchive<&File>,
    extract_to: &str,
    file_to_extract: &str,
) -> Result<(), io::Error> {
    // Get the file at the specified index
    let mut file = archive.by_name(file_to_extract)?;

    // Create the destination path
    let dest_path = Path::new(extract_to).join(file_to_extract);

    // Create directories if needed
    if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Create the file
    let mut dest_file = File::create(&dest_path)?;

    // Copy the contents of the file from the zip archive to the destination file
    io::copy(&mut file, &mut dest_file)?;

    Ok(())
}

/**
 * Function to extract specific files from a zip archive to a specified folder
 */
fn extract_zip(
    archive: &mut ZipArchive<&File>,
    extract_to: &PathBuf,
    files_to_extract: Vec<String>,
) -> Result<(), io::Error> {
    // Iterate over each file in the zip archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        // Get the file's name
        let file_name = &file.name().to_string();

        // Check if the file should be extracted
        if files_to_extract.contains(&file_name) {
            let dest_path = Path::new(extract_to).join(file_name);

            // Create directories if needed
            if let Some(parent) = dest_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }

            // Create the file
            let mut dest_file = File::create(&dest_path)?;

            // Copy the contents of the file from the zip archive to the destination file
            io::copy(&mut file, &mut dest_file)?;
            let dest_name = dest_path.to_string_lossy();
            println!("Extracting '{file_name}' -> '{dest_name}'");
        } else {
            println!("Skipping '{file_name}'");
        }
    }

    Ok(())
}

fn read_metadata(zip: &mut zip::ZipArchive<&File>) -> Result<GameMetadata, Error> {
    let zip_file = zip.by_name("metadata.json")?;
    let value = serde_json::from_reader(zip_file)?;
    Ok(value)
}

fn read_config(zip: &mut zip::ZipArchive<&File>) -> Result<GameConfig, Error> {
    let zip_file = zip.by_name("config.json")?;
    let value = serde_json::from_reader(zip_file)?;
    Ok(value)
}

fn main() {
    let run = tauri::Builder::default()
        .manage(Profiles(Default::default()))
        .invoke_handler(tauri::generate_handler![
            close,
            launch,
            import,
            load_profiles
        ])
        .run(tauri::generate_context!());
    if run.is_err() {
        let _ = show_error(&run.unwrap_err().to_string());
        panic!("Error Occurred");
    }
}
