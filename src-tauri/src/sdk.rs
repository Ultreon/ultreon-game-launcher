use std::collections::HashMap;
use std::env::consts::{ARCH, OS};
use std::fs::File;
use std::path::Path;

use flate2::read::GzDecoder;
use reqwest::Client;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use tar::Archive;
use tauri::AppHandle;
use zip::ZipArchive;

use crate::archive;
use crate::game::{GameConfig, GameMetadata};
use crate::util::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct SDK {
    /// Deprecated use 'versions' instead.
    #[deprecated = "Use version range using 'versions'"]
    pub(crate) version: String,
    #[serde(default)]
    pub(crate) versions: VersionReq,
    pub(crate) r#type: String,
}

#[derive(Debug, Deserialize, Hash, PartialEq)]
pub enum SDKOperatingSystem {
    #[serde(alias = "win")]
    Windows,
    #[serde(alias = "lin")]
    Linux,
    #[serde(alias = "mac")]
    MacOS,
}

impl Default for SDKOperatingSystem {
    fn default() -> Self {
        match OS {
            "windows" => Self::Windows,
            "linux" => Self::Linux,
            "macos" => Self::MacOS,
            _ => panic!("Unsupported platform!"),
        }
    }
}

impl Eq for SDKOperatingSystem {}

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
                    Self::WinX86
                }
                "x86_64" => {
                    Self::WinX64
                }
                _ => {
                    panic!("Unsupported platform!");
                }
            },
            "linux" => match ARCH {
                "x86_64" => {
                    Self::LinX64
                }
                "arm" => {
                    Self::LinArm
                }
                _ => {
                    panic!("Unsupported platform!");
                }
            },
            "macos" => match ARCH {
                "x86_64" => {
                    Self::MacX64
                }
                "arm" => {
                    Self::MacArm
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
pub struct SDKExecutablePaths(HashMap<SDKOperatingSystem, String>);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct SDKInfo {
    download: SDKDownloadInfo,
    pub(crate) version: Version,
    date: String,
    executable_path: String,
    #[serde(default)]
    executable_paths: HashMap<SDKOperatingSystem, String>,
    #[serde(default)]
    pub(crate) inner_path: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SDKList(pub(crate) HashMap<String, HashMap<String, SDKInfo>>);

pub async fn retrieve_sdk(
    app_: AppHandle,
    client: Client,
    sdk_info: &SDKInfo,
    cfg: &GameConfig,
    _meta: &GameMetadata,
) -> Result<bool, String> {
    let app = app_;

    let platform = &Default::default();

    let url = sdk_info
        .download
        .0
        .get(platform)
        .ok_or_else(|| format!("Can't find SDK for platform {:?}", platform))?;
    let name = url.rsplit_once('/').map(|v| v.1).unwrap_or(url);

    let data_dir = &crate::util::get_data_dir();

    let output_dir = data_dir
        .join("sdks/".to_string() + &cfg.sdk.r#type + "/" + &sdk_info.version.to_string())
        .to_str()
        .unwrap()
        .to_string();

    if Path::new(&output_dir).exists() {
        return Ok(false);
    }

    let file_path = &data_dir.join(format!("temp/{}", name));

    std::fs::create_dir_all(data_dir.join("temp"))
        .map_err(|e| format!("Failed to create output directory: {:?}", e))?;

    crate::net::download_file(
        app.to_owned(),
        client,
        url.to_string(),
        file_path.to_owned(),
    )
        .await
        .map_err(|e| format!("Failed to download SDK: {:?}", e))?;

    let file = File::open(file_path).map_err(|e| format!("Failed to open SDK package: {:?}", e))?;
    if file_path.file_name().unwrap().to_string_lossy().ends_with(".tar.gz") {
        let decompressed = GzDecoder::new(file);
        let mut archive = Archive::new(decompressed);
        archive::extract_tar_gz(app, name, &output_dir, &mut archive)?;
    } else if file_path.file_name().unwrap().to_string_lossy().ends_with(".zip") {
        let archive = &mut ZipArchive::new(file).map_err(|e| format!("Failed to open SDK package: {:?}", e))?;
        archive::extract_zip(app, name, &output_dir, archive)?;
    }

    Ok(true)
}

pub async fn fetch_sdk(client: Client) -> Result<SDKList, Error> {
    let value: SDKList = serde_json::from_slice(
        &client
            .get("https://ultreon.github.io/metadata/sdks.json")
            .send()
            .await?
            .bytes()
            .await?,
    )?;
    Ok(value)
}
