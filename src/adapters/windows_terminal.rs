// Windows Terminal adapter.
// Injects a "Murf" scheme into settings.json and points profiles at it.

use super::{AppAdapter, Installation, WriteOp};
use crate::palette::Palette;
use anyhow::Result;
use std::path::PathBuf;

pub struct WindowsTerminal;

impl WindowsTerminal {
    fn candidate_paths() -> Vec<PathBuf> {
        let local = std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA not set");
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
}

impl AppAdapter for WindowsTerminal {
    fn id(&self) -> &'static str { "windows_terminal" }
    fn display_name(&self) -> &'static str { "Windows Terminal" }

    fn detect(&self) -> Option<Installation> {
        let found: Vec<PathBuf> = Self::candidate_paths()
            .into_iter()
            .filter(|p| p.exists())
            .collect();
        if found.is_empty() {
            None
        } else {
            Some(Installation {
                app_id: self.id(),
                config_paths: found,
            })
        }
    }

    fn plan(&self, _palette: &Palette, _install: &Installation) -> Result<Vec<WriteOp>> {
        // TODO: read each settings.json as text, find or create
        // the "Murf" scheme, point profiles at it, write back.
        todo!()
    }

    fn reload(&self, _install: &Installation) -> Result<()> {
        Ok(()) // Windows Terminal watches its own file.
    }
}
