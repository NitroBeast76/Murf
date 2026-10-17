#![windows_subsystem = "windows"]

mod adapters;
mod autostart;
mod color;
mod config;
mod journal;
mod matugen;
mod palette;
mod registry;
mod watcher;

use adapters::{
    cava::Cava, chronoterm::Chronoterm, fastfetch::Fastfetch,
    glazewm::GlazeWm, starship::Starship,
    windows_accent::WindowsAccent, windows_theme::WindowsTheme,
    windows_terminal::WindowsTerminal, yasb::Yasb, AppAdapter,
};
use anyhow::Result;
use color::AnsiMode;
use global_hotkey::{hotkey::{Code, HotKey, Modifiers}, GlobalHotKeyEvent, GlobalHotKeyManager};
use std::fs;
use std::path::PathBuf;
use std::os::windows::process::CommandExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
    Icon, TrayIconBuilder,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

enum UserEvent {
    Menu(MenuEvent),
    WallpaperChanged(String),
    AutostartDelayElapsed,
    Hotkey,
}

fn make_icon() -> Icon {
    let bytes = include_bytes!("../assets/icon.png");
    let img = image::load_from_memory(bytes).expect("failed to decode icon.png");
    
    // Tray icons are tiny. If we pass a 256x256 image to Windows, it uses
    // a fast, ugly downscaler. We resize it ourselves using a high-quality
    // filter (Lanczos3) to 32x32 first.
    let resized = image::imageops::resize(&img, 32, 32, image::imageops::FilterType::Lanczos3);
    
    let rgba = resized.into_raw();
    Icon::from_rgba(rgba, 32, 32).expect("valid icon")
}

fn ansi_mode_from_str(s: &str) -> AnsiMode {
    match s {
        "material" => AnsiMode::Material,
        _ => AnsiMode::Semantic,
    }
}

fn build_adapters(cfg: &config::Config) -> Vec<Box<dyn AppAdapter>> {
    let ansi_mode = ansi_mode_from_str(&cfg.palette.ansi_mapping);
    vec![
        Box::new(WindowsTerminal::new(ansi_mode)),
        Box::new(Chronoterm),
        Box::new(Cava),
        Box::new(Starship),
        Box::new(GlazeWm),
        Box::new(Yasb),
        Box::new(WindowsAccent),
        Box::new(WindowsTheme),
        Box::new(Fastfetch),
    ]
}

fn write_palette_dump(wallpaper: &str, palette: &palette::Palette, cfg: &config::Config) -> Result<PathBuf> {
    let Some(local) = std::env::var_os("LOCALAPPDATA") else { anyhow::bail!("LOCALAPPDATA not set"); };
    let dir = PathBuf::from(local).join("Murf");
    fs::create_dir_all(&dir)?;
    let path = dir.join("last-palette.json");

    let mut roles = serde_json::Map::new();
    for (k, v) in &palette.roles {
        roles.insert(k.clone(), serde_json::Value::String(v.hex.clone()));
    }

    let primary = palette.hex_or("primary", "#808080");
    let error   = palette.hex_or("error",   "#ff0000");
    let bg      = palette.hex_or("background", "#000000");
    let sem = color::semantic_ansi(&primary, &error, &bg, palette.is_dark);
    
    let doc = serde_json::json!({
        "wallpaper": wallpaper, "mode": cfg.palette.mode, "is_dark": palette.is_dark,
        "background": bg, "roles": roles,
        "ansi_semantic": {
            "red": sem.red, "yellow": sem.yellow, "green": sem.green,
            "cyan": sem.cyan, "blue": sem.blue, "purple": sem.purple,
        },
    });

    fs::write(&path, serde_json::to_string_pretty(&doc)?)?;
    Ok(path)
}

