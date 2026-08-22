pub mod about;
pub mod export;
pub mod preferences;

use gpui::prelude::*;
use gpui::*;

use crate::state::ActiveModal;
use crate::ui::theme::ThemeColors;
use crate::ui::MainView;

pub fn render_modal_container(
    title: String,
    content: impl IntoElement,
    theme: &ThemeColors,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    div()
        .id("modal-backdrop")
        .absolute()
        .inset_0()
        .bg(theme.modal_backdrop)
        .flex()
        .items_center()
        .justify_center()
        .p_4()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _, _, cx| {
                this.state.active_modal = ActiveModal::None;
                cx.notify();
            }),
        )
        .child(
            div()
                .id("modal-card")
                .w(px(460.0))
                .max_h(px(580.0))
                .flex()
                .flex_col()
                .rounded_2xl()
                .bg(theme.surface)
                .border_1()
                .border_color(theme.border)
                .shadow_2xl()
                .overflow_hidden()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .child(
                    // Modal Header
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .when(crate::i18n::is_rtl(), |s| s.flex_row_reverse())
                        .px_6()
                        .py_4()
                        .border_b_1()
                        .border_color(theme.border)
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child(title),
                        )
                        .child(
                            div()
                                .id("modal-close-btn")
                                .size(px(28.0))
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
                                        this.state.active_modal = ActiveModal::None;
                                        cx.notify();
                                    }),
                                )
                                .child("✕"),
                        ),
                )
                .child(
                    // Modal Body
                    div()
                        .p_6()
                        .overflow_hidden()
                        .child(content),
                ),
        )
}
