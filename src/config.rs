use std::path::PathBuf;

use hex_color::HexColor;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    #[serde(default = "default_file")]
    pub file: PathBuf,
    #[serde(default = "default_color_priority")]
    pub color_priority: HexColor,
    #[serde(default = "default_color_project")]
    pub color_project: HexColor,
    #[serde(default = "default_color_context")]
    pub color_context: HexColor,
}

impl Config {
    pub fn new() -> Self {
        match envy::prefixed("ROFI_TODO_").from_env() {
            Ok(config) => config,
            Err(err) => {
                log::error!("{err:?}");

                Config::default()
            }
        }
    }
}

fn default_file() -> PathBuf {
    let data = std::env::var("XDG_DATA_HOME").map(|data| format!("{data}/rofi-todo.txt"));
    let home = std::env::var("HOME").map(|home| format!("{home}/.local/share/rofi-todo.txt"));

    data.unwrap_or(home.unwrap_or_else(|_| {
        log::error!("Unable to find location for todo.txt");

        "/tmp/rofi-todo.txt".into()
    }))
    .into()
}

fn default_color_priority() -> HexColor {
    HexColor::RED
}

fn default_color_project() -> HexColor {
    HexColor::rgb(0, 128, 0)
}

fn default_color_context() -> HexColor {
    HexColor::rgb(255, 165, 0)
}
