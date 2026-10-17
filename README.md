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