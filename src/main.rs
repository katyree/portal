#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod catalog;
mod search_input;
mod ui;

use anyhow::Result;
use gpui::prelude::*;
use gpui::{App, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions, px, size};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "--scan") {
        for launcher in catalog::Catalog::load()?.scan() {
            println!(
                "{}: {}{}",
                launcher.id.name(),
                if launcher.target.is_some() {
                    "installed"
                } else {
                    "not found"
                },
                if launcher.running { " (running)" } else { "" }
            );
        }
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "--launch") {
        let id = catalog::LauncherId::ALL
            .into_iter()
            .find(|id| args.get(1).is_some_and(|slug| slug == id.slug()))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Use --launch steam|ubisoft|epicgames|xbox|ea|rockstargames|battledotnet|amazon"
                )
            })?;
        return catalog::Catalog::load()?.launch(id);
    }
    Application::new()
        .with_assets(ui::Assets)
        .run(|cx: &mut App| {
            search_input::bind_keys(cx);
            let bounds = Bounds::centered(None, size(px(620.), px(570.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("portal".into()),
                        ..Default::default()
                    }),
                    window_min_size: Some(size(px(490.), px(480.))),
                    ..Default::default()
                },
                |_, cx| cx.new(ui::Portal::new),
            )
            .expect("Could not create Portal window");
            cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            cx.activate(true);
        });
    Ok(())
}
