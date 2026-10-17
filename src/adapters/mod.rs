// Adapter trait and write operations.

pub mod cava;
pub mod chronoterm;
pub mod fastfetch;
pub mod glazewm;
pub mod starship;
pub mod windows_accent;
pub mod windows_terminal;
pub mod yasb;

use crate::palette::Palette;
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum WriteOp {
    File {
        adapter: &'static str,
        target: PathBuf,
        content: String,
    },
    Registry {
        adapter: &'static str,
        hive: RegistryHive,
        path: String,
        value: String,
        data: RegistryData,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryHive { CurrentUser }

#[derive(Debug, Clone)]
pub enum RegistryData {
    Dword(u32),
    Binary(Vec<u8>),
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ReloadOutcome {
    pub live_reload: usize,
    pub restarted: usize,
    pub deferred: usize,
    pub skipped: usize,
}

pub trait AppAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    #[allow(dead_code)]
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
        match op {
            WriteOp::File { adapter, target, content } => {
                let backup = backup_path(target);
                let temp = temp_path(target);

                let entry = journal.record_file(
                    adapter,
                    target,
                    Some(&backup),
                    Some(&temp),
                )?;

                if !backup.exists() && target.exists() {
                    fs::copy(target, &backup)?;
                    tracing::info!(backup = %backup.display(), "backup created");
                }

                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&temp, content)?;
                fs::rename(&temp, target)?;

                journal.mark_done(entry)?;
                tracing::info!(
                    adapter = adapter,
                    path = %target.display(),
                    "wrote file"
                );
            }
            WriteOp::Registry { adapter, hive, path, value, data } => {
                let entry = journal.record_registry(
                    adapter,
                    *hive,
                    path,
                    value,
                    data,
                )?;

                crate::registry::write(*hive, path, value, data)?;

                journal.mark_done(entry)?;
                tracing::info!(
                    adapter = adapter,
                    path = %path,
                    value = %value,
                    "wrote registry"
                );
            }
        }
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

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(PathBuf::from)
}

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

pub fn inject_at_top(
    original: &str,
    marker_open: &str,
    marker_close: &str,
    body: &str,
) -> String {
    // If the markers exist elsewhere, extract the block and reinsert
    // it at the top.
    if let Some(start) = original.find(marker_open) {
        if let Some(rel_end) = original[start + marker_open.len()..].find(marker_close) {
            let close_end = start + marker_open.len() + rel_end + marker_close.len();
            let mut after = close_end;
            if original[after..].starts_with('\n') { after += 1; }

            let _before = &original[..start];
            let rest  = &original[after..];

            // Strip leading blank lines from `rest`.
            let rest_trimmed = rest.trim_start_matches('\n');

            return format!(
                "{}\n{}\n{}\n{}",
                marker_open,
                body.trim_end(),
                marker_close,
                rest_trimmed
            );
        }
    }

    let bom = "\u{FEFF}";
    let (prefix, rest) = if original.starts_with(bom) {
        (bom, &original[bom.len()..])
    } else {
        ("", original)
    };

    format!(
        "{}{}\n{}\n{}\n{}",
        prefix,
        marker_open,
        body.trim_end(),
        marker_close,
        rest
    )
}

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

pub fn remove_block(original: &str, marker_open: &str, marker_close: &str) -> String {
    if let Some(start) = original.find(marker_open) {
        if let Some(rel_end) = original[start + marker_open.len()..].find(marker_close) {
            let close_end = start + marker_open.len() + rel_end + marker_close.len();
            let mut end = close_end;
            if original[end..].starts_with('\n') { end += 1; }
            if original[end..].starts_with('\n') { end += 1; }
            let mut out = String::new();
            out.push_str(&original[..start]);
            out.push_str(&original[end..]);
            return out;
        }
    }
    original.to_string()
}




/// Check if a process is currently running using tasklist.
pub fn is_process_running(exe_name: &str) -> bool {
    use std::process::Command;
    let output = Command::new("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {}", exe_name), "/NH"])
        .output();
        
    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
            text.contains(&exe_name.to_lowercase())
        }
        Err(_) => false,
    }
}

/// Kill a process by exe name and immediately relaunch it,
/// but ONLY if it was already running.
pub fn restart_process(exe_name: &str, args: &[&str]) -> Result<ReloadOutcome> {
    use std::process::Command;
    
    if !is_process_running(exe_name) {
        tracing::debug!(exe = exe_name, "not running, skipping restart");
        return Ok(ReloadOutcome { skipped: 1, ..Default::default() });
    }

    // 1. Kill
    let _ = Command::new("taskkill")
        .args(["/IM", exe_name, "/F"])
        .output();
        
    // 2. Wait a moment for the OS to release file locks
    std::thread::sleep(std::time::Duration::from_millis(200));

    // 3. Relaunch
    let mut cmd = Command::new("cmd");
    cmd.args(["/C", "start", ""]);
    cmd.arg(exe_name);
    cmd.args(args);
    
    match cmd.spawn() {
        Ok(_) => {
            tracing::info!(exe = exe_name, "restarted process");
            Ok(ReloadOutcome { restarted: 1, ..Default::default() })
        }
        Err(e) => {
            tracing::warn!(exe = exe_name, error = %e, "failed to restart");
            Err(anyhow::anyhow!("failed to restart {}", exe_name))
        }
    }
}

