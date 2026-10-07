// Wallpaper watcher. Polls the registry every 3 seconds, debounces,
// emits on a channel when the wallpaper has settled.

use anyhow::{Context, Result};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

pub struct Watcher {
    pub rx: Receiver<String>,
    _handle: thread::JoinHandle<()>,
}

impl Watcher {
    /// Start watching. Returns immediately.
    ///
    /// `settle_delay_secs` — after the wallpaper path changes, wait
    /// this long for further changes before emitting.
    pub fn start(settle_delay_secs: u64) -> Result<Self> {
        let (tx, rx) = channel();
        let handle = thread::spawn(move || {
            if let Err(e) = watch_loop(tx, settle_delay_secs) {
                tracing::error!(error = %e, "watcher loop exited");
            }
        });
        Ok(Watcher { rx, _handle: handle })
    }
}

fn watch_loop(tx: Sender<String>, settle_delay_secs: u64) -> Result<()> {
    let poll_interval = Duration::from_secs(3);
    let settle = Duration::from_secs(settle_delay_secs);

    let mut last_seen: Option<String> = read_wallpaper();
    let mut last_emitted: Option<String> = None;
    let mut pending: Option<(String, Instant)> = None;

    tracing::info!(
        current = last_seen.as_deref().unwrap_or("<none>"),
        "watcher started"
    );

    loop {
        thread::sleep(poll_interval);

        let current = read_wallpaper();

        // Detect a change in the wallpaper path.
        if current != last_seen {
            if let Some(path) = current.clone() {
                tracing::debug!(path = %path, "wallpaper path changed");
                pending = Some((path, Instant::now()));
            }
            last_seen = current;
        }

        // If we have a pending change and the settle window has elapsed,
        // emit it.
        if let Some((path, since)) = pending.clone() {
            if since.elapsed() >= settle {
                if Some(&path) != last_emitted.as_ref() {
                    tracing::info!(path = %path, "wallpaper settled");
                    let _ = tx.send(path.clone());
                    last_emitted = Some(path);
                }
                pending = None;
            }
        }
    }
}

/// Read the current wallpaper path from HKCU via reg.exe.
fn read_wallpaper() -> Option<String> {
    let out = std::process::Command::new("reg")
        .args([
            "query",
            "HKCU\\Control Panel\\Desktop",
            "/v",
            "Wallpaper",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if line.contains("Wallpaper") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(last) = parts.last() {
                return Some(last.to_string());
            }
        }
    }
    None
}
