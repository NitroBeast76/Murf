// Adapter trait and registry. v0.1: one adapter (Windows Terminal).

pub mod windows_terminal;

use crate::palette::Palette;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Installation {
    pub app_id: &'static str,
    pub config_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum WriteOp {
    File { path: PathBuf, content: String },
}

pub trait AppAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;

    fn detect(&self) -> Option<Installation>;

    fn plan(&self, palette: &Palette, install: &Installation) -> Result<Vec<WriteOp>>;

    fn reload(&self, install: &Installation) -> Result<()>;
}
