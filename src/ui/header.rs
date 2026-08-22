use gpui::*;

use crate::config::Theme;
use crate::state::ActiveModal;
use crate::ui::theme::ThemeColors;
use crate::ui::MainView;

pub fn render_header(
    has_file: bool,
    theme: &ThemeColors,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    div()
        .id("header-bar")
        .w_full()
        .h(px(48.0))
        .flex()
        .items_center()
        .justify_between()
        .px_4()
        .bg(theme.surface)
        .border_b_1()
        .border_color(theme.border)
        .child(
            // Left actions
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
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
                            .ml_4()
                            .child(
                                render_header_btn(
                                    "close-file-btn",
                                    "✕",
                                    theme,
                                    cx.listener(|this, _, _, cx| {
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
                .child(
                    render_header_btn(
                        "open-file-action-btn",
                        "📂",
                        theme,
                        cx.listener(|this, _, _, cx| {
                            this.open_file_dialog(cx);
                        }),
                    ),
                )
                .child(
                    render_header_btn(
                        "theme-toggle-btn",
                        "🌓",
                        theme,
                        cx.listener(|this, _, _, cx| {
                            let next_theme = match this.state.config.theme {
                                Theme::System => Theme::Dark,
                                Theme::Dark => Theme::Light,
                                Theme::Light => Theme::System,
                            };
                            this.state.update_theme(next_theme);
                            cx.notify();
                        }),
                    ),
                )
                .child(
                    render_header_btn(
                        "preferences-btn",
                        "🛠",
                        theme,
                        cx.listener(|this, _, _, cx| {
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
                            this.state.active_modal = ActiveModal::About;
                            cx.notify();
                        }),
                    ),
                ),
        )
        .into_any_element()
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
