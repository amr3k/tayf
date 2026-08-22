use gpui::*;
use image::{Frame, RgbaImage};
use std::sync::Arc;

use crate::state::AppState;
use crate::ui::theme::{ThemeColors, parse_hex_color};

pub fn render_animation_view(
    state: &mut AppState,
    theme: &ThemeColors,
    is_dark: bool,
    _cx: &mut Context<crate::ui::MainView>,
) -> impl IntoElement {
    let has_file = state.animation.is_some();
    let bg_hex = state.effective_canvas_background(is_dark).to_string();
    let bg_color = parse_hex_color(&bg_hex);

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

    let frame = state.current_frame;
    let mut rendered_image: Option<Arc<RenderImage>> = None;

    if let Some(anim) = &mut state.animation {
        let orig_w = anim.metadata.width.max(10.0);
        let orig_h = anim.metadata.height.max(10.0);

        // Render at crisp resolution (e.g. 1000px max dimension or native)
        let render_w = (orig_w.max(800.0).min(2048.0)).round() as u32;
        let aspect = orig_w / orig_h;
        let render_h = ((render_w as f32 / aspect).round() as u32).max(1);

        if let Ok(rgba_bytes) = anim.render_frame_rgba(frame, render_w, render_h) {
            if let Some(rgba_img) = RgbaImage::from_raw(render_w, render_h, rgba_bytes.to_vec()) {
                let img_frame = Frame::new(rgba_img);
                rendered_image = Some(Arc::new(RenderImage::new([img_frame])));
            }
        }
    }

    div()
        .id("animation-canvas-container")
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(bg_color)
        .p_4()
        .child(
            if let Some(img_source) = rendered_image {
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        img(img_source)
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .id("lottie-frame"),
                    )
                    .into_any_element()
            } else {
                div().size_full().into_any_element()
            }
        )
        .into_any_element()
}
