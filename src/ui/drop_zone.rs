use gpui::*;
use rust_i18n::t;

use crate::ui::theme::ThemeColors;
use crate::ui::MainView;

pub fn render_drop_zone(theme: &ThemeColors, cx: &mut Context<MainView>) -> impl IntoElement {
    div()
        .id("drop-zone-container")
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .p_8()
        .bg(theme.background)
        .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
            if let Some(path) = paths.paths().first() {
                if let Err(e) = this.state.load_file(path) {
                    this.state.status_message = Some((format!("Failed to load file: {}", e), true));
                }
                cx.notify();
            }
        }))
        .child(
            div()
                .id("drop-card")
                .w(px(460.0))
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .p_10()
                .rounded_2xl()
                .border_2()
                .border_dashed()
                .border_color(theme.border)
                .bg(theme.surface)
                .shadow_lg()
                .child(
                    div()
                        .size(px(64.0))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(theme.surface_hover)
                        .border_1()
                        .border_color(theme.border)
                        .child(
                            crate::ui::icon::render_icon(crate::ui::icon::Icon::Upload)
                                .size(px(32.0))
                                .text_color(theme.accent),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text_primary)
                                .child(t!("drag_drop_prompt").to_string()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child(t!("supports_formats").to_string()),
                        ),
                )
                .child(
                    div()
                        .id("choose-file-btn")
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_5()
                        .py_2p5()
                        .rounded_lg()
                        .bg(theme.accent)
                        .hover(|s| s.bg(theme.accent_hover))
                        .active(|s| s.bg(theme.accent_active))
                        .text_color(theme.accent_text)
                        .font_weight(FontWeight::MEDIUM)
                        .text_sm()
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.open_file_dialog(cx);
                            }),
                        )
                        .child(
                            crate::ui::icon::render_icon(crate::ui::icon::Icon::Folder)
                                .size(px(16.0))
                                .text_color(theme.accent_text),
                        )
                        .child(t!("choose_file").to_string()),
                ),
        )
}
