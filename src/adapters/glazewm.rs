// GlazeWM adapter.
//
// Injects a `# <murf>` block containing a `colors:` map into
// %USERPROFILE%\.glaze-wm\config.yaml. Users reference the anchors
// from their window_effects section.

use super::{home_dir, inject_section, AppAdapter, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct GlazeWm;

const MARKER: &str = "# <murf>";

impl GlazeWm {
    fn config_path() -> Option<PathBuf> {
        Some(home_dir()?.join(".glaze-wm").join("config.yaml"))
    }
}

impl AppAdapter for GlazeWm {
    fn id(&self) -> &'static str { "glazewm" }
    fn display_name(&self) -> &'static str { "GlazeWM" }

    fn detect(&self) -> Vec<PathBuf> {
        match Self::config_path() {
            Some(p) if p.exists() => vec![p],
            _ => vec![],
        }
    }

    fn plan(&self, palette: &Palette) -> Result<Vec<WriteOp>> {
        let Some(path) = Self::config_path() else { return Ok(vec![]) };

        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;

        let body = format!(
            "colors:\n\
             \x20 primary:    \"{primary}\"\n\
             \x20 secondary:  \"{secondary}\"\n\
             \x20 tertiary:   \"{tertiary}\"\n\
             \x20 background: \"{background}\"\n\
             \x20 surface:    \"{surface}\"\n\
             \x20 on_surface: \"{on_surface}\"\n\
             \x20 error:      \"{error}\"\n\
             \x20 outline:    \"{outline}\"",
            primary    = palette.hex_or("primary", "#ffffff"),
            secondary  = palette.hex_or("secondary", "#cccccc"),
            tertiary   = palette.hex_or("tertiary", "#aaaaaa"),
            background = palette.hex_or("background", "#000000"),
            surface    = palette.hex_or("surface", "#181818"),
            on_surface = palette.hex_or("on_surface", "#ffffff"),
            error      = palette.hex_or("error", "#ff0000"),
            outline    = palette.hex_or("outline", "#888888"),
        );

        let new_text = inject_section(&text, MARKER, &body);

        Ok(vec![WriteOp {
            adapter: "glazewm",
            target: path,
            content: new_text,
        }])
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        // `glazewm command wm-reload-config` would be run here.
        // For v0.2 it's logged as deferred.
        Ok(ReloadOutcome { deferred: 1, ..Default::default() })
    }
}
