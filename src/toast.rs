//! Non-blocking notification toasts.
//!
//! Two types cooperate:
//!
//! * [`Toast`] — a builder that **enqueues** one notification via
//!   [`Toast::show`]. Takes only `&Context`, so it can fire from any
//!   callback that has access to the egui context (button handlers, input
//!   events, async completion callbacks, …).
//! * [`Toasts`] — the renderer. Call [`Toasts::new()`]`.render(ctx)` once
//!   per frame in your top-level `update`. Without this call, enqueued
//!   toasts silently accumulate and nothing is shown.
//!
//! # Usage
//!
//! ```no_run
//! # use elegance::{BadgeTone, Toast, Toasts};
//! # let ctx = egui::Context::default();
//! // Somewhere in your update loop:
//! Toasts::new().render(&ctx);
//!
//! // From any callback with access to the context:
//! Toast::new("Deploy complete")
//!     .tone(BadgeTone::Ok)
//!     .description("Rolled out to us-east-1")
//!     .show(&ctx);
//! ```

use std::{collections::VecDeque, time::Duration};

use egui::{
    accesskit, Align2, Area, Color32, Context, CornerRadius, Id, Order, Pos2, Rect, Response,
    Sense, Stroke, StrokeKind, Ui, Vec2,
};

use crate::theme::Theme;
use crate::BadgeTone;

/// How long the fade-out animation takes, in seconds. Counted against
/// a toast's total lifetime (i.e., the toast disappears at
/// `birth + duration + FADE_OUT`).
const FADE_OUT: f64 = 0.20;
/// Default auto-dismiss duration, in seconds.
const DEFAULT_DURATION: f64 = 4.0;
/// Default stack cap — older toasts are dropped when this is exceeded.
const DEFAULT_MAX_VISIBLE: usize = 5;
/// Default width of a toast card, in points.
const DEFAULT_WIDTH: f32 = 320.0;
/// Vertical gap between stacked toasts, in points.
const STACK_GAP: f32 = 8.0;

fn storage_id() -> Id {
    Id::new("elegance::toasts")
}

/// A single enqueued notification.
///
/// Construct with [`Toast::new`], configure via the builder methods, then
/// call [`Toast::show`] to enqueue. The toast is rendered the next time
/// [`Toasts::render`] runs.
#[derive(Debug, Clone)]
#[must_use = "Call `show(ctx)` to enqueue the toast."]
pub struct Toast {
    title: String,
    description: Option<String>,
    tone: BadgeTone,
    duration: Option<Duration>,
}

impl Toast {
    /// Create a toast with a title. Defaults: [`BadgeTone::Info`], auto-dismiss
    /// after `DEFAULT_DURATION` seconds.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            tone: BadgeTone::Info,
            duration: Some(Duration::from_secs_f64(DEFAULT_DURATION)),
        }
    }

    /// Pick the tone (drives the left accent bar colour).
    pub fn tone(mut self, tone: BadgeTone) -> Self {
        self.tone = tone;
        self
    }

    /// Add a secondary line below the title.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Override how long the toast stays visible before it starts fading out.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Disable auto-dismiss. The toast stays until the user clicks × or
    /// another toast pushes it out of the stack (see [`Toasts::max_visible`]).
    pub fn persistent(mut self) -> Self {
        self.duration = None;
        self
    }

    /// Enqueue the toast. It is shown on the next frame that renders
    /// [`Toasts`].
    pub fn show(self, ctx: &Context) {
        let now = ctx.input(|i| i.time);
        ctx.data_mut(|d| {
            let mut state = d.get_temp::<ToastState>(storage_id()).unwrap_or_default();
            let id = state.next_id;
            state.next_id = state.next_id.wrapping_add(1);
            state.queue.push_back(ToastEntry {
                id,
                title: self.title,
                description: self.description,
                tone: self.tone,
                duration: self.duration.map(|d| d.as_secs_f64()),
                birth: now,
                dismiss_start: None,
            });
            d.insert_temp(storage_id(), state);
        });
        ctx.request_repaint();
    }
}

/// Renderer for the enqueued toast stack.
///
/// Configure placement via the builder, then call [`Toasts::render`] once
/// per frame. Multiple `Toasts::render` calls per frame are a mistake —
/// each one will paint the whole stack.
#[derive(Debug, Clone)]
#[must_use = "Call `.render(ctx)` to draw the toast stack."]
pub struct Toasts {
    anchor: Align2,
    offset: Vec2,
    max_visible: usize,
    width: f32,
}

impl Default for Toasts {
    fn default() -> Self {
        Self::new()
    }
}

