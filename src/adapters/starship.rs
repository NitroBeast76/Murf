// Starship adapter.
//
// Injects a [palettes.murf] block with the full set of names
// Starship knows, and forces the root palette = "murf" setting.

use super::{home_dir, inject_section, AppAdapter, ReloadOutcome, WriteOp};
use crate::color::{semantic_ansi, AnsiMode};
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

        // Include both the M3 names and the ANSI set, so format
        // strings using `bg:red`, `bg:green`, etc. resolve.
        let primary = palette.hex_or("primary", "#ffffff");
        let error   = palette.hex_or("error",   "#ff0000");
        let bg      = palette.hex_or("background", "#000000");
        let ansi = semantic_ansi(&primary, &error, &bg, palette.is_dark);

        let body = format!(
            "[palettes.{name}]\n\
             # M3 roles\n\
             primary    = \"{primary}\"\n\
             secondary  = \"{secondary}\"\n\
             tertiary   = \"{tertiary}\"\n\
             surface    = \"{surface}\"\n\
             on_surface = \"{on_surface}\"\n\
             error      = \"{error}\"\n\
             \n\
             # ANSI set\n\
             black   = \"{black}\"\n\
             red     = \"{red}\"\n\
             green   = \"{green}\"\n\
             yellow  = \"{yellow}\"\n\
             blue    = \"{blue}\"\n\
             purple  = \"{purple}\"\n\
             cyan    = \"{cyan}\"\n\
             white   = \"{white}\"",
            name = PALETTE_NAME,
            primary    = primary,
            secondary  = palette.hex_or("secondary", "#cccccc"),
            tertiary   = palette.hex_or("tertiary", "#aaaaaa"),
            surface    = palette.hex_or("surface", "#000000"),
            on_surface = palette.hex_or("on_surface", "#ffffff"),
            error      = error,
            black  = palette.hex_or("surface_container_lowest", "#000000"),
            red    = ansi.red,
            green  = ansi.green,
            yellow = ansi.yellow,
            blue   = ansi.blue,
            purple = ansi.purple,
            cyan   = ansi.cyan,
            white  = palette.hex_or("on_surface", "#ffffff"),
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
/// the first section header. Also removes any duplicate `palette = ...`
/// lines that appear after the first one.
fn set_root_palette(text: &str, name: &str) -> String {
    let lines: Vec<String> = text.lines().map(String::from).collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut in_multiline = false;
    let mut in_section = false;
    let mut seen = false;

    for line in lines {
        let triple = line.matches("\"\"\"").count();

        if in_multiline {
            out.push(line.clone());
            if triple % 2 == 1 { in_multiline = false; }
            continue;
        }
        if triple % 2 == 1 {
            out.push(line.clone());
            in_multiline = true;
            continue;
        }

        let trimmed = line.trim();

        if !in_section && trimmed.starts_with('[') {
            if let Some(end) = trimmed.find(']') {
                let after = trimmed[end + 1..].trim();
                if after.is_empty() || after.starts_with('#') {
                    in_section = true;
                }
            }
        }

        if !in_section {
            if let Some(rest) = trimmed.strip_prefix("palette") {
                if rest.trim_start().starts_with('=') {
                    if !seen {
                        out.push(format!("palette = \"{}\"", name));
                        seen = true;
                    }
                    // Skip duplicates.
                    continue;
                }
            }
        }

        out.push(line);
    }

    if !seen {
        out.insert(0, format!("palette = \"{}\"", name));
    }

    let mut s = out.join("\n");
    if text.ends_with('\n') { s.push('\n'); }
    s
}
