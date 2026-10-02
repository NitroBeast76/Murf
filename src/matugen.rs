// Matugen bridge. See docs/matugen-contract.md for the verified invocation.

use crate::palette::{Color, Palette};
use anyhow::{anyhow, Context, Result};
use std::collections::BTreeMap;
use std::process::Command;

pub fn generate(
    image_path: &str,
    mode: &str,
    scheme_type: &str,
    contrast: f32,
) -> Result<Palette> {
    let output = Command::new("matugen")
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
        .context("failed to run matugen; is it on PATH?")?;

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

    Ok(Palette { roles })
}

/// Current wallpaper path from the Windows registry.
pub fn current_wallpaper() -> Option<String> {
    use std::process::Command;
    // Read HKCU\Control Panel\Desktop\Wallpaper via reg.exe.
    // Avoids the `windows` crate dependency for this one call.
    let out = Command::new("reg")
        .args([
            "query",
            "HKCU\\Control Panel\\Desktop",
            "/v",
            "Wallpaper",
        ])
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
