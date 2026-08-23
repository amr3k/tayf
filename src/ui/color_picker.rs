use gpui::prelude::*;
use gpui::*;
use rust_i18n::t;

use crate::config::HexColor;
use crate::state::{AppState, ColorPickerSlot};
use crate::ui::theme::{ThemeColors, parse_hex_color};
use crate::ui::MainView;

pub const CANVAS_WHITE: &str = "#FFFFFF";
pub const CANVAS_BLACK: &str = "#000000";

const SV_AREA_HEIGHT: f32 = 132.0;
const HUE_BAR_HEIGHT: f32 = 12.0;
const SWATCH_SIZE: f32 = 26.0;

/// Hue wheel segments (start, end) as fractions of a full turn.
const HUE_SEGMENTS: [(f32, f32); 6] = [
    (0.0, 1.0 / 6.0),
    (1.0 / 6.0, 2.0 / 6.0),
    (2.0 / 6.0, 3.0 / 6.0),
    (3.0 / 6.0, 4.0 / 6.0),
    (4.0 / 6.0, 5.0 / 6.0),
    (5.0 / 6.0, 1.0),
];

impl AppState {
    /// Expands the custom picker for `slot` (collapsing any other), adopting
    /// the current color's hue when the color is chromatic.
    pub fn toggle_custom_picker(&mut self, slot: ColorPickerSlot, target_is_dark: bool) {
        if self.custom_picker.open_slot == Some(slot) {
            self.custom_picker.open_slot = None;
            return;
        }

        self.custom_picker.open_slot = Some(slot);
        let (h, s, _) = rgb_to_hsv(hex_to_rgb(self.canvas_color_for(target_is_dark).as_str()));
        if s > 0.002 {
            self.custom_picker.working_hue = h;
        }
    }

    pub fn close_custom_picker(&mut self) {
        self.custom_picker.open_slot = None;
    }

    /// Maps a pointer position onto the saturation/value area and stores the
    /// resulting color.
    pub fn apply_custom_sv_pick(&mut self, position: Point<Pixels>, target_is_dark: bool) {
        let Some(bounds) = self.custom_picker.sv_bounds else {
            return;
        };

        let width: f32 = bounds.size.width.into();
        let height: f32 = bounds.size.height.into();
        let origin_x: f32 = bounds.origin.x.into();
        let origin_y: f32 = bounds.origin.y.into();
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        let x: f32 = position.x.into();
        let y: f32 = position.y.into();
        let s = ((x - origin_x) / width).clamp(0.0, 1.0);
        let v = 1.0 - ((y - origin_y) / height).clamp(0.0, 1.0);
        let hex = hsv_to_hex(self.custom_picker.working_hue, s, v);
        self.update_canvas_color(HexColor::new(hex), target_is_dark);
    }

    /// Maps a pointer position onto the hue bar and re-tints the current
    /// color with the picked hue.
    pub fn apply_custom_hue_pick(&mut self, position: Point<Pixels>, target_is_dark: bool) {
        let Some(bounds) = self.custom_picker.hue_bounds else {
            return;
        };

        let width: f32 = bounds.size.width.into();
        let origin_x: f32 = bounds.origin.x.into();
        if width <= 0.0 {
            return;
        }

        let x: f32 = position.x.into();
        let hue = ((x - origin_x) / width).clamp(0.0, 1.0);
        self.custom_picker.working_hue = hue;

        let (_, mut s, mut v) =
            rgb_to_hsv(hex_to_rgb(self.canvas_color_for(target_is_dark).as_str()));
        // Pure white/black carry no meaningful s/v; jump to the vivid variant
        // of the newly picked hue so the change is visible.
        if s <= 0.002 || v <= 0.002 {
            s = 1.0;
            v = 1.0;
        }

        let hex = hsv_to_hex(hue, s, v);
        self.update_canvas_color(HexColor::new(hex), target_is_dark);
    }
}