impl Toasts {
    /// Start a new configuration. Defaults: anchored to the bottom-right
    /// with a 12-pt offset, up to 5 toasts visible, 320-pt wide.
    pub fn new() -> Self {
        Self {
            anchor: Align2::RIGHT_BOTTOM,
            offset: Vec2::new(12.0, 12.0),
            max_visible: DEFAULT_MAX_VISIBLE,
            width: DEFAULT_WIDTH,
        }
    }

    /// Anchor corner for the stack. Default: [`Align2::RIGHT_BOTTOM`].
    pub fn anchor(mut self, anchor: Align2) -> Self {
        self.anchor = anchor;
        self
    }

    /// Offset from the anchor corner, in points. Default: `(12, 12)`.
    pub fn offset(mut self, offset: impl Into<Vec2>) -> Self {
        self.offset = offset.into();
        self
    }

    /// Maximum number of toasts rendered at once. Oldest are dropped when
    /// the cap is exceeded. Default: 5.
    pub fn max_visible(mut self, max_visible: usize) -> Self {
        self.max_visible = max_visible.max(1);
        self
    }

    /// Width of each toast card in points. Default: 320.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(120.0);
        self
    }

    /// Render the enqueued toast stack. Call once per frame.
    pub fn render(self, ctx: &Context) {
        let theme = Theme::current(ctx);
        let now = ctx.input(|i| i.time);

        // Snapshot state under a short lock, then hand the lock back.
        let mut state = ctx
            .data_mut(|d| d.get_temp::<ToastState>(storage_id()))
            .unwrap_or_default();

        // Expire fully-faded toasts, then cap the queue to max_visible by
        // dropping oldest (front).
        state.queue.retain(|entry| !entry.is_expired(now));
        while state.queue.len() > self.max_visible {
            state.queue.pop_front();
        }

        if state.queue.is_empty() {
            ctx.data_mut(|d| d.insert_temp(storage_id(), state));
            return;
        }

        // Paint toasts. We lay out manually (not via egui's own stacking)
        // so we can track sizes on the Area that each toast lives in.
        let screen = ctx.content_rect();
        let stack_up = matches!(self.anchor.y(), egui::Align::Max);

        // Compute each toast's height so we can stack them without depending
        // on previous-frame measurements.
        let entry_heights: Vec<f32> = state
            .queue
            .iter()
            .map(|e| measure_height(ctx, &theme, e, self.width))
            .collect();

        // x position of the stack.
        let x = match self.anchor.x() {
            egui::Align::Min => screen.min.x + self.offset.x,
            egui::Align::Center => screen.center().x - self.width * 0.5,
            egui::Align::Max => screen.max.x - self.offset.x - self.width,
        };

        // Starting y and step direction.
        let (mut y, step_sign): (f32, f32) = if stack_up {
            (screen.max.y - self.offset.y, -1.0)
        } else {
            (screen.min.y + self.offset.y, 1.0)
        };

        // Newest toast sits closest to the anchor edge; iterate accordingly.
        let order_is_new_to_old = stack_up;
        let indices: Vec<usize> = if order_is_new_to_old {
            (0..state.queue.len()).rev().collect()
        } else {
            (0..state.queue.len()).collect()
        };

        let mut dismiss_ids: Vec<u64> = Vec::new();
        let mut earliest_next_event: Option<f64> = None;
        let mut any_animating = false;

        for i in indices {
            let entry = &state.queue[i];
            let h = entry_heights[i];

            let (top, bottom) = if step_sign < 0.0 {
                (y - h, y)
            } else {
                (y, y + h)
            };
            let rect = Rect::from_min_max(Pos2::new(x, top), Pos2::new(x + self.width, bottom));

            // Animating = currently in fade-in or fade-out.
            let (alpha, is_animating, next_event) = entry.alpha_and_schedule(now);
            any_animating |= is_animating;
            if let Some(t) = next_event {
                earliest_next_event = Some(match earliest_next_event {
                    Some(prev) => prev.min(t),
                    None => t,
                });
            }

            let area_id = Id::new(("elegance::toast", entry.id));
            let resp = Area::new(area_id)
                .order(Order::Tooltip)
                .fixed_pos(rect.min)
                .show(ctx, |ui| paint_toast(ui, &theme, entry, rect, alpha));

            if resp.inner {
                dismiss_ids.push(entry.id);
            }

            // Advance the cursor for the next toast.
            let delta = (h + STACK_GAP) * step_sign;
            y += delta;
        }

        // Record clicks into dismiss_start so next frame's alpha math picks
        // them up.
        if !dismiss_ids.is_empty() {
            for entry in state.queue.iter_mut() {
                if dismiss_ids.contains(&entry.id) && entry.dismiss_start.is_none() {
                    entry.dismiss_start = Some(now);
                }
            }
        }

        ctx.data_mut(|d| d.insert_temp(storage_id(), state));

        // Keep animating smoothly; otherwise schedule the next transition.
        if any_animating {
            ctx.request_repaint();
        } else if let Some(at) = earliest_next_event {
            let remaining = (at - now).max(0.0);
            ctx.request_repaint_after(Duration::from_secs_f64(remaining));
        }
    }
}

