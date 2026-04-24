//! Small rounded badges — used for status labels like "OK", "Connected",
//! "Pending", "Error".

use egui::{
    Color32, CornerRadius, FontSelection, Response, Sense, Stroke, Ui, Vec2, Widget, WidgetInfo,
    WidgetText, WidgetType,
};

use crate::theme::{with_alpha, Theme};

/// Colour tones for a [`Badge`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BadgeTone {
    /// Success — green.
    Ok,
    /// Caution — amber.
    Warning,
    /// Error — red.
    Danger,
    /// Informational — sky.
    Info,
    /// Neutral grey, for status that isn't success/warning/error.
    Neutral,
}

impl BadgeTone {
    fn colours(self, theme: &Theme) -> (Color32, Color32) {
        let p = &theme.palette;
        match self {
            BadgeTone::Ok => (with_alpha(p.green, 64), p.success),
            BadgeTone::Warning => (with_alpha(p.amber, 64), p.warning),
            BadgeTone::Danger => (with_alpha(p.red, 64), p.danger),
            BadgeTone::Info => (with_alpha(p.sky, 64), p.sky),
            BadgeTone::Neutral => (with_alpha(p.text_muted, 40), p.text_muted),
        }
    }
}

/// A compact rounded status badge.
///
/// ```no_run
/// # use elegance::{Badge, BadgeTone};
/// # egui::__run_test_ui(|ui| {
/// ui.add(Badge::new("OK", BadgeTone::Ok));
/// # });
/// ```
#[must_use = "Add with `ui.add(...)`."]
pub struct Badge {
    text: WidgetText,
    tone: BadgeTone,
}

impl std::fmt::Debug for Badge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Badge")
            .field("text", &self.text.text())
            .field("tone", &self.tone)
            .finish()
    }
}

impl Badge {
    /// Create a badge with the given text and tone.
    pub fn new(text: impl Into<WidgetText>, tone: BadgeTone) -> Self {
        Self {
            text: text.into(),
            tone,
        }
    }
}

impl Widget for Badge {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());
        let t = &theme.typography;
        let (bg, fg) = self.tone.colours(&theme);

        let font = egui::FontId::proportional(t.small);
        let galley = egui::WidgetText::from(
            egui::RichText::new(self.text.text().to_uppercase())
                .color(fg)
                .size(t.small)
                .strong(),
        )
        .into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            FontSelection::FontId(font),
        );

        let pad = Vec2::new(9.0, 3.0);
        let desired = galley.size() + pad * 2.0;
        let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());

        if ui.is_rect_visible(rect) {
            ui.painter().rect(
                rect,
                CornerRadius::same(99),
                bg,
                Stroke::NONE,
                egui::StrokeKind::Inside,
            );
            let text_pos = egui::pos2(rect.min.x + pad.x, rect.center().y - galley.size().y * 0.5);
            ui.painter().galley(text_pos, galley, fg);
        }

        response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, self.text.text()));
        response
    }
}
