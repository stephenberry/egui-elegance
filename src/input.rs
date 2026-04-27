//! Styled single-line text input.
//!
//! Wraps [`egui::TextEdit`] with:
//!
//! * The slate input background
//! * A crisp 1-px border that turns sky-coloured on focus
//! * An optional "dirty" indicator (a 3-px sky-coloured bar down the left
//!   edge) to signal unsaved changes
//! * An optional label and optional hint text
//! * Success / error flash animations triggered via
//!   [`ResponseFlashExt`](crate::ResponseFlashExt)

use egui::{
    CornerRadius, FontSelection, Response, Stroke, TextEdit, Ui, Vec2, Widget, WidgetInfo,
    WidgetText, WidgetType,
};

use crate::{flash, theme::Theme};

/// A styled single-line text input.
///
/// ```no_run
/// # use elegance::TextInput;
/// # egui::__run_test_ui(|ui| {
/// let mut email = String::new();
/// ui.add(TextInput::new(&mut email).label("Email").hint("you@example.com"));
/// # });
/// ```
///
/// # Ids, focus, and flash state
///
/// Flash animations, focus, and cursor state are keyed off the widget's
/// egui id. Without [`id_salt`](Self::id_salt), the id is derived from
/// egui's auto-id counter, which is layout-dependent — if a sibling
/// appears or disappears above this input between frames, the id shifts
/// and any in-flight flash, focus, or cursor state is lost. Any input
/// you flash via [`ResponseFlashExt`](crate::ResponseFlashExt) should pin
/// its id with [`id_salt`](Self::id_salt).
#[must_use = "Add with `ui.add(...)`."]
pub struct TextInput<'a> {
    text: &'a mut String,
    label: Option<WidgetText>,
    hint: Option<&'a str>,
    dirty: bool,
    password: bool,
    desired_width: Option<f32>,
    id_salt: Option<egui::Id>,
    compact: bool,
}

impl<'a> std::fmt::Debug for TextInput<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextInput")
            .field("dirty", &self.dirty)
            .field("password", &self.password)
            .field("desired_width", &self.desired_width)
            .field("compact", &self.compact)
            .finish()
    }
}

impl<'a> TextInput<'a> {
    /// Create a text input bound to `text`.
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            label: None,
            hint: None,
            dirty: false,
            password: false,
            desired_width: None,
            id_salt: None,
            compact: false,
        }
    }

    /// Show a label above the input.
    pub fn label(mut self, text: impl Into<WidgetText>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Show placeholder-style hint text when the field is empty.
    pub fn hint(mut self, text: &'a str) -> Self {
        self.hint = Some(text);
        self
    }

    /// Mark the input as having unsaved changes. Shows a sky-coloured
    /// accent bar down the left side.
    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    /// Mask the text as a password field.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Desired width (points) for the editor portion of the widget.
    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    /// Supply a stable id salt. Useful when two inputs share the same label.
    pub fn id_salt(mut self, id: impl std::hash::Hash) -> Self {
        self.id_salt = Some(egui::Id::new(id));
        self
    }

    /// Render with reduced vertical padding so the input matches the
    /// height of [`RemovableChip`](crate::RemovableChip) and small-size
    /// controls. Useful for inline path / chip rows where a full-height
    /// input would dominate. Default: `false` (standard control height).
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
}

impl<'a> Widget for TextInput<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());
        let p = &theme.palette;
        let t = &theme.typography;

        ui.vertical(|ui| {
            if let Some(label) = &self.label {
                ui.add_space(2.0);
                let rich = egui::RichText::new(label.text())
                    .color(p.text_muted)
                    .size(t.label);
                ui.add(egui::Label::new(rich).wrap_mode(egui::TextWrapMode::Extend));
                ui.add_space(2.0);
            }

            // Pin a stable id_salt so the TextEdit id is predictable —
            // we need it to look up flash state before painting.
            //
            // `TextEdit::id_salt` internally wraps its input in
            // `Id::new(...)` before calling `make_persistent_id`, so we
            // mirror that step here to get the *same* widget id.
            let id_salt = self.id_salt.unwrap_or_else(|| ui.next_auto_id());
            let widget_id = ui.make_persistent_id(egui::Id::new(id_salt));

            let flash = flash::active_flash(ui.ctx(), widget_id);
            let bg_fill = flash::background_fill(&theme, p.input_bg, flash);

            let desired_width = self.desired_width.unwrap_or_else(|| ui.available_width());
            // Compact mode shrinks the vertical padding to match
            // RemovableChip's editor and Button::size(Small) at ~22 pt
            // total. Horizontal padding is unchanged so the caret has
            // room either way.
            let margin_y = if self.compact {
                3.0
            } else {
                theme.control_padding_y
            };
            let margin = Vec2::new(theme.control_padding_x * 0.5, margin_y);

            // Swap visuals so the TextEdit picks up our look, then restore.
            let response = crate::theme::with_themed_visuals(ui, |ui| {
                let v = ui.visuals_mut();
                crate::theme::themed_input_visuals(v, &theme, bg_fill);
                v.extreme_bg_color = bg_fill;
                v.selection.bg_fill = crate::theme::with_alpha(p.sky, 90);
                v.selection.stroke = Stroke::new(1.0, p.sky);

                let mut edit = TextEdit::singleline(self.text)
                    .id_salt(id_salt)
                    .font(FontSelection::FontId(egui::FontId::proportional(t.body)))
                    .text_color(p.text)
                    .margin(margin)
                    .desired_width(desired_width);
                if let Some(hint) = self.hint {
                    edit = edit.hint_text(egui::RichText::new(hint).color(p.text_faint));
                }
                if self.password {
                    edit = edit.password(true);
                }

                ui.add(edit)
            });

            if self.dirty && ui.is_rect_visible(response.rect) {
                // Hug the inside of the border with a fixed geometry that does
                // *not* change across hover/focus. The bar is a status
                // indicator, not an interactive element — jittering it with the
                // cursor reads as a bug. All input states use a 1 pt stroke
                // (only the colour changes on focus), so inset = 1.0 matches
                // the inner edge of the border in every state, and the bar's
                // 5 pt inner corner radius matches the border's inner arc.
                let stroke_w = 1.0;
                let bar_w = 3.0;
                let r = (theme.control_radius - stroke_w).max(0.0) as u8;
                let bar = egui::Rect::from_min_max(
                    egui::pos2(
                        response.rect.min.x + stroke_w,
                        response.rect.min.y + stroke_w,
                    ),
                    egui::pos2(
                        response.rect.min.x + stroke_w + bar_w,
                        response.rect.max.y - stroke_w,
                    ),
                );
                let corner = CornerRadius {
                    nw: r,
                    sw: r,
                    ne: 0,
                    se: 0,
                };
                ui.painter().rect_filled(bar, corner, p.sky);
            }

            // Expose the field label via accesskit so screen readers announce
            // the field purpose. TextEdit sets its own widget_info (with the
            // current text value as the label); replacing it with ours makes
            // the field queryable by its semantic label instead.
            if let Some(label) = &self.label {
                let label = label.text().to_string();
                response.widget_info(|| WidgetInfo::labeled(WidgetType::TextEdit, true, &label));
            }

            response
        })
        .inner
    }
}
