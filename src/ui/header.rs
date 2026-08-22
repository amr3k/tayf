use gpui::prelude::*;
use gpui::*;
use rust_i18n::t;

use crate::config::Theme;
use crate::state::{ActiveModal, AppState};
use crate::ui::theme::ThemeColors;
use crate::ui::MainView;

pub fn render_header(
    state: &AppState,
    theme: &ThemeColors,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let is_rtl = crate::i18n::is_rtl();
    let has_file = state.animation.is_some();

    div()
        .id("header-bar")
        .w_full()
        .h(px(48.0))
        .flex()
        .items_center()
        .justify_between()
        .when(is_rtl, |s| s.flex_row_reverse())
        .px_4()
        .bg(theme.surface)
        .border_b_1()
        .border_color(theme.border)
        .child(
            // Left / Start actions
            div()
                .flex()
                .items_center()
                .gap_2()
                .when(is_rtl, |s| s.flex_row_reverse())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .when(is_rtl, |s| s.flex_row_reverse())
                        .child(
                            div()
                                .size(px(24.0))
                                .rounded_md()
                                .bg(theme.accent)
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(theme.accent_text)
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .child("A"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child("AnimaView"),
                        ),
                )
                .children(if has_file {
                    Some(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .when_else(is_rtl, |s| s.mr_4(), |s| s.ml_4())
                            .when(is_rtl, |s| s.flex_row_reverse())
                            .child(
                                render_header_btn(
                                    "close-file-btn",
                                    "✕",
                                    theme,
                                    cx.listener(|this, _, _, cx| {
                                        this.state.is_theme_dropdown_open = false;
                                        this.state.reset();
                                        cx.notify();
                                    }),
                                ),
                            )
                            .child(
                                render_header_btn(
                                    "export-btn",
                                    "⤓",
                                    theme,
                                    cx.listener(|this, _, _, cx| {
                                        this.state.is_theme_dropdown_open = false;
                                        this.state.active_modal = ActiveModal::Export;
                                        cx.notify();
                                    }),
                                ),
                            )
                            .child(
                                render_header_btn(
                                    "toggle-sidebar-btn",
                                    "⚙",
                                    theme,
                                    cx.listener(|this, _, _, cx| {
                                        this.state.is_theme_dropdown_open = false;
                                        this.state.is_sidebar_open = !this.state.is_sidebar_open;
                                        cx.notify();
                                    }),
                                ),
                            ),
                    )
                } else {
                    None
                }),
        )
        .child(
            // Right actions
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .when(is_rtl, |s| s.flex_row_reverse())
                .child(
                    render_header_btn(
                        "open-file-action-btn",
                        "📂",
                        theme,
                        cx.listener(|this, _, _, cx| {
                            this.state.is_theme_dropdown_open = false;
                            this.open_file_dialog(cx);
                        }),
                    ),
                )
                .child(render_theme_dropdown(state, theme, is_rtl, cx))
                .child(
                    render_header_btn(
                        "preferences-btn",
                        "🛠",
                        theme,
                        cx.listener(|this, _, _, cx| {
                            this.state.is_theme_dropdown_open = false;
                            this.state.active_modal = ActiveModal::Preferences;
                            cx.notify();
                        }),
                    ),
                )
                .child(
                    render_header_btn(
                        "about-btn",
                        "ℹ",
                        theme,
                        cx.listener(|this, _, _, cx| {
                            this.state.is_theme_dropdown_open = false;
                            this.state.active_modal = ActiveModal::About;
                            cx.notify();
                        }),
                    ),
                ),
        )
        .into_any_element()
}

