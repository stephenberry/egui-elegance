//! Percentage slider — a 0–100% control whose central UI element is the
//! percentage value itself.
//!
//! [`PercentSlider`] differs from [`Slider`](crate::Slider) in three ways:
//!
//! 1. The value is always a 0–100 percentage (`f32`). Pair it with
//!    [`PercentSlider::total_fmt`] when the percentage maps to a meaningful
//!    absolute quantity (a duration, a file size, a budget share) and the
//!    absolute value will surface in a callout while the user drags.
//! 2. The visual hierarchy puts the percentage value front and centre,
//!    rendered large in the top-right of the widget. An optional small
//!    label sits in the top-left.
//! 3. Quartile ticks (`0`, `25%`, `50%`, `75%`, `100%`) sit beneath the
//!    track so "what fraction of the total am I setting?" reads at a
//!    glance. Hide them with [`PercentSlider::show_ticks`] for compact
//!    layouts.
//!
//! Three snap modes are available, in order of specificity:
//! [`step`](PercentSlider::step) snaps to multiples of a fixed size
//! (`5.0` → 0, 5, 10, …); [`steps`](PercentSlider::steps) snaps to `n`
//! evenly-spaced positions including both endpoints (`steps(5)` → 0, 25,
//! 50, 75, 100); [`stops`](PercentSlider::stops) snaps to an explicit,
//! possibly non-uniform list of positions. When `steps` or `stops` is
//! set, the tick row renders at exactly those positions and the arrow
//! keys jump between them.

use egui::{
    Color32, CornerRadius, CursorIcon, Event, EventFilter, Key, Pos2, Rect, Response, Sense, Shape,
    Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo, WidgetText, WidgetType,
};

use crate::theme::{placeholder_galley, with_alpha, Accent, Theme, BASELINE_FRAC};

/// A 0–100% slider whose central UI element is the percentage value itself.
///
/// ```no_run
/// # use elegance::PercentSlider;
/// # egui::__run_test_ui(|ui| {
/// let mut share = 45.0_f32;
/// ui.add(
///     PercentSlider::new(&mut share)
///         .label("Cache window")
///         .total_fmt(|p| {
///             let mins = (p * 60.0 / 100.0).round() as i32;
///             format!("{mins} min")
///         }),
/// );
/// # });
/// ```
#[must_use = "Call `ui.add(...)` to render the widget."]
pub struct PercentSlider<'a> {
    value: &'a mut f32,
    label: Option<WidgetText>,
    accent: Accent,
    show_ticks: bool,
    step: Option<f32>,
    stops: Option<Vec<f32>>,
    decimals: usize,
    total_fmt: Option<Box<dyn Fn(f32) -> String + 'a>>,
    desired_width: Option<f32>,
}

impl<'a> std::fmt::Debug for PercentSlider<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PercentSlider")
            .field("accent", &self.accent)
            .field("show_ticks", &self.show_ticks)
            .field("step", &self.step)
            .field("stops", &self.stops)
            .field("decimals", &self.decimals)
            .field("desired_width", &self.desired_width)
            .finish()
    }
}

impl<'a> PercentSlider<'a> {
    /// Create a slider bound to `value`, clamped to `0.0..=100.0`.
    pub fn new(value: &'a mut f32) -> Self {
        Self {
            value,
            label: None,
            accent: Accent::Sky,
            show_ticks: true,
            step: None,
            stops: None,
            decimals: 0,
            total_fmt: None,
            desired_width: None,
        }
    }

    /// Add a small label in the top-left of the header row.
    #[inline]
    pub fn label(mut self, label: impl Into<WidgetText>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Pick the fill colour from one of the theme accents. Default: [`Accent::Sky`].
    #[inline]
    pub fn accent(mut self, accent: Accent) -> Self {
        self.accent = accent;
        self
    }

    /// Show or hide the quartile tick row beneath the track. Default: `true`.
    #[inline]
    pub fn show_ticks(mut self, show: bool) -> Self {
        self.show_ticks = show;
        self
    }

    /// Snap the value to multiples of `step` percentage points. Default:
    /// continuous. Common values: `5.0` for "round to 5%", `25.0` for
    /// quartile snap.
    ///
    /// Mutually exclusive with [`steps`](Self::steps) and
    /// [`stops`](Self::stops); calling this clears either of them.
    #[inline]
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self.stops = None;
        self
    }

