// Autostart: HKCU\...\Run entry with --autostart flag, plus respect
// for the StartupApproved bit set by Task Manager.

use crate::adapters::{RegistryData, RegistryHive};
use anyhow::Result;
use std::path::PathBuf;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
const VALUE_NAME: &str = "Murf";

fn current_exe() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

pub fn is_registered() -> bool {
    match crate::registry::read(RegistryHive::CurrentUser, RUN_KEY, VALUE_NAME) {
        Ok(Some(RegistryData::String(_))) => true,
        _ => false,
    }
}

pub fn is_approved_by_windows() -> bool {
    match crate::registry::read(RegistryHive::CurrentUser, APPROVED_KEY, VALUE_NAME) {
        Ok(Some(RegistryData::Binary(b))) if !b.is_empty() => (b[0] & 0x01) == 0,
        _ => true,
    }
}

pub fn register() -> Result<()> {
    let Some(exe) = current_exe() else {
        anyhow::bail!("could not resolve current exe path");
    };
    let value = format!("\"{}\" --autostart", exe.display());

    crate::registry::write(
        RegistryHive::CurrentUser,
        RUN_KEY,
        VALUE_NAME,
        &RegistryData::String(value),
    )?;

    let approved = vec![0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let _ = crate::registry::write(
        RegistryHive::CurrentUser,
        APPROVED_KEY,
        VALUE_NAME,
        &RegistryData::Binary(approved),
    );

    Ok(())
}

pub fn unregister() -> Result<()> {
    crate::registry::delete(RegistryHive::CurrentUser, RUN_KEY, VALUE_NAME)?;
    Ok(())
}

pub fn toggle(enabled: bool) -> Result<()> {
    if enabled { register() } else { unregister() }
}

pub fn sync_from_windows() {
    if is_registered() && !is_approved_by_windows() {
        tracing::info!("autostart disabled in Task Manager; leaving it alone");
    }
}