// -- internals ---------------------------------------------------------------

#[derive(Clone, Default)]
struct ToastState {
    queue: VecDeque<ToastEntry>,
    next_id: u64,
}

#[derive(Clone)]
struct ToastEntry {
    id: u64,
    title: String,
    description: Option<String>,
    tone: BadgeTone,
    /// Auto-dismiss duration in seconds. `None` = persistent.
    duration: Option<f64>,
    /// Context time when the toast was enqueued.
    birth: f64,
    /// Context time when the user clicked ×. Triggers an immediate fade-out.
    dismiss_start: Option<f64>,
}

impl ToastEntry {
    /// Has the fade-out animation completed?
    fn is_expired(&self, now: f64) -> bool {
        if let Some(ds) = self.dismiss_start {
            return now >= ds + FADE_OUT;
        }
        if let Some(d) = self.duration {
            return now >= self.birth + d + FADE_OUT;
        }
        false
    }

    /// Returns `(alpha, is_animating, next_transition_time)`.
    ///
    /// Toasts appear at full opacity and fade out only. `is_animating` is
    /// true while the fade-out is in progress (we repaint continuously
    /// during it). `next_transition_time` is `Some(t)` when the toast is
    /// still at full opacity and we want a single deferred repaint at `t`
    /// to start the fade-out.
    fn alpha_and_schedule(&self, now: f64) -> (f32, bool, Option<f64>) {
        // Fade-out: either explicit dismiss, or past the auto-dismiss instant.
        let fade_out_start = match self.dismiss_start {
            Some(ds) => Some(ds),
            None => self.duration.map(|d| self.birth + d),
        };

        match fade_out_start {
            Some(t0) if now >= t0 => {
                let progress = ((now - t0) / FADE_OUT).clamp(0.0, 1.0) as f32;
                (1.0 - progress, progress < 1.0, None)
            }
            Some(t0) => (1.0, false, Some(t0)),
            None => (1.0, false, None),
        }
    }
}

fn tone_accent(theme: &Theme, tone: BadgeTone) -> Color32 {
    let p = &theme.palette;
    match tone {
        BadgeTone::Ok => p.success,
        BadgeTone::Warning => p.warning,
        BadgeTone::Danger => p.danger,
        BadgeTone::Info => p.sky,
        BadgeTone::Neutral => p.text_muted,
    }
}

