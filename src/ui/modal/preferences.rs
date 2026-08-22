use gpui::*;
use rust_i18n::t;

use crate::config::Theme;
use crate::state::AppState;
use crate::ui::theme::{ThemeColors, parse_hex_color};
use crate::ui::MainView;

pub fn render_preferences_modal(
    state: &AppState,
    theme: &ThemeColors,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let current_theme = state.config.theme;
    let current_lang = state.config.lang.clone();
    let light_bg = state.config.canvas_background_color.clone();
    let dark_bg = state.config.canvas_background_color_dark.clone();

    let color_presets = [
        "#FFFFFF", "#F8FAFC", "#F1F5F9", "#E2E8F0",
        "#0F1115", "#181A20", "#1E293B", "#000000",
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
                                .text_color(theme.text_primary)
                                .child(light_bg.clone()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .children(color_presets[0..4].iter().map(|&hex| {
                            let is_active = light_bg.eq_ignore_ascii_case(hex);
                            let swatch_color = parse_hex_color(hex);
                            let hex_string = hex.to_string();

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
                                        this.state.update_canvas_color(hex_string.clone(), false);
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
                                .text_color(theme.text_primary)
                                .child(dark_bg.clone()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .children(color_presets[4..8].iter().map(|&hex| {
                            let is_active = dark_bg.eq_ignore_ascii_case(hex);
                            let swatch_color = parse_hex_color(hex);
                            let hex_string = hex.to_string();

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
                                        this.state.update_canvas_color(hex_string.clone(), true);
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
