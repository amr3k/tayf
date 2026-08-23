use gpui::prelude::*;
use gpui::*;
use rust_i18n::t;

use crate::config::HexColor;
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

    let is_rtl = crate::i18n::is_rtl();
    let meta = &anim.metadata;
    let current_bg = state.effective_canvas_background(is_dark);

    let color_presets = [
        "#FFFFFF", "#F8FAFC", "#E2E8F0", "#94A3B8",
        "#1E293B", "#0F1115", "#000000", "#EF4444",
        "#F59E0B", "#10B981", "#06B6D4", "#3B82F6",
        "#8B5CF6", "#EC4899",
    ];

    div()
        .id("sidebar-panel")
        .w(px(280.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(theme.surface)
        .when_else(is_rtl, |s| s.border_r_1(), |s| s.border_l_1())
        .border_color(theme.border)
        .p_4()
        .justify_between()
        .overflow_hidden()
        .child(
            // Top and Middle content
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap_6()
                .overflow_hidden()
                .child(
                    // Header
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .when(is_rtl, |s| s.flex_row_reverse())
                        .border_b_1()
                        .border_color(theme.border)
                        .pb_3()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .when(is_rtl, |s| s.flex_row_reverse())
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child(
                                    crate::ui::icon::render_icon(crate::ui::icon::Icon::Settings)
                                        .size(px(16.0))
                                        .text_color(theme.text_primary),
                                )
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
                                .child(
                                    crate::ui::icon::render_icon(crate::ui::icon::Icon::Cancel)
                                        .size(px(14.0))
                                        .text_color(theme.text_muted),
                                )
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
                                        .when(is_rtl, |s| s.flex_row_reverse())
                                        .text_xs()
                                        .text_color(theme.text_secondary)
                                        .child(t!("background").to_string())
                                        .child(
                                            div()
                                                .font_family(".SystemUIFont")
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme.text_primary)
                                                .child(current_bg.as_str().to_string()),
                                        ),
                                )
                                .child(
                                    // Color preset swatches
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .gap_2()
                                        .children(color_presets.iter().map(|&hex| {
                                            let is_active = current_bg.as_str().eq_ignore_ascii_case(hex);
                                            let swatch_color = parse_hex_color(hex);
                                            let hex_color = HexColor::new(hex);

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
                                                        this.state.update_canvas_color(hex_color.clone(), is_dark);
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
                                .child(render_meta_row(t!("name").to_string(), &meta.file_name, theme, is_rtl))
                                .child(render_meta_row(t!("type").to_string(), meta.file_type_str(), theme, is_rtl))
                                .child(render_meta_row(t!("size").to_string(), &meta.formatted_size(), theme, is_rtl))
                                .child(render_meta_row(t!("dimensions").to_string(), &meta.formatted_dimensions(), theme, is_rtl))
                                .child(render_meta_row(t!("fps").to_string(), &meta.formatted_fps(), theme, is_rtl))
                                .child(render_meta_row(t!("duration").to_string(), &meta.formatted_duration(), theme, is_rtl))
                                .child(render_meta_row(t!("frames").to_string(), &format!("{}", meta.total_frames.round() as u32), theme, is_rtl)),
                        ),
                ),
        )
        // Bottom Action buttons (Export and Close File)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .pt_3()
                .border_t_1()
                .border_color(theme.border)
                // Export Animation Button
                .child(
                    div()
                        .id("sidebar-export-btn")
                        .w_full()
                        .h(px(32.0))
                        .px_2p5()
                        .rounded_lg()
                        .flex()
                        .items_center()
                        .justify_between()
                        .when(is_rtl, |s| s.flex_row_reverse())
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .bg(theme.surface_hover)
                        .text_color(theme.text_primary)
                        .border_1()
                        .border_color(theme.border)
                        .hover(|s| s.bg(theme.surface_active).border_color(theme.accent))
                        .active(|s| s.bg(theme.surface_active))
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.state.is_app_menu_open = false;
                                this.state.active_modal = crate::state::ActiveModal::Export;
                                cx.notify();
                            }),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .when(is_rtl, |s| s.flex_row_reverse())
                                .child(
                                    crate::ui::icon::render_icon(crate::ui::icon::Icon::Download)
                                        .size(px(14.0))
                                        .text_color(theme.text_secondary),
                                )
                                .child(
                                    div().child(t!("export_animation").to_string()),
                                ),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .bg(theme.surface)
                                .border_1()
                                .border_color(theme.border)
                                .text_xs()
                                .font_weight(FontWeight::NORMAL)
                                .text_color(theme.text_muted)
                                .child(if cfg!(target_os = "macos") { "⌘E" } else { "Ctrl+E" }),
                        ),
                )
                // Close File Button
                .child(
                    div()
                        .id("sidebar-close-file-btn")
                        .w_full()
                        .h(px(32.0))
                        .px_2p5()
                        .rounded_lg()
                        .flex()
                        .items_center()
                        .justify_between()
                        .when(is_rtl, |s| s.flex_row_reverse())
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .bg(theme.surface)
                        .text_color(theme.text_secondary)
                        .border_1()
                        .border_color(theme.border)
                        .hover(|s| s.bg(theme.surface_hover).text_color(theme.danger).border_color(theme.danger))
                        .active(|s| s.bg(theme.surface_active))
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.state.is_app_menu_open = false;
                                this.state.reset();
                                cx.notify();
                            }),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .when(is_rtl, |s| s.flex_row_reverse())
                                .child(
                                    crate::ui::icon::render_icon(crate::ui::icon::Icon::Cancel)
                                        .size(px(14.0))
                                        .text_color(theme.text_secondary),
                                )
                                .child(
                                    div().child(t!("close_file").to_string()),
                                ),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .bg(theme.surface_hover)
                                .border_1()
                                .border_color(theme.border)
                                .text_xs()
                                .font_weight(FontWeight::NORMAL)
                                .text_color(theme.text_muted)
                                .child("Esc"),
                        ),
                ),
        )
        .into_any_element()
}

fn render_meta_row(label: String, value: &str, theme: &ThemeColors, is_rtl: bool) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .when(is_rtl, |s| s.flex_row_reverse())
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
