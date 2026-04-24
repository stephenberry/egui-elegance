//! Card container — a rounded, bordered surface for grouping widgets.

use egui::{Color32, CornerRadius, InnerResponse, Margin, Stroke, Ui, WidgetText};

use crate::theme::Theme;

/// A styled card surface.
///
/// ```no_run
/// # use elegance::Card;
/// # egui::__run_test_ui(|ui| {
/// Card::new().heading("Setup").show(ui, |ui| {
///     ui.label("Card contents go here.");
/// });
/// # });
/// ```
#[derive(Default)]
#[must_use = "Call `.show(ui, ...)` to render the card."]
pub struct Card {
    heading: Option<WidgetText>,
    padding: Option<f32>,
    fill: Option<Color32>,
    bordered: bool,
    corner_radius: Option<CornerRadius>,
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Card")
            .field("heading", &self.heading.as_ref().map(|h| h.text()))
            .field("padding", &self.padding)
            .field("fill", &self.fill)
            .field("bordered", &self.bordered)
            .field("corner_radius", &self.corner_radius)
            .finish()
    }
}

impl Card {
    /// Create a card with the default padding, fill, and border.
    pub fn new() -> Self {
        Self {
            heading: None,
            padding: None,
            fill: None,
            bordered: true,
            corner_radius: None,
        }
    }

    /// Show a small caption at the top of the card.
    pub fn heading(mut self, heading: impl Into<WidgetText>) -> Self {
        self.heading = Some(heading.into());
        self
    }

    /// Override the default inner padding (points).
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = Some(padding);
        self
    }

    /// Override the fill colour.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Toggle the 1-px border. Defaults to on.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Override the corner radius (per-corner). Useful for segmented
    /// layouts where only some corners should be rounded.
    pub fn corner_radius(mut self, radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = Some(radius.into());
        self
    }

    /// Render the card and its body contents, returning whatever the
    /// closure returns inside an [`InnerResponse`].
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = Theme::current(ui.ctx());
        let p = &theme.palette;

        let padding = self.padding.unwrap_or(theme.card_padding);
        let stroke = if self.bordered {
            Stroke::new(1.0, p.border)
        } else {
            Stroke::NONE
        };

        let radius = self
            .corner_radius
            .unwrap_or_else(|| CornerRadius::same(theme.card_radius as u8));

        let frame = egui::Frame::new()
            .fill(self.fill.unwrap_or(p.card))
            .stroke(stroke)
            .corner_radius(radius)
            .inner_margin(Margin::same(padding as i8));

        frame.show(ui, |ui| {
            if let Some(h) = &self.heading {
                let rt = egui::RichText::new(h.text())
                    .color(p.text_muted)
                    .size(theme.typography.heading)
                    .strong();
                ui.add(egui::Label::new(rt).wrap_mode(egui::TextWrapMode::Extend));
                ui.add_space(8.0);
            }
            add_contents(ui)
        })
    }
}
