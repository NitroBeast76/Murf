// Chronoterm adapter.
//
// Locates config.toml in either %APPDATA%\chronoterm\ or
// %USERPROFILE%\.config\chronoterm\, preferring whichever exists.
// Replaces the [colors] table between # <murf> markers.

use super::{home_dir, inject_section, AppAdapter, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct Chronoterm;

const MARKER: &str = "# <murf>";

impl Chronoterm {
    fn candidate_paths() -> Vec<PathBuf> {
        let mut out = Vec::new();
        if let Some(appdata) = std::env::var_os("APPDATA") {
            out.push(
                PathBuf::from(appdata)
                    .join("chronoterm")
                    .join("config.toml"),
            );
        }
        if let Some(home) = home_dir() {
            out.push(
                home.join(".config")
                    .join("chronoterm")
                    .join("config.toml"),
            );
        }
        out
    }

    fn config_path() -> Option<PathBuf> {
        Self::candidate_paths().into_iter().find(|p| p.exists())
    }
}

impl AppAdapter for Chronoterm {
    fn id(&self) -> &'static str { "chronoterm" }
    fn display_name(&self) -> &'static str { "Chronoterm" }

    fn detect(&self) -> Vec<PathBuf> {
        Self::config_path().into_iter().collect()
    }

    fn plan(&self, palette: &Palette) -> Result<Vec<WriteOp>> {
        let Some(path) = Self::config_path() else {
            return Ok(vec![]);
        };

        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;

        let body = format!(
            "[colors]\nhours   = \"{}\"\nminutes = \"{}\"\nseconds = \"{}\"\ndate    = \"{}\"",
            palette.hex_or("primary", "#ffffff"),
            palette.hex_or("secondary", "#cccccc"),
            palette.hex_or("tertiary", "#aaaaaa"),
            palette.hex_or("on_surface", "#ffffff"),
        );

        let new_text = inject_section(&text, MARKER, &body);

        Ok(vec![WriteOp {
            adapter: "chronoterm",
            target: path,
            content: new_text,
        }])
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        Ok(ReloadOutcome { skipped: 1, ..Default::default() })
    }
}
