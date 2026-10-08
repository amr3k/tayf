use gpui::prelude::*;
use gpui::*;
use rust_i18n::t;

use crate::config::HexColor;
use crate::state::{AppState, ColorPickerSlot};
use crate::ui::theme::{parse_hex_color, ThemeColors};
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
        self.custom_picker.last_outside_pos = None;
        if self.custom_picker.open_slot == Some(slot) {
            self.custom_picker.open_slot = None;
            self.custom_picker.hex_draft = None;
            self.custom_picker.hex_pristine = false;
            return;
        }

        self.custom_picker.open_slot = Some(slot);
        self.custom_picker.hex_draft = None;
        self.custom_picker.hex_pristine = false;
        self.custom_picker.hex_target_is_dark = target_is_dark;
        let (h, s, _) = rgb_to_hsv(hex_to_rgb(self.canvas_color_for(target_is_dark).as_str()));
        if s > 0.002 {
            self.custom_picker.working_hue = h;
        }
    }

    pub fn close_custom_picker(&mut self) {
        self.custom_picker.open_slot = None;
        self.custom_picker.hex_draft = None;
        self.custom_picker.hex_pristine = false;
        self.custom_picker.last_outside_pos = None;
    }

    /// Toggle-swatches click handler. The panel's outside-click listener runs
    /// in the capture phase before this bubble handler, so a click on the
    /// swatch that just dismissed the panel must stay closed instead of
    /// reopening in the same gesture.
    pub fn handle_custom_swatch_click(
        &mut self,
        slot: ColorPickerSlot,
        target_is_dark: bool,
        position: Point<Pixels>,
    ) {
        if self.custom_picker.last_outside_pos == Some(position) {
            self.custom_picker.last_outside_pos = None;
            self.custom_picker.open_slot = None;
            self.custom_picker.hex_draft = None;
            self.custom_picker.hex_pristine = false;
            return;
        }
        self.toggle_custom_picker(slot, target_is_dark);
    }

    /// Returns true when a hex field is being edited for `slot`.
    pub fn is_hex_editing(&self, slot: ColorPickerSlot) -> bool {
        self.custom_picker.open_slot == Some(slot) && self.custom_picker.hex_draft.is_some()
    }

    /// Text to show in the hex field: the in-progress draft while editing,
    /// otherwise the applied color.
    pub fn hex_field_text(&self, current: &HexColor) -> String {
        self.custom_picker
            .hex_draft
            .clone()
            .unwrap_or_else(|| current.as_str().to_string())
    }

    /// Whether the current hex draft is invalid (false when not editing).
    pub fn hex_draft_is_invalid(&self) -> bool {
        match &self.custom_picker.hex_draft {
            Some(draft) => !HexColor::is_valid(draft),
            None => false,
        }
    }

    /// Begins hex editing from the current color.
    pub fn start_hex_edit(&mut self, current: &HexColor, target_is_dark: bool) {
        self.custom_picker.hex_draft = Some(current.as_str().to_string());
        self.custom_picker.hex_target_is_dark = target_is_dark;
        self.custom_picker.hex_pristine = true;
    }

    /// Cancels hex editing, reverting the field to the applied color.
    pub fn cancel_hex_edit(&mut self) {
        self.custom_picker.hex_draft = None;
        self.custom_picker.hex_pristine = false;
    }

    /// Applies `text` if valid (normalizing case and leading `#`), updating
    /// the canvas color and working hue. Returns true when applied.
    fn apply_valid_hex_text(&mut self, text: &str, target_is_dark: bool) -> bool {
        if !HexColor::is_valid(text) {
            return false;
        }
        let normalized = HexColor::normalize(text);
        let (h, s, _) = rgb_to_hsv(hex_to_rgb(&normalized));
        if s > 0.002 {
            self.custom_picker.working_hue = h;
        }
        self.update_canvas_color(HexColor::new(normalized), target_is_dark);
        true
    }

    /// Appends a typed character to the hex draft. The first hex keystroke
    /// after focus replaces the mirrored value so typing `34C3EB` over
    /// `#FFFFFF` yields `#34C3EB` instead of appending.
    pub fn push_hex_char(&mut self, ch: char, current: &HexColor) {
        let target_is_dark = self.custom_picker.hex_target_is_dark;
        let draft = self
            .custom_picker
            .hex_draft
            .clone()
            .unwrap_or_else(|| current.as_str().to_string());
        let pristine = self.custom_picker.hex_pristine && self.custom_picker.hex_draft.is_some();

        let Some(next) = advance_hex_draft(&draft, pristine, ch) else {
            return;
        };
        let applied = self.apply_valid_hex_text(&next, target_is_dark);
        // Store the draft regardless so invalid states stay visible;
        // valid states were already applied live (SV/hue stay in sync).
        let _ = applied;
        self.custom_picker.hex_draft = Some(next);
        self.custom_picker.hex_pristine = false;
    }

    /// Deletes the last character of the hex draft, live-applying when the
    /// remainder is still a valid color.
    pub fn pop_hex_char(&mut self) {
        let Some(mut draft) = self.custom_picker.hex_draft.clone() else {
            return;
        };
        draft.pop();
        let target_is_dark = self.custom_picker.hex_target_is_dark;
        if HexColor::is_valid(&draft) {
            self.apply_valid_hex_text(&draft, target_is_dark);
        }
        self.custom_picker.hex_draft = Some(draft);
        self.custom_picker.hex_pristine = false;
    }

    /// Replaces the draft with pasted text (filtered to hex chars + `#`),
    /// live-applying when valid.
    pub fn set_hex_draft(&mut self, raw: &str) {
        let filtered = sanitize_hex_paste(raw);
        let text = if filtered.is_empty() {
            raw.trim().to_string()
        } else {
            filtered
        };
        let target_is_dark = self.custom_picker.hex_target_is_dark;
        if HexColor::is_valid(&text) {
            self.apply_valid_hex_text(&text, target_is_dark);
        }
        self.custom_picker.hex_draft = Some(text);
        self.custom_picker.hex_pristine = false;
    }

    /// Commits the draft on Enter: valid values are applied (already live)
    /// and editing ends; invalid values stay visible with an error.
    /// Returns true when editing ended.
    pub fn commit_hex_edit(&mut self) -> bool {
        let Some(draft) = self.custom_picker.hex_draft.clone() else {
            return true;
        };
        if draft.trim().is_empty() {
            return false;
        }
        let target_is_dark = self.custom_picker.hex_target_is_dark;
        if self.apply_valid_hex_text(&draft, target_is_dark) {
            self.custom_picker.hex_draft = None;
            self.custom_picker.hex_pristine = false;
            true
        } else {
            false
        }
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
        self.custom_picker.hex_draft = None;
        self.custom_picker.hex_pristine = false;
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
        self.custom_picker.hex_draft = None;
        self.custom_picker.hex_pristine = false;
        self.update_canvas_color(HexColor::new(hex), target_is_dark);
    }
}

