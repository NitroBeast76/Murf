// Murf entry point. v0.2 step 4: journal + recovery.

mod adapters;
mod color;
mod config;
mod journal;
mod matugen;
mod palette;
mod watcher;

use anyhow::Result;
use color::AnsiMode;
use std::fs;
use std::path::PathBuf;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIconBuilder,
};

enum UserEvent {
    Menu(MenuEvent),
    WallpaperChanged(String),
}

fn make_icon() -> Icon {
    let size: u32 = 32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;
    let outer_r: f32 = 15.0;
    let inner_r: f32 = 7.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - cx + 0.5;
            let dy = y as f32 - cy + 0.5;
            let d = (dx * dx + dy * dy).sqrt();
            let i = ((y * size + x) * 4) as usize;

            if d <= inner_r {
                rgba[i]     = 0x4a;
                rgba[i + 1] = 0x5f;
                rgba[i + 2] = 0xa8;
                rgba[i + 3] = 0xff;
            } else if d <= outer_r {
                rgba[i]     = 0xe8;
                rgba[i + 1] = 0xe8;
                rgba[i + 2] = 0xe8;
                rgba[i + 3] = 0xff;
            }
        }
    }

    Icon::from_rgba(rgba, size, size).expect("valid icon")
}

fn ansi_mode_from_str(s: &str) -> AnsiMode {
    match s {
        "material" => AnsiMode::Material,
        _ => AnsiMode::Semantic,
    }
}

fn write_palette_dump(
    wallpaper: &str,
    palette: &palette::Palette,
    cfg: &config::Config,
) -> Result<PathBuf> {
    let Some(local) = std::env::var_os("LOCALAPPDATA") else {
        anyhow::bail!("LOCALAPPDATA not set");
    };
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

    let ratio = |hex: &str| -> f64 { color::contrast_ratio(hex, &bg).unwrap_or(0.0) as f64 };

    let doc = serde_json::json!({
        "wallpaper": wallpaper,
        "mode": cfg.palette.mode,
        "scheme_type": cfg.palette.scheme_type,
        "contrast": cfg.palette.contrast,
        "ansi_mapping": cfg.palette.ansi_mapping,
        "is_dark": palette.is_dark,
        "background": bg,
        "roles": roles,
        "ansi_semantic": {
            "red": sem.red, "yellow": sem.yellow, "green": sem.green,
            "cyan": sem.cyan, "blue": sem.blue, "purple": sem.purple,
            "brightRed": sem.bright_red, "brightYellow": sem.bright_yellow,
            "brightGreen": sem.bright_green, "brightCyan": sem.bright_cyan,
            "brightBlue": sem.bright_blue, "brightPurple": sem.bright_purple,
        },
        "ansi_contrast": {
            "red": ratio(&sem.red), "yellow": ratio(&sem.yellow),
            "green": ratio(&sem.green), "cyan": ratio(&sem.cyan),
            "blue": ratio(&sem.blue), "purple": ratio(&sem.purple),
            "brightRed": ratio(&sem.bright_red),
            "brightYellow": ratio(&sem.bright_yellow),
            "brightGreen": ratio(&sem.bright_green),
            "brightCyan": ratio(&sem.bright_cyan),
            "brightBlue": ratio(&sem.bright_blue),
            "brightPurple": ratio(&sem.bright_purple),
        },
    });

    fs::write(&path, serde_json::to_string_pretty(&doc)?)?;
    Ok(path)
}

