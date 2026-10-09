// Adapter trait and write operations.
//
// An adapter has two jobs:
//   1. plan() — pure. Given a palette, produce the writes that would
//      be performed. No side effects, no file writes, no journal.
//   2. reload() — after writes succeed, ask the app to pick them up.
//
// The apply pipeline takes the Vec<WriteOp>, journals each one, then
// executes. Adapters never write files themselves.

pub mod windows_terminal;

use crate::palette::Palette;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WriteOp {
    /// Adapter id, recorded in the journal.
    pub adapter: &'static str,
    /// The file to write.
    pub target: PathBuf,
    /// Full new content of the file.
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

    /// Return every file path the adapter considers a valid install.
    /// Empty when the app is not installed.
    fn detect(&self) -> Vec<PathBuf>;

    /// Pure. Reads files to compute new content, but never writes.
    fn plan(&self, palette: &Palette) -> Result<Vec<WriteOp>>;

    /// Called after all writes for this adapter have succeeded.
    fn reload(&self) -> Result<ReloadOutcome>;
}

/// Apply a batch of WriteOps under a journal. Each op is journaled
/// before its write, marked done after, and the journal is committed
/// (and deleted) once all succeed.
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

        // Backup once, never overwritten.
        if !backup.exists() && op.target.exists() {
            fs::copy(&op.target, &backup)?;
            tracing::info!(backup = %backup.display(), "backup created");
        }

        // Stage to temp in same directory.
        fs::write(&temp, &op.content)?;

        // Rename over target.
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

pub fn backup_path(target: &std::path::Path) -> PathBuf {
    let mut name = target.file_name().unwrap_or_default().to_os_string();
    name.push(".murf-bak");
    target.with_file_name(name)
}

pub fn temp_path(target: &std::path::Path) -> PathBuf {
    let mut name = target.file_name().unwrap_or_default().to_os_string();
    name.push(".murf.tmp");
    target.with_file_name(name)
}
