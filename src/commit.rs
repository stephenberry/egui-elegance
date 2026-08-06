//! Commit signal — "this adjustment has settled".
//!
//! [`egui::Response::changed`] fires on every intermediate value a drag sweeps
//! through, which is what a live preview wants. It is the wrong signal for work
//! you would not want to repeat dozens of times for one gesture: a network
//! write, a disk persist, a device reprogram. Dragging a
//! [`MetricSlider`](crate::MetricSlider) in `stops` mode from one end of the
//! rail to the other reports `changed()` once per stop crossed, each carrying a
//! value the user never meant to commit.
//!
//! [`ResponseCommitExt::committed`] reports the frame an adjustment settles
//! instead, and applies to every elegance value widget.
//!
//! # Usage
//!
//! ```no_run
//! # use elegance::{MetricSlider, ResponseCommitExt};
//! # fn push_to_backend(_: f32) {}
//! # fn preview(_: f32) {}
//! # egui::__run_test_ui(|ui| {
//! let mut buffer = 16.0_f32;
//! let resp = ui.add(
//!     MetricSlider::new(&mut buffer, 0.0..=32.0)
//!         .suffix("GiB")
//!         .stops([4.0, 8.0, 16.0, 32.0]),
//! );
//! if resp.changed() {
//!     preview(buffer); // live, every intermediate stop
//! }
//! if resp.committed() {
//!     push_to_backend(buffer); // once per settled adjustment
//! }
//! # });
//! ```

use egui::{Context, Id, PointerButton, Response};

/// Memory key for a widget-reported commit. Stores the pass number the commit
/// was reported on, so reading it is idempotent within a frame and stale
/// entries cannot fire on a later one.
fn commit_pass_id(id: Id) -> Id {
    id.with("elegance::commit_pass")
}

/// Report that an adjustment on `id` settled this pass.
///
/// For input a [`Response`] alone cannot classify. The generic predicate reads
/// pointer release and key presses; a widget driven by *continuous non-pointer*
/// input (a scroll wheel, whose delta egui deliberately smooths across many
/// frames) has to decide for itself when the gesture stopped and say so.
pub(crate) fn mark_commit(ctx: &Context, id: Id) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| d.insert_temp(commit_pass_id(id), pass));
}

/// Whether a widget reported a commit for the pass now being laid out.
fn widget_reported_commit(ctx: &Context, id: Id) -> bool {
    let pass = ctx.cumulative_pass_nr();
    ctx.data(|d| d.get_temp::<u64>(commit_pass_id(id)) == Some(pass))
}

/// Did the user press a key this frame?
///
/// This is what separates a keyboard nudge from a scroll: both change the value
/// with no pointer button down, but only one of them is a discrete, already
/// settled adjustment.
fn key_pressed_this_frame(ctx: &Context) -> bool {
    ctx.input(|i| {
        i.events
            .iter()
            .any(|e| matches!(e, egui::Event::Key { pressed: true, .. }))
    })
}

/// Extension trait for reading the *commit* signal off a value widget's
/// [`Response`]. Import this trait (or `use elegance::*`) to bring the method
/// into scope.
pub trait ResponseCommitExt {
    /// True on the frame an adjustment settles, and only then.
    ///
    /// Where [`Response::changed`] fires on every intermediate value a drag
    /// passes through, this fires once per deliberate adjustment: on pointer
    /// release after a drag or a click, and immediately on a keyboard nudge,
    /// which is already atomic. Use it to gate expensive or outward-facing
    /// work.
    ///
    /// Read the bound value as usual; it is already up to date on the frame
    /// this returns true.
    ///
    /// # Which widgets this is for
    ///
    /// The elegance value widgets: [`MetricSlider`](crate::MetricSlider),
    /// [`PercentSlider`](crate::PercentSlider), [`Slider`](crate::Slider),
    /// [`RangeSlider`](crate::RangeSlider), [`Knob`](crate::Knob).
    ///
    /// The trait is implemented on [`Response`] itself, so the method is
    /// callable on any widget's response, but it is only meaningful where a
    /// gesture has an end. It is **not** for text widgets: a
    /// [`TextInput`](crate::TextInput) reports `changed()` per keystroke with no
    /// pointer down, so `committed()` would fire per keystroke too. Use
    /// `lost_focus()` plus an Enter check there, as
    /// [`ResponseFlashExt`](crate::ResponseFlashExt) does.
    ///
    /// # This reports a settled interaction, not a different value
    ///
    /// Grabbing a handle and releasing it without moving it still reports
    /// committed once, as does pressing Space or Enter on a focused widget
    /// (egui synthesises a click), or aborting a drag with Escape (egui has no
    /// drag-revert, so the value stays where the pointer left it). A widget
    /// disabled between a drag and its release loses the commit for a value it
    /// already wrote. Each is one signal per gesture rather than the
    /// one-per-frame `changed()` would give. If a redundant write is costly,
    /// compare against your last-written value.
    fn committed(&self) -> bool;
}

impl ResponseCommitExt for Response {
    #[inline]
    fn committed(&self) -> bool {
        // Each term covers one way a gesture can end. They can overlap without
        // harm — `||` still yields one commit per frame — but between them they
        // must cover every modality that can move a bound value, because a
        // missed term means a write the consumer silently never performs.
        //
        // * `drag_stopped` — pointer release that ended a drag. Button-agnostic
        //   in egui, so a right-drag ends here too.
        // * `clicked` / `clicked_by` — pointer release that never became a
        //   drag. `clicked()` is Primary-only, but every elegance value widget
        //   gates its write on `is_pointer_button_down_on()`, which is
        //   button-agnostic, so a secondary or middle click moves the value and
        //   has to commit as well.
        // * `long_touched` — a touch held past the click duration. egui clears
        //   both potential-click and potential-drag ids at that point, so the
        //   eventual lift produces neither `clicked` nor `drag_stopped`; without
        //   this term a touch user who pauses to aim never commits at all.
        // * `changed && !is_pointer_button_down_on && key_pressed` — a keyboard
        //   nudge, discrete and therefore already settled.
        // * `widget_reported_commit` — the escape hatch for input the terms
        //   above cannot classify. See `mark_commit`.
        //
        // Two guards are load-bearing and look over-specified:
        //
        // The keyboard term must test `!is_pointer_button_down_on()` and not
        // `!dragged()`. A widget sensing both click and drag (which every
        // elegance value widget does) does not become `dragged` until the
        // pointer is decidedly dragging, so on the press frame of a
        // click-to-set the value has already been written while `dragged()` is
        // still false. Guarding on `!dragged()` would commit on that press frame
        // and then again via `clicked()` on release.
        // `is_pointer_button_down_on()` is true from the press frame onward and
        // is forced false on the release frame, which is exactly the window to
        // exclude.
        //
        // It must also test `key_pressed_this_frame`. Without that, any
        // pointerless change commits — including a scroll wheel, whose delta
        // egui smooths across many frames, which would fire a burst of commits
        // for one notch on `Knob`. That is the precise failure this signal
        // exists to prevent.
        self.drag_stopped()
            || self.clicked()
            || self.clicked_by(PointerButton::Secondary)
            || self.clicked_by(PointerButton::Middle)
            || self.long_touched()
            || (self.changed()
                && !self.is_pointer_button_down_on()
                && key_pressed_this_frame(&self.ctx))
            || widget_reported_commit(&self.ctx, self.id)
    }
}
