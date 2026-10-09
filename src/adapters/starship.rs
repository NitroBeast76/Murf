// Starship adapter.
//
// Injects a [palettes.murf] block into starship.toml between markers,
// and sets `palette = "murf"` at the root level.

use super::{home_dir, inject_section, AppAdapter, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct Starship;

const MARKER: &str = "# <murf>";

impl Starship {
    fn config_path() -> Option<PathBuf> {
        Some(home_dir()?.join(".config").join("starship.toml"))
    }
}

impl AppAdapter for Starship {
    fn id(&self) -> &'static str { "starship" }
    fn display_name(&self) -> &'static str { "Starship" }

    fn detect(&self) -> Vec<PathBuf> {
        match Self::config_path() {
            Some(p) if p.exists() => vec![p],
            _ => vec![],
        }
    }

    fn plan(&self, palette: &Palette) -> Result<Vec<WriteOp>> {
        let Some(path) = Self::config_path() else { return Ok(vec![]) };
        if !path.exists() { return Ok(vec![]); }

        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;

        let body = format!(
            "[palettes.murf]\n\
             primary   = \"{}\"\n\
             secondary = \"{}\"\n\
             tertiary  = \"{}\"\n\
             surface   = \"{}\"\n\
             on_surface = \"{}\"\n\
             error     = \"{}\"",
            palette.hex_or("primary", "#ffffff"),
            palette.hex_or("secondary", "#cccccc"),
            palette.hex_or("tertiary", "#aaaaaa"),
            palette.hex_or("surface", "#000000"),
            palette.hex_or("on_surface", "#ffffff"),
            palette.hex_or("error", "#ff0000"),
        );

        let mut new_text = inject_section(&text, MARKER, &body);

        // Ensure root-level `palette = "murf"`.
        let has_root_palette = new_text.lines().any(|l| {
            let t = l.trim();
            t.starts_with("palette") && t.contains("=") && !t.starts_with("palettes")
        });

        if !has_root_palette {
            new_text = format!("palette = \"murf\"\n{}", new_text);
        }

        Ok(vec![WriteOp {
            adapter: "starship",
            target: path,
            content: new_text,
        }])
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        Ok(ReloadOutcome { live_reload: 1, ..Default::default() })
    }
}