/// Renders the three canvas background options (white, black, custom) plus
/// the expandable custom picker panel when `slot` is active.
pub fn render_background_options(
    state: &AppState,
    current: &HexColor,
    slot: ColorPickerSlot,
    target_is_dark: bool,
    theme: &ThemeColors,
    cx: &mut Context<MainView>,
) -> Div {
    let is_rtl = crate::i18n::is_rtl();
    let is_white_active = current.as_str().eq_ignore_ascii_case(CANVAS_WHITE);
    let is_black_active = current.as_str().eq_ignore_ascii_case(CANVAS_BLACK);

    let mut row = div()
        .flex()
        .flex_wrap()
        .gap_2()
        .when(is_rtl, |s| s.flex_row_reverse())
        .child(preset_swatch(
            "bg-opt-white",
            CANVAS_WHITE,
            is_white_active,
            theme,
            cx.listener(move |this, _, _, cx| {
                this.state.update_canvas_color(HexColor::new(CANVAS_WHITE), target_is_dark);
                cx.notify();
            }),
        ))
        .child(preset_swatch(
            "bg-opt-black",
            CANVAS_BLACK,
            is_black_active,
            theme,
            cx.listener(move |this, _, _, cx| {
                this.state.update_canvas_color(HexColor::new(CANVAS_BLACK), target_is_dark);
                cx.notify();
            }),
        ))
        .child(custom_swatch(
            "bg-opt-custom",
            !is_white_active && !is_black_active,
            theme,
            cx.listener(move |this, _, _, cx| {
                this.state.toggle_custom_picker(slot, target_is_dark);
                cx.notify();
            }),
        ));

    if state.custom_picker.open_slot == Some(slot) {
        row = row.child(render_custom_picker_panel(state, current, target_is_dark, theme, cx));
    }

    row
}

fn preset_swatch(
    id: &'static str,
    hex: &'static str,
    is_active: bool,
    theme: &ThemeColors,
    handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id(id)
        .size(px(SWATCH_SIZE))
        .rounded_md()
        .bg(parse_hex_color(hex))
        .border_2()
        .border_color(if is_active { theme.accent } else { theme.border })
        .shadow_sm()
        .cursor_pointer()
        .hover(|s| s.border_color(theme.accent))
        .on_mouse_down(MouseButton::Left, handler)
}

fn custom_swatch(
    id: &'static str,
    is_active: bool,
    theme: &ThemeColors,
    handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    div()
        .id(id)
        .size(px(SWATCH_SIZE))
        .rounded_md()
        .flex()
        .items_center()
        .justify_center()
        .bg(theme.surface_hover)
        .border_2()
        .border_color(if is_active { theme.accent } else { theme.border })
        .shadow_sm()
        .cursor_pointer()
        .hover(|s| s.border_color(theme.accent))
        .on_mouse_down(MouseButton::Left, handler)
        .child(
            crate::ui::icon::render_icon(crate::ui::icon::Icon::PaintBoard)
                .size(px(14.0))
                .text_color(if is_active {
                    theme.accent
                } else {
                    theme.text_secondary
                }),
        )
}

fn render_custom_picker_panel(
    state: &AppState,
    current: &HexColor,
    target_is_dark: bool,
    theme: &ThemeColors,
    cx: &mut Context<MainView>,
) -> Stateful<Div> {
    let working_hue = state.custom_picker.working_hue;
    let (_, sat, val) = rgb_to_hsv(hex_to_rgb(current.as_str()));
    let is_rtl = crate::i18n::is_rtl();

    div()
        .id("custom-picker-panel")
        .mt_3()
        .w_full()
        .flex()
        .flex_col()
        .gap_3()
        .p_3()
        .rounded_xl()
        .bg(theme.background)
        .border_1()
        .border_color(theme.border)
        .child(render_sv_area(working_hue, sat, val, target_is_dark, cx))
        .child(render_hue_bar(working_hue, target_is_dark, cx))
        .child(
            div()
                .id("custom-picker-footer")
                .flex()
                .items_center()
                .justify_between()
                .when(is_rtl, |s| s.flex_row_reverse())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .when(is_rtl, |s| s.flex_row_reverse())
                        .child(
                            div()
                                .size(px(14.0))
                                .rounded_sm()
                                .bg(parse_hex_color(current.as_str()))
                                .border_1()
                                .border_color(theme.border),
                        )
                        .child(
                            div()
                                .font_family(".SystemUIFont")
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text_primary)
                                .child(current.as_str().to_string()),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(t!("custom_color").to_string()),
                ),
        )
}

