// Matugen bridge.
//
// Extracts the embedded matugen.exe to %LOCALAPPDATA%\Murf\bin on first run,
// then invokes it.

use crate::palette::{Color, Palette};
use anyhow::{anyhow, Context, Result};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::os::windows::process::CommandExt;

// Embed the matugen executable directly into Murf.
const MATUGEN_BINARY: &[u8] = include_bytes!("../assets/matugen.exe");

/// Ensure matugen.exe is extracted to disk and return its path.
fn get_matugen_path() -> Result<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| anyhow!("LOCALAPPDATA not set"))?;
    let dir = PathBuf::from(local).join("Murf").join("bin");
    let exe = dir.join("matugen.exe");

    if !exe.exists() {
        std::fs::create_dir_all(&dir)?;
        std::fs::write(&exe, MATUGEN_BINARY)
            .context("failed to extract embedded matugen.exe")?;
        tracing::info!(path = %exe.display(), "extracted embedded matugen");
    }

    Ok(exe)
}

pub fn generate(
    image_path: &str,
    mode: &str,
    scheme_type: &str,
    contrast: f32,
) -> Result<Palette> {
    let matugen_exe = get_matugen_path()?;

    let output = Command::new(&matugen_exe)
        .creation_flags(0x08000000)
        .args([
            "image",
            image_path,
            "-j",
            "hex",
            "--dry-run",
            "--mode",
            mode,
            "--type",
            scheme_type,
            "--contrast",
            &contrast.to_string(),
            "--source-color-index",
            "0",
        ])
        .output()
        .context("failed to run matugen")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("matugen exited with {}: {}", output.status, stderr));
    }

    let stdout = String::from_utf8(output.stdout)
        .context("matugen stdout was not valid UTF-8")?;

    parse_palette(&stdout)
}

fn parse_palette(json: &str) -> Result<Palette> {
    let v: serde_json::Value =
        serde_json::from_str(json).context("matugen did not emit valid JSON")?;

    let colors = v
        .get("colors")
        .and_then(|c| c.as_object())
        .ok_or_else(|| anyhow!("matugen JSON missing top-level \"colors\" object"))?;

    let mut roles = BTreeMap::new();
    for (role_name, role_obj) in colors {
        let hex = role_obj
            .get("default")
            .and_then(|d| d.get("color"))
            .and_then(|c| c.as_str());
        if let Some(hex) = hex {
            roles.insert(role_name.clone(), Color { hex: hex.to_string() });
        }
    }

    if roles.is_empty() {
        return Err(anyhow!("matugen JSON contained no usable role colors"));
    }

    let is_dark = match v.get("is_dark_mode").and_then(|b| b.as_bool()) {
        Some(b) => b,
        None => infer_is_dark(&roles),
    };

    Ok(Palette { roles, is_dark })
}

fn infer_is_dark(roles: &BTreeMap<String, Color>) -> bool {
    let Some(bg) = roles.get("background").map(|c| c.hex.as_str()) else {
        return true;
    };
    match crate::color::hex_to_oklch(bg) {
        Some(c) => c.l < 0.5,
        None => true,
    }
}

pub fn current_wallpaper() -> Option<String> {
    let out = Command::new("reg")
        .creation_flags(0x08000000)
        .args(["query", "HKCU\\Control Panel\\Desktop", "/v", "Wallpaper"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if line.contains("Wallpaper") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(last) = parts.last() {
                return Some(last.to_string());
            }
        }
    }
    None
}