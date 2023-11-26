#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{exit, ExitStatus, Stdio};
use std::process;

use tauri::Window;

use crate::game::GameConfig;
use crate::sdk::SDKInfo;
use crate::util::{Error, PATH_SEPARATOR};

#[cfg(target_os = "windows")]
const DETACHED_PROCESS: u32 = 0x00000008;

pub fn run_with_sdk(window: &Window, sdk_info: &SDKInfo, cfg: &GameConfig, data_dir: &String, cp: Vec<String>) -> Result<i32, Result<i32, Error>> {
    let sdk_path = prepare_run(sdk_info, cfg, data_dir);

    println!("Running SDK: {}", sdk_path.to_string_lossy());

    let cp = cp.join(PATH_SEPARATOR);
    let code = match run_game(data_dir, &cp, cfg, sdk_path) {
        Ok(v) => v.code().unwrap_or(0),
        Err(e) => {
            window.show().expect("Failed to show window again.");
            return Err(Err(Error::Launch(format!("Game Crashed: {:?}", e))));
        }
    };

    if code == 0 {
        exit(0);
    }
    Ok(code)
}

fn prepare_run(sdk_info: &SDKInfo, cfg: &GameConfig, data_dir: &String) -> PathBuf {
    let mut sdk_path =
        PathBuf::from(data_dir).join(format!("sdks/{}/{}/", cfg.sdk.r#type, sdk_info.version));
    if sdk_info.inner_path.is_some() {
        let inner_path = sdk_info.inner_path.as_ref().unwrap();
        sdk_path = sdk_path.join(inner_path);
    }

    sdk_path = sdk_path.join("bin/java");
    sdk_path
}

fn run_game(
    data_dir: &String,
    cp: &String,
    cfg: &GameConfig,
    sdk_path: PathBuf,
) -> Result<ExitStatus, Error> {
    #[allow(unused_mut)]
    #[cfg(target_os = "linux")]
    let command = process::Command::new(sdk_path)
        .args(&["-cp", &cp, &cfg.main_class])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .current_dir((&data_dir).to_string() + "/games/" + &cfg.game);

    #[allow(unused_mut)]
    #[cfg(target_os = "macos")]
    let command = process::Command::new(sdk_path)
        .args(&["-cp", &cp, &cfg.main_class])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .current_dir((&data_dir).to_string() + "/games/" + &cfg.game);

    #[allow(unused_mut)]
    #[cfg(target_os = "windows")]
    let command = process::Command::new(sdk_path)
        .args(&["-cp", &cp, &cfg.main_class])
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .current_dir((&data_dir).to_string() + "/games/" + &cfg.game)
        .creation_flags(DETACHED_PROCESS); // Be careful: This only works on windows

    let status = &mut command.spawn()?.wait()?;

    Ok(*status)
}
