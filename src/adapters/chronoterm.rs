// Chronoterm adapter.
//
// Replaces the [colors] table in %USERPROFILE%\.config\chronoterm\config.toml
// between `# <murf>` and `# </murf>` markers.

use super::{home_dir, inject_section, AppAdapter, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct Chronoterm;

const MARKER: &str = "# <murf>";

impl Chronoterm {
    fn config_path() -> Option<PathBuf> {
        Some(home_dir()?.join(".config").join("chronoterm").join("config.toml"))
    }
}

impl AppAdapter for Chronoterm {
    fn id(&self) -> &'static str { "chronoterm" }
    fn display_name(&self) -> &'static str { "Chronoterm" }

    fn detect(&self) -> Vec<PathBuf> {
        match Self::config_path() {
            Some(p) if p.exists() => vec![p],
            _ => vec![],
        }
    }

    fn plan(&self, palette: &Palette) -> Result<Vec<WriteOp>> {
        let Some(path) = Self::config_path() else {
            return Ok(vec![]);
        };
        if !path.exists() {
            return Ok(vec![]);
        }

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
