// GlazeWM adapter.

use super::{home_dir, inject_at_top, AppAdapter, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct GlazeWm;

const MARKER_OPEN: &str = "# <murf>";
const MARKER_CLOSE: &str = "# </murf>";

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
             \x20 primary:    &murf_primary    \"{primary}\"\n\
             \x20 secondary:  &murf_secondary  \"{secondary}\"\n\
             \x20 tertiary:   &murf_tertiary   \"{tertiary}\"\n\
             \x20 background: &murf_background \"{background}\"\n\
             \x20 surface:    &murf_surface    \"{surface}\"\n\
             \x20 on_surface: &murf_on_surface \"{on_surface}\"\n\
             \x20 error:      &murf_error      \"{error}\"\n\
             \x20 outline:    &murf_outline    \"{outline}\"",
            primary    = palette.hex_or("primary", "#ffffff"),
            secondary  = palette.hex_or("secondary", "#cccccc"),
            tertiary   = palette.hex_or("tertiary", "#aaaaaa"),
            background = palette.hex_or("background", "#000000"),
            surface    = palette.hex_or("surface", "#181818"),
            on_surface = palette.hex_or("on_surface", "#ffffff"),
            error      = palette.hex_or("error", "#ff0000"),
            outline    = palette.hex_or("outline", "#888888"),
        );

        let new_text = inject_at_top(&text, MARKER_OPEN, MARKER_CLOSE, &body);

        Ok(vec![WriteOp::File {
            adapter: "glazewm",
            target: path,
            content: new_text,
        }])
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        use std::process::Command;

        let candidates = [
            "glazewm",
            r"C:\Program Files\glzr.io\GlazeWM\cli\glazewm.exe",
        ];

        for bin in candidates {
            let result = Command::new(bin)
                .args(["command", "wm-reload-config"])
                .output();
            if let Ok(out) = result {
                if out.status.success() {
                    tracing::info!(binary = bin, "glazewm reloaded");
                    return Ok(ReloadOutcome { live_reload: 1, ..Default::default() });
                }
            }
        }

        tracing::warn!("could not invoke glazewm; config written but not reloaded");
        Ok(ReloadOutcome { deferred: 1, ..Default::default() })
    }
}
