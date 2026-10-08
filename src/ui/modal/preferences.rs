use gpui::prelude::*;
use gpui::*;
use rust_i18n::t;

use crate::config::Theme;
use crate::state::{AppState, ColorPickerSlot};
use crate::ui::color_picker::render_background_options;
use crate::ui::theme::ThemeColors;
use crate::ui::MainView;

pub fn render_preferences_modal(
    state: &AppState,
    theme: &ThemeColors,
    window: &mut Window,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let is_rtl = crate::i18n::is_rtl();
    let current_theme = state.config.theme;
    let current_lang = state.config.lang.clone();
    let light_bg = state.config.canvas_background_color.clone();
    let dark_bg = state.config.canvas_background_color_dark.clone();

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
                            Some(crate::ui::icon::Icon::Computer),
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
                            Some(crate::ui::icon::Icon::Sun),
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
                            Some(crate::ui::icon::Icon::Moon),
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
                            None,
                            t!("language_en").to_string(),
                            current_lang == "en",
                            theme,
                            cx.listener(|this, _, window, cx| {
                                this.state.update_language("en");
                                window.set_window_title(&t!("app_name"));
                                cx.notify();
                            }),
                        ))
                        .child(render_theme_button(
                            "lang-opt-ar",
                            None,
                            t!("language_ar").to_string(),
                            current_lang == "ar",
                            theme,
                            cx.listener(|this, _, window, cx| {
                                this.state.update_language("ar");
                                window.set_window_title(&t!("app_name"));
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
                    // White / Black / Custom color options
                    render_background_options(
                        state,
                        &light_bg,
                        ColorPickerSlot::PreferencesLight,
                        false,
                        theme,
                        window,
                        cx,
                    ),
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
                    // White / Black / Custom color options
                    render_background_options(
                        state,
                        &dark_bg,
                        ColorPickerSlot::PreferencesDark,
                        true,
                        theme,
                        window,
                        cx,
                    ),
                ),
        )
        .into_any_element()
}

fn render_theme_button(
    id: &'static str,
    icon: Option<crate::ui::icon::Icon>,
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
        .flex()
        .items_center()
        .justify_center()
        .gap_1p5()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .border_1()
        .border_color(if is_active {
            theme.accent
        } else {
            theme.border
        })
        .bg(if is_active {
            theme.surface_active
        } else {
            theme.surface
        })
        .text_color(if is_active {
            theme.accent
        } else {
            theme.text_secondary
        })
        .hover(|s| s.bg(theme.surface_hover))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, handler)
        .children(icon.map(|ico| {
            crate::ui::icon::render_icon(ico)
                .size(px(14.0))
                .text_color(if is_active {
                    theme.accent
                } else {
                    theme.text_secondary
                })
        }))
        .child(label)
}
