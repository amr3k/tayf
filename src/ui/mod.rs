pub mod canvas;
pub mod color_picker;
pub mod controls;
pub mod drop_zone;
pub mod header;
pub mod icon;
pub mod modal;
pub mod sidebar;
pub mod system_accent;
pub mod theme;

use gpui::prelude::*;
use gpui::*;
use rust_i18n::t;
use std::path::PathBuf;
use std::time::Duration;

use crate::config::Theme;
use crate::export::{export_animation, ExportFormat, ExportOptions};
use crate::state::{ActiveModal, AppState, ExportProgressState};
use crate::ui::canvas::render_animation_view;
use crate::ui::controls::render_playback_controls;
use crate::ui::drop_zone::render_drop_zone;
use crate::ui::header::render_header;
use crate::ui::modal::about::render_about_modal;
use crate::ui::modal::export::render_export_modal;
use crate::ui::modal::preferences::render_preferences_modal;
use crate::ui::modal::render_modal_container;
use crate::ui::sidebar::render_sidebar;
use crate::ui::theme::ThemeColors;

fn format_load_error(path: &std::path::Path, err: &anyhow::Error) -> String {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let err_str = err.to_string();
    if err_str.contains("File too large") {
        t!("file_too_large", size = "100").to_string()
    } else if err_str.contains("Invalid or unsupported")
        || err_str.contains("Failed to extract")
        || err_str.contains("ThorVG")
    {
        // Known invalid-Lottie cases — show the localized invalid-file message
        format!("{} — {}", file_name, t!("invalid_file"))
    } else {
        // Fallback for IO / unexpected errors: show file name plus raw error
        // so the user still gets an explicit failure instead of silent revert.
        format!("{} — {}", file_name, err_str)
    }
}

pub struct MainView {
    pub state: AppState,
    pub scrub_track_bounds: Option<Bounds<Pixels>>,
    pub focus_handle: FocusHandle,
    pub previous_rendered_image: Option<std::sync::Arc<RenderImage>>,
}

