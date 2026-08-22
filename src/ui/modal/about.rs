use gpui::*;
use rust_i18n::t;

use crate::ui::theme::ThemeColors;
use crate::ui::MainView;

pub fn render_about_modal(
    theme: &ThemeColors,
    _cx: &mut Context<MainView>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .text_center()
        .gap_4()
        .p_2()
        .child(
            // App Logo Badge
            div()
                .size(px(72.0))
                .rounded_2xl()
                .bg(theme.accent)
                .flex()
                .items_center()
                .justify_center()
                .text_color(theme.accent_text)
                .text_3xl()
                .font_weight(FontWeight::BOLD)
                .shadow_xl()
                .child("A"),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child("AnimaView"),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_full()
                                .bg(theme.surface_hover)
                                .border_1()
                                .border_color(theme.border)
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child(t!("version", version => "0.1.0").to_string()),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_secondary)
                        .max_w(px(320.0))
                        .child(t!("app_description").to_string()),
                ),
        )
        .child(
            // External Links
            div()
                .flex()
                .items_center()
                .gap_3()
                .mt_2()
                .child(
                    div()
                        .id("github-link-btn")
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_4()
                        .py_2()
                        .rounded_lg()
                        .bg(theme.surface_hover)
                        .hover(|s| s.bg(theme.surface_active))
                        .text_color(theme.text_primary)
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, |_, _, _| {
                            let _ = open::that("https://github.com/amr3k/AnimaView");
                        })
                        .child("🐙")
                        .child(t!("github").to_string()),
                )
                .child(
                    div()
                        .id("author-website-btn")
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_4()
                        .py_2()
                        .rounded_lg()
                        .bg(theme.surface_hover)
                        .hover(|s| s.bg(theme.surface_active))
                        .text_color(theme.text_primary)
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, |_, _, _| {
                            let _ = open::that("https://a3k.me");
                        })
                        .child("🌐")
                        .child(t!("author_website").to_string()),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(theme.text_muted)
                .mt_4()
                .child(t!("created_by", name => "Amr").to_string()),
        )
        .into_any_element()
}
