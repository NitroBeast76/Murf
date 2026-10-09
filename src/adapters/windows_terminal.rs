// Windows Terminal adapter.

use crate::color::{material_ansi, semantic_ansi, AnsiMode};
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

pub fn apply(palette: &Palette, mode: AnsiMode) -> Result<Vec<PathBuf>> {
    let paths = detect();
    if paths.is_empty() {
        tracing::info!("Windows Terminal not found; skipping");
        return Ok(vec![]);
    }

    let scheme = build_scheme(palette, mode);
    for path in &paths {
        apply_to_file(path, &scheme)
            .with_context(|| format!("applying to {}", path.display()))?;
    }
    Ok(paths)
}

fn build_scheme(p: &Palette, mode: AnsiMode) -> serde_json::Value {
    use serde_json::json;

    let primary = p.hex_or("primary", "#808080");
    let error   = p.hex_or("error",   "#ff0000");
    let bg      = p.hex_or("background", "#000000");

    let ansi = match mode {
        AnsiMode::Semantic => semantic_ansi(&primary, &error, &bg, p.is_dark),
        AnsiMode::Material => material_ansi(
            &error,
            &p.hex_or("tertiary",              "#00ff00"),
            &p.hex_or("secondary",             "#ffff00"),
            &primary,
            &p.hex_or("tertiary_container",    "#ff00ff"),
            &p.hex_or("secondary_container",   "#00ffff"),
            &error,
            &p.hex_or("tertiary",              "#00ff00"),
            &p.hex_or("secondary",             "#ffff00"),
            &primary,
            &p.hex_or("tertiary_container",    "#ff00ff"),
            &p.hex_or("secondary_container",   "#00ffff"),
        ),
    };

    json!({
        "name": "Murf",
        "background":          p.hex_or("surface",                  "#000000"),
        "foreground":          p.hex_or("on_surface",               "#ffffff"),
        "cursorColor":         p.hex_or("primary",                  "#ffffff"),
        "selectionBackground": p.hex_or("surface_container_high",   "#333333"),
        "black":               p.hex_or("surface_container_lowest", "#000000"),
        "red":                 ansi.red,
        "green":               ansi.green,
        "yellow":              ansi.yellow,
        "blue":                ansi.blue,
        "purple":              ansi.purple,
        "cyan":                ansi.cyan,
        "white":               p.hex_or("on_surface",               "#ffffff"),
        "brightBlack":         p.hex_or("surface_container_low",    "#333333"),
        "brightRed":           ansi.bright_red,
        "brightGreen":         ansi.bright_green,
        "brightYellow":        ansi.bright_yellow,
        "brightBlue":          ansi.bright_blue,
        "brightPurple":        ansi.bright_purple,
        "brightCyan":          ansi.bright_cyan,
        "brightWhite":         p.hex_or("on_background",            "#ffffff"),
    })
}

fn apply_to_file(path: &Path, scheme: &serde_json::Value) -> Result<()> {
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

    if settings.get("schemes").is_none() {
        settings["schemes"] = serde_json::json!([]);
    }
    let schemes = settings["schemes"]
        .as_array_mut()
        .ok_or_else(|| anyhow::anyhow!("\"schemes\" is not an array"))?;

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

    if settings.get("profiles").is_none() {
        settings["profiles"] = serde_json::json!({});
    }
    if settings["profiles"].get("defaults").is_none() {
        settings["profiles"]["defaults"] = serde_json::json!({});
    }

    let previous = settings["profiles"]["defaults"]
        .get("colorScheme")
        .and_then(|v| v.as_str())
        .map(String::from);
    if let Some(prev) = previous {
        if prev != "Murf" {
            tracing::info!(previous = %prev, "overriding user colorScheme");
        }
    }
    settings["profiles"]["defaults"]["colorScheme"] = serde_json::json!("Murf");

    let new_text = serde_json::to_string_pretty(&settings)?;
    fs::write(path, new_text)
        .with_context(|| format!("writing {}", path.display()))?;

    tracing::info!(path = %path.display(), "wrote Windows Terminal settings");
    Ok(())
}