/// Keeps at most a leading `#` plus 8 hex digits from pasted text.
pub fn sanitize_hex_paste(raw: &str) -> String {
    let trimmed = raw.trim();
    let mut out = String::new();
    for (i, ch) in trimmed.chars().enumerate() {
        if ch == '#' && i == 0 && out.is_empty() {
            out.push('#');
        } else if ch.is_ascii_hexdigit() {
            if out.starts_with('#') {
                if out.len() < 9 {
                    out.push(ch.to_ascii_uppercase());
                }
            } else if out.len() < 8 {
                out.push(ch.to_ascii_uppercase());
            }
        }
        if out.len() >= 9 {
            break;
        }
    }
    // Normalize bare hex to include `#` for display consistency.
    if !out.is_empty() && !out.starts_with('#') {
        out.insert(0, '#');
    }
    out
}

/// Computes the next hex draft for a typed character.
/// Returns `None` when the keystroke is ignored (extra `#`, length cap).
/// Pure (no IO) so the typing sequence is unit-testable.
pub fn advance_hex_draft(draft: &str, pristine: bool, ch: char) -> Option<String> {
    if pristine && ch != '#' {
        // First keystroke after focus replaces the mirrored value.
        return Some(format!("#{}", ch.to_ascii_uppercase()));
    }
    if ch == '#' {
        if draft.is_empty() {
            return Some("#".to_string());
        }
        // Only a leading `#` is meaningful; ignore extras.
        return None;
    }
    if draft.is_empty() {
        return Some(format!("#{}", ch.to_ascii_uppercase()));
    }
    if draft.len() >= 9 {
        return None;
    }
    let mut next = draft.to_string();
    next.push(ch.to_ascii_uppercase());
    if next.len() > 9 {
        next.truncate(9);
    }
    Some(next)
}

