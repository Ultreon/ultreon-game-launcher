use std::fs::File;
use std::io;
use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;
use tauri::{AppHandle, Manager};
use zip::ZipArchive;

use crate::net::DownloadInfo;

pub fn extract_tar_gz(app: AppHandle, name: &str, output_dir: &String, archive: &mut Archive<GzDecoder<File>>) -> Result<(), String> {
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

    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create output directory: {:?}", e))?;

    let entries = &mut archive
        .entries()
        .map_err(|e| format!("Failed to list SDK package entries: {:?}", e))?;
    let mut buf: Vec<u8> = vec![];

    for (extracted, entry) in entries.enumerate() {
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
                downloaded: extracted as u64,
                total: extracted as u64,
                downloading: true,
                status: format!("Extracting: {:?}", path),
                percent: (100),
            },
        )
            .map_err(|e| format!("Failed to emit extract event: {:?}", e))?;

        let target_path = format!("{}/{}", out_dir, path.to_string_lossy());

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(format!("{}/{}", out_dir, parent.to_string_lossy()))
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
    }

    // Emit event
    app.emit_all(
        "downloadProgress",
        DownloadInfo {
            downloaded: 1,
            total: 1,
            downloading: false,
            status: "Completed!".to_string(),
            percent: 100,
        },
    )
        .map_err(|e| format!("Failed to emit extract event: {:?}", e))?;
    Ok(())
}

pub fn extract_zip(app: AppHandle, name: &str, output_dir: &String, archive: &mut ZipArchive<File>) -> Result<(), String> {
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

    // Iterate over each file in the zip archive

    let len = archive.len();

    for (extracted, i) in (0..len).enumerate() {
        let mut file = archive.by_index(i).map_err(|e| format!("Failed to get zip file: {:?}", e))?;

        // Emit event
        app.emit_all(
            "downloadProgress",
            DownloadInfo {
                downloaded: extracted as u64,
                total: len as u64,
                downloading: true,
                status: format!("Extracting: {:?}", file.name()),
                percent: (100),
            },
        )
            .map_err(|e| format!("Failed to emit extract event: {:?}", e))?;

        // Get the file's name
        let file_name = &file.name().to_string();

        let dest_path = Path::new(output_dir).join(file_name);

        // Create directories if needed
        if let Some(parent) = dest_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create entry destination directory: {:?}", e))?;
            }
        }

        if file.is_dir() {
            std::fs::create_dir_all(&dest_path).map_err(|e| format!("Failed to create entry destination directory: {:?}", e))?;
            println!("Extracting '{file_name}' -> '{:?}'", &dest_path.to_string_lossy());
        } else {
            // Create the file
            let mut dest_file = File::create(&dest_path).map_err(|e| format!("Failed to create destination file: {:?}", e))?;

            // Copy the contents of the file from the zip archive to the destination file
            io::copy(&mut file, &mut dest_file).map_err(|e| format!("Failed to extract zip entry: {:?}", e))?;
            let dest_name = dest_path.to_string_lossy();
            println!("Extracting '{file_name}' -> '{dest_name}'");
        }
    }

    // Emit event
    app.emit_all(
        "downloadProgress",
        DownloadInfo {
            downloaded: 1,
            total: 1,
            downloading: false,
            status: "Completed!".to_string(),
            percent: 100,
        },
    )
        .map_err(|e| format!("Failed to emit extract event: {:?}", e))?;
    Ok(())
}
