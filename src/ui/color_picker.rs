use std::time::Instant;

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
        self.custom_picker.last_closed_slot = None;
        self.custom_picker.last_closed_palette_index = None;
        if self.custom_picker.open_slot == Some(slot) {
            self.custom_picker.open_slot = None;
            self.cancel_hex_edit();
            if slot == ColorPickerSlot::Palette {
                self.palette_edit_index = None;
            }
            return;
        }

        self.custom_picker.open_slot = Some(slot);
        self.cancel_hex_edit();
        self.custom_picker.hex_target_is_dark = target_is_dark;
        if slot != ColorPickerSlot::Palette {
            self.palette_edit_index = None;
        }
        if let Some(current) = self.canvas_color_for(target_is_dark) {
            let (h, s, _) = rgb_to_hsv(hex_to_rgb(current.as_str()));
            if s > 0.002 {
                self.custom_picker.working_hue = h;
            }
        }
    }

    pub fn close_custom_picker(&mut self) {
        self.custom_picker.open_slot = None;
        self.custom_picker.last_outside_pos = None;
        self.custom_picker.last_closed_slot = None;
        self.custom_picker.last_closed_palette_index = None;
        self.palette_edit_index = None;
        self.cancel_hex_edit();
    }

    /// Records an outside-click dismissal, remembering what was open so a
    /// click on a *different* swatch can replace the picker instead of
    /// staying closed.
    pub fn close_custom_picker_from_outside(&mut self, position: Point<Pixels>) {
        self.custom_picker.last_closed_slot = self.custom_picker.open_slot;
        self.custom_picker.last_closed_palette_index = self.palette_edit_index;
        self.custom_picker.open_slot = None;
        self.palette_edit_index = None;
        self.cancel_hex_edit();
        self.custom_picker.last_outside_pos = Some(position);
    }

    /// Toggle-swatches click handler. The panel's outside-click listener runs
    /// in the capture phase before this bubble handler, so a click on the
    /// swatch that just dismissed the panel must stay closed instead of
    /// reopening in the same gesture. A click on a *different* swatch
    /// replaces the picker with the new target.
    pub fn handle_custom_swatch_click(
        &mut self,
        slot: ColorPickerSlot,
        target_is_dark: bool,
        position: Point<Pixels>,
    ) {
        if self.custom_picker.last_outside_pos == Some(position) {
            let closed_slot = self.custom_picker.last_closed_slot;
            self.custom_picker.last_outside_pos = None;
            self.custom_picker.last_closed_slot = None;
            self.custom_picker.last_closed_palette_index = None;
            if closed_slot == Some(slot) {
                self.custom_picker.open_slot = None;
                self.cancel_hex_edit();
                return;
            }
            self.toggle_custom_picker(slot, target_is_dark);
            return;
        }
        self.custom_picker.last_closed_slot = None;
        self.custom_picker.last_closed_palette_index = None;
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

    /// Begins hex editing from the current color, caret at the end.
    pub fn start_hex_edit(&mut self, current: &HexColor, target_is_dark: bool) {
        self.custom_picker.hex_draft = Some(current.as_str().to_string());
        self.custom_picker.hex_cursor = current.as_str().len();
        self.custom_picker.hex_anchor = None;
        self.custom_picker.hex_selecting = false;
        self.custom_picker.hex_target_is_dark = target_is_dark;
        self.reset_hex_blink();
    }

    /// Cancels hex editing, reverting the field to the applied color.
    pub fn cancel_hex_edit(&mut self) {
        self.custom_picker.hex_draft = None;
        self.custom_picker.hex_cursor = 0;
        self.custom_picker.hex_anchor = None;
        self.custom_picker.hex_selecting = false;
    }

    /// Restarts the caret blink (visible) after an edit.
    fn reset_hex_blink(&mut self) {
        self.custom_picker.hex_blink_visible = true;
        self.custom_picker.last_hex_blink = Instant::now();
    }

    /// Clamped caret offset for the current draft.
    fn hex_cursor_clamped(&self) -> usize {
        match &self.custom_picker.hex_draft {
            Some(draft) => clamp_hex_index(draft, self.custom_picker.hex_cursor),
            None => 0,
        }
    }

    /// Current selection as an ordered byte range, if any.
    pub fn hex_selection_range(&self) -> Option<(usize, usize)> {
        let draft = self.custom_picker.hex_draft.as_ref()?;
        let anchor = self.custom_picker.hex_anchor?;
        let cursor = self.hex_cursor_clamped();
        let anchor = clamp_hex_index(draft, anchor);
        if anchor == cursor {
            None
        } else {
            Some((anchor.min(cursor), anchor.max(cursor)))
        }
    }

    /// Places the caret (clearing any selection).
    pub fn place_hex_caret(&mut self, index: usize) {
        let Some(draft) = self.custom_picker.hex_draft.clone() else {
            return;
        };
        self.custom_picker.hex_cursor = clamp_hex_index(&draft, index);
        self.custom_picker.hex_anchor = None;
        self.reset_hex_blink();
    }

    /// Extends (or starts) a selection to `index`, keeping the anchor.
    pub fn extend_hex_selection(&mut self, index: usize) {
        let Some(draft) = self.custom_picker.hex_draft.clone() else {
            return;
        };
        let index = clamp_hex_index(&draft, index);
        if self.custom_picker.hex_anchor.is_none() {
            self.custom_picker.hex_anchor = Some(self.hex_cursor_clamped());
        }
        self.custom_picker.hex_cursor = index;
        self.reset_hex_blink();
    }

    /// Inserts a typed character at the caret, replacing any selection.
    /// Hex letters are uppercased so the draft matches the normalized form.
    pub fn insert_hex_char(&mut self, ch: char) {
        if ch.is_control() {
            return;
        }
        let Some(draft) = self.custom_picker.hex_draft.clone() else {
            return;
        };
        let cursor = self.hex_cursor_clamped();
        let (start, end) = self.hex_selection_range().unwrap_or((cursor, cursor));
        if draft.len() - (end - start) + ch.len_utf8() > MAX_HEX_DRAFT_LEN {
            return;
        }
        let (next, next_cursor) = hex_replace_range(&draft, (start, end), &ch.to_string());
        self.apply_hex_edit(next, next_cursor);
    }

    /// Deletes the selection, or the character before the caret.
    pub fn delete_hex_backward(&mut self) {
        let Some(draft) = self.custom_picker.hex_draft.clone() else {
            return;
        };
        let cursor = self.hex_cursor_clamped();
        let (start, end) = match self.hex_selection_range() {
            Some(range) => range,
            None => {
                if cursor == 0 {
                    return;
                }
                (step_hex_index(&draft, cursor, -1), cursor)
            }
        };
        let (next, next_cursor) = hex_replace_range(&draft, (start, end), "");
        self.apply_hex_edit(next, next_cursor);
    }

    /// Deletes the selection, or the character after the caret.
    pub fn delete_hex_forward(&mut self) {
        let Some(draft) = self.custom_picker.hex_draft.clone() else {
            return;
        };
        let cursor = self.hex_cursor_clamped();
        let (start, end) = match self.hex_selection_range() {
            Some(range) => range,
            None => {
                if cursor >= draft.len() {
                    return;
                }
                (cursor, step_hex_index(&draft, cursor, 1))
            }
        };
        let (next, next_cursor) = hex_replace_range(&draft, (start, end), "");
        self.apply_hex_edit(next, next_cursor);
    }

    /// Moves the caret by `delta` characters, optionally extending the selection.
    pub fn move_hex_cursor(&mut self, delta: isize, extend: bool) {
        let Some(draft) = self.custom_picker.hex_draft.clone() else {
            return;
        };
        let cursor = self.hex_cursor_clamped();
        let next = step_hex_index(&draft, cursor, delta);
        if extend {
            if self.custom_picker.hex_anchor.is_none() {
                self.custom_picker.hex_anchor = Some(cursor);
            }
            self.custom_picker.hex_cursor = next;
            if self.custom_picker.hex_anchor == Some(next) {
                self.custom_picker.hex_anchor = None;
            }
        } else {
            self.custom_picker.hex_cursor = next;
            self.custom_picker.hex_anchor = None;
        }
        self.reset_hex_blink();
    }

    /// Moves the caret to the start/end of the draft, optionally selecting.
    pub fn move_hex_home_end(&mut self, to_start: bool, extend: bool) {
        if self.custom_picker.hex_draft.is_none() {
            return;
        };
        let cursor = self.hex_cursor_clamped();
        let next = if to_start {
            0
        } else {
            self.custom_picker.hex_draft.as_ref().map(|d| d.len()).unwrap_or(0)
        };
        if extend {
            if self.custom_picker.hex_anchor.is_none() {
                self.custom_picker.hex_anchor = Some(cursor);
            }
            self.custom_picker.hex_cursor = next;
            if self.custom_picker.hex_anchor == Some(next) {
                self.custom_picker.hex_anchor = None;
            }
        } else {
            self.custom_picker.hex_cursor = next;
            self.custom_picker.hex_anchor = None;
        }
        self.reset_hex_blink();
    }

    /// Selects the whole draft.
    pub fn select_all_hex(&mut self) {
        let Some(draft) = self.custom_picker.hex_draft.clone() else {
            return;
        };
        self.custom_picker.hex_anchor = Some(0);
        self.custom_picker.hex_cursor = draft.len();
        self.reset_hex_blink();
    }

    /// Text for clipboard copy: the selection, or the whole draft.
    pub fn copyable_hex_text(&self) -> Option<String> {
        let draft = self.custom_picker.hex_draft.as_ref()?;
        match self.hex_selection_range() {
            Some((start, end)) => Some(draft[start..end].to_string()),
            None => Some(draft.clone()),
        }
    }

    /// Pastes text at the caret, replacing any selection (single-line,
    /// truncated to fit). Invalid content stays visible with an error.
    pub fn paste_hex_text(&mut self, raw: &str) {
        let Some(draft) = self.custom_picker.hex_draft.clone() else {
            return;
        };
        let cursor = self.hex_cursor_clamped();
        let (start, end) = self.hex_selection_range().unwrap_or((cursor, cursor));
        let (next, next_cursor) = hex_paste_into(&draft, (start, end), raw);
        self.apply_hex_edit(next, next_cursor);
    }

    /// Stores a new draft after an edit, live-applying when valid so the
    /// SV/hue pickers stay in sync. Invalid drafts stay visible with an error.
    fn apply_hex_edit(&mut self, next: String, cursor: usize) {
        if self.custom_picker.open_slot == Some(ColorPickerSlot::Palette) {
            if HexColor::is_valid(&next) {
                self.apply_valid_palette_hex(&next);
            }
        } else {
            let target_is_dark = self.custom_picker.hex_target_is_dark;
            if HexColor::is_valid(&next) {
                self.apply_valid_hex_text(&next, target_is_dark);
            }
        }
        self.custom_picker.hex_cursor = clamp_hex_index(&next, cursor);
        self.custom_picker.hex_anchor = None;
        self.custom_picker.hex_draft = Some(next);
        self.reset_hex_blink();
    }

    /// Applies `text` if valid (normalizing case and leading `#`), updating
    /// the canvas color and working hue. Returns true when applied.
    fn apply_valid_hex_text(&mut self, text: &str, target_is_dark: bool) -> bool {
        if self.custom_picker.open_slot == Some(ColorPickerSlot::Palette) {
            return self.apply_valid_palette_hex(text);
        }
        if !HexColor::is_valid(text) {
            return false;
        }
        let normalized = HexColor::normalize(text);
        let (h, s, _) = rgb_to_hsv(hex_to_rgb(&normalized));
        if s > 0.002 {
            self.custom_picker.working_hue = h;
        }
        self.update_canvas_color(Some(HexColor::new(normalized)), target_is_dark);
        true
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
        if self.custom_picker.open_slot == Some(ColorPickerSlot::Palette) {
            if self.apply_valid_palette_hex(&draft) {
                self.cancel_hex_edit();
                true
            } else {
                false
            }
        } else {
            let target_is_dark = self.custom_picker.hex_target_is_dark;
            if self.apply_valid_hex_text(&draft, target_is_dark) {
                self.cancel_hex_edit();
                true
            } else {
                false
            }
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
        self.cancel_hex_edit();
        self.update_canvas_color(Some(HexColor::new(hex)), target_is_dark);
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

        let (mut s, mut v) = match self.canvas_color_for(target_is_dark) {
            Some(c) => {
                let (_, s, v) = rgb_to_hsv(hex_to_rgb(c.as_str()));
                (s, v)
            }
            None => (1.0, 1.0),
        };
        // Pure white/black carry no meaningful s/v; jump to the vivid variant
        // of the newly picked hue so the change is visible.
        if s <= 0.002 || v <= 0.002 {
            s = 1.0;
            v = 1.0;
        }

        let hex = hsv_to_hex(hue, s, v);
        self.cancel_hex_edit();
        self.update_canvas_color(Some(HexColor::new(hex)), target_is_dark);
    }

    // ---- Animation palette editing (targets LoadedAnimation, not canvas) ----

    /// Original hex currently targeted by the palette picker, if any.
    pub fn palette_edit_original(&self) -> Option<String> {
        let idx = self.palette_edit_index?;
        self.animation
            .as_ref()
            .and_then(|a| a.palette_colors().get(idx).cloned())
    }

    /// Current (possibly overridden) hex for the palette edit target.
    pub fn palette_edit_current(&self) -> Option<(String, String)> {
        let original = self.palette_edit_original()?;
        let current = self.palette_current(&original);
        Some((original, current))
    }

    /// Opens the palette picker for `index`, adopting the current replacement
    /// color's hue when chromatic. Clicking the active swatch closes it.
    pub fn toggle_palette_picker(&mut self, index: usize) {
        self.custom_picker.last_outside_pos = None;
        self.custom_picker.last_closed_slot = None;
        self.custom_picker.last_closed_palette_index = None;
        if self.custom_picker.open_slot == Some(ColorPickerSlot::Palette)
            && self.palette_edit_index == Some(index)
        {
            self.custom_picker.open_slot = None;
            self.palette_edit_index = None;
            self.cancel_hex_edit();
            return;
        }
        self.custom_picker.open_slot = Some(ColorPickerSlot::Palette);
        self.palette_edit_index = Some(index);
        self.cancel_hex_edit();
        if let Some((_, current)) = self.palette_edit_current() {
            let (h, s, _) = rgb_to_hsv(hex_to_rgb(&current));
            if s > 0.002 {
                self.custom_picker.working_hue = h;
            }
        }
    }

    pub fn handle_palette_swatch_click(&mut self, index: usize, position: Point<Pixels>) {
        if self.custom_picker.last_outside_pos == Some(position) {
            let closed_slot = self.custom_picker.last_closed_slot;
            let closed_index = self.custom_picker.last_closed_palette_index;
            self.custom_picker.last_outside_pos = None;
            self.custom_picker.last_closed_slot = None;
            self.custom_picker.last_closed_palette_index = None;
            if closed_slot == Some(ColorPickerSlot::Palette) && closed_index == Some(index) {
                self.custom_picker.open_slot = None;
                self.palette_edit_index = None;
                self.cancel_hex_edit();
                return;
            }
            self.toggle_palette_picker(index);
            return;
        }
        self.custom_picker.last_closed_slot = None;
        self.custom_picker.last_closed_palette_index = None;
        self.toggle_palette_picker(index);
    }

    fn apply_palette_hex_value(&mut self, normalized: &str) {
        let (h, s, _) = rgb_to_hsv(hex_to_rgb(normalized));
        if s > 0.002 {
            self.custom_picker.working_hue = h;
        }
        if let Some(original) = self.palette_edit_original() {
            let _ = self.set_palette_override(&original, Some(normalized));
        }
    }

    fn apply_valid_palette_hex(&mut self, text: &str) -> bool {
        if !HexColor::is_valid(text) {
            return false;
        }
        // Palette identity is RGB-only; collapse alpha suffixes to `#RRGGBB`.
        let full = HexColor::normalize(text);
        let rgb_only = if full.len() == 9 {
            full[..7].to_string()
        } else if full.len() == 4 {
            // Expand `#RGB` -> `#RRGGBB` for consistent palette keys.
            let c: Vec<char> = full[1..].chars().collect();
            if c.len() == 3 {
                format!("#{}{}{}{}{}{}", c[0], c[0], c[1], c[1], c[2], c[2]).to_uppercase()
            } else {
                full
            }
        } else {
            full
        };
        self.apply_palette_hex_value(&rgb_only);
        true
    }

    /// SV-area pick targeting the palette edit color (live preview reload).
    pub fn apply_palette_sv_pick(&mut self, position: Point<Pixels>) {
        let Some(bounds) = self.custom_picker.sv_bounds else {
            return;
        };
        if self.palette_edit_original().is_none() {
            return;
        }
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
        self.cancel_hex_edit();
        self.apply_palette_hex_value(&hex);
    }

    /// Hue-bar pick targeting the palette edit color.
    pub fn apply_palette_hue_pick(&mut self, position: Point<Pixels>) {
        let Some(bounds) = self.custom_picker.hue_bounds else {
            return;
        };
        let Some((_, current)) = self.palette_edit_current() else {
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
        let (_, mut s, mut v) = rgb_to_hsv(hex_to_rgb(&current));
        if s <= 0.002 || v <= 0.002 {
            s = 1.0;
            v = 1.0;
        }
        let hex = hsv_to_hex(hue, s, v);
        self.cancel_hex_edit();
        self.apply_palette_hex_value(&hex);
    }

    /// Starts hex editing for the palette target (caret at end).
    pub fn start_palette_hex_edit(&mut self, current: &str) {
        self.custom_picker.hex_draft = Some(current.to_string());
        self.custom_picker.hex_cursor = current.len();
        self.custom_picker.hex_anchor = None;
        self.custom_picker.hex_selecting = false;
        self.reset_hex_blink();
    }
}

/// Maximum draft size in bytes: a leading `#` plus 8 hex digits.
pub const MAX_HEX_DRAFT_LEN: usize = 9;

/// Font size of the hex field in px. Must match the `.text_size(px(12.))`
/// style on the field so hit-testing lines up with the rendered text.
pub const HEX_FONT_SIZE_PX: f32 = 12.0;

/// Clamps a byte offset into `draft` (char-boundary safe).
pub fn clamp_hex_index(draft: &str, index: usize) -> usize {
    let mut index = index.min(draft.len());
    while !draft.is_char_boundary(index) {
        index -= 1;
    }
    index
}

/// Steps a caret offset by `delta` characters (negative = left),
/// staying on char boundaries.
pub fn step_hex_index(draft: &str, cursor: usize, delta: isize) -> usize {
    let mut index = clamp_hex_index(draft, cursor);
    if delta > 0 {
        for _ in 0..delta {
            if index >= draft.len() {
                break;
            }
            index += draft[index..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
        }
    } else {
        for _ in 0..-delta {
            if index == 0 {
                break;
            }
            index -= 1;
            while !draft.is_char_boundary(index) {
                index -= 1;
            }
        }
    }
    index
}

/// Replaces `range` in `draft` with `insert`, returning the new text and the
/// caret offset right after the insertion. Replacing a whole `#`-draft with
/// text that lacks `#` keeps the `#` convention (`#FFFFFF` + select-all +
/// `F` -> `#F`). Pure (no IO) for unit tests.
pub fn hex_replace_range(draft: &str, range: (usize, usize), insert: &str) -> (String, usize) {
    let (start, end) = (range.0.min(range.1), range.0.max(range.1));
    let start = clamp_hex_index(draft, start);
    let end = clamp_hex_index(draft, end);
    let insert = if start == 0
        && end == draft.len()
        && draft.starts_with('#')
        && !insert.starts_with('#')
    {
        format!("#{insert}")
    } else {
        insert.to_string()
    };
    // Uppercase hex letters so the draft matches the normalized form.
    let insert: String = insert
        .chars()
        .map(|c| {
            if c.is_ascii_hexdigit() {
                c.to_ascii_uppercase()
            } else {
                c
            }
        })
        .collect();
    let mut next = String::with_capacity(draft.len() + insert.len());
    next.push_str(&draft[..start]);
    next.push_str(&insert);
    next.push_str(&draft[end..]);
    (next, start + insert.len())
}

/// Pastes clipboard text over `range`: first line only, no control chars,
/// truncated to fit. Pure (no IO) for unit tests.
pub fn hex_paste_into(draft: &str, range: (usize, usize), raw: &str) -> (String, usize) {
    let (start, end) = (range.0.min(range.1), range.0.max(range.1));
    let start = clamp_hex_index(draft, start);
    let end = clamp_hex_index(draft, end);
    let chunk: String = raw
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    let full_replace =
        start == 0 && end == draft.len() && draft.starts_with('#') && !chunk.starts_with('#');
    let mut budget = MAX_HEX_DRAFT_LEN.saturating_sub(draft.len() - (end - start));
    if full_replace {
        budget = budget.saturating_sub(1);
    }
    let mut kept = String::new();
    for ch in chunk.chars() {
        if kept.len() + ch.len_utf8() > budget {
            break;
        }
        kept.push(ch);
    }
    hex_replace_range(draft, (start, end), &kept)
}

/// Best-effort coercion of user-typed hex to a canvas color.
pub fn coerce_hex_input(raw: &str) -> Option<HexColor> {
    if HexColor::is_valid(raw) {
        Some(HexColor::new(HexColor::normalize(raw)))
    } else {
        None
    }
}

/// Shapes the hex draft for caret/selection geometry and click hit-testing.
/// Must use the same family/size/weight as the rendered field.
pub fn shape_hex_text(text: &str, color: Hsla, window: &mut Window) -> ShapedLine {
    let mut font = font(".SystemUIFont");
    font.weight = FontWeight::MEDIUM;
    let owned = text.to_owned();
    let runs = if owned.is_empty() {
        vec![]
    } else {
        vec![TextRun {
            len: owned.len(),
            font,
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        }]
    };
    window
        .text_system()
        .shape_line(owned.into(), px(HEX_FONT_SIZE_PX), &runs, None)
}

/// Renders the canvas background options (clear, white, black, custom) plus
/// the expandable custom picker panel when `slot` is active.
/// `current` is `None` for a transparent (checkerboard) background.
pub fn render_background_options(
    state: &AppState,
    current: Option<&HexColor>,
    slot: ColorPickerSlot,
    target_is_dark: bool,
    theme: &ThemeColors,
    window: &mut Window,
    cx: &mut Context<MainView>,
) -> Div {
    let is_rtl = crate::i18n::is_rtl();
    let is_clear_active = current.is_none();
    let is_white_active = current
        .map(|c| c.as_str().eq_ignore_ascii_case(CANVAS_WHITE))
        .unwrap_or(false);
    let is_black_active = current
        .map(|c| c.as_str().eq_ignore_ascii_case(CANVAS_BLACK))
        .unwrap_or(false);
    let slot_tag = match slot {
        ColorPickerSlot::Sidebar => "sidebar",
        ColorPickerSlot::PreferencesLight => "prefs-light",
        ColorPickerSlot::PreferencesDark => "prefs-dark",
        ColorPickerSlot::Palette => "palette",
    };

    let mut row = div()
        .flex()
        .flex_wrap()
        .gap_2()
        .when(is_rtl, |s| s.flex_row_reverse())
        .child(clear_swatch(
            SharedString::from(format!("bg-opt-clear-{slot_tag}")),
            is_clear_active,
            theme,
            cx.listener(move |this, _, _, cx| {
                this.state.clear_canvas_color(target_is_dark);
                cx.notify();
            }),
        ))
        .child(preset_swatch(
            SharedString::from(format!("bg-opt-white-{slot_tag}")),
            CANVAS_WHITE,
            is_white_active,
            theme,
            cx.listener(move |this, _, _, cx| {
                this.state.update_canvas_color(
                    Some(HexColor::new(CANVAS_WHITE)),
                    target_is_dark,
                );
                cx.notify();
            }),
        ))
        .child(preset_swatch(
            SharedString::from(format!("bg-opt-black-{slot_tag}")),
            CANVAS_BLACK,
            is_black_active,
            theme,
            cx.listener(move |this, _, _, cx| {
                this.state.update_canvas_color(
                    Some(HexColor::new(CANVAS_BLACK)),
                    target_is_dark,
                );
                cx.notify();
            }),
        ))
        .child(custom_swatch(
            SharedString::from(format!("bg-opt-custom-{slot_tag}")),
            !is_clear_active && !is_white_active && !is_black_active,
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
            window,
            cx,
        ));
    }

    row
}

fn preset_swatch(
    id: SharedString,
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

/// Checkerboard swatch for the transparent ("Clear") background option.
fn clear_swatch(
    id: SharedString,
    is_active: bool,
    theme: &ThemeColors,
    handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let half = px(SWATCH_SIZE / 2.0);
    div()
        .id(id)
        .size(px(SWATCH_SIZE))
        .rounded_md()
        .overflow_hidden()
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
            div()
                .size_full()
                .flex()
                .flex_wrap()
                .child(div().w(half).h(half).bg(gpui::white()))
                .child(div().w(half).h(half).bg(parse_hex_color("#CBD5E1")))
                .child(div().w(half).h(half).bg(parse_hex_color("#CBD5E1")))
                .child(div().w(half).h(half).bg(gpui::white())),
        )
}

fn custom_swatch(
    id: SharedString,
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
    current: Option<&HexColor>,
    slot: ColorPickerSlot,
    target_is_dark: bool,
    theme: &ThemeColors,
    window: &mut Window,
    cx: &mut Context<MainView>,
) -> Stateful<Div> {
    let working_hue = state.custom_picker.working_hue;
    let fallback = HexColor::new(CANVAS_WHITE);
    let display = current.unwrap_or(&fallback);
    let (_, sat, val) = rgb_to_hsv(hex_to_rgb(display.as_str()));
    let is_rtl = crate::i18n::is_rtl();
    let is_invalid = state.hex_draft_is_invalid();
    let is_editing = state.custom_picker.hex_draft.is_some();
    let is_transparent = current.is_none();

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
            this.state.close_custom_picker_from_outside(e.position);
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
                                .child(if is_transparent {
                                    div()
                                        .size(px(14.0))
                                        .rounded_sm()
                                        .overflow_hidden()
                                        .border_1()
                                        .border_color(theme.border)
                                        .flex()
                                        .flex_wrap()
                                        .child(
                                            div()
                                                .w(px(7.0))
                                                .h(px(7.0))
                                                .bg(gpui::white()),
                                        )
                                        .child(
                                            div()
                                                .w(px(7.0))
                                                .h(px(7.0))
                                                .bg(parse_hex_color("#CBD5E1")),
                                        )
                                        .child(
                                            div()
                                                .w(px(7.0))
                                                .h(px(7.0))
                                                .bg(parse_hex_color("#CBD5E1")),
                                        )
                                        .child(
                                            div()
                                                .w(px(7.0))
                                                .h(px(7.0))
                                                .bg(gpui::white()),
                                        )
                                        .into_any_element()
                                } else {
                                    div()
                                        .size(px(14.0))
                                        .rounded_sm()
                                        .bg(parse_hex_color(display.as_str()))
                                        .border_1()
                                        .border_color(theme.border)
                                        .into_any_element()
                                })
                                .child(render_hex_input(
                                    state,
                                    display,
                                    slot,
                                    target_is_dark,
                                    is_invalid,
                                    theme,
                                    window,
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
                )
                .child(
                    div()
                        .id("custom-picker-clear-btn")
                        .w_full()
                        .h(px(28.0))
                        .rounded_lg()
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .when(is_rtl, |s| s.flex_row_reverse())
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .bg(theme.surface)
                        .text_color(if is_transparent {
                            theme.accent
                        } else {
                            theme.text_secondary
                        })
                        .border_1()
                        .border_color(if is_transparent {
                            theme.accent
                        } else {
                            theme.border
                        })
                        .hover(|s| s.bg(theme.surface_hover).border_color(theme.accent))
                        .active(|s| s.bg(theme.surface_active))
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.state.clear_canvas_color(target_is_dark);
                                cx.notify();
                            }),
                        )
                        .child(
                            crate::ui::icon::render_icon(crate::ui::icon::Icon::Cancel)
                                .size(px(14.0))
                                .text_color(if is_transparent {
                                    theme.accent
                                } else {
                                    theme.text_secondary
                                }),
                        )
                        .child(div().child(t!("clear").to_string())),
                ),
        )
}

