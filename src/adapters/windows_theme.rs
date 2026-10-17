// Windows Theme (Light/Dark mode) adapter.
//
// Writes the user's system theme to match the palette's is_dark flag.
// Disabled by default in murf.toml; user must opt in.

use super::{AppAdapter, RegistryData, RegistryHive, ReloadOutcome, WriteOp};
use crate::palette::Palette;
use anyhow::Result;
use std::path::PathBuf;

pub struct WindowsTheme;

const THEME_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";

impl AppAdapter for WindowsTheme {
    fn id(&self) -> &'static str { "windows_theme" }
    fn display_name(&self) -> &'static str { "Windows Theme" }

    fn detect(&self) -> Vec<PathBuf> {
        vec![PathBuf::from("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize")]
    }

    fn plan(&self, palette: &Palette) -> Result<Vec<WriteOp>> {
        // 1 = Light, 0 = Dark
        let val = if palette.is_dark { 0 } else { 1 };

        Ok(vec![
            WriteOp::Registry {
                adapter: "windows_theme",
                hive: RegistryHive::CurrentUser,
                path: THEME_PATH.to_string(),
                value: "AppsUseLightTheme".to_string(),
                data: RegistryData::Dword(val),
            },
            WriteOp::Registry {
                adapter: "windows_theme",
                hive: RegistryHive::CurrentUser,
                path: THEME_PATH.to_string(),
                value: "SystemUsesLightTheme".to_string(),
                data: RegistryData::Dword(val),
            },
        ])
    }

    fn reload(&self) -> Result<ReloadOutcome> {
        crate::registry::broadcast_setting_change()?;
        Ok(ReloadOutcome { live_reload: 1, ..Default::default() })
    }
}
