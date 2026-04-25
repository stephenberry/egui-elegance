//! Tooltip — a hover-triggered, themed callout that explains a trigger widget.
//!
//! A [`Tooltip`] attaches to any [`Response`] and surfaces a small bordered
//! card after a short hover delay. The card has a required body line plus an
//! optional bold heading and an optional shortcut row (label + key chips).
//! Reach for it for one-line labels on icon buttons, "what is this?" hints
//! next to field labels, and short explanations of status pills.
//!
//! Visibility is driven by egui's tooltip system, so all the standard niceties
//! come for free: a delay before the first tooltip in a session, a grace
//! window after that during which moving onto a sibling shows its tooltip
//! immediately, and dismiss-on-click / dismiss-on-scroll. For a
//! click-anchored panel that the user can interact with, reach for
//! [`Popover`](crate::Popover) instead.
//!
//! ```no_run
//! # use elegance::{Button, Tooltip};
//! # egui::__run_test_ui(|ui| {
//! let trigger = ui.add(Button::new("Save"));
//! Tooltip::new("Write the working tree to disk.")
//!     .heading("Save changes")
//!     .shortcut("\u{2318} S")
//!     .show(&trigger);
//! # });
//! ```

use egui::{
    emath::RectAlign, Color32, CornerRadius, Frame, Margin, Pos2, Rect, Response, Sense, Shape,
    Stroke, Ui, Vec2, WidgetText,
};

use crate::theme::Theme;

/// Where the tooltip opens relative to its trigger.
///
/// The chosen side is honoured even when there's not enough room on that
/// side; pick the side with care for triggers near the viewport edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TooltipSide {
    /// Above the trigger; arrow points down. The default.
    Top,
    /// Below the trigger; arrow points up.
    Bottom,
    /// Left of the trigger; arrow points right.
    Left,
    /// Right of the trigger; arrow points left.
    Right,
}

impl TooltipSide {
    fn to_rect_align(self) -> RectAlign {
        match self {
            TooltipSide::Top => RectAlign::TOP,
            TooltipSide::Bottom => RectAlign::BOTTOM,
            TooltipSide::Left => RectAlign::LEFT,
            TooltipSide::Right => RectAlign::RIGHT,
        }
    }
}

/// A hover-triggered tooltip attached to a [`Response`].
///
/// Construct with the body text and optionally layer on a heading or a
/// keyboard-shortcut row, then call [`Tooltip::show`] immediately after
/// rendering the trigger.
#[derive(Clone, Debug)]
#[must_use = "Call `.show(&trigger)` to render the tooltip."]
pub struct Tooltip {
    body: WidgetText,
    heading: Option<WidgetText>,
    shortcut: Option<String>,
    shortcut_label: String,
    side: TooltipSide,
    width: Option<f32>,
    arrow: bool,
    gap: f32,
}

impl Tooltip {
    /// Create a tooltip with the given body text.
    ///
    /// Defaults: anchored above the trigger, no heading, no shortcut row,
    /// arrow on, ~8 pt gap between trigger and tooltip.
    pub fn new(body: impl Into<WidgetText>) -> Self {
        Self {
            body: body.into(),
            heading: None,
            shortcut: None,
            shortcut_label: "Shortcut".into(),
            side: TooltipSide::Top,
            width: None,
            arrow: true,
            gap: 8.0,
        }
    }

    /// Add a bold heading line above the body.
    #[inline]
    pub fn heading(mut self, heading: impl Into<WidgetText>) -> Self {
        self.heading = Some(heading.into());
        self
    }

    /// Add a keyboard-shortcut row at the bottom of the tooltip.
    ///
    /// The string is split on whitespace and each token is rendered as a
    /// small monospace chip, so `"\u{2318} S"` renders as two chips.
    #[inline]
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Override the leading label on the shortcut row. Default: `"Shortcut"`.
    #[inline]
    pub fn shortcut_label(mut self, label: impl Into<String>) -> Self {
        self.shortcut_label = label.into();
        self
    }

    /// Which side of the trigger to anchor on. Default: [`TooltipSide::Top`].
    #[inline]
    pub fn side(mut self, side: TooltipSide) -> Self {
        self.side = side;
        self
    }

    /// Fix the tooltip's max content width, in points. Long body text wraps
    /// at this width. Default: 260.
    #[inline]
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    /// Toggle the small arrow that points at the trigger. Default: on.
    #[inline]
    pub fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
        self
    }

    /// Gap between the trigger and the tooltip, in points. Default: 8.
    #[inline]
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Render the tooltip attached to `trigger`. Returns `Some(response)`
    /// while the tooltip is visible, `None` otherwise.
    pub fn show(self, trigger: &Response) -> Option<Response> {
        let theme = Theme::current(&trigger.ctx);
        let p = &theme.palette;

        let frame = Frame::new()
            .fill(p.card)
            .stroke(Stroke::new(1.0, p.border))
            .corner_radius(CornerRadius::same(theme.control_radius as u8))
            .inner_margin(Margin::symmetric(10, 8));

        let mut tip = egui::Tooltip::for_enabled(trigger);
        tip.popup = tip
            .popup
            .frame(frame)
            .align(self.side.to_rect_align())
            .align_alternatives(&[])
            .gap(self.gap);
        tip.popup = tip.popup.width(self.width.unwrap_or(260.0));

        let trigger_rect = trigger.rect;
        let trigger_ctx = trigger.ctx.clone();
        let arrow = self.arrow;
        let side = self.side;
        let theme_for_paint = theme.clone();
        let heading = self.heading;
        let body = self.body;
        let shortcut = self.shortcut;
        let shortcut_label = self.shortcut_label;

        let inner = tip.show(move |ui| {
            paint_contents(
                ui,
                &theme_for_paint,
                heading.as_ref(),
                &body,
                shortcut.as_deref(),
                &shortcut_label,
            );
        })?;

        if arrow {
            let actual_side = detect_side(trigger_rect, inner.response.rect, side);
            paint_arrow(
                &trigger_ctx,
                inner.response.layer_id,
                inner.response.rect,
                trigger_rect,
                actual_side,
                theme.palette.card,
                theme.palette.border,
            );
        }

        Some(inner.response)
    }
}

