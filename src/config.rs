// Config: %APPDATA%\Murf\murf.toml
//
// Loaded on startup. Written if missing. Environment variables in
// ${VAR} or %VAR% form are expanded on load.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: General,
    pub palette: PaletteCfg,
    #[serde(default)]
    pub apps: BTreeMap<String, AppCfg>,
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
    pub ansi_mapping: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppCfg {
    pub enabled: bool,
    #[serde(default)]
    pub config_path: Option<String>,
    #[serde(default)]
    pub config_paths: Vec<String>,
    #[serde(default)]
    pub config_dir: Option<String>,
    #[serde(default)]
    pub reload: String,
    #[serde(default)]
    pub reload_cmd: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: General {
                version: 1,
                settle_delay_secs: 6,
                log_level: "info".into(),
            },
            palette: PaletteCfg {
                source: "system".into(),
                mode: "smart".into(),
                scheme_type: "scheme-tonal-spot".into(),
                contrast: 0.0,
                ansi_mapping: "semantic".into(),
            },
            apps: BTreeMap::new(),
        }
    }
}

pub fn config_dir() -> PathBuf {
    let appdata = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    appdata.join("Murf")
}

pub fn config_path() -> PathBuf {
    config_dir().join("murf.toml")
}

pub fn load_or_default() -> Result<Config> {
    let path = config_path();
    if path.exists() {
        load_from(&path)
    } else {
        let cfg = Config::default();
        if let Err(e) = write_to(&path, &cfg) {
            tracing::warn!(error = %e, "could not write default config");
        } else {
            tracing::info!(path = %path.display(), "default config written");
        }
        Ok(cfg)
    }
}

fn load_from(path: &Path) -> Result<Config> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let expanded = expand_env(&raw);

    let cfg: Config = toml::from_str(&expanded)
        .with_context(|| format!("parsing {}", path.display()))?;

    tracing::info!(
        path = %path.display(),
        mode = %cfg.palette.mode,
        scheme = %cfg.palette.scheme_type,
        ansi = %cfg.palette.ansi_mapping,
        "config loaded"
    );
    Ok(cfg)
}

fn write_to(path: &Path, cfg: &Config) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = toml::to_string_pretty(cfg)?;
    let header = "\
# Murf configuration.
# Delete this file to reset Murf to defaults.
# Environment variables: ${VAR} or %VAR%.
\n";
    fs::write(path, format!("{}{}", header, text))?;
    Ok(())
}

/// Expand ${VAR} and %VAR% to their environment values.
/// Unknown variables are left as-is.
fn expand_env(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < bytes.len() {
        // %VAR% form
        if bytes[i] == '%' {
            if let Some(end) = bytes[i + 1..].iter().position(|&c| c == '%') {
                let name: String = bytes[i + 1..i + 1 + end].iter().collect();
                if let Ok(val) = std::env::var(&name) {
                    out.push_str(&val);
                    i += end + 2;
                    continue;
                }
            }
        }

        // ${VAR} form
        if bytes[i] == '$' && i + 1 < bytes.len() && bytes[i + 1] == '{' {
            if let Some(end) = bytes[i + 2..].iter().position(|&c| c == '}') {
                let name: String = bytes[i + 2..i + 2 + end].iter().collect();
                if let Ok(val) = std::env::var(&name) {
                    out.push_str(&val);
                    i += end + 3;
                    continue;
                }
            }
        }

        out.push(bytes[i]);
        i += 1;
    }
    out
}
