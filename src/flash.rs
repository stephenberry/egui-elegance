//! Success / error flash feedback.
//!
//! A short green or red tint on the widget background that fades back to
//! the normal colour over ~0.8 s, used to confirm the outcome of a submit.
//!
//! # Usage
//!
//! After a submit completes, call [`ResponseFlashExt::flash_success`] or
//! [`ResponseFlashExt::flash_error`] on the widget's [`egui::Response`].
//! The next frames will render the flash and animate it out.
//!
//! ```no_run
//! # use elegance::{TextInput, ResponseFlashExt};
//! # egui::__run_test_ui(|ui| {
//! let mut text = String::new();
//! let resp = ui.add(TextInput::new(&mut text).id_salt("mix_freq"));
//! if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
//!     // Pretend we submitted and got a result.
//!     let ok: bool = true;
//!     if ok { resp.flash_success(); } else { resp.flash_error(); }
//! }
//! # });
//! ```
//!
//! Currently consumed by [`crate::TextInput`].

use egui::{Color32, Context, Id, Response};

use crate::theme::{mix, Theme};

/// The duration of a flash animation, in seconds.
pub const FLASH_DURATION: f64 = 0.8;

/// Flash outcome. Controls the colour that tints the widget background.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlashKind {
    /// Green tint — submit succeeded.
    Success,
    /// Red tint — submit failed.
    Error,
}

/// Extension trait that lets you trigger a flash directly on an
/// [`egui::Response`]. Import this trait (or `use elegance::*`) to bring
/// the methods into scope.
pub trait ResponseFlashExt {
    /// Begin a success flash on the widget this [`Response`] came from.
    ///
    /// Call once right after a submit returns `Ok`. Safe to re-call — the
    /// animation restarts from the beginning.
    fn flash_success(&self);

    /// Begin an error flash on the widget this [`Response`] came from.
    fn flash_error(&self);

    /// Immediately clear any flash state on the widget this [`Response`]
    /// came from. Rarely needed — flashes clear themselves after
    /// [`FLASH_DURATION`].
    fn clear_flash(&self);
}

impl ResponseFlashExt for Response {
    fn flash_success(&self) {
        start(&self.ctx, self.id, FlashKind::Success);
    }

    fn flash_error(&self) {
        start(&self.ctx, self.id, FlashKind::Error);
    }

    fn clear_flash(&self) {
        self.ctx.data_mut(|d| d.remove::<FlashState>(self.id));
        self.ctx.request_repaint();
    }
}

/// Begin a success flash on the widget with the given id. Use when you only
/// have the widget id and a [`Context`], e.g. when completing an async callback
/// that returned after the originating [`Response`] went out of scope.
pub fn flash_success(ctx: &Context, id: Id) {
    start(ctx, id, FlashKind::Success);
}

/// Begin an error flash on the widget with the given id. Async counterpart
/// to [`ResponseFlashExt::flash_error`].
pub fn flash_error(ctx: &Context, id: Id) {
    start(ctx, id, FlashKind::Error);
}

// --- internals --------------------------------------------------------------

#[derive(Clone, Copy)]
pub(crate) struct FlashState {
    kind: FlashKind,
    started_at: f64,
}

pub(crate) fn start(ctx: &Context, id: Id, kind: FlashKind) {
    let now = ctx.input(|i| i.time);
    ctx.data_mut(|d| {
        d.insert_temp(
            id,
            FlashState {
                kind,
                started_at: now,
            },
        );
    });
    ctx.request_repaint();
}

/// Return the current flash for `id`, if one is active. `progress` is in
/// `0.0..=1.0` where `0.0` is "just started" and `1.0` is "just about to
/// finish".
///
/// Side effects:
///  * clears expired flash state
///  * requests a repaint while a flash is active
pub(crate) fn active_flash(ctx: &Context, id: Id) -> Option<(FlashKind, f32)> {
    let state = ctx.data(|d| d.get_temp::<FlashState>(id))?;
    let now = ctx.input(|i| i.time);
    let elapsed = now - state.started_at;
    if !(0.0..FLASH_DURATION).contains(&elapsed) {
        ctx.data_mut(|d| d.remove::<FlashState>(id));
        return None;
    }
    ctx.request_repaint();
    Some((state.kind, (elapsed / FLASH_DURATION) as f32))
}

/// Compute the widget background fill for an input-style widget, given
/// the normal fill and an optional active flash. Uses quadratic ease-out
/// so the tint fades quickly at first then lingers slightly.
pub(crate) fn background_fill(
    theme: &Theme,
    base_fill: Color32,
    flash: Option<(FlashKind, f32)>,
) -> Color32 {
    let Some((kind, progress)) = flash else {
        return base_fill;
    };
    // Ease-out: remaining = (1 - t)^2, so intensity drops fast then settles.
    let remaining = (1.0 - progress).powi(2);
    // Mix the accent 25% (0x40 alpha) into the base fill — the peak tint
    // at flash start.
    const PEAK_MIX: f32 = 0x40 as f32 / 255.0;
    let accent = match kind {
        FlashKind::Success => theme.palette.green,
        FlashKind::Error => theme.palette.red,
    };
    mix(base_fill, accent, PEAK_MIX * remaining)
}
