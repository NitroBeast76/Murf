// Cava adapter.
//
// Replaces the [color] table in cava's config in place with a
// Murf-managed version between markers. Removes the earlier
// companion-file include block if present.

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

    fn detect(&self) -> Vec<PathBuf> {
        Self::config_path().into_iter().collect()
    }

    fn plan(&self, palette: &Palette) -> Result<Vec<WriteOp>> {
        let Some(path) = Self::config_path() else {
            return Ok(vec![]);
        };

        let mut text = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;

        // Strip any earlier include block from the previous adapter
        // revision.
        if text.contains("murf-colors.ini") {
            text = remove_block(&text, MARKER_OPEN, MARKER_CLOSE);
        }

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

        Ok(vec![WriteOp {
            adapter: "cava",
            target: path,
            content: new_text,
        }])
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        Ok(ReloadOutcome { skipped: 1, ..Default::default() })
    }
}
