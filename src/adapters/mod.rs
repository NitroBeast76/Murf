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

/// Home directory of the current user.
pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(PathBuf::from)
}

/// Replace an existing `[table]` section with `body`, wrapped in
/// marker lines. If the table is absent, append a marked block at
/// the end of the file.
///
/// A section ends at the next line whose trimmed form is a bracketed
/// header (`[name]` alone on the line) or at EOF.
pub fn replace_table(
    original: &str,
    table_name: &str,
    marker_open: &str,
    marker_close: &str,
    body: &str,
) -> String {
    let header = format!("[{}]", table_name);
    let lines: Vec<&str> = original.lines().collect();

    let is_header = |s: &str| -> bool {
        let t = s.trim();
        t.starts_with('[') && t.ends_with(']') && t.len() > 2
    };

    let header_idx = lines.iter().position(|l| l.trim() == header);

    if let Some(idx) = header_idx {
        let end_idx = lines
            .iter()
            .enumerate()
            .skip(idx + 1)
            .find(|(_, l)| is_header(l))
            .map(|(i, _)| i)
            .unwrap_or(lines.len());

        let mut out = String::new();
        for l in &lines[..idx] {
            out.push_str(l);
            out.push('\n');
        }
        out.push_str(marker_open);
        out.push('\n');
        out.push_str(body.trim_end());
        out.push('\n');
        out.push_str(marker_close);
        out.push('\n');
        for l in &lines[end_idx..] {
            out.push_str(l);
            out.push('\n');
        }
        if !original.ends_with('\n') && out.ends_with('\n') {
            out.pop();
        }
        out
    } else {
        let mut out = original.to_string();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(marker_open);
        out.push('\n');
        out.push_str(body.trim_end());
        out.push('\n');
        out.push_str(marker_close);
        out.push('\n');
        out
    }
}

/// Remove a marker block delimited by `marker_open` and `marker_close`,
/// including one trailing newline after the close marker.
pub fn remove_block(original: &str, marker_open: &str, marker_close: &str) -> String {
    if let Some(start) = original.find(marker_open) {
        if let Some(rel_end) = original[start + marker_open.len()..].find(marker_close) {
            let close_end = start + marker_open.len() + rel_end + marker_close.len();
            let mut end = close_end;
            if original[end..].starts_with('\n') {
                end += 1;
            }
            if original[end..].starts_with('\n') {
                end += 1;
            }
            let mut out = String::new();
            out.push_str(&original[..start]);
            out.push_str(&original[end..]);
            return out;
        }
    }
    original.to_string()
}
