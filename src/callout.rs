//! Callout — a full-width inline banner announcing persistent context.
//!
//! A [`Callout`] is a strip of chrome that sits inline in a layout to flag
//! important context to the reader: experimental features, unsaved changes,
//! failed builds, maintenance windows. Unlike [`Toast`](crate::Toast) it does
//! not auto-dismiss, and unlike [`FlashKind`](crate::FlashKind) it's a whole
//! surface rather than a pulse on another widget.
//!
//! The visual treatment: a `card`-colored banner with a 3px accent stripe on
//! the leading edge, a tone-tinted icon, a bold title inline with muted body
//! text, and optional action buttons plus a dismiss button pinned to the
//! right.

use egui::{
    Align, Color32, CornerRadius, InnerResponse, Layout, Margin, Rect, Response, Sense, Stroke,
    StrokeKind, Ui, Vec2, WidgetInfo, WidgetText, WidgetType,
};

use crate::theme::Theme;

/// Semantic tones for a [`Callout`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CalloutTone {
    /// Informational — sky accent. For neutral announcements.
    Info,
    /// Success — green accent. For affirmative context ("Deploy complete").
    Success,
    /// Caution — amber accent. For conditions the user should notice.
    Warning,
    /// Error or destructive — red accent. For failures or dangerous state.
    Danger,
    /// Grey — for announcements that have no urgency at all.
    Neutral,
}

impl CalloutTone {
    fn stripe(self, theme: &Theme) -> Color32 {
        let p = &theme.palette;
        match self {
            Self::Info => p.sky,
            Self::Success => p.green,
            Self::Warning => p.amber,
            Self::Danger => p.red,
            Self::Neutral => p.text_muted,
        }
    }

    fn icon_color(self, theme: &Theme) -> Color32 {
        let p = &theme.palette;
        match self {
            Self::Info => p.sky,
            Self::Success => p.success,
            Self::Warning => p.warning,
            Self::Danger => p.danger,
            Self::Neutral => p.text_muted,
        }
    }

    fn default_icon(self) -> &'static str {
        match self {
            Self::Info => "ℹ",
            Self::Success => "✓",
            Self::Warning => "⚠",
            Self::Danger => "×",
            Self::Neutral => "•",
        }
    }
}

/// A full-width inline banner in the elegance style.
///
/// ```no_run
/// # use elegance::{Callout, CalloutTone};
/// # egui::__run_test_ui(|ui| {
/// Callout::new(CalloutTone::Warning)
///     .title("Unsaved changes.")
///     .body("You have 3 edits that haven't been written to disk.")
///     .show(ui, |_| {});
/// # });
/// ```
#[must_use = "Call `.show(ui, ...)` to render the callout."]
pub struct Callout<'a> {
    tone: CalloutTone,
    title: Option<WidgetText>,
    body: Option<WidgetText>,
    icon: Option<WidgetText>,
    open: Option<&'a mut bool>,
}

impl std::fmt::Debug for Callout<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Callout")
            .field("tone", &self.tone)
            .field("title", &self.title.as_ref().map(|t| t.text()))
            .field("body", &self.body.as_ref().map(|b| b.text()))
            .field("icon", &self.icon.as_ref().map(|i| i.text()))
            .field("dismissable", &self.open.is_some())
            .finish()
    }
}

impl<'a> Callout<'a> {
    /// Create a new callout with the given tone.
    pub fn new(tone: CalloutTone) -> Self {
        Self {
            tone,
            title: None,
            body: None,
            icon: None,
            open: None,
        }
    }

    /// Set the bolded title text rendered inline at the left of the banner.
    #[inline]
    pub fn title(mut self, text: impl Into<WidgetText>) -> Self {
        self.title = Some(text.into());
        self
    }

    /// Set the muted body text, rendered to the right of the title.
    #[inline]
    pub fn body(mut self, text: impl Into<WidgetText>) -> Self {
        self.body = Some(text.into());
        self
    }

