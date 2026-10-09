// Cava adapter.
//
// Writes murf-colors.ini next to the cava config, and ensures the
// main config has `include = murf-colors.ini` between markers.
// The include line is added at the top of the file, once.

use super::{home_dir, AppAdapter, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct Cava;

const MARKER_OPEN: &str = "# <murf>";
const MARKER_CLOSE: &str = "# </murf>";
const INCLUDE_LINE: &str = "include = murf-colors.ini";

impl Cava {
    fn dir() -> Option<PathBuf> {
        Some(home_dir()?.join(".config").join("cava"))
    }
    fn main_config() -> Option<PathBuf> {
        Some(Self::dir()?.join("config"))
    }
    fn companion() -> Option<PathBuf> {
        Some(Self::dir()?.join("murf-colors.ini"))
    }
}

impl AppAdapter for Cava {
    fn id(&self) -> &'static str { "cava" }
    fn display_name(&self) -> &'static str { "Cava" }

    fn detect(&self) -> Vec<PathBuf> {
        match Self::main_config() {
            Some(p) if p.exists() => vec![p],
            _ => vec![],
        }
    }

    fn plan(&self, palette: &Palette) -> Result<Vec<WriteOp>> {
        let Some(main) = Self::main_config() else { return Ok(vec![]) };
        let Some(companion) = Self::companion() else { return Ok(vec![]) };
        if !main.exists() { return Ok(vec![]); }

        let mut ops = Vec::new();

        // Companion file: always rewritten.
        let mut ini = String::from("[color]\n");
        ini.push_str(&format!(
            "background = '{}'\n",
            palette.hex_or("surface", "#000000")
        ));
        ini.push_str(&format!(
            "foreground = '{}'\n",
            palette.hex_or("on_surface", "#ffffff")
        ));
        let gradient_roles = [
            "primary", "secondary", "tertiary",
            "primary_container", "secondary_container", "tertiary_container",
            "primary_fixed", "secondary_fixed",
        ];
        for (i, role) in gradient_roles.iter().enumerate() {
            ini.push_str(&format!(
                "gradient_color_{} = '{}'\n",
                i + 1,
                palette.hex_or(role, "#ffffff")
            ));
        }
        ops.push(WriteOp {
            adapter: "cava",
            target: companion,
            content: ini,
        });

        // Main config: add include line once.
        let text = fs::read_to_string(&main)
            .with_context(|| format!("reading {}", main.display()))?;

        let has_include = text.lines().any(|l| {
            let t = l.trim();
            t == INCLUDE_LINE
                || t == "# <murf>"
                || t.starts_with("include")
                    && t.contains("murf-colors.ini")
        });

        if !has_include {
            let block = format!("{}\n{}\n{}", MARKER_OPEN, INCLUDE_LINE, MARKER_CLOSE);
            let new_text = format!("{}\n\n{}", block, text);
            ops.push(WriteOp {
                adapter: "cava",
                target: main,
                content: new_text,
            });
        }

        Ok(ops)
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        Ok(ReloadOutcome { skipped: 1, ..Default::default() })
    }
}
