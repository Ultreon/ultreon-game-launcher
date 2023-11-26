use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use futures_util::StreamExt;
use crate::util::Error;

#[derive(Deserialize, Serialize, Clone)]
pub struct DownloadInfo {
    pub(crate) downloaded: u64,
    pub(crate) total: u64,
    pub(crate) percent: u32,
    pub(crate) downloading: bool,
    pub(crate) status: String,
}

pub fn build_client() -> Result<Client, Error> {
    let client = Client::builder()
        .build()
        .map_err(|e| Error::Launch(format!("Failed to create url client: {:?}", e)))?;
    Ok(client)
}

pub async fn download_file<R: Runtime>(
    app: AppHandle<R>,
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
        .ok_or_else(|| Error::Download("Failed to get file path".to_string()))?
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

    Ok(())
}
