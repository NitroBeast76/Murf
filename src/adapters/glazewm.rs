// GlazeWM adapter.
//
// Locates config.yaml in any of the paths GlazeWM has used across
// versions, in priority order. Injects a `# <murf>` block with a
// colors: map.

use super::{home_dir, inject_section, AppAdapter, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct GlazeWm;

const MARKER: &str = "# <murf>";

impl GlazeWm {
    fn candidate_paths() -> Vec<PathBuf> {
        let mut out = Vec::new();
        if let Some(home) = home_dir() {
            out.push(home.join(".glzr").join("glazewm").join("config.yaml"));
            out.push(home.join(".glaze-wm").join("config.yaml"));
            out.push(home.join(".config").join("glazewm").join("config.yaml"));
        }
        if let Some(appdata) = std::env::var_os("APPDATA") {
            out.push(PathBuf::from(appdata).join("glazewm").join("config.yaml"));
        }
        out
    }

    fn config_path() -> Option<PathBuf> {
        Self::candidate_paths().into_iter().find(|p| p.exists())
    }
}

impl AppAdapter for GlazeWm {
    fn id(&self) -> &'static str { "glazewm" }
    fn display_name(&self) -> &'static str { "GlazeWM" }

    fn detect(&self) -> Vec<PathBuf> {
        Self::config_path().into_iter().collect()
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
        Ok(ReloadOutcome { deferred: 1, ..Default::default() })
    }
}