    /// Override the icon glyph. Defaults to a tone-dependent symbol.
    #[inline]
    pub fn icon(mut self, icon: impl Into<WidgetText>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Render a trailing × button that sets `*open = false` when clicked.
    ///
    /// The caller is responsible for gating the `.show(...)` call on the
    /// same `bool` so the banner disappears after dismissal.
    #[inline]
    pub fn dismissable(mut self, open: &'a mut bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Render the callout and return the closure's result.
    ///
    /// `add_actions` is invoked with a **right-to-left** layout so buttons
    /// you add slot into the action area between the body text and the
    /// dismiss button. Inside the closure the first widget added appears
    /// furthest right, so **add your primary action first**:
    ///
    /// ```no_run
    /// # use elegance::{Accent, Button, Callout, CalloutTone};
    /// # egui::__run_test_ui(|ui| {
    /// Callout::new(CalloutTone::Warning)
    ///     .title("Unsaved changes.")
    ///     .show(ui, |ui| {
    ///         ui.add(Button::new("Save now").accent(Accent::Amber)); // rightmost
    ///         ui.add(Button::new("Discard").outline());              // to its left
    ///     });
    /// # });
    /// ```
    ///
    /// Pass `|_| {}` when no actions are needed.
    pub fn show<R>(self, ui: &mut Ui, add_actions: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        const STRIPE_WIDTH: f32 = 3.0;

        let theme = Theme::current(ui.ctx());
        let p = &theme.palette;
        let body_size = theme.typography.body;
        let stripe = self.tone.stripe(&theme);
        let icon_color = self.tone.icon_color(&theme);
        let default_icon = self.tone.default_icon();

        let a11y_label = self
            .title
            .as_ref()
            .or(self.body.as_ref())
            .map(|w| w.text().to_string())
            .unwrap_or_else(|| "callout".to_string());

        let Self {
            tone: _,
            title,
            body,
            icon,
            open,
        } = self;

        // Left inner margin accounts for the 3px stripe (24 pt from stripe
        // to content, matching the HTML mockups).
        let frame = egui::Frame::new().fill(p.card).inner_margin(Margin {
            left: (STRIPE_WIDTH as i8) + 18,
            right: 16,
            top: 10,
            bottom: 10,
        });

        let frame_response: InnerResponse<R> = frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;

                // Icon.
                let icon_str = icon
                    .as_ref()
                    .map(|w| w.text().to_string())
                    .unwrap_or_else(|| default_icon.to_string());
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(icon_str)
                            .color(icon_color)
                            .size(body_size + 1.0),
                    )
                    .wrap_mode(egui::TextWrapMode::Extend),
                );

                // Title (strong).
                if let Some(title) = title {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(title.text())
                                .color(p.text)
                                .size(body_size)
                                .strong(),
                        )
                        .wrap_mode(egui::TextWrapMode::Extend),
                    );
                }

                // Body (muted).
                if let Some(body) = body {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(body.text())
                                .color(p.text_muted)
                                .size(body_size),
                        )
                        .wrap_mode(egui::TextWrapMode::Truncate),
                    );
                }

                // Right-aligned action slot and optional dismiss button.
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(open) = open {
                        if dismiss_button(ui, &theme).clicked() {
                            *open = false;
                        }
                    }
                    add_actions(ui)
                })
                .inner
            })
            .inner
        });

        // Paint the accent stripe and bottom border on top of the frame.
        let rect = frame_response.response.rect;
        let painter = ui.painter();
        painter.rect(
            Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.left() + STRIPE_WIDTH, rect.bottom()),
            ),
            CornerRadius::ZERO,
            stripe,
            Stroke::NONE,
            StrokeKind::Inside,
        );
        painter.hline(rect.x_range(), rect.bottom(), Stroke::new(1.0, p.border));

        frame_response
            .response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, &a11y_label));

        frame_response
    }
}

/// Compact × dismiss button. Renders as two diagonal strokes so it doesn't
/// depend on the presence of a specific "×" glyph in the active font.
fn dismiss_button(ui: &mut Ui, theme: &Theme) -> Response {
    let size = Vec2::splat(theme.typography.body + 8.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let color = if response.hovered() || response.has_focus() {
            theme.palette.text
        } else {
            theme.palette.text_faint
        };
        let painter = ui.painter();
        let c = rect.center();
        let r = 4.5;
        let stroke = Stroke::new(1.5, color);
        painter.line_segment([c + Vec2::new(-r, -r), c + Vec2::new(r, r)], stroke);
        painter.line_segment([c + Vec2::new(r, -r), c + Vec2::new(-r, r)], stroke);
    }
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, "Dismiss"));
    response
}