    /// Snap to `n` evenly-spaced positions, including both endpoints.
    /// `steps(2)` snaps to `{0, 100}`, `steps(5)` to `{0, 25, 50, 75, 100}`,
    /// `steps(7)` to seven positions whose spacing the widget computes for
    /// you (so the awkward `100 / 6` math stays out of caller code).
    ///
    /// When set, the tick row renders at exactly those positions and the
    /// arrow keys jump between adjacent stops. Values below `2` are
    /// promoted to `2`. Mutually exclusive with [`step`](Self::step) and
    /// [`stops`](Self::stops); calling this clears either of them.
    #[inline]
    pub fn steps(mut self, n: usize) -> Self {
        let n = n.max(2);
        let last = (n - 1) as f32;
        self.stops = Some((0..n).map(|i| (i as f32 / last) * 100.0).collect());
        self.step = None;
        self
    }

    /// Snap to an explicit, possibly non-uniform list of positions in
    /// `0.0..=100.0`. Out-of-range, `NaN`, and duplicate values are filtered
    /// out; the result is sorted ascending. If fewer than two valid
    /// positions remain, falls back to `[0.0, 100.0]`.
    ///
    /// When set, the tick row renders at exactly these positions and the
    /// arrow keys jump between adjacent stops. Mutually exclusive with
    /// [`step`](Self::step) and [`steps`](Self::steps); calling this clears
    /// either of them.
    pub fn stops(mut self, positions: impl IntoIterator<Item = f32>) -> Self {
        let mut v: Vec<f32> = positions
            .into_iter()
            .filter(|p| p.is_finite() && (0.0..=100.0).contains(p))
            .collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        v.dedup_by(|a, b| (*a - *b).abs() < f32::EPSILON);
        if v.len() < 2 {
            v = vec![0.0, 100.0];
        }
        self.stops = Some(v);
        self.step = None;
        self
    }

    /// Number of decimal places in the headline value. Default: `0`.
    #[inline]
    pub fn decimals(mut self, n: usize) -> Self {
        self.decimals = n;
        self
    }

    /// Supply a callback to format the percentage as an *absolute* quantity,
    /// shown in a callout above the thumb while the user drags. The callback
    /// receives the current percentage in `0.0..=100.0` and returns a display
    /// string such as `"27 min"` or `"3.2 GB"`. When unset, no callout is
    /// shown.
    #[inline]
    pub fn total_fmt(mut self, fmt: impl Fn(f32) -> String + 'a) -> Self {
        self.total_fmt = Some(Box::new(fmt));
        self
    }

    /// Override the slider width. Defaults to `ui.available_width()`.
    #[inline]
    pub fn desired_width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }
}

