rust_i18n::i18n!("locales");

pub mod config;
pub mod engine;
pub mod export;
pub mod i18n;
pub mod state;
pub mod ui;

use gpui::*;
use std::path::PathBuf;

fn wayland_app_id() -> String {
    if let Some(data_dir) = directories::BaseDirs::new().map(|d| d.data_dir().to_path_buf()) {
        let mut matches: Vec<PathBuf> = std::fs::read_dir(data_dir.join("applications"))
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name.starts_with("appimagekit_")
                            && name.to_lowercase().ends_with("-tayf.desktop")
                    })
            })
            .collect();
        matches.sort();
        if let Some(path) = matches.first() {
            return path.to_string_lossy().into_owned();
        }
    }
    "tayf".to_string()
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let file_arg = std::env::args().nth(1).map(PathBuf::from);

    Application::new()
        .with_assets(ui::icon::IconAssets)
        .run(move |cx: &mut App| {
            let config = config::AppConfig::load();
            let initial_file = file_arg.clone();

            let window_options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point {
                        x: px(100.0),
                        y: px(100.0),
                    },
                    size: Size {
                        width: px(config.window_width as f32),
                        height: px(config.window_height as f32),
                    },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("Tayf".into()),
                    ..Default::default()
                }),
                app_id: Some(wayland_app_id()),
                window_decorations: Some(WindowDecorations::Client),
                focus: true,
                show: true,
                kind: WindowKind::Normal,
                is_movable: true,
                window_min_size: Some(Size {
                    width: px(500.0),
                    height: px(400.0),
                }),
                window_background: WindowBackgroundAppearance::Transparent,
                ..Default::default()
            };

            let window = cx
                .open_window(window_options, move |_, cx| {
                    cx.new(|cx| ui::MainView::new(initial_file, cx))
                })
                .expect("Failed to open Tayf window");

            window
                .update(cx, |view, window, cx| {
                    window.set_window_title("Tayf");
                    window.focus(&view.focus_handle(cx));
                    cx.activate(true);
                })
                .ok();
        });
}
