use gpui::prelude::*;
use gpui::*;
use rust_i18n::t;

use crate::engine::palette::preset_palettes;
use crate::state::AppState;
use crate::ui::color_picker::render_palette_picker_panel;
use crate::ui::theme::{parse_hex_color, ThemeColors};
use crate::ui::MainView;

const PALETTE_SWATCH: f32 = 26.0;
const PRESET_SWATCH: f32 = 16.0;

pub fn render_palette_section(
    state: &AppState,
    theme: &ThemeColors,
    window: &mut Window,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let is_rtl = crate::i18n::is_rtl();
    let colors = state.palette_colors();
    let has_override = state.has_palette_override();

    let mut section = div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .when(is_rtl, |s| s.flex_row_reverse())
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_muted)
                        .child(t!("palette").to_uppercase()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(format!("{}", colors.len())),
                ),
        );

    if colors.is_empty() {
        section = section.child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .child(t!("palette_empty").to_string()),
        );
        return section.into_any_element();
    }

    // Detected colors grid.
    section = section.child(
        div()
            .flex()
            .flex_wrap()
            .gap_2()
            .when(is_rtl, |s| s.flex_row_reverse())
            .children(colors.iter().enumerate().map(|(i, original)| {
                let current = state.palette_current(original);
                let is_active = state.palette_edit_index == Some(i);
                let is_overridden = !current.eq_ignore_ascii_case(original);
                div()
                    .id(SharedString::from(format!("palette-swatch-{i}")))
                    .size(px(PALETTE_SWATCH))
                    .rounded_md()
                    .bg(parse_hex_color(&current))
                    .border_2()
                    .border_color(if is_active {
                        theme.accent
                    } else if is_overridden {
                        theme.accent
                    } else {
                        theme.border
                    })
                    .shadow_sm()
                    .cursor_pointer()
                    .hover(|s| s.border_color(theme.accent))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                            this.state.handle_palette_swatch_click(i, e.position);
                            cx.notify();
                        }),
                    )
            })),
    );

    // Per-color editor (original -> current + custom picker).
    if let Some(idx) = state.palette_edit_index {
        if let Some(original) = colors.get(idx).cloned() {
            let current = state.palette_current(&original);
            let is_overridden = !current.eq_ignore_ascii_case(&original);
            let original_owned = original.clone();
            section = section.child(
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
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .when(is_rtl, |s| s.flex_row_reverse())
                                    .child(
                                        div()
                                            .size(px(14.0))
                                            .rounded_sm()
                                            .bg(parse_hex_color(&original))
                                            .border_1()
                                            .border_color(theme.border),
                                    )
                                    .child(
                                        div()
                                            .font_family(".SystemUIFont")
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.text_secondary)
                                            .child(original.clone()),
                                    )
                                    .child(div().text_color(theme.text_muted).child("→"))
                                    .child(
                                        div()
                                            .size(px(14.0))
                                            .rounded_sm()
                                            .bg(parse_hex_color(&current))
                                            .border_1()
                                            .border_color(theme.border),
                                    )
                                    .child(
                                        div()
                                            .font_family(".SystemUIFont")
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(theme.text_primary)
                                            .child(current.clone()),
                                    ),
                            )
                            .child(if is_overridden {
                                div()
                                    .id(SharedString::from(format!("palette-reset-{idx}")))
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .border_1()
                                    .border_color(theme.border)
                                    .hover(|s| s.text_color(theme.danger).border_color(theme.danger))
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, _, cx| {
                                            this.state.set_palette_override(&original_owned, None);
                                            cx.notify();
                                        }),
                                    )
                                    .child(t!("palette_reset_one").to_string())
                                    .into_any_element()
                            } else {
                                div().into_any_element()
                            }),
                    )
                    .child(render_palette_picker_panel(state, &current, theme, window, cx)),
            );
        }
    }

    // Preset palettes.
    section = section.child(
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .child(t!("palette_presets").to_string()),
            )
            .children(preset_palettes().into_iter().map(|preset| {
                let preset_colors: Vec<String> =
                    preset.colors.iter().map(|c| c.to_string()).collect();
                let preset_id = preset.id;
                let preset_name = preset.name;
                div()
                    .id(SharedString::from(format!("palette-preset-{preset_id}")))
                    .w_full()
                    .px_2()
                    .py_1p5()
                    .rounded_lg()
                    .flex()
                    .items_center()
                    .justify_between()
                    .when(is_rtl, |s| s.flex_row_reverse())
                    .bg(theme.surface)
                    .border_1()
                    .border_color(theme.border)
                    .hover(|s| s.border_color(theme.accent))
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.state.apply_palette_preset(&preset_colors);
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .when(is_rtl, |s| s.flex_row_reverse())
                            .children(
                                preset
                                    .colors
                                    .iter()
                                    .map(|c| {
                                        div()
                                            .size(px(PRESET_SWATCH))
                                            .rounded_sm()
                                            .bg(parse_hex_color(c))
                                            .border_1()
                                            .border_color(theme.border)
                                    })
                                    .collect::<Vec<_>>(),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_secondary)
                            .child(preset_name.to_string()),
                    )
            })),
    );

    // Reset all.
    if has_override {
        section = section.child(
            div()
                .id("palette-reset-all-btn")
                .w_full()
                .h(px(28.0))
                .rounded_lg()
                .flex()
                .items_center()
                .justify_center()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .bg(theme.surface)
                .text_color(theme.text_secondary)
                .border_1()
                .border_color(theme.border)
                .hover(|s| {
                    s.bg(theme.surface_hover)
                        .text_color(theme.danger)
                        .border_color(theme.danger)
                })
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.state.reset_palette();
                        cx.notify();
                    }),
                )
                .child(t!("palette_reset").to_string()),
        );
    }

    section.into_any_element()
}