impl<'a> Widget for PercentSlider<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());
        let p = &theme.palette;
        let t = &theme.typography;
        let accent_fill = p.accent_fill(self.accent);

        // Typography sizes for each row.
        let value_size = (t.heading * 1.5).round().max(20.0);
        let sign_size = t.label;
        let label_size = t.label;
        let tick_size = t.small;

        // Probe galley to discover the real header height; egui font metrics
        // vary slightly between platforms, so measure rather than guess.
        let probe = placeholder_galley(ui, "0", value_size, true, f32::INFINITY);
        let header_h = probe.size().y;

        let track_h: f32 = 6.0;
        let thumb_d: f32 = 14.0;
        let row_track_h = thumb_d.max(22.0);
        let header_gap: f32 = 8.0;
        let tick_gap: f32 = if self.show_ticks { 4.0 } else { 0.0 };
        let tick_row_h: f32 = if self.show_ticks {
            tick_size + 8.0
        } else {
            0.0
        };

        let total_h = header_h + header_gap + row_track_h + tick_gap + tick_row_h;
        let total_w = self
            .desired_width
            .unwrap_or_else(|| ui.available_width())
            .max(180.0);

        // Normalize and clamp the incoming value.
        let mut current = if self.value.is_nan() {
            0.0
        } else {
            *self.value
        };
        current = current.clamp(0.0, 100.0);

        let step = self.step.filter(|s| s.is_finite() && *s > 0.0);
        let stops = self.stops.as_deref();

        // Snap a raw 0..=100 percentage to whichever snap mode (if any) is
        // configured: explicit stops > step > continuous.
        let snap = |raw: f32| -> f32 {
            let v = if let Some(stops) = stops {
                nearest_stop(stops, raw)
            } else if let Some(s) = step {
                (raw / s).round() * s
            } else {
                raw
            };
            v.clamp(0.0, 100.0)
        };

        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::new(total_w, total_h), Sense::click_and_drag());

        // Geometry: the thumb-centre travels between `track_left` and
        // `track_right`; the visible pill extends a thumb-radius further on
        // each side so the thumb sits cleanly inside it at the extremes.
        let thumb_pad = thumb_d * 0.5;
        let track_y = rect.min.y + header_h + header_gap + row_track_h * 0.5;
        let track_left = rect.min.x + thumb_pad;
        let track_right = rect.max.x - thumb_pad;
        let track_span = (track_right - track_left).max(1.0);
        let track_rect = Rect::from_min_max(
            Pos2::new(rect.min.x, track_y - track_h * 0.5),
            Pos2::new(rect.max.x, track_y + track_h * 0.5),
        );

        // Update value while the pointer button is held within the widget.
        if response.is_pointer_button_down_on() {
            if let Some(pos) = response.interact_pointer_pos() {
                let clamped_x = pos.x.clamp(track_left, track_right);
                let frac = ((clamped_x - track_left) / track_span).clamp(0.0, 1.0);
                let next = snap(frac * 100.0);
                if (next - current).abs() > f32::EPSILON {
                    current = next;
                    *self.value = current;
                    response.mark_changed();
                }
            }
        }

        // Keyboard nudges when the slider has focus. Left / Right step by
        // `step` (or 1 percentage point); Shift bumps to 10x; Home / End jump
        // to 0 / 100. Up / Down are intentionally left to egui's focus
        // navigation so the user can move between stacked controls vertically.
        //
        // `set_focus_lock_filter` claims horizontal arrows for this widget
        // while it's focused; without it, egui's focus system consumes
        // Left / Right for spatial navigation before the value handler below
        // ever sees them.
        if response.has_focus() {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    response.id,
                    EventFilter {
                        horizontal_arrows: true,
                        ..Default::default()
                    },
                );
            });

            let small_step = step.unwrap_or(1.0);
            let big_step = step.map(|s| s * 10.0).unwrap_or(10.0);
            let events = ui.input(|input| input.events.clone());
            for ev in &events {
                if let Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = ev
                {
                    let nudge = if modifiers.shift {
                        big_step
                    } else {
                        small_step
                    };
                    // With discrete stops, arrows jump to the adjacent stop;
                    // Home / End snap to the first / last. Without stops,
                    // fall back to nudge-and-snap by `step` (or continuous).
                    let raw = match (stops, key) {
                        (Some(stops), Key::ArrowLeft) => adjacent_stop(stops, current, -1),
                        (Some(stops), Key::ArrowRight) => adjacent_stop(stops, current, 1),
                        (Some(stops), Key::Home) => stops.first().copied(),
                        (Some(stops), Key::End) => stops.last().copied(),
                        (None, Key::ArrowLeft) => Some(current - nudge),
                        (None, Key::ArrowRight) => Some(current + nudge),
                        (None, Key::Home) => Some(0.0),
                        (None, Key::End) => Some(100.0),
                        _ => None,
                    };
                    if let Some(next) = raw {
                        let next = snap(next);
                        if (next - current).abs() > f32::EPSILON {
                            current = next;
                            *self.value = current;
                            response.mark_changed();
                        }
                    }
                }
            }
        }

        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }
        if response.is_pointer_button_down_on() {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter().clone();

            // ---- Header row ----------------------------------------------
            let decimals = self.decimals;
            let num_text = format!("{current:.decimals$}");
            let num_galley = placeholder_galley(ui, &num_text, value_size, true, f32::INFINITY);
            let sign_galley = placeholder_galley(ui, "%", sign_size, false, f32::INFINITY);

            // Baseline-align the small "%" with the large numeric.
            let baseline_y = rect.min.y + num_galley.size().y * BASELINE_FRAC;
            let sign_x = rect.max.x - sign_galley.size().x;
            let num_x = sign_x - 2.0 - num_galley.size().x;
            painter.galley(Pos2::new(num_x, rect.min.y), num_galley, p.text);
            painter.galley(
                Pos2::new(sign_x, baseline_y - sign_galley.size().y * BASELINE_FRAC),
                sign_galley,
                p.text_muted,
            );

            if let Some(label) = &self.label {
                let label_wrap = (num_x - rect.min.x - 8.0).max(0.0);
                let label_galley =
                    placeholder_galley(ui, label.text(), label_size, false, label_wrap);
                let label_top = baseline_y - label_galley.size().y * BASELINE_FRAC;
                painter.galley(Pos2::new(rect.min.x, label_top), label_galley, p.text);
            }

            // ---- Track ----------------------------------------------------
            let frac = (current / 100.0).clamp(0.0, 1.0);
            let thumb_x = track_left + track_span * frac;
            let thumb_center = Pos2::new(thumb_x, track_y);

            let track_radius = CornerRadius::same((track_h * 0.5).round() as u8);

            painter.rect(
                track_rect,
                track_radius,
                p.input_bg,
                Stroke::new(1.0, p.border),
                StrokeKind::Inside,
            );
            if thumb_x > track_rect.min.x + 0.5 {
                let fill_rect = Rect::from_min_max(
                    Pos2::new(track_rect.min.x, track_rect.min.y),
                    Pos2::new(thumb_x, track_rect.max.y),
                );
                painter.rect_filled(fill_rect, track_radius, accent_fill);
            }

            // Faint 10% interior divisions, slightly stronger at the midpoint.
            // Painted on top of both the filled and unfilled portions so the
            // proportion reads at a glance like a battery indicator without
            // breaking the smooth-fill feel.
            let div_base = if p.is_dark {
                Color32::WHITE
            } else {
                Color32::BLACK
            };
            for i in 1..10 {
                let f = i as f32 / 10.0;
                let x = track_rect.min.x + track_rect.width() * f;
                let alpha = if i == 5 { 48 } else { 24 };
                painter.line_segment(
                    [
                        Pos2::new(x, track_rect.min.y + 1.5),
                        Pos2::new(x, track_rect.max.y - 1.5),
                    ],
                    Stroke::new(1.0, with_alpha(div_base, alpha)),
                );
            }

            // Focus / drag halo.
            if response.has_focus() || response.is_pointer_button_down_on() {
                painter.circle_filled(
                    thumb_center,
                    thumb_d * 0.5 + 4.0,
                    with_alpha(accent_fill, 55),
                );
            }
            // Thumb: pale fill, accent-coloured ring.
            painter.circle(
                thumb_center,
                thumb_d * 0.5,
                p.text,
                Stroke::new(2.0, accent_fill),
            );

            // ---- Ticks ---------------------------------------------------
            // With explicit stops: render a tick at each, all medium weight
            // (no major/minor distinction). Without stops: the default
            // quartile row, with 0 / 50% / 100% drawn slightly heavier than
            // 25% / 75%.
            if self.show_ticks {
                let tick_top_y = rect.min.y + header_h + header_gap + row_track_h + tick_gap;
                let ticks: Vec<(f32, String, bool)> = match stops {
                    Some(stops) => stops
                        .iter()
                        .map(|p| {
                            let label = if *p == 0.0 {
                                "0".to_string()
                            } else {
                                format!("{}%", p.round() as i32)
                            };
                            (p / 100.0, label, false)
                        })
                        .collect(),
                    None => vec![
                        (0.00, "0".to_string(), true),
                        (0.25, "25%".to_string(), false),
                        (0.50, "50%".to_string(), true),
                        (0.75, "75%".to_string(), false),
                        (1.00, "100%".to_string(), true),
                    ],
                };
                for (f, label, major) in &ticks {
                    let x = track_left + track_span * f;
                    let tick_h = if *major { 5.0 } else { 3.5 };
                    let tick_color = if *major { p.text_muted } else { p.border };
                    painter.line_segment(
                        [Pos2::new(x, tick_top_y), Pos2::new(x, tick_top_y + tick_h)],
                        Stroke::new(1.0, tick_color),
                    );
                    let g = placeholder_galley(ui, label, tick_size, false, f32::INFINITY);
                    let raw_x = x - g.size().x * 0.5;
                    let tx = raw_x.max(rect.min.x).min(rect.max.x - g.size().x);
                    let ty = tick_top_y + tick_h + 1.0;
                    painter.galley(Pos2::new(tx, ty), g, p.text_faint);
                }
            }

            // ---- Drag callout --------------------------------------------
            // The callout floats above the track during interaction. It's
            // the only place the absolute (`total_fmt`) value appears, so it
            // skips rendering when no formatter is configured.
            if response.is_pointer_button_down_on() {
                if let Some(fmt) = &self.total_fmt {
                    paint_callout(
                        ui,
                        &theme,
                        &painter,
                        thumb_center,
                        rect,
                        thumb_d,
                        current,
                        decimals,
                        fmt(current),
                    );
                }
            }
        }

        let label_str = self
            .label
            .as_ref()
            .map(|l| l.text().to_string())
            .unwrap_or_default();
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Slider, true, &label_str));
        response
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_callout(
    ui: &egui::Ui,
    theme: &Theme,
    painter: &egui::Painter,
    thumb_center: Pos2,
    widget_rect: Rect,
    thumb_d: f32,
    current: f32,
    decimals: usize,
    total: String,
) {
    let p = &theme.palette;
    let t = &theme.typography;
    let text = format!("{current:.decimals$}% \u{00B7} {total}");
    let g = placeholder_galley(ui, &text, t.label, false, f32::INFINITY);

    let pad_x: f32 = 9.0;
    let pad_y: f32 = 5.0;
    let tail_h: f32 = 5.0;
    let cw = g.size().x + 2.0 * pad_x;
    let ch = g.size().y + 2.0 * pad_y;

    // Centre over the thumb, clamping into the widget's horizontal extent
    // so the callout never escapes the parent layout.
    let half_w = cw * 0.5;
    let edge_pad = 4.0;
    let cx = thumb_center.x.clamp(
        widget_rect.min.x + half_w + edge_pad,
        widget_rect.max.x - half_w - edge_pad,
    );
    let cy_bottom = thumb_center.y - thumb_d * 0.5 - tail_h - 2.0;
    let crect = Rect::from_min_max(
        Pos2::new(cx - half_w, cy_bottom - ch),
        Pos2::new(cx + half_w, cy_bottom),
    );

    let fill = p.depth_tint(p.input_bg, 0.04);
    let border = p.border;

    painter.rect(
        crect,
        CornerRadius::same(5),
        fill,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );

    // Tail: triangle pointing at the thumb. Drawn so its top edge sits flush
    // with the callout's bottom; a small fill cap hides the seam between the
    // bordered rect and the triangle.
    let tail_x = thumb_center
        .x
        .clamp(crect.min.x + tail_h + 2.0, crect.max.x - tail_h - 2.0);
    let tail_top_y = crect.max.y;
    let tip = Pos2::new(tail_x, tail_top_y + tail_h);
    let left = Pos2::new(tail_x - tail_h, tail_top_y);
    let right = Pos2::new(tail_x + tail_h, tail_top_y);
    painter.add(Shape::convex_polygon(
        vec![left, tip, right],
        fill,
        Stroke::new(1.0, border),
    ));
    // Hide the seam where the triangle meets the rect.
    painter.line_segment(
        [
            Pos2::new(tail_x - tail_h + 1.0, tail_top_y),
            Pos2::new(tail_x + tail_h - 1.0, tail_top_y),
        ],
        Stroke::new(1.5, fill),
    );

    painter.galley(
        Pos2::new(crect.min.x + pad_x, crect.min.y + pad_y),
        g,
        p.text,
    );
}

/// Snap `raw` to the nearest entry in `stops`. `stops` is assumed sorted and
/// non-empty; the builder maintains both invariants.
fn nearest_stop(stops: &[f32], raw: f32) -> f32 {
    let mut best = stops[0];
    let mut best_d = (raw - best).abs();
    for &s in &stops[1..] {
        let d = (raw - s).abs();
        if d < best_d {
            best_d = d;
            best = s;
        }
    }
    best
}

/// Return the stop immediately above (`dir > 0`) or below (`dir < 0`) `current`,
/// or `None` if `current` is already at the corresponding extreme. A small
/// epsilon lets the comparison treat a value that just snapped to a stop as
/// being "on" that stop rather than infinitesimally past it.
fn adjacent_stop(stops: &[f32], current: f32, dir: i32) -> Option<f32> {
    let eps = 0.001;
    if dir > 0 {
        stops.iter().copied().find(|&s| s > current + eps)
    } else {
        stops.iter().rev().copied().find(|&s| s < current - eps)
    }
}
