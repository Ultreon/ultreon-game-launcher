// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate msgbox;

use msgbox::IconType;
use msgbox::MsgBoxError;
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use tauri::api::dialog::blocking::FileDialogBuilder;
use tauri::InvokeError;
use uuid::Uuid;
use zip::ZipArchive; // Note the updated import

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

#[tauri::command]
fn close() -> () {
    exit(0);
}

#[tauri::command]
fn launch(profile: Profile) -> () {
    let game: String = profile.game;
    let name: String = profile.name;
    let version: String = profile.version;

    let version_dir = "games/".to_string() + "/" + &game + "/versions/" + &version + "/";

    fn read_cfg(dir: &String) -> Result<GameConfig, io::Error> {
        let file = File::open(dir.to_string() + "config.json")?;
        let cfg = from_reader::<&File, GameConfig>(&file)?;
        drop(file);
        return Ok(cfg);
    }

    fn read_meta(dir: &String) -> Result<GameConfig, io::Error> {
        let file = File::open(dir.to_string() + "metadata.json")?;
        let cfg = from_reader::<&File, GameConfig>(&file)?;
        drop(file);
        return Ok(cfg);
    }

    let cfg = match read_cfg(&version_dir) {
        Ok(v) => v,
        Err(err) => return show_error(&err.to_string()).expect("Failed to show error message"),
    };
    let meta = match read_meta(&version_dir) {
        Ok(v) => v,
        Err(err) => return show_error(&err.to_string()).expect("Failed to show error message"),
    };

    cfg.classpath.join(PATH_SEPARATOR);
    return;
}

#[tauri::command]
fn load_profiles(mut profiles: Vec<Profile>) -> () {
    let profile = Profile {
        game: "ultracraft".to_string(),
        name: "Hello world".to_string(),
        version: "0.1.0".to_string(),
    };
    profiles.push(profile);
}

#[tauri::command(async)]
fn import(name: String) -> Result<Profile, InvokeError> {
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

    let open = &OpenOptions::new().read(true).open(path);
    if open.is_err() {
        let _ = show_error(&(open.as_ref()).unwrap_err().to_string());
        show_error(&(open.as_ref()).unwrap_err().to_string())
            .expect("Failed to open error message.");
        return Err(InvokeError::from(
            "IO Error: ".to_string() + &(&(open.as_ref()).unwrap_err().to_string()),
        ));
    }
    let file = match File::open(path) {
        Ok(val) => val,
        Err(err) => return Err(InvokeError::from(err.to_string())),
    };
    let profile = match list_zip_contents(&file, &name) {
        Ok(v) => v,
        Err(err) => return Err(err.into()),
    };
    drop(file);
    return Ok(profile);
}

fn show_error(x: &str) -> Result<(), MsgBoxError> {
    return msgbox::create("An error occurred!", &x.to_string(), IconType::Error);
}

fn list_zip_contents(reader: &File, name: &String) -> Result<Profile, String> {
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
    let mut zip = match zip::ZipArchive::new(reader) {
        Ok(it) => it,
        Err(err) => return Err(err.to_string()),
    };

    let metadata = match read_metadata(&mut zip) {
        Ok(value) => value,
        Err(value) => return Err(value),
    };
    let config = match read_config(&mut zip) {
        Ok(value) => value,
        Err(value) => return Err(value),
    };

    let version: &str = &metadata.version;
    println!("Version: {}", version);

    let game_name = config.game.as_str();

    match extract_single_file(
        &mut zip,
        &data_dir
            .join("games/".to_string() + game_name + "/versions/" + version)
            .to_str()
            .unwrap(),
        &(metadata.version.to_string() + ".jar"),
    ) {
        Ok(it) => it,
        Err(err) => return Err(err.to_string()),
    };

    match extract_single_file(
        &mut zip,
        &data_dir
            .join("games/".to_string() + game_name + "/versions/" + version)
            .to_str()
            .unwrap(),
        "config.json",
    ) {
        Ok(it) => it,
        Err(err) => return Err(err.to_string()),
    };

    match extract_single_file(
        &mut zip,
        &data_dir
            .join("games/".to_string() + game_name + "/versions/" + version)
            .to_str()
            .unwrap(),
        "metadata.json",
    ) {
        Ok(it) => it,
        Err(err) => return Err(err.to_string()),
    };

    extract_zip(&mut zip, &data_dir, config.classpath).unwrap_or_else(|error| {
        return show_error(error.to_string().as_str()).expect("Failed to show error message");
    });

    return Ok(Profile {
        game: game_name.to_owned(),
        name: (name).to_string(),
        version: version.to_owned(),
    });
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

fn read_metadata(zip: &mut zip::ZipArchive<&File>) -> Result<GameMetadata, String> {
    let zip_file = match zip.by_name("metadata.json") {
        Ok(it) => it,
        Err(err) => return Err(err.to_string()),
    };
    let value = match serde_json::from_reader(zip_file) {
        Ok(value) => value,
        Err(value) => return Err(value.to_string()),
    };
    Ok(value)
}

fn read_config(zip: &mut zip::ZipArchive<&File>) -> Result<GameConfig, String> {
    let zip_file = match zip.by_name("config.json") {
        Ok(it) => it,
        Err(err) => return Err(err.to_string()),
    };
    let value = match serde_json::from_reader(zip_file) {
        Ok(value) => value,
        Err(value) => return Err(value.to_string()),
    };
    Ok(value)
}

fn main() {
    let run = tauri::Builder::default()
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
