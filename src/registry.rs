// Registry reads/writes via the `windows` crate.

use crate::adapters::{RegistryData, RegistryHive};
use anyhow::{anyhow, Result};
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_BINARY, REG_DWORD,
    REG_VALUE_TYPE, RegCloseKey, RegOpenKeyExW, RegQueryValueExW,
    RegSetValueExW,
};

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn hive_handle(hive: RegistryHive) -> HKEY {
    match hive {
        RegistryHive::CurrentUser => HKEY_CURRENT_USER,
    }
}

pub fn write(
    hive: RegistryHive,
    path: &str,
    value: &str,
    data: &RegistryData,
) -> Result<()> {
    let path_w = to_wide(path);
    let value_w = to_wide(value);

    unsafe {
        let mut key = HKEY::default();
        let status = RegOpenKeyExW(
            hive_handle(hive),
            PCWSTR(path_w.as_ptr()),
            0,
            KEY_WRITE,
            &mut key,
        );
        if status.is_err() {
            return Err(anyhow!("RegOpenKeyExW failed for {}", path));
        }

        let (reg_type, bytes): (REG_VALUE_TYPE, Vec<u8>) = match data {
            RegistryData::Dword(v) => (REG_DWORD, v.to_le_bytes().to_vec()),
            RegistryData::Binary(v) => (REG_BINARY, v.clone()),
        };

        let status = RegSetValueExW(
            key,
            PCWSTR(value_w.as_ptr()),
            0,
            reg_type,
            Some(&bytes),
        );
        let _ = RegCloseKey(key);
        if status.is_err() {
            return Err(anyhow!("RegSetValueExW failed for {}", value));
        }
    }
    Ok(())
}

pub fn read(
    hive: RegistryHive,
    path: &str,
    value: &str,
) -> Result<Option<RegistryData>> {
    let path_w = to_wide(path);
    let value_w = to_wide(value);

    unsafe {
        let mut key = HKEY::default();
        let status = RegOpenKeyExW(
            hive_handle(hive),
            PCWSTR(path_w.as_ptr()),
            0,
            KEY_READ,
            &mut key,
        );
        if status.is_err() {
            return Ok(None);
        }

        let mut ty = REG_VALUE_TYPE::default();
        let mut size = 0u32;

        let status = RegQueryValueExW(
            key,
            PCWSTR(value_w.as_ptr()),
            None,
            Some(&mut ty),
            None,
            Some(&mut size),
        );
        if status.is_err() {
            let _ = RegCloseKey(key);
            return Ok(None);
        }

        let mut buf = vec![0u8; size as usize];
        let status = RegQueryValueExW(
            key,
            PCWSTR(value_w.as_ptr()),
            None,
            Some(&mut ty),
            Some(buf.as_mut_ptr()),
            Some(&mut size),
        );
        let _ = RegCloseKey(key);

        if status.is_err() {
            return Ok(None);
        }

        if ty == REG_DWORD {
            if buf.len() >= 4 {
                let v = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                Ok(Some(RegistryData::Dword(v)))
            } else {
                Ok(None)
            }
        } else if ty == REG_BINARY {
            Ok(Some(RegistryData::Binary(buf)))
        } else {
            Ok(None)
        }
    }
}

pub fn delete(hive: RegistryHive, path: &str, value: &str) -> Result<()> {
    use windows::Win32::System::Registry::RegDeleteValueW;

    let path_w = to_wide(path);
    let value_w = to_wide(value);

    unsafe {
        let mut key = HKEY::default();
        let status = RegOpenKeyExW(
            hive_handle(hive),
            PCWSTR(path_w.as_ptr()),
            0,
            KEY_WRITE,
            &mut key,
        );
        if status.is_err() { return Ok(()); }
        let _ = RegDeleteValueW(key, PCWSTR(value_w.as_ptr()));
        let _ = RegCloseKey(key);
    }
    Ok(())
}

/// Broadcast WM_SETTINGCHANGE with "ImmersiveColorSet" so Explorer and
/// other listening processes pick up the new accent.
pub fn broadcast_setting_change() -> Result<()> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };

    let param = to_wide("ImmersiveColorSet");

    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(param.as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            1000,
            None,
        );
    }
    Ok(())
}

pub fn show_message_box(title: &str, text: &str) {
    use windows::Win32::UI::WindowsAndMessaging::{
        MessageBoxW, MB_ICONINFORMATION, MB_OK,
    };
    let t = to_wide(title);
    let b = to_wide(text);
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(b.as_ptr()),
            PCWSTR(t.as_ptr()),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}