/// Best-effort coercion of user-typed hex to a canvas color.
pub fn coerce_hex_input(raw: &str) -> Option<HexColor> {
    if HexColor::is_valid(raw) {
        Some(HexColor::new(HexColor::normalize(raw)))
    } else {
        None
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
                this.state
                    .update_canvas_color(HexColor::new(CANVAS_WHITE), target_is_dark);
                cx.notify();
            }),
        ))
        .child(preset_swatch(
            "bg-opt-black",
            CANVAS_BLACK,
            is_black_active,
            theme,
            cx.listener(move |this, _, _, cx| {
                this.state
                    .update_canvas_color(HexColor::new(CANVAS_BLACK), target_is_dark);
                cx.notify();
            }),
        ))
        .child(custom_swatch(
            "bg-opt-custom",
            !is_white_active && !is_black_active,
            theme,
            cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                this.state
                    .handle_custom_swatch_click(slot, target_is_dark, e.position);
                cx.notify();
            }),
        ));

    if state.custom_picker.open_slot == Some(slot) {
        row = row.child(render_custom_picker_panel(
            state,
            current,
            slot,
            target_is_dark,
            theme,
            cx,
        ));
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
        .border_color(if is_active {
            theme.accent
        } else {
            theme.border
        })
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
        .border_color(if is_active {
            theme.accent
        } else {
            theme.border
        })
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
    slot: ColorPickerSlot,
    target_is_dark: bool,
    theme: &ThemeColors,
    cx: &mut Context<MainView>,
) -> Stateful<Div> {
    let working_hue = state.custom_picker.working_hue;
    let (_, sat, val) = rgb_to_hsv(hex_to_rgb(current.as_str()));
    let is_rtl = crate::i18n::is_rtl();
    let is_invalid = state.hex_draft_is_invalid();
    let is_editing = state.custom_picker.hex_draft.is_some();

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
        .on_mouse_down_out(cx.listener(|this, e: &MouseDownEvent, _, cx| {
            this.state.close_custom_picker();
            this.state.custom_picker.last_outside_pos = Some(e.position);
            cx.notify();
        }))
        .child(render_sv_area(working_hue, sat, val, target_is_dark, cx))
        .child(render_hue_bar(working_hue, target_is_dark, cx))
        .child(
            div()
                .id("custom-picker-footer")
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
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
                                .child(render_hex_input(
                                    state,
                                    current,
                                    slot,
                                    target_is_dark,
                                    is_invalid,
                                    theme,
                                    cx,
                                )),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child(
                                    if is_editing {
                                        t!("hex_input_hint").to_string()
                                    } else {
                                        t!("custom_color").to_string()
                                    },
                                ),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.danger)
                        .when(!is_invalid, |s| s.invisible())
                        .child(t!("invalid_hex").to_string()),
                ),
        )
}

