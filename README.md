# Murf

**Wallpaper-driven theming for Windows.** 

Murf watches your desktop, extracts a Material You color palette using [matugen](https://github.com/InioX/matugen), and seamlessly injects those colors into your terminal, text editor, status bar, window manager, and system UI. When you change your wallpaper, your entire desktop morphs to match.

## Features
- **Instant Morphing:** Automatically detects wallpaper changes and applies the new palette in milliseconds.
- **Non-Destructive:** Uses a transactional journal. If an apply fails, Murf rolls back the files to their exact previous state.
- **Zero Config:** Works out of the box with standard installation paths for popular Windows ricing tools.
- **Live Reloads:** Automatically refreshes apps like YASB, GlazeWM, and Windows Terminal without breaking your layout.
- **Global Hotkey:** Press `Ctrl+Alt+M` anywhere to force a manual re-apply.

## Supported Applications
Murf currently themes the following apps (and will automatically skip any you don't have installed):
- **Windows Terminal**
- **GlazeWM**
- **YASB** (Yet Another Status Bar)
- **Starship** prompt
- **Fastfetch**
- **Windows 11 Accent Color** (Title bars and UI elements)
- **Windows 11 Theme** (Light/Dark mode toggle)
- **Chronoterm**
- **Cava**

*(Note: CLI apps like Cava and Chronoterm cannot live-reload safely. Murf updates their config files instantly, and they will morph the next time you launch them.)*

## Installation
1. Download `murf.exe` from the Releases page.
2. Place it in a permanent folder (e.g., `C:\Users\YourName\Tools\`).
3. Double-click to run. It will appear in your system tray.
4. Right-click the tray icon and select **Start at login**.

## Configuration
Murf generates a heavily commented configuration file on first run at:
`%APPDATA%\Murf\murf.toml`

### Custom Paths
By default, Murf looks in standard locations (e.g., `~/.config/yasb`, `%LOCALAPPDATA%\Packages\...`). If you use portable apps or custom dotfile directories, you can point Murf directly to them:

```toml
[apps.yasb]
enabled = true
config_dir = "D:\\Dotfiles\\yasb"

[apps.glazewm]
enabled = true
config_path = "D:\\Dotfiles\\glazewm\\config.yaml"
```

### Tuning the Morph Speed
The `settle_delay_ms` value controls how long Murf waits after you change the wallpaper before applying the theme.
```toml
[general]
# 500ms is snappy. Increase to 2000ms if you have slow-loading wallpapers.
settle_delay_ms = 500 
```

### Custom Fastfetch ASCII Art
If you want to use your own ASCII logo instead of the default Windows logo, save your art as a `.txt` file and point to it in your `murf.toml`:

```toml
[apps.fastfetch]
enabled = true
logo_source = "C:\\Users\\YourName\\my-ascii-art.txt"
```
*(Make sure to use double backslashes `\\` in the TOML file).*

## Example Configs
Setting up Murf is designed to be copy-paste friendly. Check the `examples/` folder for ready-to-use blocks:

- **PowerShell Profile:** Copy the contents of `examples/profile.ps1` into your `$PROFILE` to ensure Fastfetch and Starship use Murf's generated configs.
- **YASB:** Copy the `:root` block from `examples/yasb-styles.css` into your `styles.css`. This replaces hardcoded colors with dynamic variables.
- **GlazeWM:** Copy the `window_effects` block from `examples/glazewm-config.yaml` into your `config.yaml` (notice the `*murf_primary` anchor). Murf injects the anchor definitions automatically at the top of the file.

## Usage
- **Tray Menu:** Right-click the Murf icon to Apply Now, Pause Watching, or toggle specific Adapters on the fly.
- **Hotkey:** `Ctrl+Alt+M` forces an immediate theme update.
- **Logs:** If something goes wrong, check `%LOCALAPPDATA%\Murf\logs\murf.log`.

## Uninstallation
1. Right-click the tray icon and select **Quit**.
2. Delete `murf.exe`.
3. (Optional) Delete `%APPDATA%\Murf` to remove your configuration, and `%LOCALAPPDATA%\Murf` to remove logs.
4. Any apps themed by Murf will simply stop updating. You can remove the `Murf` blocks from their configs manually if you wish to restore their original colors.

---
*Built with Rust. Powered by matugen and Material You.*
