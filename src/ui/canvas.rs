use gpui::*;
use image::{Frame, RgbaImage};
use std::sync::Arc;

use crate::state::AppState;
use crate::ui::theme::{parse_hex_color, ThemeColors};

const CHECKER_CELL_PX: f32 = 16.0;

pub fn render_animation_view(
    state: &mut AppState,
    theme: &ThemeColors,
    is_dark: bool,
    cx: &mut Context<crate::ui::MainView>,
) -> impl IntoElement {
    let has_file = state.animation.is_some();
    let bg_opt = state
        .effective_canvas_background(is_dark)
        .map(|c| c.to_hsla());

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

    match bg_opt {
        Some(bg_color) => div()
            .id("animation-canvas-container")
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(bg_color)
            .p_4()
            .child(render_animation_canvas(view))
            .into_any_element(),
        None => div()
            .id("animation-canvas-container")
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .relative()
            .bg(gpui::white())
            .p_4()
            .child(
                canvas(
                    |_, _, _| (),
                    |bounds, _, window, _| {
                        paint_checkerboard(bounds, window);
                    },
                )
                .absolute()
                .inset_0(),
            )
            .child(render_animation_canvas(view))
            .into_any_element(),
    }
}

/// Light-gray squares over the white container background to signal
/// transparency.
fn paint_checkerboard(bounds: Bounds<Pixels>, window: &mut Window) {
    let origin_x: f32 = bounds.origin.x.into();
    let origin_y: f32 = bounds.origin.y.into();
    let width: f32 = bounds.size.width.into();
    let height: f32 = bounds.size.height.into();
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    let dark = parse_hex_color("#CBD5E1");
    let cols = (width / CHECKER_CELL_PX).ceil() as i32;
    let rows = (height / CHECKER_CELL_PX).ceil() as i32;
    for row in 0..rows {
        for col in 0..cols {
            if (row + col) % 2 == 0 {
                continue;
            }
            let x = origin_x + col as f32 * CHECKER_CELL_PX;
            let y = origin_y + row as f32 * CHECKER_CELL_PX;
            let w = CHECKER_CELL_PX.min((origin_x + width) - x);
            let h = CHECKER_CELL_PX.min((origin_y + height) - y);
            if w <= 0.0 || h <= 0.0 {
                continue;
            }
            window.paint_quad(gpui::fill(
                Bounds {
                    origin: Point {
                        x: px(x),
                        y: px(y),
                    },
                    size: Size {
                        width: px(w),
                        height: px(h),
                    },
                },
                dark,
            ));
        }
    }
}

fn render_animation_canvas(view: Entity<crate::ui::MainView>) -> Canvas<()> {
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
            .size_full()
}
