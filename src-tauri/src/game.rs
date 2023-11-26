use serde::{Deserialize, Serialize};
use crate::sdk::SDK;

#[derive(Debug, Deserialize, Serialize)]
pub struct GameMetadata {
    pub(crate) version: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct GameConfig {
    pub(crate) classpath: Vec<String>,
    pub(crate) sdk: SDK,
    pub(crate) main_class: String,
    pub(crate) game: String,
}