fn apply_alpha(color: Color32, alpha: f32) -> Color32 {
    let a = (color.a() as f32 * alpha.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}

/// Layout constants shared between measurement and painting.
mod layout {
    pub const PAD_X: f32 = 14.0;
    pub const PAD_Y: f32 = 10.0;
    pub const BAR_W: f32 = 3.0;
    pub const BAR_GAP: f32 = 10.0;
    pub const TITLE_DESC_GAP: f32 = 3.0;
    pub const CLOSE_W: f32 = 18.0;
    pub const CLOSE_GAP: f32 = 8.0;
    pub const TEXT_LEFT_NUDGE: f32 = 4.0;

    /// Shared so `measure_height` and `paint_toast` wrap against the same
    /// width — otherwise the stack lays out against a height the paint
    /// path doesn't reproduce.
    pub fn text_wrap_width(card_width: f32) -> f32 {
        (card_width - PAD_X * 1.5 - BAR_W - BAR_GAP - CLOSE_W - CLOSE_GAP + TEXT_LEFT_NUDGE)
            .max(1.0)
    }
}

fn measure_height(ctx: &Context, theme: &Theme, entry: &ToastEntry, width: f32) -> f32 {
    use layout::*;
    let t = &theme.typography;

    // Lay out with Color32::PLACEHOLDER so the galley cache entry is shared
    // with paint_toast, which fills the final (alpha'd) color at paint time
    // via painter.galley(..., fallback_color). Using a concrete color here
    // would produce a different cache key and double the work during fades.
    let text_width = text_wrap_width(width);
    let title_galley = ctx.fonts_mut(|f| {
        f.layout(
            entry.title.clone(),
            egui::FontId::proportional(t.body),
            Color32::PLACEHOLDER,
            text_width,
        )
    });

    let mut h = PAD_Y * 2.0 + title_galley.size().y;
    if let Some(desc) = &entry.description {
        let desc_galley = ctx.fonts_mut(|f| {
            f.layout(
                desc.clone(),
                egui::FontId::proportional(t.small),
                Color32::PLACEHOLDER,
                text_width,
            )
        });
        h += TITLE_DESC_GAP + desc_galley.size().y;
    }
    h.max(44.0)
}

/// Paint a single toast inside its area. Returns `true` if the close button
/// was clicked this frame.
fn paint_toast(ui: &mut Ui, theme: &Theme, entry: &ToastEntry, rect: Rect, alpha: f32) -> bool {
    use layout::*;
    let p = &theme.palette;
    let t = &theme.typography;

    // Upgrade the Area's Ui role from `GenericContainer` (set by
    // `Ui::new`) to an ARIA live-region role. Danger/Warning toasts use
    // `Role::Alert` (assertive — interrupts the user); others use
    // `Role::Status` (polite — announced after current speech).
    let role = match entry.tone {
        BadgeTone::Danger | BadgeTone::Warning => accesskit::Role::Alert,
        _ => accesskit::Role::Status,
    };
    let label = entry.title.clone();
    let description = entry.description.clone();
    ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
        node.set_role(role);
        node.set_label(label);
        if let Some(d) = description {
            node.set_description(d);
        }
    });

    // Claim the full toast rect so clicks don't pass through to widgets beneath.
    ui.allocate_rect(rect, Sense::hover());
    let painter = ui.painter();

    // Card background.
    let bg = apply_alpha(p.depth_tint(p.card, 0.04), alpha);
    let border = apply_alpha(p.border, alpha);
    painter.rect(
        rect,
        CornerRadius::same(theme.card_radius as u8),
        bg,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );

    // Left accent bar.
    let accent = apply_alpha(tone_accent(theme, entry.tone), alpha);
    let bar_rect = Rect::from_min_max(
        Pos2::new(rect.min.x + 4.0, rect.min.y + 6.0),
        Pos2::new(rect.min.x + 4.0 + BAR_W, rect.max.y - 6.0),
    );
    painter.rect_filled(bar_rect, CornerRadius::same(2), accent);

    // Close × in the top-right.
    let close_rect = Rect::from_min_size(
        Pos2::new(rect.max.x - PAD_X * 0.5 - CLOSE_W, rect.min.y + 6.0),
        Vec2::new(CLOSE_W, CLOSE_W),
    );
    let close_resp: Response = ui.allocate_rect(close_rect, Sense::click());
    let close_color = if close_resp.hovered() {
        apply_alpha(p.text, alpha)
    } else {
        apply_alpha(p.text_muted, alpha)
    };
    let close_galley = crate::theme::placeholder_galley(ui, "×", t.body + 2.0, true, f32::INFINITY);
    let close_text_pos = Pos2::new(
        close_rect.center().x - close_galley.size().x * 0.5,
        close_rect.center().y - close_galley.size().y * 0.5,
    );
    ui.painter()
        .galley(close_text_pos, close_galley, close_color);

    // Text block: title + optional description, to the right of the bar.
    let text_left = rect.min.x + PAD_X + BAR_W + BAR_GAP - TEXT_LEFT_NUDGE;
    let text_width = text_wrap_width(rect.width());

    let title_color = apply_alpha(p.text, alpha);
    let desc_color = apply_alpha(p.text_muted, alpha);

    // Lay out with Color32::PLACEHOLDER and supply the real (alpha'd) color
    // to painter.galley as fallback_color. This shares the cache entry with
    // measure_height and avoids a fresh layout every frame during the fade.
    let title_galley = ui.ctx().fonts_mut(|f| {
        f.layout(
            entry.title.clone(),
            egui::FontId::proportional(t.body),
            Color32::PLACEHOLDER,
            text_width,
        )
    });
    let title_size_y = title_galley.size().y;
    let title_pos = Pos2::new(text_left, rect.min.y + PAD_Y);
    ui.painter().galley(title_pos, title_galley, title_color);

    if let Some(desc) = &entry.description {
        let desc_galley = ui.ctx().fonts_mut(|f| {
            f.layout(
                desc.clone(),
                egui::FontId::proportional(t.small),
                Color32::PLACEHOLDER,
                text_width,
            )
        });
        let desc_pos = Pos2::new(
            text_left,
            rect.min.y + PAD_Y + title_size_y + TITLE_DESC_GAP,
        );
        ui.painter().galley(desc_pos, desc_galley, desc_color);
    }

    close_resp.clicked()
}
