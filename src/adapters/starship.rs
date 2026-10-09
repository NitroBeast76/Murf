// Starship adapter.
//
// Injects a [palettes.murf] block into starship.toml between markers,
// and forces the root-level `palette = "murf"` setting. The user's
// original palette name is not preserved in a separate file; the
// journaled backup of starship.toml handles restoration.

use super::{home_dir, inject_section, AppAdapter, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct Starship;

const MARKER: &str = "# <murf>";
const PALETTE_NAME: &str = "murf";

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

        // Inject/refresh the [palettes.murf] block.
        let body = format!(
            "[palettes.{name}]\n\
             primary    = \"{primary}\"\n\
             secondary  = \"{secondary}\"\n\
             tertiary   = \"{tertiary}\"\n\
             surface    = \"{surface}\"\n\
             on_surface = \"{on_surface}\"\n\
             error      = \"{error}\"",
            name = PALETTE_NAME,
            primary    = palette.hex_or("primary", "#ffffff"),
            secondary  = palette.hex_or("secondary", "#cccccc"),
            tertiary   = palette.hex_or("tertiary", "#aaaaaa"),
            surface    = palette.hex_or("surface", "#000000"),
            on_surface = palette.hex_or("on_surface", "#ffffff"),
            error      = palette.hex_or("error", "#ff0000"),
        );
        let with_block = inject_section(&text, MARKER, &body);

        // Force root-level `palette = "murf"`.
        let final_text = set_root_palette(&with_block, PALETTE_NAME);

        Ok(vec![WriteOp {
            adapter: "starship",
            target: path,
            content: final_text,
        }])
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        Ok(ReloadOutcome { live_reload: 1, ..Default::default() })
    }
}

/// Find the root-level `palette = ...` line and replace its value with
/// `palette = "murf"`. If no such line exists, insert one at the top.
/// Section headers (lines starting with `[`) disable the search for
/// the rest of the file, because keys inside a section are not
/// root-level.
fn set_root_palette(text: &str, name: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    let mut found = false;
    let mut in_section = false;

    for line in lines.iter_mut() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            in_section = true;
            continue;
        }
        if in_section {
            continue;
        }

        // Root-level key.
        if let Some(rest) = trimmed.strip_prefix("palette") {
            if rest.trim_start().starts_with('=') {
                *line = format!("palette = \"{}\"", name);
                found = true;
                break;
            }
        }
    }

    if !found {
        // Insert at the top, before the first line.
        lines.insert(0, format!("palette = \"{}\"", name));
    }

    let mut out = lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    out
}
