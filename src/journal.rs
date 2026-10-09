// Apply journal.
//
// Before any write, an entry is recorded. Entries are updated to
// "done" as each write completes, with FlushFileBuffers after each
// update so the journal survives a crash. On successful completion
// the journal is marked committed and deleted.
//
// On startup, if the journal exists and is not committed, Murf
// restores every "done" entry from its backup and deletes the
// journal. If restore fails repeatedly, give_up is set and Murf
// stops retrying.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub adapter: String,
    pub kind: String, // "file"
    pub target: PathBuf,
    pub backup: Option<PathBuf>,
    pub temp: Option<PathBuf>,
    pub status: String, // "pending" | "done"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalDoc {
    pub started_at: String,
    pub committed: bool,
    pub give_up: bool,
    pub rollback_attempts: u32,
    pub entries: Vec<Entry>,
}

pub struct Journal {
    path: PathBuf,
    doc: JournalDoc,
}

pub fn journal_path() -> PathBuf {
    let appdata = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    appdata.join("Murf").join("apply.journal")
}

impl Journal {
    /// Create a fresh journal. Fails if one already exists
    /// (indicates an unrecovered prior apply).
    pub fn begin() -> Result<Self> {
        let path = journal_path();
        if path.exists() {
            anyhow::bail!(
                "journal already exists at {}; run recovery first",
                path.display()
            );
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let doc = JournalDoc {
            started_at: format!("{:?}", std::time::SystemTime::now()),
            committed: false,
            give_up: false,
            rollback_attempts: 0,
            entries: Vec::new(),
        };
        let mut j = Journal { path, doc };
        j.flush()?;
        Ok(j)
    }

    pub fn record(
        &mut self,
        adapter: &str,
        target: &Path,
        backup: Option<&Path>,
        temp: Option<&Path>,
    ) -> Result<usize> {
        self.doc.entries.push(Entry {
            adapter: adapter.to_string(),
            kind: "file".to_string(),
            target: target.to_path_buf(),
            backup: backup.map(|p| p.to_path_buf()),
            temp: temp.map(|p| p.to_path_buf()),
            status: "pending".to_string(),
        });
        self.flush()?;
        Ok(self.doc.entries.len() - 1)
    }

    pub fn mark_done(&mut self, index: usize) -> Result<()> {
        if let Some(e) = self.doc.entries.get_mut(index) {
            e.status = "done".to_string();
        }
        self.flush()?;
        Ok(())
    }

    pub fn commit(mut self) -> Result<()> {
        self.doc.committed = true;
        self.flush()?;
        // Committed flag is now on disk. Safe to delete.
        let _ = fs::remove_file(&self.path);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        let text = serde_json::to_string_pretty(&self.doc)?;
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .with_context(|| format!("opening {}", self.path.display()))?;
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum RecoveryResult {
    NoJournal,
    AlreadyCommitted,
    RolledBack { restored: usize, failed: usize },
    GaveUp { attempts: u32 },
}

/// Run at startup. If a journal is present, restore from backups.
pub fn recover() -> Result<RecoveryResult> {
    let path = journal_path();
    if !path.exists() {
        return Ok(RecoveryResult::NoJournal);
    }

    let text = fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let mut doc: JournalDoc = serde_json::from_str(&text)
        .with_context(|| format!("parsing {}", path.display()))?;

    if doc.committed {
        let _ = fs::remove_file(&path);
        return Ok(RecoveryResult::AlreadyCommitted);
    }

    if doc.give_up {
        return Ok(RecoveryResult::GaveUp {
            attempts: doc.rollback_attempts,
        });
    }

    // Roll back every "done" entry, in reverse.
    let mut restored = 0usize;
    let mut failed = 0usize;

    for e in doc.entries.iter().rev() {
        if e.status != "done" {
            continue;
        }
        let Some(backup) = &e.backup else {
            continue;
        };
        match fs::copy(backup, &e.target) {
            Ok(_) => {
                tracing::info!(
                    target = %e.target.display(),
                    "restored from backup"
                );
                restored += 1;
            }
            Err(err) => {
                tracing::error!(
                    target = %e.target.display(),
                    error = %err,
                    "restore failed"
                );
                failed += 1;
            }
        }
    }

    // Clean up temps.
    for e in &doc.entries {
        if let Some(t) = &e.temp {
            let _ = fs::remove_file(t);
        }
    }

    if failed > 0 {
        doc.rollback_attempts += 1;
        if doc.rollback_attempts >= 3 {
            doc.give_up = true;
        }
        let text = serde_json::to_string_pretty(&doc)?;
        fs::write(&path, text)?;
        if doc.give_up {
            return Ok(RecoveryResult::GaveUp {
                attempts: doc.rollback_attempts,
            });
        }
        anyhow::bail!("rollback failed for {failed} entries; will retry on next startup");
    }

    let _ = fs::remove_file(&path);
    Ok(RecoveryResult::RolledBack { restored, failed })
}