/// Saturation (x) / value (y) area: pure-hue base with white and black
/// gradient overlays, matching classic HSV picker squares.
fn render_sv_area(
    working_hue: f32,
    sat: f32,
    val: f32,
    target_is_dark: bool,
    cx: &mut Context<MainView>,
) -> Stateful<Div> {
    let view = cx.entity().clone();

    div()
        .id("custom-picker-sv-area")
        .relative()
        .w_full()
        .h(px(SV_AREA_HEIGHT))
        .rounded_lg()
        .overflow_hidden()
        .cursor_pointer()
        .child(
            canvas(
                move |bounds, _window, cx| {
                    view.update(cx, |this, _| {
                        this.state.custom_picker.sv_bounds = Some(bounds);
                    });
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full(),
        )
        .child(div().absolute().inset_0().bg(hsla(working_hue, 1.0, 0.5, 1.0)))
        .child(
            div().absolute().inset_0().bg(linear_gradient(
                90.0,
                linear_color_stop(hsla(0.0, 0.0, 1.0, 1.0), 0.0),
                linear_color_stop(hsla(0.0, 0.0, 1.0, 0.0), 1.0),
            )),
        )
        .child(
            div().absolute().inset_0().bg(linear_gradient(
                180.0,
                linear_color_stop(hsla(0.0, 0.0, 0.0, 0.0), 0.0),
                linear_color_stop(hsla(0.0, 0.0, 0.0, 1.0), 1.0),
            )),
        )
        .child(sv_thumb(sat, val))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                this.state.apply_custom_sv_pick(e.position, target_is_dark);
                cx.notify();
            }),
        )
        .on_mouse_move(cx.listener(move |this, e: &MouseMoveEvent, _, cx| {
            if e.pressed_button == Some(MouseButton::Left) {
                this.state.apply_custom_sv_pick(e.position, target_is_dark);
                cx.notify();
            }
        }))
}

fn sv_thumb(sat: f32, val: f32) -> Div {
    div()
        .absolute()
        .left(relative(sat))
        .top(relative(1.0 - val))
        .child(
            div()
                .absolute()
                .left(px(-8.0))
                .top(px(-8.0))
                .size(px(16.0))
                .rounded_full()
                .border_2()
                .border_color(hsla(0.0, 0.0, 1.0, 0.95))
                .bg(hsla(0.0, 0.0, 0.0, 0.12))
                .shadow_md(),
        )
}

/// Hue bar: six abutting two-stop gradients forming a smooth wheel strip.
/// The thumb lives in an unclipped wrapper so it can overflow the pill.
fn render_hue_bar(
    working_hue: f32,
    target_is_dark: bool,
    cx: &mut Context<MainView>,
) -> Stateful<Div> {
    let view = cx.entity().clone();

    div()
        .id("custom-picker-hue-bar")
        .relative()
        .w_full()
        .h(px(HUE_BAR_HEIGHT))
        .cursor_pointer()
        .child(
            canvas(
                move |bounds, _window, cx| {
                    view.update(cx, |this, _| {
                        this.state.custom_picker.hue_bounds = Some(bounds);
                    });
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full(),
        )
        .child(
            div()
                .h_full()
                .w_full()
                .rounded_full()
                .overflow_hidden()
                .flex()
                .children(HUE_SEGMENTS.map(|(start, end)| {
                    div().flex_1().h_full().bg(linear_gradient(
                        90.0,
                        linear_color_stop(hsla(start, 1.0, 0.5, 1.0), 0.0),
                        linear_color_stop(hsla(end, 1.0, 0.5, 1.0), 1.0),
                    ))
                })),
        )
        .child(
            div()
                .absolute()
                .top_0()
                .h_full()
                .left(relative(working_hue))
                .child(
                    div()
                        .absolute()
                        .left(px(-5.0))
                        .top(px(-4.0))
                        .w(px(10.0))
                        .h(px(20.0))
                        .rounded_md()
                        .border_2()
                        .border_color(hsla(0.0, 0.0, 1.0, 0.95))
                        .bg(hsla(0.0, 0.0, 0.0, 0.12))
                        .shadow_sm(),
                ),
        )
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                this.state.apply_custom_hue_pick(e.position, target_is_dark);
                cx.notify();
            }),
        )
        .on_mouse_move(cx.listener(move |this, e: &MouseMoveEvent, _, cx| {
            if e.pressed_button == Some(MouseButton::Left) {
                this.state.apply_custom_hue_pick(e.position, target_is_dark);
                cx.notify();
            }
        }))
}

