use gpui::prelude::*;
use gpui::*;
use rust_i18n::t;

use crate::config::{HexColor, Theme};
use crate::state::AppState;
use crate::ui::theme::{ThemeColors, parse_hex_color};
use crate::ui::MainView;

pub fn render_preferences_modal(
    state: &AppState,
    theme: &ThemeColors,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let is_rtl = crate::i18n::is_rtl();
    let current_theme = state.config.theme;
    let current_lang = state.config.lang.clone();
    let light_bg = state.config.canvas_background_color.clone();
    let dark_bg = state.config.canvas_background_color_dark.clone();

    let light_presets = [
        "#FFFFFF", "#F8FAFC", "#F1F5F9", "#E2E8F0",
        "#FEF3C7", "#DCFCE7", "#E0E7FF", "#FCE7F3",
    ];

    let dark_presets = [
        "#0F1115", "#181A20", "#1E293B", "#000000",
        "#450A0A", "#064E3B", "#1E1B4B", "#3B0764",
    ];

    div()
        .flex()
        .flex_col()
        .gap_6()
        .child(
            // Theme Section
            div()
                .flex()
                .flex_col()
                .gap_2p5()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_muted)
                        .child(t!("theme").to_uppercase()),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .when(is_rtl, |s| s.flex_row_reverse())
                        .child(render_theme_button(
                            "theme-opt-system",
                            t!("theme_system").to_string(),
                            current_theme == Theme::System,
                            theme,
                            cx.listener(|this, _, _, cx| {
                                this.state.update_theme(Theme::System);
                                cx.notify();
                            }),
                        ))
                        .child(render_theme_button(
                            "theme-opt-light",
                            t!("theme_light").to_string(),
                            current_theme == Theme::Light,
                            theme,
                            cx.listener(|this, _, _, cx| {
                                this.state.update_theme(Theme::Light);
                                cx.notify();
                            }),
                        ))
                        .child(render_theme_button(
                            "theme-opt-dark",
                            t!("theme_dark").to_string(),
                            current_theme == Theme::Dark,
                            theme,
                            cx.listener(|this, _, _, cx| {
                                this.state.update_theme(Theme::Dark);
                                cx.notify();
                            }),
                        )),
                ),
        )
        .child(
            // Language Section
            div()
                .flex()
                .flex_col()
                .gap_2p5()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_muted)
                        .child(t!("language").to_uppercase()),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .when(is_rtl, |s| s.flex_row_reverse())
                        .child(render_theme_button(
                            "lang-opt-en",
                            t!("language_en").to_string(),
                            current_lang == "en",
                            theme,
                            cx.listener(|this, _, _, cx| {
                                this.state.update_language("en");
                                cx.notify();
                            }),
                        ))
                        .child(render_theme_button(
                            "lang-opt-ar",
                            t!("language_ar").to_string(),
                            current_lang == "ar",
                            theme,
                            cx.listener(|this, _, _, cx| {
                                this.state.update_language("ar");
                                cx.notify();
                            }),
                        )),
                ),
        )
        .child(
            // Canvas Background (Light)
            div()
                .flex()
                .flex_col()
                .gap_2p5()
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
                                .child(t!("canvas_background_light").to_uppercase()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family(".SystemUIFont")
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child(light_bg.as_str().to_string()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .children(light_presets.iter().map(|&hex| {
                            let is_active = light_bg.as_str().eq_ignore_ascii_case(hex);
                            let swatch_color = parse_hex_color(hex);
                            let hex_color = HexColor::new(hex);

                            div()
                                .id(SharedString::from(format!("light-bg-{}", hex)))
                                .size(px(28.0))
                                .rounded_md()
                                .bg(swatch_color)
                                .border_2()
                                .border_color(if is_active { theme.accent } else { theme.border })
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.state.update_canvas_color(hex_color.clone(), false);
                                        cx.notify();
                                    }),
                                )
                        })),
                ),
        )
        .child(
            // Canvas Background (Dark)
            div()
                .flex()
                .flex_col()
                .gap_2p5()
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
                                .child(t!("canvas_background_dark").to_uppercase()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_family(".SystemUIFont")
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child(dark_bg.as_str().to_string()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .children(dark_presets.iter().map(|&hex| {
                            let is_active = dark_bg.as_str().eq_ignore_ascii_case(hex);
                            let swatch_color = parse_hex_color(hex);
                            let hex_color = HexColor::new(hex);

                            div()
                                .id(SharedString::from(format!("dark-bg-{}", hex)))
                                .size(px(28.0))
                                .rounded_md()
                                .bg(swatch_color)
                                .border_2()
                                .border_color(if is_active { theme.accent } else { theme.border })
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.state.update_canvas_color(hex_color.clone(), true);
                                        cx.notify();
                                    }),
                                )
                        })),
                ),
        )
        .into_any_element()
}

fn render_theme_button(
    id: &'static str,
    label: String,
    is_active: bool,
    theme: &ThemeColors,
    handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex_1()
        .py_2()
        .px_3()
        .rounded_lg()
        .text_center()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .border_1()
        .border_color(if is_active { theme.accent } else { theme.border })
        .bg(if is_active { theme.surface_active } else { theme.surface })
        .text_color(if is_active { theme.accent } else { theme.text_secondary })
        .hover(|s| s.bg(theme.surface_hover))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, handler)
        .child(label)
}