/// Palette picker panel: SV + hue + hex targeting a single animation color
/// (live preview via ThorVG reload). Reuses the same hex editor state;
/// hex apply branches to the palette when `Palette` slot is open.
pub fn render_palette_picker_panel(
    state: &AppState,
    current_hex: &str,
    theme: &ThemeColors,
    window: &mut Window,
    cx: &mut Context<MainView>,
) -> Stateful<Div> {
    let working_hue = state.custom_picker.working_hue;
    let (_, sat, val) = rgb_to_hsv(hex_to_rgb(current_hex));
    let is_rtl = crate::i18n::is_rtl();
    let is_invalid = state.hex_draft_is_invalid();
    let is_editing = state.custom_picker.hex_draft.is_some();
    let current = HexColor::new(current_hex);

    div()
        .id("palette-picker-panel")
        .mt_2()
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
            this.state.close_custom_picker_from_outside(e.position);
            cx.notify();
        }))
        .child(render_palette_sv_area(working_hue, sat, val, cx))
        .child(render_palette_hue_bar(working_hue, cx))
        .child(
            div()
                .id("palette-picker-footer")
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
                                    &current,
                                    ColorPickerSlot::Palette,
                                    false,
                                    is_invalid,
                                    theme,
                                    window,
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

fn render_palette_sv_area(
    working_hue: f32,
    sat: f32,
    val: f32,
    cx: &mut Context<MainView>,
) -> Stateful<Div> {
    let view = cx.entity().clone();

    div()
        .id("palette-picker-sv-area")
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
                this.state.apply_palette_sv_pick(e.position);
                cx.notify();
            }),
        )
        .on_mouse_move(cx.listener(move |this, e: &MouseMoveEvent, _, cx| {
            if e.pressed_button == Some(MouseButton::Left) {
                this.state.apply_palette_sv_pick(e.position);
                cx.notify();
            }
        }))
}

