use gpui::*;
use rust_i18n::t;

use crate::export::ExportFormat;
use crate::state::AppState;
use crate::ui::theme::ThemeColors;
use crate::ui::MainView;

pub fn render_export_modal(
    state: &AppState,
    theme: &ThemeColors,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let is_exporting = state.export_progress.as_ref().map(|p| p.is_exporting).unwrap_or(false);
    let progress_percent = state.export_progress.as_ref().map(|p| p.progress_percent).unwrap_or(0);
    let export_error = state.export_progress.as_ref().and_then(|p| p.error.clone());

    let current_format = state.export_format;
    let current_width = state.export_options.width;
    let current_fps = state.export_options.fps;
    let is_transparent = state.export_options.transparent;
    let is_looping = state.export_options.loop_gif;
    let current_quality = state.export_options.quality;

    let native_width = state
        .animation
        .as_ref()
        .map(|a| (a.metadata.width.round() as u32).max(10))
        .unwrap_or(800);

    let format_str = match current_format {
        ExportFormat::Gif => "GIF",
        ExportFormat::Mp4 => "MP4",
    };

    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(
            // Format selector (GIF vs MP4)
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_muted)
                        .child(t!("format").to_uppercase()),
                )
                .child(
                    div()
                        .flex()
                        .gap_3()
                        .child(
                            render_format_card(
                                "format-gif-card",
                                "GIF",
                                "Animated GIF Image",
                                current_format == ExportFormat::Gif,
                                theme,
                                cx.listener(|this, _, _, cx| {
                                    this.state.set_export_format(ExportFormat::Gif);
                                    cx.notify();
                                }),
                            ),
                        )
                        .child(
                            render_format_card(
                                "format-mp4-card",
                                "MP4",
                                "H.264 Video",
                                current_format == ExportFormat::Mp4,
                                theme,
                                cx.listener(|this, _, _, cx| {
                                    this.state.set_export_format(ExportFormat::Mp4);
                                    cx.notify();
                                }),
                            ),
                        ),
                ),
        )
        .child(
            // Dimensions Selector
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_muted)
                                .child(t!("width_px").to_string()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family(".SystemUIFont")
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.accent)
                                .child(format!("{} px", current_width)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .children([native_width, 512, 800, 1080, 1920].into_iter().map(|w| {
                            let is_selected = current_width == w;
                            let label = if w == native_width {
                                format!("{} (Native)", w)
                            } else {
                                format!("{}px", w)
                            };

                            div()
                                .id(SharedString::from(format!("dim-preset-{}", w)))
                                .flex_1()
                                .py_1p5()
                                .rounded_lg()
                                .text_center()
                                .text_xs()
                                .font_weight(if is_selected { FontWeight::BOLD } else { FontWeight::MEDIUM })
                                .bg(if is_selected { theme.accent } else { theme.surface_hover })
                                .text_color(if is_selected { theme.accent_text } else { theme.text_secondary })
                                .hover(|s| s.bg(if is_selected { theme.accent_hover } else { theme.surface_active }))
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.state.set_export_width(w);
                                        cx.notify();
                                    }),
                                )
                                .child(label)
                        })),
                ),
        )
        .child(
            // FPS Selector
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_muted)
                        .child(t!("frame_rate").to_string()),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .children([15u32, 24, 30, 60].into_iter().map(|fps| {
                            let is_selected = current_fps == fps;

                            div()
                                .id(SharedString::from(format!("fps-preset-{}", fps)))
                                .flex_1()
                                .py_1p5()
                                .rounded_lg()
                                .text_center()
                                .text_xs()
                                .font_weight(if is_selected { FontWeight::BOLD } else { FontWeight::MEDIUM })
                                .bg(if is_selected { theme.accent } else { theme.surface_hover })
                                .text_color(if is_selected { theme.accent_text } else { theme.text_secondary })
                                .hover(|s| s.bg(if is_selected { theme.accent_hover } else { theme.surface_active }))
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.state.set_export_fps(fps);
                                        cx.notify();
                                    }),
                                )
                                .child(format!("{} FPS", fps))
                        })),
                ),
        )
        .child(
            // Quality Selector
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_muted)
                        .child(t!("quality").to_string()),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .children([(50u8, "Low (50%)"), (75u8, "Medium (75%)"), (90u8, "High (90%)")].into_iter().map(|(q, label)| {
                            let is_selected = (current_quality as i16 - q as i16).abs() <= 10;

                            div()
                                .id(SharedString::from(format!("quality-preset-{}", q)))
                                .flex_1()
                                .py_1p5()
                                .rounded_lg()
                                .text_center()
                                .text_xs()
                                .font_weight(if is_selected { FontWeight::BOLD } else { FontWeight::MEDIUM })
                                .bg(if is_selected { theme.accent } else { theme.surface_hover })
                                .text_color(if is_selected { theme.accent_text } else { theme.text_secondary })
                                .hover(|s| s.bg(if is_selected { theme.accent_hover } else { theme.surface_active }))
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.state.set_export_quality(q);
                                        cx.notify();
                                    }),
                                )
                                .child(label)
                        })),
                ),
        )
        .child(
            // Options: Transparency & Loop
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .id("toggle-transparency-btn")
                        .flex()
                        .items_center()
                        .justify_between()
                        .p_2p5()
                        .rounded_lg()
                        .bg(theme.surface_hover)
                        .border_1()
                        .border_color(theme.border)
                        .hover(|s| s.bg(theme.surface_active))
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.state.toggle_export_transparent();
                                cx.notify();
                            }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text_primary)
                                .child(t!("transparent_background").to_string()),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(if is_transparent { theme.accent } else { theme.surface })
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(if is_transparent { theme.accent_text } else { theme.text_muted })
                                .child(if is_transparent { "ON" } else { "OFF" }),
                        ),
                )
                .child(
                    div()
                        .id("toggle-loop-btn")
                        .flex()
                        .items_center()
                        .justify_between()
                        .p_2p5()
                        .rounded_lg()
                        .bg(theme.surface_hover)
                        .border_1()
                        .border_color(theme.border)
                        .hover(|s| s.bg(theme.surface_active))
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.state.toggle_export_loop();
                                cx.notify();
                            }),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text_primary)
                                .child(t!("loop_animation").to_string()),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(if is_looping { theme.accent } else { theme.surface })
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(if is_looping { theme.accent_text } else { theme.text_muted })
                                .child(if is_looping { "ON" } else { "OFF" }),
                        ),
                ),
        )
        .children(if is_exporting {
            Some(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_3()
                    .rounded_lg()
                    .bg(theme.surface_hover)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text_secondary)
                            .child(t!("exporting_progress", percent => progress_percent.to_string()).to_string())
                            .child(format!("{}%", progress_percent)),
                    )
                    .child(
                        div()
                            .h(px(8.0))
                            .w_full()
                            .rounded_full()
                            .bg(theme.surface)
                            .child(
                                div()
                                    .h_full()
                                    .w(relative((progress_percent as f32 / 100.0).clamp(0.0, 1.0)))
                                    .rounded_full()
                                    .bg(theme.accent),
                            ),
                    ),
            )
        } else {
            None
        })
        .children(if let Some(err) = export_error {
            Some(
                div()
                    .p_3()
                    .rounded_lg()
                    .bg(theme.danger)
                    .text_color(theme.accent_text)
                    .text_xs()
                    .child(err),
            )
        } else {
            None
        })
        .child(
            // Action Buttons
            div()
                .flex()
                .items_center()
                .justify_end()
                .gap_3()
                .mt_1()
                .child(
                    div()
                        .id("cancel-export-btn")
                        .px_4()
                        .py_2()
                        .rounded_lg()
                        .bg(theme.surface_hover)
                        .hover(|s| s.bg(theme.surface_active))
                        .text_color(theme.text_primary)
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.state.active_modal = crate::state::ActiveModal::None;
                                cx.notify();
                            }),
                        )
                        .child(t!("cancel").to_string()),
                )
                .child(
                    div()
                        .id("start-export-btn")
                        .px_5()
                        .py_2()
                        .rounded_lg()
                        .bg(theme.accent)
                        .hover(|s| s.bg(theme.accent_hover))
                        .active(|s| s.bg(theme.accent_active))
                        .text_color(theme.accent_text)
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                let format = this.state.export_format;
                                let options = this.state.export_options.clone();
                                this.start_export(format, options, cx);
                            }),
                        )
                        .child(t!("export_button", format => format_str).to_string()),
                ),
        )
        .into_any_element()
}

fn render_format_card(
    id: &'static str,
    title: &'static str,
    desc: &'static str,
    is_active: bool,
    theme: &ThemeColors,
    on_click: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex_1()
        .p_3p5()
        .rounded_xl()
        .border_2()
        .border_color(if is_active { theme.accent } else { theme.border })
        .bg(if is_active { theme.surface_active } else { theme.surface_hover })
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, on_click)
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if is_active { theme.accent } else { theme.text_primary })
                .child(title),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child(desc),
        )
}
