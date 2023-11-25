// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate msgbox;

use flate2::read::GzDecoder;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::collections::HashMap;
use std::env::consts::ARCH;
use std::env::consts::OS;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::process::Stdio;
use std::process::exit;
use std::sync::Mutex;
use tauri::api::dialog::blocking::FileDialogBuilder;
use tauri::InvokeError;
use tauri::Manager;
use tauri::Runtime;
use tauri::State;
use zip::ZipArchive; // Note the updated import

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

    fn msg(to_string: &str) -> Error {
        return Error::Generic(anyhow::Error::msg(to_string.to_string()));
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

#[derive(Debug, Deserialize, Serialize)]
pub struct GameMetadata {
    version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SDK {
    version: String,
    r#type: String,
}

#[derive(Debug, Deserialize, Hash, PartialEq)]
pub enum SDKPlatform {
    #[serde(alias = "win-x64")]
    WinX64,
    #[serde(alias = "win-x86")]
    WinX86,
    #[serde(alias = "lin-x64")]
    LinX64,
    #[serde(alias = "lin-arm")]
    LinArm,
    #[serde(alias = "mac-x64")]
    MacX64,
    #[serde(alias = "mac-arm")]
    MacArm,
}

impl Default for SDKPlatform {
    fn default() -> Self {
        match OS {
            "windows" => match ARCH {
                "x86" => {
                    return Self::WinX86;
                }
                "x86_64" => {
                    return Self::WinX64;
                }
                _ => {
                    panic!("Unsupported platform!");
                }
            },
            "linux" => match ARCH {
                "x86_64" => {
                    return Self::LinX64;
                }
                "arm" => {
                    return Self::LinArm;
                }
                _ => {
                    panic!("Unsupported platform!");
                }
            },
            "macos" => match ARCH {
                "x86_64" => {
                    return Self::MacX64;
                }
                "arm" => {
                    return Self::MacArm;
                }
                _ => {
                    panic!("Unsupported platform!");
                }
            },
            _ => {
                panic!("Unsupported platform!");
            }
        }
    }
}

impl Eq for SDKPlatform {}

#[derive(Deserialize)]
pub struct SDKDownloadInfo(HashMap<SDKPlatform, String>);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct SDKInfo {
    download: SDKDownloadInfo,
    version: String,
    date: String,
    executable_path: String,
    #[serde(default)]
    inner_path: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SDKList(HashMap<String, HashMap<String, SDKInfo>>);

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct GameConfig {
    classpath: Vec<String>,
    sdk: SDK,
    main_class: String,
    game: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DownloadInfo {
    downloaded: u64,
    total: u64,
    percent: u32,
    downloading: bool,
    status: String,
}

#[derive(Deserialize, Serialize, Clone)]
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

fn read_cfg(dir: &String) -> Result<GameConfig, io::Error> {
    let file = File::open(get_data_dir().join(dir.to_string() + "config.json"))?;
    let cfg = from_reader::<&File, GameConfig>(&file)?;
    drop(file);
    return Ok(cfg);
}

fn read_meta(dir: &String) -> Result<GameMetadata, io::Error> {
    let file = File::open(get_data_dir().join(dir.to_string() + "metadata.json"))?;
    let meta = from_reader::<&File, GameMetadata>(&file)?;
    drop(file);
    return Ok(meta);
}

#[tauri::command]
async fn launch(
    app: tauri::AppHandle,
    window: tauri::Window,
    profile: Profile,
) -> Result<i32, Error> {
    let client = Client::builder()
        .build()
        .map_err(|e| Error::Launch(format!("Failed to create url client: {:?}", e)))?;

    let sdk_list: SDKList = fetch_sdk(client.to_owned())
        .await
        .map_err(|e| Error::Fetch(format!("Failed to fetch SDK: {:?}", e)))?;

    
    let version_dir = "games/".to_string() + "/" + &profile.game + "/versions/" + &profile.version + "/";
    let cfg = read_cfg(&version_dir)
        .map_err(|e| Error::Launch(format!("Failed to read version config: {:?}", e)))?;
    let _meta = read_meta(&version_dir)
        .map_err(|e| Error::Launch(format!("Failed to read version metadata, {:?}", e)))?;
    let sdk_info = sdk_list
        .0.get(&cfg.sdk.r#type)
        .ok_or_else(|| Error::Launch(format!("Unknown SDK type: {}", &cfg.sdk.r#type)))?
        .get(&cfg.sdk.version)
        .ok_or_else(|| Error::Launch(format!("Unknown SDK type: {}", &cfg.sdk.r#type)))?;
    get_sdk(app, client, sdk_info, &cfg, &_meta)
        .await
        .map_err(|e| Error::Launch(e))?;

    let game: String = profile.game;
    let version: String = profile.version;

    let version_dir = "games/".to_string() + "/" + &game + "/versions/" + &version + "/";

    let cfg = read_cfg(&version_dir)
        .map_err(|e| Error::Launch(format!("Failed to read version config: {:?}", e)))?;
    let meta = read_meta(&version_dir)
        .map_err(|e| Error::Launch(format!("Failed to read version metadata, {:?}", e)))?;

    let binding = get_data_dir();
    let data_dir_unstripped = binding.to_str().unwrap();
    let data_dir = &data_dir_unstripped
        .strip_suffix("/")
        .unwrap_or_else(|| data_dir_unstripped)
        .to_string();

    let mut cp = vec![];
    for entry in cfg.classpath.iter() {
        cp.push(data_dir.to_string() + "/" + entry)
    }

    cp.push(
        data_dir.to_string()
            + &"/games/".to_string()
            + &cfg.game
            + "/versions/"
            + &meta.version
            + "/"
            + &meta.version
            + ".jar",
    );

    println!("{:?}", cp);

    window.hide().expect("Failed to hide window.");

    let mut sdk_path = PathBuf::from(data_dir).join(format!("sdks/{}/{}/", cfg.sdk.r#type, cfg.sdk.version));
    if sdk_info.inner_path.is_some() {
        let inner_path = sdk_info.inner_path.as_ref().unwrap();
        sdk_path = sdk_path.join(inner_path);
    }

    sdk_path = sdk_path.join("bin/java");

    println!("Running SDK: {}", sdk_path.to_string_lossy());

    let cp = cp.join(PATH_SEPARATOR);
    let status = process::Command::new(sdk_path)
        .args(["-cp", &cp, &cfg.main_class])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .current_dir((&data_dir).to_string() + &"/games/".to_string() + &cfg.game)
        .spawn()?
        .wait()?;

    let code = status
        .code()
        .unwrap_or(0);

    if status.success() {
        exit(0);
    }

    window.show().expect("Failed to show window again.");

    return Ok(code);
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

async fn get_sdk(
    app_: tauri::AppHandle,
    client: Client,
    sdk_info: &SDKInfo,
    cfg: &GameConfig,
    _meta: &GameMetadata
) -> Result<bool, String> {
    let app = app_;

    let platform = &Default::default();

    let url = sdk_info
        .download
        .0
        .get(platform)
        .ok_or_else(|| format!("Can't find SDK for platform {:?}", platform))?;
    let name = url.rsplit_once("/").map(|v| v.1).unwrap_or(url);

    let data_dir = &get_data_dir();

    let output_dir = data_dir
        .join("sdks/".to_string() + &cfg.sdk.r#type + &"/" + &cfg.sdk.version)
        .to_str()
        .unwrap()
        .to_string();

    if Path::new(&output_dir).exists() {
        return Ok(false);
    }

    let file_path = data_dir.join(format!("temp/{}", name));

    std::fs::create_dir_all(data_dir.join("temp"))
        .map_err(|e| format!("Failed to create output directory: {:?}", e))?;

    download_file(
        app.to_owned(),
        client,
        url.to_string(),
        file_path.to_owned(),
    )
    .await
    .map_err(|e| format!("Failed to download SDK: {:?}", e))?;

    // Replace "your_archive.tar.gz" with the actual path to your tar.gz file
    let file = File::open(file_path).map_err(|e| format!("Failed to open SDK package: {:?}", e))?;
    let decompressed = GzDecoder::new(file);
    let mut archive = tar::Archive::new(decompressed);

    // Emit event
    app.emit_all(
        "downloadProgress",
        DownloadInfo {
            downloaded: 0,
            total: 0,
            downloading: true,
            status: format!("Extracting: {}", name),
            percent: 100,
        },
    )
    .map_err(|e| format!("Failed to emit extract event: {:?}", e))?;

    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {:?}", e))?;

    let entries = &mut archive
        .entries()
        .map_err(|e| format!("Failed to list SDK package entries: {:?}", e))?;
    let mut extracted = 0;
    let mut buf: Vec<u8> = vec![];

    for entry in entries {
        let out_dir = output_dir.clone();
        let mut entry = entry.map_err(|e| format!("Failed to get entry: {:?}", e))?;
        let path = entry
            .path()
            .map_err(|e| format!("Failed to get entry: {:?}", e))?;

        println!("Extracting: {}/{}", name, path.to_string_lossy());

        // Emit event
        app.emit_all(
            "downloadProgress",
            DownloadInfo {
                downloaded: extracted,
                total: extracted,
                downloading: true,
                status: format!("Extracting: {:?}", path),
                percent: (100) as u32,
            },
        )
        .map_err(|e| format!("Failed to emit extract event: {:?}", e))?;

        let target_path = format!("{}/{}", out_dir.to_string(), path.to_string_lossy());

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(format!("{}/{:?}", out_dir, parent))
                .map_err(|e| format!("Failed to output directories: {:?}", e))?;
        }

        // If the entry is a directory, create it
        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&target_path)
                .map_err(|e| format!("Failed to create directory: {:?}", e))?;
        } else {
            entry
                .unpack_in(out_dir)
                .map_err(|e| format!("Failed to unpack file: {:?}", e))?;
            buf.clear();
        }
        extracted += 1;
    }

    // Emit event
    app.emit_all(
        "downloadProgress",
        DownloadInfo {
            downloaded: 1,
            total: 1,
            downloading: false,
            status: format!("Completed!"),
            percent: 100,
        },
    )
    .map_err(|e| format!("Failed to emit extract event: {:?}", e))?;

    return Ok(true);
}

async fn fetch_sdk(client: Client) -> Result<SDKList, Error> {
    let value: SDKList = serde_json::from_slice(
        &client
            .get("https://ultreon.github.io/metadata/sdks.json")
            .send()
            .await?
            .bytes()
            .await?,
    )?;
    return Ok(value);
}

async fn download_file<R: Runtime>(
    app: tauri::AppHandle<R>,
    client: Client,
    url: String,
    file_path: PathBuf,
) -> Result<(), Error> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| Error::Download(format!("Failed to make request: {:?}", e)))?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded_size = 0;

    let mut file = File::create(&file_path)?;
    let file_name = Path::new(&file_path)
        .file_name()
        .ok_or_else(|| Error::Download(format!("Failed to get file path")))?
        .to_str()
        .unwrap()
        .to_owned();

    let mut response = response.bytes_stream();
    while let Some(chunk) = response.next().await {
        let chunk = chunk.map_err(|e| Error::Download(format!("Failed to read chunk: {:?}", e)))?;
        downloaded_size += chunk.len() as u64;
        file.write_all(&chunk)
            .map_err(|e| Error::Download(format!("Failed to write to file: {:?}", e)))?;

        // Emit event
        app.emit_all(
            "downloadProgress",
            DownloadInfo {
                downloaded: downloaded_size,
                total: total_size,
                downloading: true,
                percent: (100 * downloaded_size / total_size) as u32,
                status: format!("Downloading: {}", file_name),
            },
        )
        .map_err(|e| Error::Download(format!("Failed to emit event: {:?}", e)))?;
    }

    return Ok(());
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
