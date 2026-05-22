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
///
/// Inner padding defaults to the active theme's `card_padding` on all
/// four sides. Override via [`Card::padding`], which accepts anything
/// convertible to an [`egui::Margin`]: a scalar for uniform padding, a
/// [`Vec2`](egui::Vec2) for per-axis padding, or an explicit `Margin`
/// for full per-side control.
#[derive(Default)]
#[must_use = "Call `.show(ui, ...)` to render the card."]
pub struct Card {
    heading: Option<WidgetText>,
    padding: Option<Margin>,
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

    /// Override the default inner padding. Accepts anything convertible
    /// to an [`egui::Margin`]:
    ///
    /// * a scalar (`i8` or `f32`) for uniform padding on all four sides,
    /// * an [`egui::Vec2`] for symmetric per-axis padding (e.g. a compact
    ///   row of widgets that wants less vertical than horizontal room),
    /// * an explicit `Margin { left, right, top, bottom }` for full
    ///   per-side control.
    ///
    /// `Margin` stores each side as `i8`, so values above 127 saturate
    /// when converted from `f32`. In practice that ceiling sits well
    /// above any sensible card padding, but is worth knowing when
    /// passing values from a theme or animation.
    pub fn padding(mut self, margin: impl Into<Margin>) -> Self {
        self.padding = Some(margin.into());
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

        let margin = self
            .padding
            .unwrap_or_else(|| Margin::same(theme.card_padding as i8));
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
            .inner_margin(margin);

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

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    /// `padding` accepts every reasonable `Into<Margin>` form and the
    /// most recent call wins. Guards against an accidental split into
    /// separate fields where the show path could read the wrong one.
    #[test]
    fn padding_accepts_all_into_margin_forms() {
        let c = Card::new();
        assert!(c.padding.is_none(), "default is unset");

        // Uniform via unsuffixed float literal (covers inference).
        let c = Card::new().padding(12.0);
        assert_eq!(c.padding, Some(Margin::same(12)));

        // Uniform via unsuffixed integer literal.
        let c = Card::new().padding(8);
        assert_eq!(c.padding, Some(Margin::same(8)));

        // Per-axis via Vec2.
        let c = Card::new().padding(vec2(10.0, 4.0));
        assert_eq!(c.padding, Some(Margin::symmetric(10, 4)));

        // Full per-side via explicit Margin.
        let m = Margin {
            left: 1,
            right: 2,
            top: 3,
            bottom: 4,
        };
        let c = Card::new().padding(m);
        assert_eq!(c.padding, Some(m));

        // Last setter wins.
        let c = Card::new().padding(20.0).padding(vec2(8.0, 2.0));
        assert_eq!(c.padding, Some(Margin::symmetric(8, 2)));
    }
}
