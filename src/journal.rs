// Apply journal.

use crate::adapters::{RegistryData, RegistryHive};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Entry {
    #[serde(rename = "file")]
    File {
        adapter: String,
        target: PathBuf,
        backup: Option<PathBuf>,
        temp: Option<PathBuf>,
        status: String,
    },
    #[serde(rename = "registry")]
    Registry {
        adapter: String,
        hive: String,
        path: String,
        value: String,
        original: Option<RegistryBackup>,
        status: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryBackup {
    Dword(u32),
    Binary(Vec<u8>),
    String(String),
    Absent,
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

    pub fn record_file(
        &mut self,
        adapter: &str,
        target: &Path,
        backup: Option<&Path>,
        temp: Option<&Path>,
    ) -> Result<usize> {
        self.doc.entries.push(Entry::File {
            adapter: adapter.to_string(),
            target: target.to_path_buf(),
            backup: backup.map(|p| p.to_path_buf()),
            temp: temp.map(|p| p.to_path_buf()),
            status: "pending".to_string(),
        });
        self.flush()?;
        Ok(self.doc.entries.len() - 1)
    }

    pub fn record_registry(
        &mut self,
        adapter: &str,
        hive: RegistryHive,
        path: &str,
        value: &str,
        _new_data: &RegistryData,
    ) -> Result<usize> {
        let existing = crate::registry::read(hive, path, value).ok().flatten();
        let original = match existing {
            Some(RegistryData::Dword(v)) => Some(RegistryBackup::Dword(v)),
            Some(RegistryData::Binary(v)) => Some(RegistryBackup::Binary(v)),
            Some(RegistryData::String(v)) => Some(RegistryBackup::String(v)),
            None => Some(RegistryBackup::Absent),
        };

        let hive_str = match hive {
            RegistryHive::CurrentUser => "HKCU",
        };

        self.doc.entries.push(Entry::Registry {
            adapter: adapter.to_string(),
            hive: hive_str.to_string(),
            path: path.to_string(),
            value: value.to_string(),
            original,
            status: "pending".to_string(),
        });
        self.flush()?;
        Ok(self.doc.entries.len() - 1)
    }

    pub fn mark_done(&mut self, index: usize) -> Result<()> {
        match self.doc.entries.get_mut(index) {
            Some(Entry::File { status, .. }) => { *status = "done".into(); }
            Some(Entry::Registry { status, .. }) => { *status = "done".into(); }
            None => {}
        }
        self.flush()?;
        Ok(())
    }

    pub fn commit(mut self) -> Result<()> {
        self.doc.committed = true;
        self.flush()?;
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
#[allow(dead_code)]
pub enum RecoveryResult {
    NoJournal,
    AlreadyCommitted,
    RolledBack { restored: usize, failed: usize },
    GaveUp { attempts: u32 },
}

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

    let mut restored = 0usize;
    let mut failed = 0usize;

    for e in doc.entries.iter().rev() {
        match e {
            Entry::File { status, target, backup, temp, .. } => {
                if status == "done" {
                    if let Some(b) = backup {
                        match fs::copy(b, target) {
                            Ok(_) => restored += 1,
                            Err(err) => {
                                tracing::error!(target = %target.display(), error = %err, "restore failed");
                                failed += 1;
                            }
                        }
                    }
                }
                if let Some(t) = temp {
                    let _ = fs::remove_file(t);
                }
            }
            Entry::Registry { status, hive, path: rpath, value, original, .. } => {
                if status != "done" { continue; }
                let hive_enum = match hive.as_str() {
                    "HKCU" => RegistryHive::CurrentUser,
                    _ => continue,
                };
                let res = match original {
                    Some(RegistryBackup::Dword(v)) => {
                        crate::registry::write(hive_enum, rpath, value, &RegistryData::Dword(*v))
                    }
                    Some(RegistryBackup::Binary(v)) => {
                        crate::registry::write(hive_enum, rpath, value, &RegistryData::Binary(v.clone()))
                    }
                    Some(RegistryBackup::String(v)) => {
                        crate::registry::write(hive_enum, rpath, value, &RegistryData::String(v.clone()))
                    }
                    Some(RegistryBackup::Absent) => {
                        crate::registry::delete(hive_enum, rpath, value)
                    }
                    None => Ok(()),
                };
                match res {
                    Ok(_) => restored += 1,
                    Err(err) => {
                        tracing::error!(path = %rpath, value = %value, error = %err, "registry restore failed");
                        failed += 1;
                    }
                }
            }
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


