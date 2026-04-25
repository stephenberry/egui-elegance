//! Determinate circular progress indicator.
//!
//! A ring-shaped cousin of [`ProgressBar`](crate::ProgressBar): a faint
//! track plus an accent-coloured arc that sweeps clockwise from 12
//! o'clock as the fraction grows. Centre text defaults to a rounded
//! percent; pass an explicit label via [`ProgressRing::text`] or add a
//! small sub-caption with [`ProgressRing::caption`].
//!
//! For *indeterminate* "still working" loaders, prefer
//! [`Spinner`](crate::Spinner). This widget renders a fixed fraction
//! each frame and never requests repaints on its own.

use std::f32::consts::{PI, TAU};

use egui::{
    epaint::{PathShape, PathStroke},
    pos2, Color32, Pos2, Response, Sense, Ui, Vec2, Widget, WidgetInfo, WidgetType,
};

use crate::theme::{Accent, Theme};

/// A themed determinate circular progress indicator.
///
/// ```no_run
/// # use elegance::{Accent, ProgressRing};
/// # egui::__run_test_ui(|ui| {
/// // Default: 56 pt diameter, sky arc, percent in the centre.
/// ui.add(ProgressRing::new(0.42));
///
/// // Larger, green, custom centre text + sub-caption.
/// ui.add(
///     ProgressRing::new(0.6)
///         .size(88.0)
///         .accent(Accent::Green)
///         .text("12 / 20")
///         .caption("files"),
/// );
///
/// // Hide the centre text entirely with an empty override.
/// ui.add(ProgressRing::new(0.3).size(32.0).text(""));
/// # });
/// ```
#[derive(Debug, Clone)]
#[must_use = "Add with `ui.add(...)`."]
pub struct ProgressRing {
    fraction: f32,
    size: f32,
    stroke_width: Option<f32>,
    color: Option<Color32>,
    accent: Option<Accent>,
    text: Option<String>,
    caption: Option<String>,
}

impl ProgressRing {
    /// Create a ring at `fraction` (0..=1). NaN and out-of-range values
    /// are clamped.
    pub fn new(fraction: f32) -> Self {
        Self {
            fraction: if fraction.is_nan() {
                0.0
            } else {
                fraction.clamp(0.0, 1.0)
            },
            size: 56.0,
            stroke_width: None,
            color: None,
            accent: None,
            text: None,
            caption: None,
        }
    }

    /// Outer diameter in points. Default: 56. Clamped to at least 8.
    #[inline]
    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(8.0);
        self
    }

    /// Arc stroke thickness in points. Defaults to ~8 % of `size`,
    /// clamped to `[3.0, 10.0]`.
    #[inline]
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = Some(width.max(1.0));
        self
    }

    /// Paint the arc with an explicit colour. Clears any previously set
    /// accent.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self.accent = None;
        self
    }

    /// Pick the arc colour from one of the theme's accents. Clears any
    /// previously set explicit colour. Default: the theme's sky.
    pub fn accent(mut self, accent: Accent) -> Self {
        self.accent = Some(accent);
        self.color = None;
        self
    }

    /// Override the centre text. By default the ring shows the rounded
    /// percent (e.g. "42%") once `size >= 40`; passing `""` hides the
    /// text entirely.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Add a small muted sub-caption below the primary text.
    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }
}

impl Widget for ProgressRing {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());
        let p = &theme.palette;
        let color = match (self.color, self.accent) {
            (Some(c), _) => c,
            (_, Some(a)) => p.accent_fill(a),
            _ => p.sky,
        };
        let stroke_w = self
            .stroke_width
            .unwrap_or((self.size * 0.08).clamp(3.0, 10.0));

        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            // Subtract half-stroke so the ring's outer edge lands on the
            // allocated rect; extra 1 pt keeps anti-aliased edges clean.
            let radius = ((self.size * 0.5) - stroke_w * 0.5 - 1.0).max(0.5);
            let track_color = p.depth_tint(p.card, 0.1);

            // Track and arc share the same sample density + PathShape
            // primitive so they sit on identical pixels; mixing
            // `circle_stroke` (internal tessellation) with a manual
            // `PathShape::line` (polyline) produces a subtle concentric
            // mismatch.
            let n_full: usize = 96;
            let point_at = |a: f32| {
                let (sin, cos) = a.sin_cos();
                pos2(center.x + radius * cos, center.y + radius * sin)
            };

            let track_points: Vec<Pos2> = (0..n_full)
                .map(|i| point_at((i as f32 / n_full as f32) * TAU))
                .collect();
            painter.add(PathShape::closed_line(
                track_points,
                PathStroke::new(stroke_w, track_color),
            ));

            // Arc, clockwise from 12 o'clock.
            if self.fraction > 0.0 {
                let sweep = TAU * self.fraction;
                let start = -PI * 0.5;
                // Match the track's per-radian sampling so the arc
                // points lie exactly on the same polygon vertices.
                let n_points = ((n_full as f32 * self.fraction).ceil() as usize).max(2);
                let points: Vec<Pos2> = (0..=n_points)
                    .map(|i| point_at(start + sweep * (i as f32 / n_points as f32)))
                    .collect();

                // Rounded endpoint caps — PathShape strokes are butt-ended.
                painter.circle_filled(points[0], stroke_w * 0.5, color);
                painter.circle_filled(points[n_points], stroke_w * 0.5, color);
                painter.add(PathShape::line(points, PathStroke::new(stroke_w, color)));
            }

            // Centre label + caption.
            let primary: String = match &self.text {
                Some(s) => s.clone(),
                None if self.size >= 40.0 => {
                    format!("{}%", (self.fraction * 100.0).round() as u32)
                }
                _ => String::new(),
            };
            let caption = self.caption.as_deref().unwrap_or("");

            let primary_size = (self.size * 0.20).clamp(10.0, 24.0);
            let caption_size = (self.size * 0.11).clamp(8.0, 12.0);

            let primary_galley = (!primary.is_empty()).then(|| {
                crate::theme::placeholder_galley(ui, &primary, primary_size, true, f32::INFINITY)
            });
            let caption_galley = (!caption.is_empty()).then(|| {
                crate::theme::placeholder_galley(ui, caption, caption_size, false, f32::INFINITY)
            });

            let primary_h = primary_galley.as_ref().map_or(0.0, |g| g.size().y);
            let caption_h = caption_galley.as_ref().map_or(0.0, |g| g.size().y);
            let line_gap = if primary_galley.is_some() && caption_galley.is_some() {
                2.0
            } else {
                0.0
            };
            let group_h = primary_h + line_gap + caption_h;
            let top_y = center.y - group_h * 0.5;

            if let Some(g) = primary_galley {
                let g_w = g.size().x;
                painter.galley(pos2(center.x - g_w * 0.5, top_y), g, p.text);
            }
            if let Some(g) = caption_galley {
                let g_w = g.size().x;
                let y = top_y + primary_h + line_gap;
                painter.galley(pos2(center.x - g_w * 0.5, y), g, p.text_muted);
            }
        }

        response
            .widget_info(|| WidgetInfo::labeled(WidgetType::ProgressIndicator, true, "progress"));
        response
    }
}
