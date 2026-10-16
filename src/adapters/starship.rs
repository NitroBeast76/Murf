// Starship adapter.
//
// Emits the full set of color names a Catppuccin/Tokyo-Night-style
// format string references, so segment backgrounds and foregrounds
// resolve to Murf colors. Forces the root palette = "murf" setting.

use super::{home_dir, inject_section, AppAdapter, ReloadOutcome, WriteOp};
use crate::color::semantic_ansi;
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

        let primary = palette.hex_or("primary", "#ffffff");
        let error   = palette.hex_or("error",   "#ff0000");
        let bg      = palette.hex_or("background", "#000000");
        let ansi = semantic_ansi(&primary, &error, &bg, palette.is_dark);

        // The full Catppuccin-compatible name set. Every name maps
        // to a Murf M3 role or derived ANSI value. This covers the
        // standard segment style names used by community starship
        // configs (crust, peach, sapphire, lavender, etc.).
        let body = format!(
            "[palettes.{name}]\n\
             \n\
             # Dark-to-light surfaces\n\
             crust      = \"{crust}\"\n\
             mantle     = \"{mantle}\"\n\
             base       = \"{base}\"\n\
             surface0   = \"{surface0}\"\n\
             surface1   = \"{surface1}\"\n\
             surface2   = \"{surface2}\"\n\
             \n\
             # Text\n\
             text       = \"{text}\"\n\
             subtext1   = \"{subtext1}\"\n\
             subtext0   = \"{subtext0}\"\n\
             overlay0   = \"{overlay0}\"\n\
             overlay1   = \"{overlay1}\"\n\
             overlay2   = \"{overlay2}\"\n\
             \n\
             # Accents\n\
             rosewater  = \"{rosewater}\"\n\
             flamingo   = \"{flamingo}\"\n\
             pink       = \"{pink}\"\n\
             mauve      = \"{mauve}\"\n\
             red        = \"{red}\"\n\
             maroon     = \"{maroon}\"\n\
             peach      = \"{peach}\"\n\
             yellow     = \"{yellow}\"\n\
             green      = \"{green}\"\n\
             teal       = \"{teal}\"\n\
             sky        = \"{sky}\"\n\
             sapphire   = \"{sapphire}\"\n\
             blue       = \"{blue}\"\n\
             lavender   = \"{lavender}\"\n\
             \n\
             # M3 role aliases (for reference)\n\
             primary    = \"{primary}\"\n\
             secondary  = \"{secondary}\"\n\
             tertiary   = \"{tertiary}\"\n\
             surface    = \"{surface}\"\n\
             on_surface = \"{on_surface}\"\n\
             error      = \"{error}\"",
            name = PALETTE_NAME,

            // Surfaces, dark → light
            crust    = palette.hex_or("surface_container_lowest", "#0a0a0a"),
            mantle   = palette.hex_or("surface_container_low",    "#111111"),
            base     = palette.hex_or("surface",                  "#181818"),
            surface0 = palette.hex_or("surface_container",        "#1e1e1e"),
            surface1 = palette.hex_or("surface_container_high",   "#242424"),
            surface2 = palette.hex_or("surface_container_highest","#2a2a2a"),

            // Text layers
            text     = palette.hex_or("on_surface",         "#ffffff"),
            subtext1 = palette.hex_or("on_surface_variant", "#dddddd"),
            subtext0 = palette.hex_or("outline",            "#bbbbbb"),
            overlay0 = palette.hex_or("outline_variant",    "#999999"),
            overlay1 = palette.hex_or("outline",            "#888888"),
            overlay2 = palette.hex_or("outline",            "#777777"),

            // Accents
            rosewater = palette.hex_or("on_primary",         "#ffe0e0"),
            flamingo  = palette.hex_or("error_container",    "#ffaaaa"),
            pink      = palette.hex_or("tertiary_container", "#ffccee"),
            mauve     = ansi.purple.clone(),
            red       = ansi.red.clone(),
            maroon    = palette.hex_or("error_container",    "#cc6666"),
            peach     = palette.hex_or("secondary",          "#ffaa88"),
            yellow    = ansi.yellow.clone(),
            green     = ansi.green.clone(),
            teal      = ansi.cyan.clone(),
            sky       = palette.hex_or("tertiary",           "#aaddff"),
            sapphire  = ansi.blue.clone(),
            blue      = primary.clone(),
            lavender  = palette.hex_or("tertiary",           "#ccccff"),

            // M3 role aliases
            primary    = primary.clone(),
            secondary  = palette.hex_or("secondary",   "#cccccc"),
            tertiary   = palette.hex_or("tertiary",    "#aaaaaa"),
            surface    = palette.hex_or("surface",     "#181818"),
            on_surface = palette.hex_or("on_surface",  "#ffffff"),
            error      = error.clone(),
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
/// Skips lines inside triple-quoted strings. Skips section content.
/// Drops duplicate root `palette` lines.
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
