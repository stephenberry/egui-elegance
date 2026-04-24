//! Buttons in the elegance style.
//!
//! A [`Button`] is a chunky, rounded rectangle with a coloured fill, bold
//! text, and smooth hover/press transitions. Six accent colours are
//! available: Blue, Green, Red, Purple, Amber, and Sky. For secondary
//! actions, [`Button::outline`] gives a transparent, bordered treatment.

use egui::{
    vec2, Color32, CornerRadius, Response, Sense, Stroke, Ui, Vec2, Widget, WidgetInfo, WidgetText,
    WidgetType,
};

use crate::theme::{mix, Accent, Theme};

/// Size presets for buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonSize {
    /// Compact — tight padding, the small typography size.
    Small,
    /// The default button size.
    Medium,
    /// Chunky — extra padding and a slightly larger font.
    Large,
}

impl ButtonSize {
    /// Resolve the `(pad_x, pad_y)` padding for a given size against the
    /// active theme. Used by [`Button`] and [`SegmentedButton`](crate::SegmentedButton)
    /// so both widgets produce identical control heights at a given size.
    pub fn padding(self, theme: &Theme) -> Vec2 {
        match self {
            ButtonSize::Small => vec2(theme.control_padding_x * 0.6, theme.control_padding_y * 0.6),
            ButtonSize::Medium => vec2(theme.control_padding_x, theme.control_padding_y),
            ButtonSize::Large => vec2(
                theme.control_padding_x * 1.25,
                theme.control_padding_y * 1.2,
            ),
        }
    }

    /// Resolve the label font size for a given size against the active theme.
    pub fn font_size(self, theme: &Theme) -> f32 {
        match self {
            ButtonSize::Small => theme.typography.small,
            ButtonSize::Medium => theme.typography.button,
            ButtonSize::Large => theme.typography.body + 1.0,
        }
    }
}

/// A coloured, rounded button.
///
/// ```no_run
/// # use elegance::{Button, Accent};
/// # egui::__run_test_ui(|ui| {
/// if ui.add(Button::new("Save").accent(Accent::Green)).clicked() {
///     // ...
/// }
/// # });
/// ```
#[must_use = "Call `ui.add(...)` to render the button."]
pub struct Button {
    text: WidgetText,
    accent: Accent,
    size: ButtonSize,
    outline: bool,
    min_width: Option<f32>,
    full_width: bool,
    enabled: bool,
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("accent", &self.accent)
            .field("size", &self.size)
            .field("outline", &self.outline)
            .field("min_width", &self.min_width)
            .field("full_width", &self.full_width)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl Button {
    /// Create a new button. Defaults to the Blue accent and medium size.
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            accent: Accent::Blue,
            size: ButtonSize::Medium,
            outline: false,
            min_width: None,
            full_width: false,
            enabled: true,
        }
    }

    /// Pick the button accent colour. Ignored when the button is set to
    /// [`Button::outline`], which has no fill colour of its own.
    #[inline]
    pub fn accent(mut self, accent: Accent) -> Self {
        self.accent = accent;
        self
    }

    /// Render the button as a transparent, bordered "ghost" treatment for
    /// secondary actions.
    #[inline]
    pub fn outline(mut self) -> Self {
        self.outline = true;
        self
    }

    /// Pick a size preset.
    #[inline]
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Set a minimum width (in points). Useful to line up button groups.
    #[inline]
    pub fn min_width(mut self, w: f32) -> Self {
        self.min_width = Some(w);
        self
    }

    /// Stretch to fill the available horizontal space.
    #[inline]
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    /// Disable the button.
    #[inline]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    fn padding(&self, theme: &Theme) -> Vec2 {
        self.size.padding(theme)
    }

    fn font_size(&self, theme: &Theme) -> f32 {
        self.size.font_size(theme)
    }
}

impl Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());
        let padding = self.padding(&theme);
        let font_size = self.font_size(&theme);

        let wrap_width = (ui.available_width() - 2.0 * padding.x).max(0.0);
        let galley =
            crate::theme::placeholder_galley(ui, self.text.text(), font_size, false, wrap_width);

        let mut desired = galley.size() + 2.0 * padding;
        desired.y = desired.y.max(font_size + 2.0 * padding.y);
        if let Some(min_w) = self.min_width {
            desired.x = desired.x.max(min_w);
        }
        if self.full_width {
            desired.x = ui.available_width().max(desired.x);
        }

        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(desired, sense);

        let visible = ui.is_rect_visible(rect);
        if visible {
            // Work out fill and text colour for the current state.
            let (fill, stroke, text_color) =
                resolve_colors(&theme, self.accent, self.outline, self.enabled, &response);

            let radius = CornerRadius::same(theme.control_radius as u8);
            ui.painter()
                .rect(rect, radius, fill, stroke, egui::StrokeKind::Inside);

            let text_pos = rect.center();
            ui.painter()
                .galley(galley_top_left(rect, galley.size()), galley, text_color);
            let _ = text_pos;
        }

        response.widget_info(|| {
            WidgetInfo::labeled(WidgetType::Button, self.enabled, self.text.text())
        });
        response
    }
}

fn galley_top_left(rect: egui::Rect, galley_size: Vec2) -> egui::Pos2 {
    let center = rect.center();
    center - galley_size * 0.5
}

fn resolve_colors(
    theme: &Theme,
    accent: Accent,
    outline: bool,
    enabled: bool,
    response: &Response,
) -> (Color32, Stroke, Color32) {
    let p = &theme.palette;
    if !enabled {
        if outline {
            return (
                Color32::TRANSPARENT,
                Stroke::new(1.0, p.border),
                mix(p.text_muted, p.card, 0.4),
            );
        }
        return (
            mix(p.accent_fill(accent), p.card, 0.55),
            Stroke::NONE,
            mix(p.text, p.card, 0.4),
        );
    }
    let is_down = response.is_pointer_button_down_on();
    let is_hovered = response.hovered();

    if outline {
        let text = if is_hovered { p.text } else { p.text_muted };
        let stroke_color = if is_hovered { p.text_muted } else { p.border };
        let fill = if is_down {
            with_alpha(p.text_muted, 30)
        } else if is_hovered {
            with_alpha(p.text_muted, 20)
        } else {
            Color32::TRANSPARENT
        };
        return (fill, Stroke::new(1.0, stroke_color), text);
    }

    let resting = p.accent_fill(accent);
    let hover = p.accent_hover(accent);
    let fill = if is_down {
        // Slightly darker than the resting hover colour for a satisfying click.
        mix(hover, Color32::BLACK, 0.08)
    } else if is_hovered {
        hover
    } else {
        resting
    };
    let stroke = if response.has_focus() {
        Stroke::new(2.0, with_alpha(p.sky, 180))
    } else {
        Stroke::NONE
    };
    (fill, stroke, Color32::WHITE)
}

fn with_alpha(c: Color32, alpha: u8) -> Color32 {
    crate::theme::with_alpha(c, alpha)
}
