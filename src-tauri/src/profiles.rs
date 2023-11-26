use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use zip::ZipArchive;

use crate::game::{GameConfig, GameMetadata};
use crate::show_error;
use crate::util::Error;

#[derive(Default)]
pub struct Profiles(pub Mutex<Vec<Profile>>);

#[derive(Deserialize, Serialize, Clone)]
pub struct Profile {
    pub game: String,
    pub name: String,
    pub version: String,
}

impl Profile {
    #[allow(dead_code)]
    fn clone(&self) -> Profile {
        Profile {
            game: self.game.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
        }
    }
}

pub fn read_cfg(dir: &String) -> Result<GameConfig, io::Error> {
    let file = File::open(crate::util::get_data_dir().join(dir.to_string() + "config.json"))?;
    let cfg = from_reader::<&File, GameConfig>(&file)?;
    drop(file);
    Ok(cfg)
}

pub fn read_meta(dir: &String) -> Result<GameMetadata, io::Error> {
    let file = File::open(crate::util::get_data_dir().join(dir.to_string() + "metadata.json"))?;
    let meta = from_reader::<&File, GameMetadata>(&file)?;
    drop(file);
    Ok(meta)
}

pub fn list_zip_contents(reader: &File, name: &String) -> Result<Profile, Error> {
    let data_dir = crate::util::get_data_dir();
    let mut zip = ZipArchive::new(reader)?;

    let metadata = read_metadata(&mut zip)?;
    let config = read_config(&mut zip)?;

    let version: &str = &metadata.version;
    println!("Version: {}", version);

    let game_name = config.game.as_str();

    extract_single_file(
        &mut zip,
        data_dir
            .join("games/".to_string() + game_name + "/versions/" + version)
            .to_str()
            .ok_or_else(|| {
                Error::msg(&("Failed to extract '".to_string() + &metadata.version + ".jar)"))
            })?,
        &(metadata.version.to_string() + ".jar"),
    )?;

    extract_single_file(
        &mut zip,
        data_dir
            .join("games/".to_string() + game_name + "/versions/" + version)
            .to_str()
            .unwrap(),
        "config.json",
    )?;

    extract_single_file(
        &mut zip,
        data_dir
            .join("games/".to_string() + game_name + "/versions/" + version)
            .to_str()
            .unwrap(),
        "metadata.json",
    )?;

    extract_package_zip(&mut zip, &data_dir, config.classpath).unwrap_or_else(|error| {
        show_error!("{}", error.to_string().as_str());
    });

    let profile = Profile {
        game: game_name.to_owned(),
        name: (name).to_string(),
        version: version.to_owned(),
    };

    Ok(profile)
}

/// Function to extract a specific file from a zip archive to a specified folder
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
fn extract_package_zip(
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
        if files_to_extract.contains(file_name) {
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

fn read_metadata(zip: &mut ZipArchive<&File>) -> Result<GameMetadata, Error> {
    let zip_file = zip.by_name("metadata.json")?;
    let value = from_reader(zip_file)?;
    Ok(value)
}

fn read_config(zip: &mut ZipArchive<&File>) -> Result<GameConfig, Error> {
    let zip_file = zip.by_name("config.json")?;
    let value = from_reader(zip_file)?;
    Ok(value)
}
