// Windows Terminal adapter.
//
// Injects a "Murf" scheme into settings.json and sets
// profiles.defaults.colorScheme = "Murf" if not already set.
//
// v0.1 writes JSON directly via serde_json. This strips comments
// if the user had any. v0.2 replaces this with text-region editing
// between marker lines, preserving everything outside the block.

use crate::palette::Palette;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn candidate_paths() -> Vec<PathBuf> {
    let Some(local) = std::env::var_os("LOCALAPPDATA") else {
        return vec![];
    };
    let base = PathBuf::from(local).join("Packages");
    vec![
        base.join("Microsoft.WindowsTerminal_8wekyb3d8bbwe")
            .join("LocalState")
            .join("settings.json"),
        base.join("Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe")
            .join("LocalState")
            .join("settings.json"),
    ]
}

pub fn detect() -> Vec<PathBuf> {
    candidate_paths()
        .into_iter()
        .filter(|p| p.exists())
        .collect()
}

pub fn apply(palette: &Palette) -> Result<Vec<PathBuf>> {
    let paths = detect();
    if paths.is_empty() {
        tracing::info!("Windows Terminal not found; skipping");
        return Ok(vec![]);
    }

    let scheme = build_scheme(palette);
    for path in &paths {
        apply_to_file(path, &scheme)
            .with_context(|| format!("applying to {}", path.display()))?;
    }
    Ok(paths)
}

fn build_scheme(p: &Palette) -> serde_json::Value {
    use serde_json::json;
    json!({
        "name": "Murf",
        "background":          p.hex_or("surface",                   "#000000"),
        "foreground":          p.hex_or("on_surface",                "#ffffff"),
        "cursorColor":         p.hex_or("primary",                   "#ffffff"),
        "selectionBackground": p.hex_or("surface_container_high",    "#333333"),
        "black":               p.hex_or("surface_container_lowest",  "#000000"),
        "red":                 p.hex_or("error",                     "#ff0000"),
        "green":               p.hex_or("tertiary",                  "#00ff00"),
        "yellow":              p.hex_or("secondary",                 "#ffff00"),
        "blue":                p.hex_or("primary",                   "#0000ff"),
        "purple":              p.hex_or("tertiary_container",        "#ff00ff"),
        "cyan":                p.hex_or("secondary_container",       "#00ffff"),
        "white":               p.hex_or("on_surface",                "#ffffff"),
        "brightBlack":         p.hex_or("surface_container_low",     "#333333"),
        "brightRed":           p.hex_or("error",                     "#ff0000"),
        "brightGreen":         p.hex_or("tertiary",                  "#00ff00"),
        "brightYellow":        p.hex_or("secondary",                 "#ffff00"),
        "brightBlue":          p.hex_or("primary",                   "#0000ff"),
        "brightPurple":        p.hex_or("tertiary_container",        "#ff00ff"),
        "brightCyan":          p.hex_or("secondary_container",       "#00ffff"),
        "brightWhite":         p.hex_or("on_background",             "#ffffff"),
    })
}

fn apply_to_file(path: &Path, scheme: &serde_json::Value) -> Result<()> {
    // Backup once. Not overwritten on subsequent runs.
    let backup = path.with_extension("json.murf-bak");
    if !backup.exists() {
        fs::copy(path, &backup)
            .with_context(|| format!("backing up to {}", backup.display()))?;
        tracing::info!(backup = %backup.display(), "backup created");
    }

    let text = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let mut settings: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("parsing {} as JSON", path.display()))?;

    // Ensure schemes[] exists.
    if settings.get("schemes").is_none() {
        settings["schemes"] = serde_json::json!([]);
    }
    let schemes = settings["schemes"]
        .as_array_mut()
        .ok_or_else(|| anyhow::anyhow!("\"schemes\" is not an array"))?;

    // Find and replace "Murf", or append.
    let mut replaced = false;
    for s in schemes.iter_mut() {
        if s.get("name").and_then(|n| n.as_str()) == Some("Murf") {
            *s = scheme.clone();
            replaced = true;
            break;
        }
    }
    if !replaced {
        schemes.push(scheme.clone());
    }

    // Ensure profiles.defaults exists and set colorScheme if absent.
    if settings.get("profiles").is_none() {
        settings["profiles"] = serde_json::json!({});
    }
    if settings["profiles"].get("defaults").is_none() {
        settings["profiles"]["defaults"] = serde_json::json!({});
    }
    if settings["profiles"]["defaults"].get("colorScheme").is_none() {
        settings["profiles"]["defaults"]["colorScheme"] =
            serde_json::json!("Murf");
    }

    // Write back.
    let new_text = serde_json::to_string_pretty(&settings)?;
    fs::write(path, new_text)
        .with_context(|| format!("writing {}", path.display()))?;

    tracing::info!(path = %path.display(), "wrote Windows Terminal settings");
    Ok(())
}
