// Config: %APPDATA%\Murf\murf.toml

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: General,
    pub palette: PaletteCfg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct General {
    pub version: u32,
    pub settle_delay_secs: u64,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteCfg {
    pub source: String,
    pub mode: String,
    pub scheme_type: String,
    pub contrast: f32,
}

pub fn config_dir() -> PathBuf {
    let appdata = std::env::var("APPDATA").expect("APPDATA not set");
    PathBuf::from(appdata).join("Murf")
}

pub fn config_path() -> PathBuf {
    config_dir().join("murf.toml")
}

pub fn load_or_default() -> Result<Config> {
    // TODO: read config_path(); if missing, write default and return it.
    Ok(Config {
        general: General {
            version: 1,
            settle_delay_secs: 6,
            log_level: "info".into(),
        },
        palette: PaletteCfg {
            source: "system".into(),
            mode: "dark".into(),
            scheme_type: "scheme-tonal-spot".into(),
            contrast: 0.0,
        },
    })
}
