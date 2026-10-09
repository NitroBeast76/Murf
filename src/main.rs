// Murf entry point. v0.1 with semantic ANSI mode.

mod adapters;
mod color;
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

const MODE: &str = "smart";
const SCHEME_TYPE: &str = "scheme-tonal-spot";
const CONTRAST: f32 = 0.0;
const ANSI_MODE: AnsiMode = AnsiMode::Semantic;

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

fn write_palette_dump(wallpaper: &str, palette: &palette::Palette) -> Result<PathBuf> {
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
    let sem = color::semantic_ansi(&primary, &error);

    let doc = serde_json::json!({
        "wallpaper": wallpaper,
        "mode": MODE,
        "scheme_type": SCHEME_TYPE,
        "contrast": CONTRAST,
        "ansi_mapping": match ANSI_MODE {
            AnsiMode::Semantic => "semantic",
            AnsiMode::Material => "material",
        },
        "roles": roles,
        "ansi_semantic": {
            "red": sem.red, "yellow": sem.yellow, "green": sem.green,
            "cyan": sem.cyan, "blue": sem.blue, "purple": sem.purple,
            "brightRed": sem.bright_red, "brightYellow": sem.bright_yellow,
            "brightGreen": sem.bright_green, "brightCyan": sem.bright_cyan,
            "brightBlue": sem.bright_blue, "brightPurple": sem.bright_purple,
        },
    });

    fs::write(&path, serde_json::to_string_pretty(&doc)?)?;
    Ok(path)
}

fn run_apply(wallpaper: &str) {
    tracing::info!(wallpaper = %wallpaper, "applying palette");

    let palette = match matugen::generate(wallpaper, MODE, SCHEME_TYPE, CONTRAST) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "matugen failed");
            return;
        }
    };

    tracing::info!(roles = palette.roles.len(), "palette parsed");

    match write_palette_dump(wallpaper, &palette) {
        Ok(path) => tracing::info!(path = %path.display(), "palette dump written"),
        Err(e) => tracing::warn!(error = %e, "failed to write palette dump"),
    }

    match adapters::windows_terminal::apply(&palette, ANSI_MODE) {
        Ok(paths) if paths.is_empty() => {
            tracing::warn!("no Windows Terminal config found");
        }
        Ok(paths) => {
            tracing::info!(count = paths.len(), "adapter applied");
        }
        Err(e) => {
            tracing::error!(error = %e, "adapter failed");
        }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "murf=info".into()),
        )
        .init();

    tracing::info!("murf starting");

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let menu_proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(UserEvent::Menu(event));
    }));

    let watcher = watcher::Watcher::start(6)?;
    let watcher_proxy = event_loop.create_proxy();
    let rx = watcher.rx;
    std::thread::spawn(move || {
        for path in rx {
            let _ = watcher_proxy.send_event(UserEvent::WallpaperChanged(path));
        }
    });

    let menu = Menu::new();
    let apply_now = MenuItem::new("Apply now", true, None);
    let quit = MenuItem::new("Quit", true, None);

    let apply_id = apply_now.id().clone();
    let quit_id = quit.id().clone();

    menu.append(&apply_now)?;
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
                        Some(w) => run_apply(&w),
                        None => tracing::warn!("no wallpaper path found"),
                    }
                } else if menu_event.id == quit_id {
                    tracing::info!("quit clicked");
                    *control_flow = ControlFlow::Exit;
                }
            }
            tao::event::Event::UserEvent(UserEvent::WallpaperChanged(path)) => {
                run_apply(&path);
            }
            _ => {}
        }
    });
}
