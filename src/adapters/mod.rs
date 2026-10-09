// Adapter trait and write operations.

pub mod cava;
pub mod chronoterm;
pub mod starship;
pub mod windows_terminal;

use crate::palette::Palette;
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WriteOp {
    pub adapter: &'static str,
    pub target: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone, Default)]
pub struct ReloadOutcome {
    pub live_reload: usize,
    pub restarted: usize,
    pub deferred: usize,
    pub skipped: usize,
}

pub trait AppAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn detect(&self) -> Vec<PathBuf>;
    fn plan(&self, palette: &Palette) -> Result<Vec<WriteOp>>;
    fn reload(&self) -> Result<ReloadOutcome>;
}

pub fn apply_ops(
    journal: &mut crate::journal::Journal,
    ops: &[WriteOp],
) -> Result<()> {
    use std::fs;

    for op in ops {
        let backup = backup_path(&op.target);
        let temp = temp_path(&op.target);

        let entry = journal.record(
            op.adapter,
            &op.target,
            Some(&backup),
            Some(&temp),
        )?;

        if !backup.exists() && op.target.exists() {
            fs::copy(&op.target, &backup)?;
            tracing::info!(backup = %backup.display(), "backup created");
        }

        if let Some(parent) = op.target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&temp, &op.content)?;
        fs::rename(&temp, &op.target)?;

        journal.mark_done(entry)?;
        tracing::info!(
            adapter = op.adapter,
            path = %op.target.display(),
            "wrote"
        );
    }

    Ok(())
}

pub fn backup_path(target: &Path) -> PathBuf {
    let mut name = target.file_name().unwrap_or_default().to_os_string();
    name.push(".murf-bak");
    target.with_file_name(name)
}

pub fn temp_path(target: &Path) -> PathBuf {
    let mut name = target.file_name().unwrap_or_default().to_os_string();
    name.push(".murf.tmp");
    target.with_file_name(name)
}

/// Replace the block between `open` and `close` markers with `body`,
/// or append a new block if the markers are absent. The markers
/// themselves are preserved.
pub fn inject_section(original: &str, marker: &str, body: &str) -> String {
    let open = marker;
    let close = marker.replace("<murf>", "</murf>");

    if let Some(start_idx) = original.find(open) {
        if let Some(rel_end) = original[start_idx + open.len()..].find(&close) {
            let close_start = start_idx + open.len() + rel_end;
            let close_end = close_start + close.len();
            let before = &original[..start_idx];
            let after = &original[close_end..];
            return format!(
                "{}{}\n{}\n{}{}",
                before, open, body.trim_end(), close, after
            );
        }
    }

    let mut out = original.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(open);
    out.push('\n');
    out.push_str(body.trim_end());
    out.push('\n');
    out.push_str(&close);
    out.push('\n');
    out
}

/// Home directory of the current user.
pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(PathBuf::from)
}
