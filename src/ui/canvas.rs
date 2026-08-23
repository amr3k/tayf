use gpui::*;
use image::{Frame, RgbaImage};
use std::sync::Arc;

use crate::state::AppState;
use crate::ui::theme::ThemeColors;

pub fn render_animation_view(
    state: &mut AppState,
    theme: &ThemeColors,
    is_dark: bool,
    cx: &mut Context<crate::ui::MainView>,
) -> impl IntoElement {
    let has_file = state.animation.is_some();
    let bg_color = state.effective_canvas_background(is_dark).to_hsla();

    if !has_file {
        return div()
            .id("empty-canvas")
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(theme.background)
            .into_any_element();
    }

    let view = cx.entity().clone();

    div()
        .id("animation-canvas-container")
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(bg_color)
        .p_4()
        .child(
            canvas(
                move |_bounds, _window, _cx| {},
                move |bounds, _, window, cx| {
                    view.update(cx, |this, _| {
                        if let Some(anim) = &mut this.state.animation {
                            let frame = this.state.current_frame;
                            let orig_w = anim.metadata.width.max(1.0);
                            let orig_h = anim.metadata.height.max(1.0);
                            let scale_factor = window.scale_factor();

                            let avail_w: f32 = bounds.size.width.into();
                            let avail_h: f32 = bounds.size.height.into();

                            if avail_w <= 0.0 || avail_h <= 0.0 {
                                return;
                            }

                            let scale = (avail_w / orig_w).min(avail_h / orig_h);
                            let fit_w = (orig_w * scale).max(1.0);
                            let fit_h = (orig_h * scale).max(1.0);

                            let render_w = ((fit_w * scale_factor).round() as u32).clamp(1, 4096);
                            let render_h = ((fit_h * scale_factor).round() as u32).clamp(1, 4096);

                            if let Ok(rgba_bytes) =
                                anim.render_frame_rgba(frame, render_w, render_h)
                            {
                                if let Some(rgba_img) =
                                    RgbaImage::from_raw(render_w, render_h, rgba_bytes.to_vec())
                                {
                                    let img_frame = Frame::new(rgba_img);
                                    let new_render_image = Arc::new(RenderImage::new([img_frame]));

                                    // Evict previous frame from GPU sprite atlas to prevent memory leak
                                    if let Some(prev) = this.previous_rendered_image.take() {
                                        let _ = window.drop_image(prev);
                                    }

                                    let origin_x = bounds.origin.x + px((avail_w - fit_w) / 2.0);
                                    let origin_y = bounds.origin.y + px((avail_h - fit_h) / 2.0);
                                    let draw_bounds = Bounds {
                                        origin: Point {
                                            x: origin_x,
                                            y: origin_y,
                                        },
                                        size: Size {
                                            width: px(fit_w),
                                            height: px(fit_h),
                                        },
                                    };

                                    let _ = window.paint_image(
                                        draw_bounds,
                                        Corners::default(),
                                        new_render_image.clone(),
                                        0,
                                        false,
                                    );

                                    this.previous_rendered_image = Some(new_render_image);
                                }
                            }
                        }
                    });
                },
            )
            .size_full(),
        )
        .into_any_element()
}