fn render_theme_dropdown(
    state: &AppState,
    theme: &ThemeColors,
    is_rtl: bool,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let current_theme = state.config.theme;
    let is_open = state.is_theme_dropdown_open;

    let (icon, label) = match current_theme {
        Theme::System => ("💻", t!("theme_system")),
        Theme::Light => ("☀️", t!("theme_light")),
        Theme::Dark => ("🌙", t!("theme_dark")),
    };

    div()
        .id("theme-dropdown-container")
        .relative()
        .child(
            div()
                .id("theme-dropdown-trigger")
                .h(px(32.0))
                .px_2p5()
                .gap_1p5()
                .rounded_lg()
                .flex()
                .items_center()
                .justify_center()
                .when(is_rtl, |s| s.flex_row_reverse())
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(if is_open { theme.text_primary } else { theme.text_secondary })
                .bg(if is_open { theme.surface_active } else { theme.surface })
                .border_1()
                .border_color(if is_open { theme.accent } else { theme.border })
                .hover(|s| s.bg(theme.surface_hover).text_color(theme.text_primary))
                .active(|s| s.bg(theme.surface_active))
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.state.is_theme_dropdown_open = !this.state.is_theme_dropdown_open;
                        cx.notify();
                    }),
                )
                .child(
                    div()
                        .text_xs()
                        .child(icon),
                )
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .child(label.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("▾"),
                ),
        )
        .children(if is_open {
            Some(
                deferred(
                    div()
                        .id("theme-dropdown-menu")
                        .absolute()
                        .top(px(36.0))
                        .when_else(is_rtl, |s| s.left_0(), |s| s.right_0())
                        .w(px(136.0))
                        .p_1()
                        .rounded_xl()
                        .bg(theme.surface)
                        .border_1()
                        .border_color(theme.border)
                        .shadow_xl()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .child(render_theme_option(
                            "theme-opt-system",
                            "💻",
                            t!("theme_system").to_string(),
                            current_theme == Theme::System,
                            Theme::System,
                            theme,
                            is_rtl,
                            cx,
                        ))
                        .child(render_theme_option(
                            "theme-opt-light",
                            "☀️",
                            t!("theme_light").to_string(),
                            current_theme == Theme::Light,
                            Theme::Light,
                            theme,
                            is_rtl,
                            cx,
                        ))
                        .child(render_theme_option(
                            "theme-opt-dark",
                            "🌙",
                            t!("theme_dark").to_string(),
                            current_theme == Theme::Dark,
                            Theme::Dark,
                            theme,
                            is_rtl,
                            cx,
                        )),
                )
            )
        } else {
            None
        })
}

fn render_theme_option(
    id: &'static str,
    icon: &'static str,
    label: String,
    is_selected: bool,
    target_theme: Theme,
    theme: &ThemeColors,
    is_rtl: bool,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    div()
        .id(id)
        .w_full()
        .h(px(30.0))
        .px_2p5()
        .rounded_lg()
        .flex()
        .items_center()
        .justify_between()
        .when(is_rtl, |s| s.flex_row_reverse())
        .text_xs()
        .font_weight(if is_selected { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
        .text_color(if is_selected { theme.accent } else { theme.text_primary })
        .bg(if is_selected { theme.surface_active } else { theme.surface })
        .hover(|s| s.bg(theme.surface_hover))
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                this.state.update_theme(target_theme);
                cx.notify();
            }),
        )
        .child(
            div()
                .flex_1()
                .flex()
                .items_center()
                .gap_2()
                .when(is_rtl, |s| s.flex_row_reverse())
                .child(
                    div()
                        .size(px(16.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(icon),
                )
                .child(
                    div()
                        .child(label),
                ),
        )
        .child(
            div()
                .w(px(14.0))
                .flex()
                .items_center()
                .justify_center()
                .text_xs()
                .text_color(theme.accent)
                .child(if is_selected { "✓" } else { "" }),
        )
}

fn render_header_btn(
    id: &'static str,
    icon: &'static str,
    theme: &ThemeColors,
    handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .size(px(32.0))
        .rounded_lg()
        .flex()
        .items_center()
        .justify_center()
        .text_sm()
        .text_color(theme.text_secondary)
        .bg(theme.surface)
        .hover(|s| s.bg(theme.surface_hover).text_color(theme.text_primary))
        .active(|s| s.bg(theme.surface_active))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, handler)
        .child(icon)
}
