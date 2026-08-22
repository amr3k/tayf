use gpui::*;
use rust_i18n::t;

use crate::state::AppState;
use crate::ui::theme::{ThemeColors, parse_hex_color};
use crate::ui::MainView;

pub fn render_sidebar(
    state: &AppState,
    theme: &ThemeColors,
    is_dark: bool,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let anim = match &state.animation {
        Some(a) => a,
        None => return div().into_any_element(),
    };

    let meta = &anim.metadata;
    let current_bg = state.effective_canvas_background(is_dark).to_string();

    let color_presets = [
        "#FFFFFF", "#0F1115", "#1E293B", "#000000", "#EF4444", "#3B82F6", "#10B981", "#8B5CF6",
    ];

    div()
        .id("sidebar-panel")
        .w(px(280.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(theme.surface)
        .border_l_1()
        .border_color(theme.border)
        .p_4()
        .gap_6()
        .overflow_hidden()
        .child(
            // Header
            div()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(theme.border)
                .pb_3()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .child("⚙")
                        .child(t!("controls").to_string()),
                )
                .child(
                    div()
                        .size(px(24.0))
                        .rounded_md()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(theme.text_muted)
                        .hover(|s| s.bg(theme.surface_hover).text_color(theme.text_primary))
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.state.is_sidebar_open = false;
                                cx.notify();
                            }),
                        )
                        .child("✕"),
                ),
        )
        .child(
            // Appearance section
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_muted)
                        .child(t!("appearance").to_uppercase()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .text_xs()
                                .text_color(theme.text_secondary)
                                .child(t!("background").to_string())
                                .child(
                                    div()
                                        .font_family(".SystemUIFont")
                                        .text_color(theme.text_primary)
                                        .child(current_bg.clone()),
                                ),
                        )
                        .child(
                            // Color preset swatches
                            div()
                                .flex()
                                .flex_wrap()
                                .gap_2()
                                .children(color_presets.iter().map(|&hex| {
                                    let is_active = current_bg.eq_ignore_ascii_case(hex);
                                    let swatch_color = parse_hex_color(hex);
                                    let hex_string = hex.to_string();

                                    div()
                                        .id(SharedString::from(format!("swatch-{}", hex)))
                                        .size(px(26.0))
                                        .rounded_md()
                                        .bg(swatch_color)
                                        .border_2()
                                        .border_color(if is_active { theme.accent } else { theme.border })
                                        .shadow_sm()
                                        .cursor_pointer()
                                        .hover(|s| s.border_color(theme.accent))
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                this.state.update_canvas_color(hex_string.clone(), is_dark);
                                                cx.notify();
                                            }),
                                        )
                                })),
                        ),
                ),
        )
        .child(
            // Metadata section
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_muted)
                        .child(t!("metadata").to_uppercase()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2p5()
                        .text_xs()
                        .child(render_meta_row(t!("name").to_string(), &meta.file_name, theme))
                        .child(render_meta_row(t!("type").to_string(), meta.file_type_str(), theme))
                        .child(render_meta_row(t!("size").to_string(), &meta.formatted_size(), theme))
                        .child(render_meta_row(t!("dimensions").to_string(), &meta.formatted_dimensions(), theme))
                        .child(render_meta_row(t!("fps").to_string(), &meta.formatted_fps(), theme))
                        .child(render_meta_row(t!("duration").to_string(), &meta.formatted_duration(), theme))
                        .child(render_meta_row(t!("frames").to_string(), &format!("{}", meta.total_frames.round() as u32), theme)),
                ),
        )
        .into_any_element()
}

fn render_meta_row(label: String, value: &str, theme: &ThemeColors) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .py_0p5()
        .child(
            div()
                .text_color(theme.text_secondary)
                .child(label),
        )
        .child(
            div()
                .font_family(".SystemUIFont")
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text_primary)
                .child(value.to_string()),
        )
}