fn paint_contents(
    ui: &mut Ui,
    theme: &Theme,
    heading: Option<&WidgetText>,
    body: &WidgetText,
    shortcut: Option<&str>,
    shortcut_label: &str,
) {
    let p = &theme.palette;
    let t = &theme.typography;

    if let Some(h) = heading {
        ui.add(
            egui::Label::new(
                egui::RichText::new(h.text())
                    .color(p.text)
                    .size(t.body)
                    .strong(),
            )
            .wrap_mode(egui::TextWrapMode::Wrap),
        );
        ui.add_space(2.0);
    }

    ui.add(
        egui::Label::new(
            egui::RichText::new(body.text())
                .color(p.text_muted)
                .size(t.small),
        )
        .wrap_mode(egui::TextWrapMode::Wrap),
    );

    if let Some(sc) = shortcut {
        ui.add_space(6.0);
        let avail = ui.available_width();
        let sep_y = ui.cursor().min.y;
        ui.painter().line_segment(
            [
                Pos2::new(ui.cursor().min.x, sep_y),
                Pos2::new(ui.cursor().min.x + avail, sep_y),
            ],
            Stroke::new(1.0, p.border),
        );
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            ui.add(egui::Label::new(
                egui::RichText::new(shortcut_label)
                    .color(p.text_faint)
                    .size(t.small),
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                // right_to_left iterates in reverse, so push tokens in reverse
                // order to render them left-to-right at the right edge.
                let tokens: Vec<&str> = sc.split_whitespace().collect();
                for token in tokens.iter().rev() {
                    add_kbd(ui, token, theme);
                }
            });
        });
    }
}

fn add_kbd(ui: &mut Ui, text: &str, theme: &Theme) -> Response {
    let p = &theme.palette;
    let font_id = egui::FontId::monospace(11.0);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font_id, p.text);
    let pad_x = 5.0;
    let width = (galley.size().x + pad_x * 2.0).max(16.0);
    let height = 18.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().rect(
            rect,
            CornerRadius::same(3),
            p.input_bg,
            Stroke::new(1.0, p.border),
            egui::StrokeKind::Inside,
        );
        let pos = Pos2::new(
            rect.center().x - galley.size().x * 0.5,
            rect.center().y - galley.size().y * 0.5,
        );
        ui.painter().galley(pos, galley, p.text);
    }
    response
}

fn detect_side(trigger: Rect, popup: Rect, requested: TooltipSide) -> TooltipSide {
    match requested {
        TooltipSide::Top | TooltipSide::Bottom => {
            if popup.center().y < trigger.center().y {
                TooltipSide::Top
            } else {
                TooltipSide::Bottom
            }
        }
        TooltipSide::Left | TooltipSide::Right => {
            if popup.center().x < trigger.center().x {
                TooltipSide::Left
            } else {
                TooltipSide::Right
            }
        }
    }
}

fn paint_arrow(
    ctx: &egui::Context,
    layer: egui::LayerId,
    popup: Rect,
    trigger: Rect,
    side: TooltipSide,
    fill: Color32,
    border: Color32,
) {
    let painter = ctx.layer_painter(layer);
    let half_base = 6.0;
    let depth = 6.0;
    let inset = 10.0;

    let (base_center, perp, base_axis) = match side {
        TooltipSide::Bottom => {
            let cx = trigger
                .center()
                .x
                .clamp(popup.min.x + inset, popup.max.x - inset);
            (
                Pos2::new(cx, popup.min.y),
                Vec2::new(0.0, -1.0),
                Vec2::new(1.0, 0.0),
            )
        }
        TooltipSide::Top => {
            let cx = trigger
                .center()
                .x
                .clamp(popup.min.x + inset, popup.max.x - inset);
            (
                Pos2::new(cx, popup.max.y),
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 0.0),
            )
        }
        TooltipSide::Right => {
            let cy = trigger
                .center()
                .y
                .clamp(popup.min.y + inset, popup.max.y - inset);
            (
                Pos2::new(popup.min.x, cy),
                Vec2::new(-1.0, 0.0),
                Vec2::new(0.0, 1.0),
            )
        }
        TooltipSide::Left => {
            let cy = trigger
                .center()
                .y
                .clamp(popup.min.y + inset, popup.max.y - inset);
            (
                Pos2::new(popup.max.x, cy),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
            )
        }
    };

    let base_a = base_center + base_axis * half_base;
    let base_b = base_center - base_axis * half_base;
    let tip = base_center + perp * depth;

    painter.add(Shape::convex_polygon(
        vec![base_a, tip, base_b],
        fill,
        Stroke::NONE,
    ));
    painter.line_segment([base_a, base_b], Stroke::new(1.5, fill));
    let stroke = Stroke::new(1.0, border);
    painter.line_segment([base_a, tip], stroke);
    painter.line_segment([base_b, tip], stroke);
}
