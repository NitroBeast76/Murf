// Starship adapter.
//
// Injects a [palettes.murf] block between markers, and replaces the
// root-level palette = "..." setting. The user's original value is
// preserved by the journaled backup of starship.toml.

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

/// Replace the root-level `palette = ...` line with the Murf value.
/// Lines inside triple-quoted strings are skipped. Scanning stops at
/// the first section header, because root-level keys cannot appear
/// after one.
fn set_root_palette(text: &str, name: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    let mut found = false;
    let mut in_multiline = false;

    for line in lines.iter_mut() {
        let triple = line.matches("\"\"\"").count();

        // If we entered this iteration inside a multiline, skip.
        if in_multiline {
            if triple % 2 == 1 {
                in_multiline = false;
            }
            continue;
        }
        // If this line opens a multiline, skip it and flip state.
        if triple % 2 == 1 {
            in_multiline = true;
            continue;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // A section header is `[something]` optionally followed by a
        // comment. Anything else (like `[x](y)\`) is content.
        if trimmed.starts_with('[') {
            if let Some(end) = trimmed.find(']') {
                let after = trimmed[end + 1..].trim();
                if after.is_empty() || after.starts_with('#') {
                    break; // root section ends here
                }
            }
        }

        if let Some(rest) = trimmed.strip_prefix("palette") {
            if rest.trim_start().starts_with('=') {
                *line = format!("palette = \"{}\"", name);
                found = true;
                break;
            }
        }
    }

    if !found {
        lines.insert(0, format!("palette = \"{}\"", name));
    }

    let mut out = lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    out
}