fn run_apply(wallpaper: &str, cfg: &config::Config) {
    tracing::info!(wallpaper = %wallpaper, "applying palette");

    let palette = match matugen::generate(wallpaper, &cfg.palette.mode, &cfg.palette.scheme_type, cfg.palette.contrast) {
        Ok(p) => p,
        Err(e) => { tracing::error!(error = %e, "matugen failed"); return; }
    };

    let _ = write_palette_dump(wallpaper, &palette, cfg);

    let adapters = build_adapters(cfg);
    let mut all_ops = Vec::new();

    for a in &adapters {
        let app_cfg = cfg.apps.get(a.id()).cloned().unwrap_or_default();
        if !app_cfg.enabled { continue; }
        if a.detect(&app_cfg).is_empty() { continue; }
        
        match a.plan(&palette, &app_cfg) {
            Ok(ops) => all_ops.extend(ops),
            Err(e) => tracing::error!(adapter = a.id(), error = %e, "planning failed"),
        }
    }

    if all_ops.is_empty() { return; }

    let mut j = match journal::Journal::begin() {
        Ok(j) => j,
        Err(e) => { tracing::error!(error = %e, "could not begin journal"); return; }
    };

    match adapters::apply_ops(&mut j, &all_ops) {
        Ok(()) => {
            if let Err(e) = j.commit() { tracing::error!(error = %e, "journal commit failed"); return; }
            for a in &adapters {
                let app_cfg = cfg.apps.get(a.id()).cloned().unwrap_or_default();
                if !app_cfg.enabled { continue; }
                if a.detect(&app_cfg).is_empty() { continue; }
                if let Err(e) = a.reload() {
                    tracing::error!(adapter = a.id(), error = %e, "reload failed");
                }
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "apply failed; recovering");
            let _ = journal::recover();
        }
    }
}

