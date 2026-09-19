//! Tray icon + menu (M0). RFC-0002 §4.2 — accessory-resident, no windows.
//!
//! The icon is generated at runtime (a simple filled disc) so the scaffold
//! carries no binary assets; real branding lands with W6 packaging.

use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

use crate::settings;

/// A 32x32 dawn-gold disc on transparency, built in memory.
fn dawn_icon() -> tauri::image::Image<'static> {
    const SIZE: u32 = 32;
    let radius = SIZE as f32 / 2.0 - 2.0;
    let mut pixels: Vec<u8> = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 + 0.5 - SIZE as f32 / 2.0;
            let dy = y as f32 + 0.5 - SIZE as f32 / 2.0;
            let inside = dx * dx + dy * dy <= radius * radius;
            let rgba: [u8; 4] = if inside {
                [0xE8, 0xC5, 0x9B, 0xFF]
            } else {
                [0x00, 0x00, 0x00, 0x00]
            };
            pixels.extend_from_slice(&rgba);
        }
    }
    tauri::image::Image::new_owned(pixels, SIZE, SIZE)
}

pub fn install(app: &tauri::AppHandle) -> tauri::Result<()> {
    let current = settings::load(app);

    let pause = CheckMenuItem::with_id(
        app,
        "pause_captures",
        "Pause captures",
        true,
        current.pause_captures,
        None::<&str>,
    )?;
    let panel = MenuItem::with_id(app, "show_panel", "Show result panel", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "About Ruoxi", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Ruoxi", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&panel, &pause, &about, &quit])?;

    TrayIconBuilder::with_id("main")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .icon(dawn_icon())
        .tooltip("Ruoxi — starting")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "quit" => app.exit(0),
            "show_panel" => crate::panel::show(app),
            "pause_captures" => {
                let mut s = settings::load(app);
                s.pause_captures = !s.pause_captures;
                if let Err(e) = settings::save(app, &s) {
                    eprintln!("ruoxi: failed to persist settings: {e}");
                }
            }
            "about" => {
                if let Some(tray) = app.tray_by_id("main") {
                    let _ = tray.set_tooltip(Some("Ruoxi — desktop companion (M0)"));
                }
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}
