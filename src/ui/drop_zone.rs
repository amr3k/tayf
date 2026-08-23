use gpui::prelude::*;
use gpui::*;
use rust_i18n::t;

use crate::state::AppState;
use crate::ui::theme::ThemeColors;
use crate::ui::MainView;

fn load_error_message(path: &std::path::Path, err: &anyhow::Error) -> String {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    let err_str = err.to_string();
    if err_str.contains("File too large") {
        t!("file_too_large", size = "100").to_string()
    } else if err_str.contains("Invalid or unsupported")
        || err_str.contains("Failed to extract")
        || err_str.contains("ThorVG")
    {
        format!("{} — {}", file_name, t!("invalid_file"))
    } else {
        format!("{} — {}", file_name, err_str)
    }
}

pub fn render_drop_zone(
    state: &AppState,
    theme: &ThemeColors,
    is_maximized: bool,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let has_error = state
        .status_message
        .as_ref()
        .is_some_and(|(_, is_err)| *is_err);

    let error_msg = state
        .status_message
        .as_ref()
        .filter(|(_, is_err)| *is_err)
        .map(|(m, _)| m.clone());

    div()
        .id("drop-zone-container")
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .p_8()
        .bg(theme.background)
        .when(!is_maximized, |s| s.rounded_b_2xl())
        .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
            if let Some(path) = paths.paths().first() {
                if let Err(e) = this.state.load_file(path) {
                    let msg = load_error_message(path, &e);
                    this.state.status_message = Some((msg, true));
                    tracing::warn!("Failed to load {:?}: {:?}", path, e);
                }
                cx.notify();
            }
        }))
        .child(if has_error {
            // Error state card — explicitly tells user file is invalid
            let msg = error_msg.clone().unwrap_or_else(|| t!("invalid_file").to_string());
            div()
                .id("drop-card-error")
                .w(px(480.0))
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_4()
                .p_8()
                .rounded_2xl()
                .border_2()
                .border_color(theme.danger)
                .bg(theme.surface)
                .shadow_lg()
                .child(
                    div()
                        .size(px(64.0))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(rgba(0xef44441a))
                        .border_1()
                        .border_color(rgba(0xef444433))
                        .child(
                            crate::ui::icon::render_icon(crate::ui::icon::Icon::Cancel)
                                .size(px(32.0))
                                .text_color(theme.danger),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.text_primary)
                                .child(t!("invalid_file_title").to_string()),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.text_secondary)
                                .text_center()
                                .child(msg),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .text_center()
                                .child(t!("invalid_file_description").to_string()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .id("choose-file-btn-error")
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
                                .child(t!("try_another_file").to_string()),
                        )
                        .child(
                            div()
                                .id("dismiss-error-btn")
                                .px_4()
                                .py_2p5()
                                .rounded_lg()
                                .border_1()
                                .border_color(theme.border)
                                .bg(theme.surface)
                                .hover(|s| s.bg(theme.surface_hover))
                                .text_color(theme.text_secondary)
                                .font_weight(FontWeight::MEDIUM)
                                .text_sm()
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.state.status_message = None;
                                        cx.notify();
                                    }),
                                )
                                .child(t!("dismiss").to_string()),
                        ),
                )
                .into_any_element()
        } else {
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
                )
                .into_any_element()
        })
}
