rust_i18n::i18n!("locales");

pub mod config;
pub mod engine;
pub mod export;
pub mod i18n;
pub mod state;
pub mod ui;

use gpui::*;
use std::path::PathBuf;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let file_arg = std::env::args().nth(1).map(PathBuf::from);

    Application::new().run(move |cx: &mut App| {
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
                title: Some("AnimaView".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            focus: true,
            show: true,
            kind: WindowKind::Normal,
            is_movable: true,
            window_min_size: Some(Size {
                width: px(500.0),
                height: px(400.0),
            }),
            window_background: WindowBackgroundAppearance::Opaque,
            ..Default::default()
        };

        let window = cx
            .open_window(window_options, move |_, cx| {
                cx.new(|cx| ui::MainView::new(initial_file, cx))
            })
            .expect("Failed to open AnimaView window");

        window
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle(cx));
                cx.activate(true);
            })
            .ok();
    });
}