/// Editable hex field: click to edit, type hex digits, Enter to commit,
/// Escape to cancel. Invalid values show a red border and keep the
/// previous color instead of corrupting the config.
fn render_hex_input(
    state: &AppState,
    current: &HexColor,
    slot: ColorPickerSlot,
    target_is_dark: bool,
    is_invalid: bool,
    theme: &ThemeColors,
    cx: &mut Context<MainView>,
) -> Stateful<Div> {
    let is_editing = state.is_hex_editing(slot);
    let text = state.hex_field_text(current);
    let current_owned = current.clone();

    div()
        .id("custom-picker-hex-input")
        .px_2()
        .py_1()
        .rounded_md()
        .flex()
        .items_center()
        .gap_1()
        .font_family(".SystemUIFont")
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .bg(theme.surface)
        .border_1()
        .border_color(if is_invalid {
            theme.danger
        } else if is_editing {
            theme.accent
        } else {
            theme.border
        })
        .text_color(if is_invalid {
            theme.danger
        } else {
            theme.text_primary
        })
        .cursor_text()
        .hover(|s| {
            s.border_color(if is_invalid {
                theme.danger
            } else {
                theme.accent
            })
        })
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, _, cx| {
                if !this.state.is_hex_editing(slot) {
                    this.state.start_hex_edit(&current_owned, target_is_dark);
                    cx.notify();
                }
            }),
        )
        .child(text)
        .children(if is_editing {
            Some(
                div()
                    .w(px(1.0))
                    .h(px(12.0))
                    .bg(if is_invalid {
                        theme.danger
                    } else {
                        theme.accent
                    }),
            )
        } else {
            None
        })
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
        .child(
            div()
                .absolute()
                .inset_0()
                .bg(hsla(working_hue, 1.0, 0.5, 1.0)),
        )
        .child(div().absolute().inset_0().bg(linear_gradient(
            90.0,
            linear_color_stop(hsla(0.0, 0.0, 1.0, 1.0), 0.0),
            linear_color_stop(hsla(0.0, 0.0, 1.0, 0.0), 1.0),
        )))
        .child(div().absolute().inset_0().bg(linear_gradient(
            180.0,
            linear_color_stop(hsla(0.0, 0.0, 0.0, 0.0), 0.0),
            linear_color_stop(hsla(0.0, 0.0, 0.0, 1.0), 1.0),
        )))
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
    use super::{
        advance_hex_draft, coerce_hex_input, hex_to_rgb, hsv_to_hex, rgb_to_hsv, sanitize_hex_paste,
    };

    fn assert_rgb_close(a: (f32, f32, f32), b: (f32, f32, f32)) {
        const TOLERANCE: f32 = 1.0 / 255.0 + 1e-4;
        assert!(
            (a.0 - b.0).abs() < TOLERANCE,
            "r mismatch: {:?} vs {:?}",
            a,
            b
        );
        assert!(
            (a.1 - b.1).abs() < TOLERANCE,
            "g mismatch: {:?} vs {:?}",
            a,
            b
        );
        assert!(
            (a.2 - b.2).abs() < TOLERANCE,
            "b mismatch: {:?} vs {:?}",
            a,
            b
        );
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

    #[test]
    fn coerce_hex_input_normalizes_valid_values() {
        assert_eq!(
            coerce_hex_input("ff0000").unwrap().as_str(),
            "#FF0000"
        );
        assert_eq!(
            coerce_hex_input("#ff0000").unwrap().as_str(),
            "#FF0000"
        );
        assert_eq!(
            coerce_hex_input("34C3EB").unwrap().as_str(),
            "#34C3EB"
        );
    }

    #[test]
    fn coerce_hex_input_rejects_invalid_values() {
        for invalid in ["#xyz", "#12345", "", "   ", "red", "#GGGGGG"] {
            assert!(
                coerce_hex_input(invalid).is_none(),
                "expected None for {invalid}"
            );
        }
    }

    #[test]
    fn sanitize_hex_paste_keeps_leading_hash_and_hex_digits() {
        assert_eq!(sanitize_hex_paste("#34c3eb"), "#34C3EB");
        assert_eq!(sanitize_hex_paste("34C3EB"), "#34C3EB");
        assert_eq!(sanitize_hex_paste("  #ff0000  "), "#FF0000");
        // Non-hex characters are dropped; extra `#` ignored.
        assert_eq!(sanitize_hex_paste("#xyz"), "#");
        // Overlong input is truncated to `#` + 8 digits.
        assert_eq!(sanitize_hex_paste("#123456789ABC"), "#12345678");
    }

    #[test]
    fn typing_six_hex_digits_appends_past_three() {
        // Regression: live-applying `#FFF` must not re-trigger the
        // replace-on-first-type behavior on the 4th keystroke.
        let mut draft = "#FFFFFF".to_string();
        let mut pristine = true;
        let mut applied: Option<String> = None;
        for ch in "ffffff".chars() {
            draft = advance_hex_draft(&draft, pristine, ch).expect("keystroke accepted");
            pristine = false;
            if crate::config::HexColor::is_valid(&draft) {
                applied = Some(crate::config::HexColor::normalize(&draft));
            }
        }
        assert_eq!(draft, "#FFFFFF");
        assert_eq!(applied.as_deref(), Some("#FFFFFF"));
    }
}