fn render_palette_hue_bar(
    working_hue: f32,
    cx: &mut Context<MainView>,
) -> Stateful<Div> {
    let view = cx.entity().clone();

    div()
        .id("palette-picker-hue-bar")
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
                this.state.apply_palette_hue_pick(e.position);
                cx.notify();
            }),
        )
        .on_mouse_move(cx.listener(move |this, e: &MouseMoveEvent, _, cx| {
            if e.pressed_button == Some(MouseButton::Left) {
                this.state.apply_palette_hue_pick(e.position);
                cx.notify();
            }
        }))
}

/// Editable hex field: a real single-line editor with caret, click/drag
/// selection, arrow-key movement, clipboard, Enter to commit and Escape to
/// cancel. Invalid values show a red border and keep the previous color
/// instead of corrupting the config.
fn render_hex_input(
    state: &AppState,
    current: &HexColor,
    slot: ColorPickerSlot,
    target_is_dark: bool,
    is_invalid: bool,
    theme: &ThemeColors,
    window: &mut Window,
    cx: &mut Context<MainView>,
) -> Stateful<Div> {
    let is_editing = state.is_hex_editing(slot);
    let text = state.hex_field_text(current);
    let text_color = if is_invalid {
        theme.danger
    } else {
        theme.text_primary
    };
    let shaped = shape_hex_text(&text, text_color, window);
    let cursor = clamp_hex_index(&text, state.custom_picker.hex_cursor);
    let caret_x = shaped.x_for_index(cursor);
    let selection = state.hex_selection_range().map(|(lo, hi)| {
        let x0 = shaped.x_for_index(lo);
        let x1 = shaped.x_for_index(hi);
        (x0, x1)
    });
    let show_caret = is_editing && state.custom_picker.hex_blink_visible;
    let selection_bg = Hsla {
        a: 0.35,
        ..theme.accent
    };
    let current_owned = current.clone();
    let view = cx.entity().clone();

    div()
        .id("custom-picker-hex-input")
        .px_2()
        .py_1()
        .rounded_md()
        .font_family(".SystemUIFont")
        .text_size(px(HEX_FONT_SIZE_PX))
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
        .text_color(text_color)
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
            cx.listener(move |this, e: &MouseDownEvent, window, cx| {
                if !this.state.is_hex_editing(slot) {
                    this.state.start_hex_edit(&current_owned, target_is_dark);
                }
                let draft = this.state.hex_field_text(&current_owned);
                let index = hex_index_at_position(&this.state, &draft, e.position, window);
                this.state.place_hex_caret(index);
                this.state.custom_picker.hex_selecting = true;
                cx.notify();
            }),
        )
        .on_mouse_move(cx.listener(move |this, e: &MouseMoveEvent, window, cx| {
            if e.pressed_button == Some(MouseButton::Left)
                && this.state.custom_picker.hex_selecting
                && this.state.is_hex_editing(slot)
            {
                let draft = this
                    .state
                    .custom_picker
                    .hex_draft
                    .clone()
                    .unwrap_or_default();
                let index = hex_index_at_position(&this.state, &draft, e.position, window);
                this.state.extend_hex_selection(index);
                cx.notify();
            }
        }))
        .on_mouse_up(
            MouseButton::Left,
            cx.listener(|this, _, _, cx| {
                if this.state.custom_picker.hex_selecting {
                    this.state.custom_picker.hex_selecting = false;
                    cx.notify();
                }
            }),
        )
        .on_mouse_up_out(
            MouseButton::Left,
            cx.listener(|this, _, _, cx| {
                if this.state.custom_picker.hex_selecting {
                    this.state.custom_picker.hex_selecting = false;
                    cx.notify();
                }
            }),
        )
        .child(
            div()
                .relative()
                .children(selection.map(|(x0, x1)| {
                    div()
                        .absolute()
                        .left(x0)
                        .top(px(1.0))
                        .bottom(px(1.0))
                        .w(x1 - x0)
                        .bg(selection_bg)
                }))
                .child(StyledText::new(text))
                .child(
                    canvas(
                        move |bounds, _window, cx| {
                            view.update(cx, |this, _| {
                                this.state.custom_picker.hex_text_bounds = Some(bounds);
                            });
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
                .children(if show_caret {
                    Some(
                        div()
                            .absolute()
                            .left(caret_x - px(0.75))
                            .top(px(1.0))
                            .bottom(px(1.0))
                            .w(px(1.5))
                            .bg(theme.accent),
                    )
                } else {
                    None
                }),
        )
}

/// Maps a mouse position onto a caret offset in `draft` using the same
/// shaping as the rendered field. Falls back to end-of-text before the
/// first paint captures bounds.
fn hex_index_at_position(
    state: &AppState,
    draft: &str,
    position: Point<Pixels>,
    window: &mut Window,
) -> usize {
    let Some(bounds) = state.custom_picker.hex_text_bounds else {
        return draft.len();
    };
    let shaped = shape_hex_text(draft, Hsla::default(), window);
    let dx = position.x - bounds.origin.x;
    let rel_x = if dx > px(0.0) { dx } else { px(0.0) };
    clamp_hex_index(draft, shaped.closest_index_for_x(rel_x))
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

pub(crate) fn hex_to_rgb(hex: &str) -> (f32, f32, f32) {
    let rgba = Rgba::from(parse_hex_color(hex));
    (rgba.r, rgba.g, rgba.b)
}

pub(crate) fn rgb_to_hsv(rgb: (f32, f32, f32)) -> (f32, f32, f32) {
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

pub(crate) fn hsv_to_hex(h: f32, s: f32, v: f32) -> String {
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
        clamp_hex_index, coerce_hex_input, hex_paste_into, hex_replace_range, hex_to_rgb,
        hsv_to_hex, rgb_to_hsv, step_hex_index, MAX_HEX_DRAFT_LEN,
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
    fn hex_caret_indices_stay_on_char_boundaries() {
        let draft = "#F😀";
        assert_eq!(clamp_hex_index(draft, 100), draft.len());
        // Byte 3 is inside the emoji; clamps back to the boundary.
        assert_eq!(clamp_hex_index(draft, 3), 2);
        assert_eq!(step_hex_index(draft, 2, 1), draft.len());
        assert_eq!(step_hex_index(draft, draft.len(), -1), 2);
        assert_eq!(step_hex_index("#FF", 0, 10), 3);
        assert_eq!(step_hex_index("#FF", 3, -10), 0);
    }

    #[test]
    fn hex_replace_range_replaces_selection_and_places_caret() {
        // Select-all then type: the classic `ffffff` over `#FFFFFF` flow.
        let (next, cursor) = hex_replace_range("#FFFFFF", (0, 7), "F");
        assert_eq!((next.as_str(), cursor), ("#F", 2));
        // Mid-string insert without selection.
        let (next, cursor) = hex_replace_range("#FF", (2, 2), "0");
        assert_eq!((next.as_str(), cursor), ("#F0F", 3));
        // Reversed range order is normalized.
        let (next, cursor) = hex_replace_range("#FFFF", (4, 1), "");
        assert_eq!((next.as_str(), cursor), ("#F", 1));
    }

    #[test]
    fn typing_six_hex_digits_appends_past_three() {
        // Regression: live-applying `#FFF` must not reset the caret; the
        // 4th-6th keystrokes keep appending instead of replacing.
        // Focus selects all; first keystroke replaces the selection.
        let (mut text, mut cursor) = hex_replace_range("#FFFFFF", (0, 7), "F");
        assert_eq!((text.as_str(), cursor), ("#F", 2));
        for ch in "fffff".chars() {
            let end = text.len();
            // No selection: pure append at caret.
            let (next, next_cursor) = hex_replace_range(&text, (end, end), &ch.to_string());
            assert_eq!(next_cursor, end + 1);
            text = next;
            cursor = next_cursor;
        }
        assert_eq!(text, "#FFFFFF");
        assert_eq!(cursor, 7);
        assert!(text.len() <= MAX_HEX_DRAFT_LEN);
    }

    #[test]
    fn hex_paste_replaces_selection_and_truncates() {
        // Full replace keeps `#` and uppercases pasted hex.
        assert_eq!(
            hex_paste_into("#FFFFFF", (0, 7), "  #34c3eb\nsecond line"),
            ("#34C3EB".to_string(), 7)
        );
        // Mid-string paste without selection.
        assert_eq!(
            hex_paste_into("#FFFFFF", (1, 1), "ab"),
            ("#ABFFFFFF".to_string(), 3)
        );
        // Overlong paste truncates to `#` + 8 digits.
        assert_eq!(
            hex_paste_into("#", (1, 1), "123456789ABC"),
            ("#12345678".to_string(), 9)
        );
    }
}
