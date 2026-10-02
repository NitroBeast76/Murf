// Murf entry point. v0.1 step 3: tray icon + menu + matugen probe.

mod matugen;
mod palette;

use anyhow::Result;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIconBuilder,
};

enum UserEvent {
    Menu(MenuEvent),
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

fn run_apply() {
    let Some(wallpaper) = matugen::current_wallpaper() else {
        tracing::warn!("no wallpaper path found in registry");
        return;
    };
    tracing::info!(wallpaper = %wallpaper, "applying palette");

    match matugen::generate(&wallpaper, "dark", "scheme-tonal-spot", 0.0) {
        Ok(palette) => {
            tracing::info!(roles = palette.roles.len(), "palette parsed");
            for role in [
                "background",
                "on_background",
                "primary",
                "secondary",
                "tertiary",
                "error",
                "surface",
                "surface_container_high",
            ] {
                tracing::info!(
                    role = role,
                    hex = palette.hex(role).unwrap_or("<missing>"),
                    "role"
                );
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "matugen failed");
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

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::Menu(event));
    }));

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

        if let tao::event::Event::UserEvent(UserEvent::Menu(menu_event)) = event {
            if menu_event.id == apply_id {
                run_apply();
            } else if menu_event.id == quit_id {
                tracing::info!("quit clicked");
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}
