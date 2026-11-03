// GlazeWM adapter.
//
// Reads config.yaml and replaces the border colors inside
// window_effects -> focused_window and other_windows.
// Never touches the rest of the file.

use super::{home_dir, AppAdapter, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::os::windows::process::CommandExt;

pub struct GlazeWm;

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

    fn detect(&self, cfg: &crate::config::AppCfg) -> Vec<PathBuf> {
        if let Some(p) = &cfg.config_path {
            let path = PathBuf::from(p);
            if path.exists() { return vec![path]; }
        }
        Self::config_path().into_iter().collect()
    }

    fn plan(&self, palette: &Palette, cfg: &crate::config::AppCfg) -> Result<Vec<WriteOp>> {
        let Some(path) = self.detect(cfg).into_iter().next() else {
            return Ok(vec![]);
        };

        let original_text = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;

        let primary = palette.hex_or("primary", "#ffffff");
        let outline = palette.hex_or("outline", "#888888");

        let mut lines: Vec<String> = original_text.lines().map(String::from).collect();
        
        // State tracking for the text replacement
        let mut in_focused = false;
        let mut in_other = false;
        let mut in_border = false;

        for line in lines.iter_mut() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') { continue; }

            // Detect the start of the blocks
            if trimmed.starts_with("focused_window:") {
                in_focused = true;
                in_other = false;
                in_border = false;
                continue;
            }
            if trimmed.starts_with("other_windows:") {
                in_focused = false;
                in_other = true;
                in_border = false;
                continue;
            }
            
            // Detect the end of the blocks (a new top-level key under window_effects)
            if trimmed.ends_with(':') && !trimmed.starts_with("border:") && !trimmed.starts_with("hide_title_bar:") && !trimmed.starts_with("corner_style:") && !trimmed.starts_with("transparency:") {
                in_focused = false;
                in_other = false;
                in_border = false;
            }

            if (in_focused || in_other) && trimmed.starts_with("border:") {
                in_border = true;
                continue;
            }

            if in_border && trimmed.starts_with("color:") {
                let target_color = if in_focused { &primary } else { &outline };
                // Preserve indentation
                let indent = &line[..line.len() - line.trim_start().len()];
                *line = format!("{}color: '{}'", indent, target_color);
                
                // Assume only one color per border. Reset state to be safe.
                in_border = false; 
                in_focused = false;
                in_other = false;
            }
        }

        let mut new_text = lines.join("\n");
        if original_text.ends_with('\n') {
            new_text.push('\n');
        }

        Ok(vec![WriteOp::File {
            adapter: "glazewm",
            target: path,
            content: new_text,
            atomic: true,
        }])
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        let candidates = [
            "glazewm",
            r"C:\Program Files\glzr.io\GlazeWM\cli\glazewm.exe",
        ];

        for bin in candidates {
            let result = Command::new(bin)
                .creation_flags(0x08000000)
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