fn main() -> Result<()> {
    let mut cfg = config::load_or_default()?;
    let default_level = cfg.general.log_level.clone();
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| format!("murf={}", default_level).into())).init();

    tracing::info!(version = VERSION, "murf starting");
    let _ = journal::recover();
    autostart::sync_from_windows();
    let autostart_enabled = autostart::is_registered() && autostart::is_approved_by_windows();

    let paused = Arc::new(AtomicBool::new(false));
    let autostart_flag = std::env::args().any(|a| a == "--autostart");

    // Register Global Hotkey (Ctrl+Alt+M)
    let hotkey_manager = GlobalHotKeyManager::new().unwrap();
    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyM);
    let _ = hotkey_manager.register(hotkey);

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let menu_proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| { let _ = menu_proxy.send_event(UserEvent::Menu(event)); }));

    let hotkey_proxy = event_loop.create_proxy();
    GlobalHotKeyEvent::set_event_handler(Some(move |_event| {
        let _ = hotkey_proxy.send_event(UserEvent::Hotkey);
    }));

    let watcher = watcher::Watcher::start(cfg.general.settle_delay_ms)?;
    let watcher_proxy = event_loop.create_proxy();
    let rx = watcher.rx;
    std::thread::spawn(move || {
        for path in rx { let _ = watcher_proxy.send_event(UserEvent::WallpaperChanged(path)); }
    });

    if autostart_flag {
        let delay = cfg.general.startup_delay_secs;
        let proxy = event_loop.create_proxy();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(delay));
            let _ = proxy.send_event(UserEvent::AutostartDelayElapsed);
        });
    }

    let menu = Menu::new();
    let apply_now        = MenuItem::new("Apply now\tCtrl+Alt+M", true, None);
    let pause_item       = CheckMenuItem::new("Pause watching", true, false, None);
    
    // Adapters Submenu
    let adapters_menu = Submenu::new("Adapters", true);
    let mut adapter_items = Vec::new();
    let adapter_list = [
        ("windows_terminal", "Windows Terminal"),
        ("yasb", "YASB"),
        ("glazewm", "GlazeWM"),
        ("chronoterm", "Chronoterm"),
        ("cava", "Cava"),
        ("starship", "Starship"),
        ("fastfetch", "Fastfetch"),
        ("windows_accent", "Windows Accent"),
        ("windows_theme", "Windows Theme"),
    ];

    for (id, name) in adapter_list {
        let is_enabled = cfg.apps.get(id).map(|a| a.enabled).unwrap_or(true);
        let item = CheckMenuItem::new(name, true, is_enabled, None);
        adapters_menu.append(&item)?;
        adapter_items.push((id.to_string(), item));
    }

    let reload_config    = MenuItem::new("Reload Config", true, None);
    let open_config      = MenuItem::new("Open Config", true, None);
    let open_logs        = MenuItem::new("Open Logs", true, None);
    let autostart_item   = CheckMenuItem::new("Start at login", true, autostart_enabled, None);
    let about            = MenuItem::new(format!("Murf {}", VERSION), true, None);
    let quit             = MenuItem::new("Quit", true, None);

    let apply_id     = apply_now.id().clone();
    let pause_id     = pause_item.id().clone();
    let reload_id    = reload_config.id().clone();
    let cfg_id       = open_config.id().clone();
    let logs_id      = open_logs.id().clone();
    let autostart_id = autostart_item.id().clone();
    let about_id     = about.id().clone();
    let quit_id      = quit.id().clone();

    let sep1 = PredefinedMenuItem::separator();
    let sep2 = PredefinedMenuItem::separator();
    let sep3 = PredefinedMenuItem::separator();

    menu.append(&apply_now)?;
    menu.append(&pause_item)?;
    menu.append(&sep1)?;
    menu.append(&adapters_menu)?;
    menu.append(&reload_config)?;
    menu.append(&sep2)?;
    menu.append(&open_config)?;
    menu.append(&open_logs)?;
    menu.append(&autostart_item)?;
    menu.append(&about)?;
    menu.append(&sep3)?;
    menu.append(&quit)?;

    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Murf")
        .with_icon(make_icon())
        .build()?;

    tracing::info!("tray icon created");

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            tao::event::Event::UserEvent(UserEvent::Menu(e)) => {
                if e.id == apply_id {
                    if let Some(w) = matugen::current_wallpaper() { run_apply(&w, &cfg); }
                } else if e.id == pause_id {
                    let now = pause_item.is_checked();
                    paused.store(now, Ordering::SeqCst);
                } else if e.id == autostart_id {
                    let _ = autostart::toggle(autostart_item.is_checked());
                } else if e.id == reload_id {
                    match config::load_or_default() {
                        Ok(new_cfg) => { cfg = new_cfg; tracing::info!("config reloaded"); }
                        Err(err) => tracing::error!(error = %err, "failed to reload config"),
                    }
                } else if e.id == cfg_id {
                    let path = config::config_path();
                    let _ = std::process::Command::new("cmd").creation_flags(0x08000000).args(["/C", "start", "", &path.to_string_lossy()]).spawn();
                } else if e.id == logs_id {
                    let dir = std::env::var("LOCALAPPDATA").map(|d| PathBuf::from(d).join("Murf").join("logs")).unwrap_or_default();
                    let _ = std::process::Command::new("cmd").creation_flags(0x08000000).args(["/C", "start", "", &dir.to_string_lossy()]).spawn();
                } else if e.id == about_id {
                    registry::show_message_box("Murf", &format!("Murf {}\n\nConfig: {}", VERSION, config::config_path().display()));
                } else if e.id == quit_id {
                    *control_flow = ControlFlow::Exit;
                } else {
                    // Handle adapter toggles
                    for (id, item) in &adapter_items {
                        if e.id == *item.id() {
                            let enabled = {
                                let app_cfg = cfg.apps.entry(id.clone()).or_insert_with(config::AppCfg::default);
                                app_cfg.enabled = item.is_checked();
                                app_cfg.enabled
                            };
                            let _ = config::save_config(&cfg);
                            tracing::info!(adapter = id, enabled = enabled, "adapter toggled");
                            break;
                        }
                    }
                }
            }
            tao::event::Event::UserEvent(UserEvent::WallpaperChanged(path)) => {
                if !paused.load(Ordering::SeqCst) { run_apply(&path, &cfg); }
            }
            tao::event::Event::UserEvent(UserEvent::AutostartDelayElapsed) => {
                if let Some(w) = matugen::current_wallpaper() { run_apply(&w, &cfg); }
            }
            tao::event::Event::UserEvent(UserEvent::Hotkey) => {
                tracing::info!("hotkey pressed, forcing apply");
                if let Some(w) = matugen::current_wallpaper() { run_apply(&w, &cfg); }
            }
            _ => {}
        }
    });
}






