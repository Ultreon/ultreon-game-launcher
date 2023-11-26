// Prevents additional console window on Wipub(crate)pub(crate)ndows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate msgbox;

use std::fs::{File, OpenOptions};
use std::path::Path;
use std::process::exit;

use serde_json::from_reader;
use tauri::{AppHandle, generate_handler, State, Window};
use tauri::api::dialog::blocking::FileDialogBuilder;

use profiles::Profiles;

use crate::profiles::Profile;
use crate::sdk::{SDKInfo, SDKList};
use crate::util::Error;

mod util;
mod launch;
mod archive;
mod sdk;
mod commands;
mod profiles;
mod net;
mod game;

#[macro_export]
macro_rules! show_error {
    ($($arg:tt)*) => {{
        println!($($arg)*);

        panic!($($arg)*);
    }};
}

#[tauri::command]
fn close() {
    exit(0);
}

#[tauri::command]
async fn launch(
    app: AppHandle,
    window: Window,
    profile: Profile,
) -> Result<i32, Error> {
    let client = crate::net::build_client()?;

    return Err(Error::Launch("Hello there!".to_string()));

    let sdk_list: SDKList = sdk::fetch_sdk(client.to_owned())
        .await
        .map_err(|e| Error::Fetch(format!("Failed to fetch SDK: {:?}", e)))?;

    let version_dir =
        "games/".to_string() + "/" + &profile.game + "/versions/" + &profile.version + "/";
    let cfg = crate::profiles::read_cfg(&version_dir)
        .map_err(|e| Error::Launch(format!("Failed to read version config: {:?}", e)))?;
    let _meta = crate::profiles::read_meta(&version_dir)
        .map_err(|e| Error::Launch(format!("Failed to read version metadata, {:?}", e)))?;
    let sdk_info_map = sdk_list
        .0
        .get(&cfg.sdk.r#type)
        .ok_or_else(|| Error::Launch(format!("Unknown SDK type: {}", &cfg.sdk.r#type)))?;

    let mut sdk_info: Option<&SDKInfo> = None;
    let vv = util::get_version_req(&cfg)?;
    let versions = &vv;

    println!("Version range: {}", versions);
    for ele in sdk_info_map {
        let v = &ele.1.version;
        let is_newer_supported = sdk_info.is_some()
            && versions.matches(v)
            && sdk_info.unwrap().version < *v;
        let is_supported = sdk_info.is_none() && versions.matches(v);
        if (is_supported) || (is_newer_supported) {
            sdk_info = Some(ele.1);
        }
    }
    let sdk_info = sdk_info.ok_or_else(|| {
        Error::Launch(format!("No compatible versions found: {}", &cfg.sdk.r#type))
    })?;
    sdk::retrieve_sdk(app, client, sdk_info, &cfg, &_meta)
        .await
        .map_err(Error::Launch)?;

    let game: String = profile.game;
    let version: String = profile.version;

    let version_dir = "games/".to_string() + "/" + &game + "/versions/" + &version + "/";

    let cfg = crate::profiles::read_cfg(&version_dir)
        .map_err(|e| Error::Launch(format!("Failed to read version config: {:?}", e)))?;
    let meta = crate::profiles::read_meta(&version_dir)
        .map_err(|e| Error::Launch(format!("Failed to read version metadata, {:?}", e)))?;

    let binding = util::get_data_dir();
    let data_dir_raw = binding.to_str().unwrap();
    let data_dir = &data_dir_raw
        .strip_suffix('/')
        .unwrap_or(data_dir_raw)
        .to_string();

    let cp = util::get_classpath(&cfg, meta, data_dir);

    window.hide().expect("Failed to hide window.");

    let code = match launch::run_with_sdk(&window, sdk_info, &cfg, data_dir, cp) {
        Ok(value) => value,
        Err(value) => return value,
    };

    window.show().expect("Failed to show window again.");
    Err(Error::Launch(format!("Game crashed, exit code: {}", code)))
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

    let binding = util::get_data_dir().join("profiles.json");
    let path = binding.as_path();
    if !Path::exists(path) {
        println!("Profiles data doesn't exist, returning empty vec.");
        return Ok(vec![]);
    }

    let open = OpenOptions::new().read(true).open(path)?;

    let mut profiles: Vec<Profile> = from_reader(open)?;
    mutex_profiles.append(&mut profiles);
    println!("Returning profile data.");
    Ok(profiles)
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
    let profile = crate::profiles::list_zip_contents(&file, &name)?;
    drop(file);

    let mut profile_mutex = profile_state.inner().0.try_lock()?;
    profile_mutex.push(profile.clone());

    let path = &Path::new(&util::get_data_dir())
        .to_path_buf()
        .join("profiles.json");
    let mut options = &mut OpenOptions::new();
    if !Path::exists(path) {
        options = options.create_new(true);
    }

    let open = options.write(true).open(path)?;

    let mut profiles = vec![];
    let binding = profile_mutex;
    for profile in binding.iter() {
        profiles.push(profile)
    }
    serde_json::to_writer(open, &profiles)?;

    Ok(profile)
}

fn main() {
    let run = tauri::Builder::default()
        .manage(Profiles(Default::default()))
        .invoke_handler(generate_handler![
            close,
            launch,
            import,
            load_profiles
        ])
        .run(tauri::generate_context!());
    if run.is_err() {
        util::show_error(&run.expect_err("").to_string());
        panic!("Error Occurred");
    }
}
