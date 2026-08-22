use gpui::prelude::*;
use gpui::*;
use rust_i18n::t;

use crate::state::AppState;
use crate::ui::theme::ThemeColors;
use crate::ui::MainView;

pub fn render_playback_controls(
    state: &AppState,
    theme: &ThemeColors,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let anim = match &state.animation {
        Some(a) => a,
        None => return div().into_any_element(),
    };

    let total_frames = anim.metadata.total_frames.max(1.0);
    let current_frame = state.current_frame;
    let progress_ratio = (current_frame / (total_frames - 1.0).max(1.0)).clamp(0.0, 1.0);

    let current_seconds = (current_frame / anim.metadata.fps.max(1.0)).max(0.0);
    let total_seconds = anim.metadata.duration_seconds;

    let is_playing = state.is_playing;
    let loop_playback = state.loop_playback;
    let current_speed = state.speed;

    div()
        .id("playback-controls-bar")
        .w_full()
        .flex()
        .flex_col()
        .gap_3()
        .p_4()
        .bg(theme.surface)
        .border_t_1()
        .border_color(theme.border)
        .child(
            // Scrubber Bar & Timestamps
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .when(crate::i18n::is_rtl(), |s| s.flex_row_reverse())
                        .text_xs()
                        .text_color(theme.text_secondary)
                        .child(
                            div()
                                .font_family(".SystemUIFont")
                                .child(format!("{:.2}s / {:.2}s", current_seconds, total_seconds)),
                        )
                        .child(
                            div()
                                .font_family(".SystemUIFont")
                                .child(format!("{} / {}", current_frame.round() as u32, total_frames.round() as u32)),
                        ),
                )
                .child({
                    let view = cx.entity().clone();
                    div()
                        .id("scrubber-track-container")
                        .relative()
                        .w_full()
                        .h(px(14.0))
                        .flex()
                        .items_center()
                        .cursor_pointer()
                        .child(
                            canvas(
                                move |bounds, _window, cx| {
                                    view.update(cx, |this, _| {
                                        this.scrub_track_bounds = Some(bounds);
                                    });
                                },
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .size_full(),
                        )
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                                if let Some(bounds) = this.scrub_track_bounds {
                                    let width: f32 = bounds.size.width.into();
                                    let x: f32 = e.position.x.into();
                                    let origin_x: f32 = bounds.origin.x.into();
                                    if width > 0.0 {
                                        let ratio = ((x - origin_x) / width).clamp(0.0, 1.0);
                                        this.handle_scrub_ratio(ratio, cx);
                                    }
                                }
                            }),
                        )
                        .on_mouse_move(
                            cx.listener(move |this, e: &MouseMoveEvent, _, cx| {
                                if e.pressed_button == Some(MouseButton::Left) {
                                    if let Some(bounds) = this.scrub_track_bounds {
                                        let width: f32 = bounds.size.width.into();
                                        let x: f32 = e.position.x.into();
                                        let origin_x: f32 = bounds.origin.x.into();
                                        if width > 0.0 {
                                            let ratio = ((x - origin_x) / width).clamp(0.0, 1.0);
                                            this.handle_scrub_ratio(ratio, cx);
                                        }
                                    }
                                }
                            }),
                        )
                        .child(
                            div()
                                .id("scrubber-track")
                                .h(px(8.0))
                                .w_full()
                                .bg(theme.surface_hover)
                                .rounded_full()
                                .relative()
                                .child(
                                    div()
                                        .id("scrubber-fill")
                                        .h_full()
                                        .w(relative(progress_ratio))
                                        .bg(theme.accent)
                                        .rounded_full()
                                        .relative()
                                        .child(
                                            div()
                                                .id("scrubber-thumb")
                                                .size(px(14.0))
                                                .rounded_full()
                                                .bg(theme.accent_text)
                                                .border_2()
                                                .border_color(theme.accent)
                                                .shadow_md()
                                                .absolute()
                                                .top(px(-3.0))
                                                .right(px(-7.0)),
                                        ),
                                ),
                        )
                }),
        )
        .child(
            // Transport Action Buttons
            div()
                .flex()
                .items_center()
                .justify_between()
                .when(crate::i18n::is_rtl(), |s| s.flex_row_reverse())
                .child(
                    // Speed Selector buttons
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(render_speed_chip(0.5, current_speed, theme, cx))
                        .child(render_speed_chip(1.0, current_speed, theme, cx))
                        .child(render_speed_chip(1.5, current_speed, theme, cx))
                        .child(render_speed_chip(2.0, current_speed, theme, cx)),
                )
                .child(
                    // Center transport buttons (Step Back, Play/Pause, Step Forward)
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .when(crate::i18n::is_rtl(), |s| s.flex_row_reverse())
                        .child(
                            div()
                                .id("step-back-btn")
                                .size(px(32.0))
                                .rounded_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(theme.surface_hover)
                                .hover(|s| s.bg(theme.surface_active))
                                .text_color(theme.text_primary)
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.state.step_frame(-1.0);
                                        cx.notify();
                                    }),
                                )
                                .child(
                                    crate::ui::icon::render_icon(crate::ui::icon::Icon::Previous)
                                        .size(px(16.0))
                                        .text_color(theme.text_primary),
                                ),
                        )
                        .child(
                            div()
                                .id("play-pause-btn")
                                .size(px(44.0))
                                .rounded_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(theme.accent)
                                .hover(|s| s.bg(theme.accent_hover))
                                .active(|s| s.bg(theme.accent_active))
                                .text_color(theme.accent_text)
                                .shadow_md()
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.state.toggle_play_pause();
                                        cx.notify();
                                    }),
                                )
                                .child(
                                    crate::ui::icon::render_icon(if is_playing {
                                        crate::ui::icon::Icon::Pause
                                    } else {
                                        crate::ui::icon::Icon::Play
                                    })
                                    .size(px(20.0))
                                    .text_color(theme.accent_text),
                                ),
                        )
                        .child(
                            div()
                                .id("step-forward-btn")
                                .size(px(32.0))
                                .rounded_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .bg(theme.surface_hover)
                                .hover(|s| s.bg(theme.surface_active))
                                .text_color(theme.text_primary)
                                .cursor_pointer()
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _, _, cx| {
                                        this.state.step_frame(1.0);
                                        cx.notify();
                                    }),
                                )
                                .child(
                                    crate::ui::icon::render_icon(crate::ui::icon::Icon::Next)
                                        .size(px(16.0))
                                        .text_color(theme.text_primary),
                                ),
                        ),
                )
                .child(
                    // Right options: Loop Toggle
                    div()
                        .id("loop-btn")
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_3()
                        .py_1p5()
                        .rounded_lg()
                        .border_1()
                        .border_color(if loop_playback { theme.accent } else { theme.border })
                        .bg(if loop_playback { theme.surface_active } else { theme.surface })
                        .text_color(if loop_playback { theme.accent } else { theme.text_secondary })
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.state.loop_playback = !this.state.loop_playback;
                                cx.notify();
                            }),
                        )
                        .child(
                            crate::ui::icon::render_icon(crate::ui::icon::Icon::Repeat)
                                .size(px(14.0))
                                .text_color(if loop_playback { theme.accent } else { theme.text_secondary }),
                        )
                        .child(t!("loop_playback").to_string()),
                ),
        )
        .into_any_element()
}

fn render_speed_chip(
    speed: f32,
    current_speed: f32,
    theme: &ThemeColors,
    cx: &mut Context<MainView>,
) -> impl IntoElement {
    let is_selected = (speed - current_speed).abs() < 0.05;

    div()
        .id(SharedString::from(format!("speed-{:.1}", speed)))
        .px_2p5()
        .py_1()
        .rounded_md()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .bg(if is_selected { theme.accent } else { theme.surface_hover })
        .text_color(if is_selected { theme.accent_text } else { theme.text_secondary })
        .hover(|s| s.bg(if is_selected { theme.accent_hover } else { theme.surface_active }))
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                this.state.set_speed(speed);
                cx.notify();
            }),
        )
        .child(format!("{:.1}x", speed))
}
