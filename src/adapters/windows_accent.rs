// Windows 11 accent adapter.
//
// Writes the current palette's primary color into HKCU registry keys
// that Explorer and WinUI read for the accent color, and broadcasts
// WM_SETTINGCHANGE so live surfaces pick it up.

use super::{AppAdapter, RegistryData, RegistryHive, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::Result;
use std::path::PathBuf;

pub struct WindowsAccent;

const DWM_PATH: &str  = r"Software\Microsoft\Windows\DWM";
const EXPLORER_ACCENT_PATH: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent";

/// Convert "#rrggbb" to a DWORD in ABGR order (Windows uses 0xAABBGGRR).
fn hex_to_abgr(hex: &str) -> Option<u32> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as u32;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as u32;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as u32;
    Some(0xFF00_0000 | (b << 16) | (g << 8) | r)
}

/// Build the 32-byte AccentPalette blob Explorer reads. It expects 8
/// ARGB dwords representing a shade ramp of the accent. We produce a
/// simple ramp from the accent color.
fn accent_palette_blob(primary_hex: &str) -> Option<Vec<u8>> {
    let hex = primary_hex.trim_start_matches('#');
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as u32;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as u32;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as u32;

    // 8 stops from dark to light. Linear interpolation toward white
    // for the light end and toward black for the dark end.
    let mut out = Vec::with_capacity(32);
    let stops: [f32; 8] = [0.2, 0.4, 0.55, 0.7, 0.85, 1.0, 1.15, 1.3];

    for s in stops {
        let mut cr = (r as f32 * s).min(255.0) as u32;
        let mut cg = (g as f32 * s).min(255.0) as u32;
        let mut cb = (b as f32 * s).min(255.0) as u32;
        if s > 1.0 {
            // lighten toward white
            let t = (s - 1.0) / 0.3;
            cr = (cr as f32 + (255.0 - cr as f32) * t).min(255.0) as u32;
            cg = (cg as f32 + (255.0 - cg as f32) * t).min(255.0) as u32;
            cb = (cb as f32 + (255.0 - cb as f32) * t).min(255.0) as u32;
        }
        // ARGB little-endian
        out.push(cb as u8);
        out.push(cg as u8);
        out.push(cr as u8);
        out.push(0xFF);
    }
    Some(out)
}

impl AppAdapter for WindowsAccent {
    fn id(&self) -> &'static str { "windows_accent" }
    fn display_name(&self) -> &'static str { "Windows 11 accent" }

    fn detect(&self) -> Vec<PathBuf> {
        // Always available. Registry path is created by Explorer.
        vec![PathBuf::from("HKCU\\Software\\Microsoft\\Windows\\DWM")]
    }

    fn plan(&self, palette: &Palette) -> Result<Vec<WriteOp>> {
        let primary = palette.hex_or("primary", "#0078D4");
        let accent = match hex_to_abgr(&primary) {
            Some(v) => v,
            None => return Ok(vec![]),
        };
        let palette_blob = match accent_palette_blob(&primary) {
            Some(b) => b,
            None => return Ok(vec![]),
        };

        Ok(vec![
            WriteOp::Registry {
                adapter: "windows_accent",
                hive: RegistryHive::CurrentUser,
                path: DWM_PATH.to_string(),
                value: "AccentColor".to_string(),
                data: RegistryData::Dword(accent),
            },
            WriteOp::Registry {
                adapter: "windows_accent",
                hive: RegistryHive::CurrentUser,
                path: DWM_PATH.to_string(),
                value: "ColorizationColor".to_string(),
                data: RegistryData::Dword(accent),
            },
            WriteOp::Registry {
                adapter: "windows_accent",
                hive: RegistryHive::CurrentUser,
                path: DWM_PATH.to_string(),
                value: "ColorizationAfterglow".to_string(),
                data: RegistryData::Dword(accent),
            },
            WriteOp::Registry {
                adapter: "windows_accent",
                hive: RegistryHive::CurrentUser,
                path: EXPLORER_ACCENT_PATH.to_string(),
                value: "AccentPalette".to_string(),
                data: RegistryData::Binary(palette_blob),
            },
        ])
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        crate::registry::broadcast_setting_change()?;
        Ok(ReloadOutcome { live_reload: 1, ..Default::default() })
    }
}