fn hex_to_rgb(hex: &str) -> (f32, f32, f32) {
    let rgba = Rgba::from(parse_hex_color(hex));
    (rgba.r, rgba.g, rgba.b)
}

fn rgb_to_hsv(rgb: (f32, f32, f32)) -> (f32, f32, f32) {
    let (r, g, b) = rgb;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let hue = if delta <= 1e-6 {
        0.0
    } else {
        let sector = if (max - r).abs() < 1e-6 {
            ((g - b) / delta).rem_euclid(6.0)
        } else if (max - g).abs() < 1e-6 {
            (b - r) / delta + 2.0
        } else {
            (r - g) / delta + 4.0
        };
        (sector / 6.0).fract()
    };

    let sat = if max <= 1e-6 { 0.0 } else { delta / max };
    (hue, sat, max)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

fn hsv_to_hex(h: f32, s: f32, v: f32) -> String {
    let (r, g, b) = hsv_to_rgb(h.clamp(0.0, 1.0), s.clamp(0.0, 1.0), v.clamp(0.0, 1.0));
    format!(
        "#{:02X}{:02X}{:02X}",
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8
    )
}

#[cfg(test)]
mod tests {
    // NOTE: no glob import here — gpui re-exports its own `test` attribute
    // macro which would shadow the built-in #[test] and recurse forever.
    use super::{hsv_to_hex, hex_to_rgb, rgb_to_hsv};

    fn assert_rgb_close(a: (f32, f32, f32), b: (f32, f32, f32)) {
        const TOLERANCE: f32 = 1.0 / 255.0 + 1e-4;
        assert!((a.0 - b.0).abs() < TOLERANCE, "r mismatch: {:?} vs {:?}", a, b);
        assert!((a.1 - b.1).abs() < TOLERANCE, "g mismatch: {:?} vs {:?}", a, b);
        assert!((a.2 - b.2).abs() < TOLERANCE, "b mismatch: {:?} vs {:?}", a, b);
    }

    #[test]
    fn hsv_to_hex_primary_colors() {
        assert_eq!(hsv_to_hex(0.0, 0.0, 1.0), "#FFFFFF");
        assert_eq!(hsv_to_hex(0.0, 0.0, 0.0), "#000000");
        assert_eq!(hsv_to_hex(0.0, 1.0, 1.0), "#FF0000");
        assert_eq!(hsv_to_hex(1.0 / 3.0, 1.0, 1.0), "#00FF00");
        assert_eq!(hsv_to_hex(2.0 / 3.0, 1.0, 1.0), "#0000FF");
    }

    #[test]
    fn hsv_out_of_range_inputs_are_clamped() {
        assert_eq!(hsv_to_hex(-0.5, 1.5, 2.0), hsv_to_hex(0.0, 1.0, 1.0));
        assert_eq!(hsv_to_hex(1.5, -1.0, -1.0), "#000000");
    }

    #[test]
    fn hex_roundtrip_preserves_color() {
        for hex in ["#FF0000", "#34C3EB", "#123ABC", "#7F7F7F", "#F59E0B"] {
            let original = hex_to_rgb(hex);
            let (h, s, v) = rgb_to_hsv(original);
            let restored = hex_to_rgb(&hsv_to_hex(h, s, v));
            assert_rgb_close(original, restored);
        }
    }

    #[test]
    fn achromatic_colors_have_zero_hue_without_nan() {
        for hex in ["#000000", "#FFFFFF", "#808080"] {
            let (r, g, b) = hex_to_rgb(hex);
            let (h, s, v) = rgb_to_hsv((r, g, b));
            assert!(h.is_finite() && s.is_finite() && v.is_finite());
            assert!(s.abs() < 1e-4 || v == 0.0);
        }
    }
}