impl Focusable for MainView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl MainView {
    pub fn new(initial_file: Option<PathBuf>, cx: &mut Context<Self>) -> Self {
        let mut state = AppState::new();
        let focus_handle = cx.focus_handle();

        if let Some(path) = initial_file {
            if let Err(e) = state.load_file(&path) {
                let msg = format_load_error(&path, &e);
                state.status_message = Some((msg, true));
                tracing::error!("Failed to load initial file {:?}: {:?}", path, e);
            }
        }

        // Spawn timer loop for animation playback
        cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(Duration::from_millis(16))
                .await;
            let should_continue = this.update(cx, |view, cx| {
                if view.state.is_playing && view.state.animation.is_some() {
                    view.state.tick();
                    cx.notify();
                }
            });
            if should_continue.is_err() {
                break;
            }
        })
        .detach();

        Self {
            state,
            scrub_track_bounds: None,
            focus_handle,
            previous_rendered_image: None,
        }
    }

    pub fn open_file_dialog(&mut self, cx: &mut Context<Self>) {
        self.state.is_theme_dropdown_open = false;
        self.state.is_app_menu_open = false;
        let dialog = rfd::FileDialog::new()
            .add_filter("Lottie Animation", &["json", "lottie"])
            .set_title("Open Lottie Animation");

        if let Some(path) = dialog.pick_file() {
            if let Err(e) = self.state.load_file(&path) {
                let msg = format_load_error(&path, &e);
                self.state.status_message = Some((msg, true));
                tracing::warn!("Failed to load {:?}: {:?}", path, e);
            }
            cx.notify();
        }
    }

    pub fn handle_scrub_ratio(&mut self, ratio: f32, cx: &mut Context<Self>) {
        self.state.seek_ratio(ratio);
        cx.notify();
    }

    pub fn start_export(
        &mut self,
        format: ExportFormat,
        options: ExportOptions,
        cx: &mut Context<Self>,
    ) {
        if self.state.animation.is_none() {
            return;
        }

        let extension = match format {
            ExportFormat::Gif => "gif",
            ExportFormat::Mp4 => "mp4",
        };

        let file_name = format!("animation.{}", extension);
        let save_dialog = rfd::FileDialog::new().set_file_name(&file_name).add_filter(
            if format == ExportFormat::Gif {
                "GIF Image"
            } else {
                "MP4 Video"
            },
            &[extension],
        );

        let output_path = match save_dialog.save_file() {
            Some(p) => p,
            None => return,
        };

        let file_path = self
            .state
            .animation
            .as_ref()
            .unwrap()
            .metadata
            .file_path
            .clone();

        self.state.export_progress = Some(ExportProgressState {
            is_exporting: true,
            current_frame: 0,
            total_frames: 100,
            progress_percent: 0,
            error: None,
            success: false,
        });
        cx.notify();

        // Run export in background thread
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<(usize, usize)>();

        cx.spawn(async move |this, cx| {
            use futures::FutureExt;
            use futures::StreamExt;

            let export_fut = cx.background_executor().spawn(async move {
                let bytes = std::fs::read(&file_path)?;
                let mut anim =
                    crate::engine::LoadedAnimation::from_bytes(&bytes, Some(&file_path))?;
                let tx = tx;
                export_animation(
                    &mut anim,
                    &output_path,
                    format,
                    &options,
                    move |cur, tot| {
                        let _ = tx.unbounded_send((cur, tot));
                    },
                )
            });

            let mut export_fut = export_fut.fuse();
            let final_result = loop {
                futures::select! {
                    res = export_fut => {
                        break res;
                    }
                    progress = rx.next() => {
                        if let Some((cur, tot)) = progress {
                            let _ = this.update(cx, |view, cx| {
                                if let Some(p) = &mut view.state.export_progress {
                                    p.current_frame = cur;
                                    p.total_frames = tot;
                                    p.progress_percent = if tot > 0 {
                                        ((cur as f32 / tot as f32) * 100.0).round() as u32
                                    } else {
                                        0
                                    };
                                    cx.notify();
                                }
                            });
                        }
                    }
                }
            };

            let _ = this.update(cx, |view, cx| {
                if let Err(e) = final_result {
                    if let Some(p) = &mut view.state.export_progress {
                        p.is_exporting = false;
                        p.error = Some(e.to_string());
                    }
                } else {
                    view.state.active_modal = ActiveModal::None;
                    view.state.export_progress = None;
                    view.state.status_message = Some((t!("export_success").to_string(), false));
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn resolve_theme(&self, window: &Window) -> (ThemeColors, bool) {
        let is_dark = match self.state.config.theme {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::System => matches!(
                window.appearance(),
                WindowAppearance::Dark | WindowAppearance::VibrantDark
            ),
        };

        let colors = if is_dark {
            ThemeColors::dark()
        } else {
            ThemeColors::light()
        };

        let colors = match system_accent::system_accent() {
            Some(accent) => colors.with_accent(accent, is_dark),
            None => colors,
        };

        (colors, is_dark)
    }
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (theme, is_dark) = self.resolve_theme(window);
        let has_file = self.state.animation.is_some();
        let is_sidebar_open = self.state.is_sidebar_open && has_file;

        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle);
        }

        let is_rtl = crate::i18n::is_rtl();

        let is_maximized = window.is_maximized();

        div()
            .id("tayf-root")
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .text_color(theme.text_primary)
            .font_family(".SystemUIFont")
            .relative()
            .when(!is_maximized, |s| {
                s.rounded_2xl()
                    .border_1()
                    .border_color(theme.border)
                    .overflow_hidden()
            })
            .track_focus(&self.focus_handle)
            // External file drop handler (Drag & Drop)
            .on_drop(
                cx.listener(|this: &mut MainView, paths: &ExternalPaths, _window, cx| {
                    if let Some(path) = paths.paths().first() {
                        if let Err(e) = this.state.load_file(path) {
                            let msg = format_load_error(path, &e);
                            this.state.status_message = Some((msg, true));
                            tracing::warn!("Failed to load {:?}: {:?}", path, e);
                        }
                        cx.notify();
                    }
                }),
            )
            // Keyboard shortcuts
            .on_key_down(cx.listener(|this: &mut MainView, e: &KeyDownEvent, _, cx| {
                let key = e.keystroke.key.as_str();
                let modifiers = e.keystroke.modifiers;

                // Hex field editing takes precedence over playback shortcuts so
                // typing `ff0000` doesn't toggle play/step frames.
                if this.state.custom_picker.hex_draft.is_some()
                    && this.state.custom_picker.open_slot.is_some()
                {
                    let is_ctrl = modifiers.control || modifiers.platform;
                    if key.eq_ignore_ascii_case("escape") || key.eq_ignore_ascii_case("esc") {
                        this.state.close_custom_picker();
                        cx.notify();
                        return;
                    }
                    if key.eq_ignore_ascii_case("enter") || key.eq_ignore_ascii_case("return") {
                        this.state.commit_hex_edit();
                        cx.notify();
                        return;
                    }
                    if key.eq_ignore_ascii_case("backspace") || key.eq_ignore_ascii_case("delete")
                    {
                        this.state.pop_hex_char();
                        cx.notify();
                        return;
                    }
                    if key.eq_ignore_ascii_case("v") && is_ctrl {
                        if let Some(text) =
                            cx.read_from_clipboard().and_then(|item| item.text())
                        {
                            this.state.set_hex_draft(&text);
                            cx.notify();
                        }
                        return;
                    }
                    // Allow app-level shortcuts (Ctrl+O, etc.) while editing.
                    if is_ctrl || modifiers.alt {
                        // Fall through to global handling below.
                    } else {
                        let typed = e
                            .keystroke
                            .key_char
                            .as_deref()
                            .unwrap_or(e.keystroke.key.as_str());
                        let mut chars = typed.chars();
                        if let (Some(ch), None) = (chars.next(), chars.next()) {
                            if ch == '#' || ch.is_ascii_hexdigit() {
                                let target = this.state.custom_picker.hex_target_is_dark;
                                let current =
                                    this.state.canvas_color_for(target).clone();
                                this.state.push_hex_char(ch, &current);
                                cx.notify();
                                return;
                            }
                        }
                        // Swallow other plain keys (e.g. space) so playback
                        // shortcuts don't fire mid-edit.
                        return;
                    }
                }

                match key {
                    "space" | " " => {
                        if this.state.active_modal == ActiveModal::None {
                            this.state.toggle_play_pause();
                            cx.notify();
                        }
                    }
                    "left" | "Left" | "arrowleft" | "ArrowLeft" => {
                        if this.state.active_modal == ActiveModal::None {
                            let step = if modifiers.shift || modifiers.control || modifiers.platform
                            {
                                10.0
                            } else {
                                1.0
                            };
                            this.state.step_frame(-step);
                            cx.notify();
                        }
                    }
                    "right" | "Right" | "arrowright" | "ArrowRight" => {
                        if this.state.active_modal == ActiveModal::None {
                            let step = if modifiers.shift || modifiers.control || modifiers.platform
                            {
                                10.0
                            } else {
                                1.0
                            };
                            this.state.step_frame(step);
                            cx.notify();
                        }
                    }
                    "escape" | "Escape" | "esc" => {
                        if this.state.custom_picker.open_slot.is_some() {
                            this.state.close_custom_picker();
                            cx.notify();
                            return;
                        }
                        if this.state.is_theme_dropdown_open || this.state.is_app_menu_open {
                            this.state.is_theme_dropdown_open = false;
                            this.state.is_app_menu_open = false;
                        } else if this.state.active_modal != ActiveModal::None {
                            this.state.active_modal = ActiveModal::None;
                        } else if this.state.animation.is_some() {
                            this.state.reset();
                        } else if this.state.status_message.is_some() {
                            // Dismiss inline load-error card when no file is open
                            this.state.status_message = None;
                        }
                        cx.notify();
                    }
                    k if k.eq_ignore_ascii_case("o")
                        && (modifiers.control || modifiers.platform) =>
                    {
                        this.state.is_app_menu_open = false;
                        this.open_file_dialog(cx);
                    }
                    k if (k == "," || k.eq_ignore_ascii_case("comma"))
                        && (modifiers.control || modifiers.platform) =>
                    {
                        this.state.is_theme_dropdown_open = false;
                        this.state.is_app_menu_open = false;
                        this.state.active_modal = ActiveModal::Preferences;
                        cx.notify();
                    }
                    k if k.eq_ignore_ascii_case("e")
                        && (modifiers.control || modifiers.platform) =>
                    {
                        if this.state.animation.is_some() {
                            this.state.is_theme_dropdown_open = false;
                            this.state.is_app_menu_open = false;
                            this.state.active_modal = ActiveModal::Export;
                            cx.notify();
                        }
                    }
                    k if k.eq_ignore_ascii_case("b")
                        && (modifiers.control || modifiers.platform) =>
                    {
                        if this.state.animation.is_some() {
                            this.state.is_theme_dropdown_open = false;
                            this.state.is_app_menu_open = false;
                            this.state.is_sidebar_open = !this.state.is_sidebar_open;
                            cx.notify();
                        }
                    }
                    k if k.eq_ignore_ascii_case("w")
                        && (modifiers.control || modifiers.platform) =>
                    {
                        if this.state.custom_picker.open_slot.is_some() {
                            this.state.close_custom_picker();
                            cx.notify();
                            return;
                        }
                        if this.state.is_theme_dropdown_open || this.state.is_app_menu_open {
                            this.state.is_theme_dropdown_open = false;
                            this.state.is_app_menu_open = false;
                        } else if this.state.active_modal != ActiveModal::None {
                            this.state.active_modal = ActiveModal::None;
                        } else if this.state.animation.is_some() {
                            this.state.reset();
                        } else if this.state.status_message.is_some() {
                            this.state.status_message = None;
                        }
                        cx.notify();
                    }
                    k if (k.eq_ignore_ascii_case("q")
                        && (modifiers.control || modifiers.platform))
                        || (k.eq_ignore_ascii_case("f4") && modifiers.alt) =>
                    {
                        cx.quit();
                    }
                    "f1" | "F1" => {
                        this.state.is_theme_dropdown_open = false;
                        this.state.is_app_menu_open = false;
                        this.state.active_modal = ActiveModal::About;
                        cx.notify();
                    }
                    _ => {}
                }
            }))
            // Header Bar
            .child(render_header(&self.state, &theme, window, cx))
            // Body container
            .child(
                div()
                    .flex_1()
                    .flex()
                    .when(is_rtl, |s| s.flex_row_reverse())
                    .relative()
                    .when(!is_maximized, |s| s.rounded_b_2xl())
                    .overflow_hidden()
                    .child(
                        // Main Viewer Area (Canvas + Controls or Drop Zone)
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .relative()
                            .overflow_hidden()
                            .child(
                                div()
                                    .flex_1()
                                    .relative()
                                    .overflow_hidden()
                                    .child(if has_file {
                                        render_animation_view(&mut self.state, &theme, is_dark, cx)
                                            .into_any_element()
                                    } else {
                                        render_drop_zone(&self.state, &theme, is_maximized, cx)
                                            .into_any_element()
                                    }),
                            )
                            .children(if has_file {
                                Some(render_playback_controls(
                                    &self.state,
                                    &theme,
                                    is_maximized,
                                    is_sidebar_open,
                                    cx,
                                ))
                            } else {
                                None
                            }),
                    )
                    // Sidebar
                    .children(if is_sidebar_open {
                        Some(render_sidebar(
                            &self.state,
                            &theme,
                            is_dark,
                            is_maximized,
                            cx,
                        ))
                    } else {
                        None
                    }),
            )
            // Toast / Status banner — hidden for load errors when no file is open
            // (the drop zone already shows an explicit inline error card)
            .children(if let Some((msg, is_err)) = &self.state.status_message {
                if !has_file && *is_err {
                    None
                } else {
                    Some(
                        div()
                            .absolute()
                            .bottom(px(80.0))
                            .left_1_2()
                            .p_3()
                            .px_4()
                            .rounded_xl()
                            .bg(if *is_err {
                                theme.danger
                            } else {
                                theme.surface_active
                            })
                            .border_1()
                            .border_color(theme.border)
                            .shadow_lg()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if *is_err {
                                gpui::rgba(0xffffffff).into()
                            } else {
                                theme.text_primary
                            })
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.state.status_message = None;
                                    cx.notify();
                                }),
                            )
                            .child(msg.clone()),
                    )
                }
            } else {
                None
            })
            // Dropdown / Menu Dismiss Backdrop
            .children(
                if self.state.is_theme_dropdown_open || self.state.is_app_menu_open {
                    Some(
                        div()
                            .id("dropdown-dismiss-backdrop")
                            .absolute()
                            .inset_0()
                            .when(!is_maximized, |s| s.rounded_2xl())
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.state.is_theme_dropdown_open = false;
                                    this.state.is_app_menu_open = false;
                                    cx.notify();
                                }),
                            ),
                    )
                } else {
                    None
                },
            )
            // Modal Dialog Overlays
            .children(match self.state.active_modal {
                ActiveModal::None => None,
                ActiveModal::Preferences => Some(
                    render_modal_container(
                        t!("preferences").to_string(),
                        render_preferences_modal(&self.state, &theme, cx),
                        &theme,
                        is_maximized,
                        cx,
                    )
                    .into_any_element(),
                ),
                ActiveModal::About => Some(
                    render_modal_container(
                        t!("about").to_string(),
                        render_about_modal(&theme, cx),
                        &theme,
                        is_maximized,
                        cx,
                    )
                    .into_any_element(),
                ),
                ActiveModal::Export => Some(
                    render_modal_container(
                        t!("export_animation").to_string(),
                        render_export_modal(&self.state, &theme, cx),
                        &theme,
                        is_maximized,
                        cx,
                    )
                    .into_any_element(),
                ),
            })
    }
}