fn run_apply(wallpaper: &str, cfg: &config::Config) {
    tracing::info!(wallpaper = %wallpaper, "applying palette");

    let palette = match matugen::generate(
        wallpaper,
        &cfg.palette.mode,
        &cfg.palette.scheme_type,
        cfg.palette.contrast,
    ) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "matugen failed");
            return;
        }
    };

    tracing::info!(
        roles = palette.roles.len(),
        is_dark = palette.is_dark,
        "palette parsed"
    );

    // Palette dump is Murf's own state; not journaled.
    match write_palette_dump(wallpaper, &palette, cfg) {
        Ok(path) => tracing::info!(path = %path.display(), "palette dump written"),
        Err(e) => tracing::warn!(error = %e, "failed to write palette dump"),
    }

    let ansi_mode = ansi_mode_from_str(&cfg.palette.ansi_mapping);

    // Adapter writes are journaled.
    let mut j = match journal::Journal::begin() {
        Ok(j) => j,
        Err(e) => {
            tracing::error!(error = %e, "could not begin journal");
            return;
        }
    };

    match adapters::windows_terminal::apply(&palette, ansi_mode, &mut j) {
        Ok(paths) if paths.is_empty() => {
            tracing::warn!("no Windows Terminal config found");
            let _ = j.commit();
        }
        Ok(paths) => {
            tracing::info!(count = paths.len(), "adapter applied");
            if let Err(e) = j.commit() {
                tracing::error!(error = %e, "journal commit failed");
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "adapter failed; rolling back");
            // Journal is left on disk. Next startup will roll back.
            // For immediate feedback, attempt recovery now.
            match journal::recover() {
                Ok(journal::RecoveryResult::RolledBack { restored, failed }) => {
                    tracing::warn!(restored, failed, "rolled back after adapter failure");
                }
                Ok(other) => tracing::warn!(?other, "recovery result"),
                Err(e) => tracing::error!(error = %e, "recovery failed"),
            }
        }
    }
}

fn main() -> Result<()> {
    let cfg = config::load_or_default()?;

    let default_level = cfg.general.log_level.clone();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("murf={}", default_level).into()),
        )
        .init();

    tracing::info!("murf starting");

    // Startup recovery: if a prior apply was interrupted, restore now.
    match journal::recover() {
        Ok(journal::RecoveryResult::NoJournal) => {}
        Ok(journal::RecoveryResult::AlreadyCommitted) => {
            tracing::info!("previous apply was committed; journal cleaned up");
        }
        Ok(journal::RecoveryResult::RolledBack { restored, failed }) => {
            tracing::warn!(restored, failed, "rolled back interrupted apply");
        }
        Ok(journal::RecoveryResult::GaveUp { attempts }) => {
            tracing::error!(
                attempts,
                "journal gave up after repeated rollback failures; manual restore required"
            );
        }
        Err(e) => {
            tracing::error!(error = %e, "journal recovery failed");
        }
    }

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let menu_proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(UserEvent::Menu(event));
    }));

    let watcher = watcher::Watcher::start(cfg.general.settle_delay_secs)?;
    let watcher_proxy = event_loop.create_proxy();
    let rx = watcher.rx;
    std::thread::spawn(move || {
        for path in rx {
            let _ = watcher_proxy.send_event(UserEvent::WallpaperChanged(path));
        }
    });

    let menu = Menu::new();
    let apply_now = MenuItem::new("Apply now", true, None);
    let open_config = MenuItem::new("Open config", true, None);
    let quit = MenuItem::new("Quit", true, None);

    let apply_id = apply_now.id().clone();
    let open_config_id = open_config.id().clone();
    let quit_id = quit.id().clone();

    menu.append(&apply_now)?;
    menu.append(&open_config)?;
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
            tao::event::Event::UserEvent(UserEvent::Menu(menu_event)) => {
                if menu_event.id == apply_id {
                    match matugen::current_wallpaper() {
                        Some(w) => run_apply(&w, &cfg),
                        None => tracing::warn!("no wallpaper path found"),
                    }
                } else if menu_event.id == open_config_id {
                    let path = config::config_path();
                    tracing::info!(path = %path.display(), "opening config");
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "start", "", &path.to_string_lossy()])
                        .spawn();
                } else if menu_event.id == quit_id {
                    tracing::info!("quit clicked");
                    *control_flow = ControlFlow::Exit;
                }
            }
            tao::event::Event::UserEvent(UserEvent::WallpaperChanged(path)) => {
                run_apply(&path, &cfg);
            }
            _ => {}
        }
    });
}
