// Cava adapter.

use super::{home_dir, remove_block, replace_table, AppAdapter, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct Cava;

const MARKER_OPEN: &str = "# <murf>";
const MARKER_CLOSE: &str = "# </murf>";

impl Cava {
    fn config_path() -> Option<PathBuf> {
        let home = home_dir()?;
        let p = home.join(".config").join("cava").join("config");
        if p.exists() { Some(p) } else { None }
    }
}

impl AppAdapter for Cava {
    fn id(&self) -> &'static str { "cava" }
    fn display_name(&self) -> &'static str { "Cava" }

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

        let mut text = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;

        // 1. Ensure Cava's native watcher is enabled
        let mut has_live_config = false;
        let mut lines: Vec<String> = text.lines().map(String::from).collect();
        
        for line in lines.iter_mut() {
            let t = line.trim();
            if t.starts_with("live-config") {
                // Replace existing line
                let indent = &line[..line.len() - line.trim_start().len()];
                *line = format!("{}live-config = 1", indent);
                has_live_config = true;
                break;
            }
        }
        
        if !has_live_config {
            // Inject it right after [general]
            let mut injected = false;
            for (i, line) in lines.iter().enumerate() {
                if line.trim() == "[general]" {
                    lines.insert(i + 1, "live-config = 1".to_string());
                    injected = true;
                    break;
                }
            }
            // If no [general], just append it at the top
            if !injected {
                lines.insert(0, "[general]\nlive-config = 1\n".to_string());
            }
        }
        
        text = lines.join("\n");
        if !text.ends_with('\n') { text.push('\n'); }

        // 2. Strip old Murf color block
        if text.contains("murf-colors.ini") || text.contains("# <murf>") {
            text = remove_block(&text, MARKER_OPEN, MARKER_CLOSE);
        }

        // 3. Generate new color block
        let body = format!(
            "[color]\n\
             background = '{bg}'\n\
             foreground = '{fg}'\n\
             gradient = 1\n\
             gradient_color_1 = '{c1}'\n\
             gradient_color_2 = '{c2}'\n\
             gradient_color_3 = '{c3}'\n\
             gradient_color_4 = '{c4}'\n\
             gradient_color_5 = '{c5}'\n\
             gradient_color_6 = '{c6}'\n\
             gradient_color_7 = '{c7}'\n\
             gradient_color_8 = '{c8}'",
            bg = palette.hex_or("surface", "#000000"),
            fg = palette.hex_or("on_surface", "#ffffff"),
            c1 = palette.hex_or("primary", "#ffffff"),
            c2 = palette.hex_or("secondary", "#cccccc"),
            c3 = palette.hex_or("tertiary", "#aaaaaa"),
            c4 = palette.hex_or("primary_container", "#888888"),
            c5 = palette.hex_or("secondary_container", "#777777"),
            c6 = palette.hex_or("tertiary_container", "#666666"),
            c7 = palette.hex_or("primary_fixed", "#555555"),
            c8 = palette.hex_or("secondary_fixed", "#444444"),
        );

        let new_text = replace_table(&text, "color", MARKER_OPEN, MARKER_CLOSE, &body);

        Ok(vec![WriteOp::File {
            adapter: "cava",
            target: path,
            content: new_text,
            atomic: false, // In-place to trigger the watcher reliably
        }])
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        tracing::info!("cava live-config enabled; watcher will reload automatically");
        Ok(ReloadOutcome { live_reload: 1, ..Default::default() })
    }
}