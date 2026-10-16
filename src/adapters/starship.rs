// Starship adapter.
//
// Every value in the emitted palette is derived from the current
// matugen palette: surfaces from the background luminance ramp,
// accents from primary hue rotations. No canonical Catppuccin hues.

use super::{home_dir, inject_section, AppAdapter, ReloadOutcome, WriteOp};
use crate::color::{starship_accents, surface_ramp};
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

        let surfaces = surface_ramp(&bg);
        let accents  = starship_accents(&palette.roles);

        let body = format!(
            "[palettes.{name}]\n\
             \n\
             # Surfaces (luminance ramp from background)\n\
             crust      = \"{crust}\"\n\
             mantle     = \"{mantle}\"\n\
             base       = \"{base}\"\n\
             surface0   = \"{s0}\"\n\
             surface1   = \"{s1}\"\n\
             surface2   = \"{s2}\"\n\
             \n\
             # Text (M3 roles)\n\
             text       = \"{text}\"\n\
             subtext1   = \"{subtext1}\"\n\
             subtext0   = \"{subtext0}\"\n\
             overlay0   = \"{overlay0}\"\n\
             overlay1   = \"{overlay1}\"\n\
             overlay2   = \"{overlay2}\"\n\
             \n\
             # Accents (hue rotations of primary)\n\
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
             # M3 role aliases\n\
             primary    = \"{primary}\"\n\
             secondary  = \"{secondary}\"\n\
             tertiary   = \"{tertiary}\"\n\
             surface    = \"{surface}\"\n\
             on_surface = \"{on_surface}\"\n\
             error      = \"{error}\"",
            name = PALETTE_NAME,

            crust = surfaces.crust,
            mantle = surfaces.mantle,
            base = surfaces.base,
            s0 = surfaces.surface0,
            s1 = surfaces.surface1,
            s2 = surfaces.surface2,

            text     = palette.hex_or("on_surface",         "#ffffff"),
            subtext1 = palette.hex_or("on_surface_variant", "#dddddd"),
            subtext0 = palette.hex_or("on_surface_variant", "#bbbbbb"),
            overlay0 = palette.hex_or("outline_variant",    "#888888"),
            overlay1 = palette.hex_or("outline",            "#777777"),
            overlay2 = palette.hex_or("outline",            "#666666"),

            rosewater = accents.rosewater,
            flamingo  = accents.flamingo,
            pink      = accents.pink,
            mauve     = accents.mauve,
            red       = accents.red,
            maroon    = accents.maroon,
            peach     = accents.peach,
            yellow    = accents.yellow,
            green     = accents.green,
            teal      = accents.teal,
            sky       = accents.sky,
            sapphire  = accents.sapphire,
            blue      = accents.blue,
            lavender  = accents.lavender,

            primary    = primary.clone(),
            secondary  = palette.hex_or("secondary",   "#cccccc"),
            tertiary   = palette.hex_or("tertiary",    "#aaaaaa"),
            surface    = palette.hex_or("surface",     "#181818"),
            on_surface = palette.hex_or("on_surface",  "#ffffff"),
            error      = error.clone(),
        );

        let with_block = inject_section(&text, MARKER, &body);
        let final_text = set_root_palette(&with_block, PALETTE_NAME);

        Ok(vec![WriteOp::File {
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


