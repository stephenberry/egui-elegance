//! Commit signal — "this adjustment has settled".
//!
//! [`egui::Response::changed`] fires on every intermediate value a drag sweeps
//! through, which is what a live preview wants. It is the wrong signal for work
//! you would not want to repeat dozens of times for one gesture: a network
//! write, a disk persist, a device reprogram.
//!
//! [`ResponseCommitExt::committed`] reports the frame an adjustment settles
//! instead. See the trait for what counts as settled.
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

use egui::{Context, Id, NUM_POINTER_BUTTONS, PointerButton, Response};

/// Every pointer button egui can report.
///
/// The length is tied to [`NUM_POINTER_BUTTONS`], so if egui ever gains a
/// button this array stops compiling instead of silently leaving that button
/// uncommitted — the failure mode this list exists to prevent.
const ALL_POINTER_BUTTONS: [PointerButton; NUM_POINTER_BUTTONS] = [
    PointerButton::Primary,
    PointerButton::Secondary,
    PointerButton::Middle,
    PointerButton::Extra1,
    PointerButton::Extra2,
];

/// Memory key for a widget-reported commit. Stores the pass number the commit
/// was reported on, so reading it is idempotent within a pass and an entry
/// cannot fire on a later one.
fn commit_pass_id(id: Id) -> Id {
    id.with("elegance::commit_pass")
}

/// Report that an adjustment on `id` settled this pass.
///
/// A [`Response`] records *that* a widget changed, never *what drove it*, so
/// non-pointer input cannot be classified from the outside: a keyboard nudge
/// and a smoothed scroll frame look identical there. Rather than guess, each
/// elegance value widget reports its own non-pointer settle through this
/// channel, at the point where it already knows which input it just handled.
///
/// `id` must be the id of the [`Response`] the widget *returns*. For a widget
/// that unions several responses that is the first one, since
/// [`Response::union`] keeps the left id.
pub(crate) fn mark_commit(ctx: &Context, id: Id) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| d.insert_temp(commit_pass_id(id), pass));
}

/// Whether a widget reported a commit for the pass now being laid out.
fn widget_reported_commit(ctx: &Context, id: Id) -> bool {
    let pass = ctx.cumulative_pass_nr();
    ctx.data(|d| d.get_temp::<u64>(commit_pass_id(id)) == Some(pass))
}

/// Was this widget clicked by *any* pointer button?
///
/// [`Response::clicked`] is primary-only, but every elegance value widget
/// gates its pointer write on a button-agnostic predicate, so a click with any
/// button moves the value and therefore has to commit.
fn clicked_by_any_button(response: &Response) -> bool {
    // `clicked()` is not redundant with the Primary entry below: it also covers
    // the click egui synthesises for Space/Enter on a focused widget and for
    // accessibility activation, neither of which involves a real button.
    response.clicked()
        || ALL_POINTER_BUTTONS
            .iter()
            .any(|&button| response.clicked_by(button))
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
    /// gesture has an end. On a widget from outside this crate it reports
    /// pointer settle only: keyboard and scroll adjustments are reported by
    /// each elegance widget from the inside, which a foreign widget cannot do.
    ///
    /// It is **not** for text widgets either: clicking into a
    /// [`TextInput`](crate::TextInput) would commit, which says nothing about
    /// whether the text is final. Use `lost_focus()` plus an Enter check there,
    /// as [`ResponseFlashExt`](crate::ResponseFlashExt) does.
    ///
    /// # This reports a settled interaction, not a different value
    ///
    /// Grabbing a handle and releasing it without moving it still reports
    /// committed once, as does pressing Space or Enter on a focused widget
    /// (egui synthesises a click) — reachable straight after a click, now that
    /// grabbing a value widget focuses it — or aborting a drag with Escape
    /// (egui has no
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
        // missed term is a write the consumer silently never performs.
        //
        // * `drag_stopped` — pointer release that ended a drag. Button-agnostic
        //   in egui, so a right-drag ends here too.
        // * `clicked_by_any_button` — pointer release that never became a drag.
        // * `long_touched` — a touch held past the click duration. egui clears
        //   both potential-click and potential-drag ids at that point, so the
        //   eventual lift produces neither `clicked` nor `drag_stopped`; without
        //   this term a touch user who pauses to aim never commits at all.
        // * `widget_reported_commit` — everything the terms above cannot see,
        //   reported by the widget itself. See `mark_commit`.
        //
        // Deliberately absent: any term reading `changed()`. Inferring "this
        // change was a settled one" from the response alone requires guessing
        // which input drove it, and that guess is wrong for scroll — whose
        // delta egui smooths across many frames — which would fire a burst of
        // commits for a single wheel notch. Attribution belongs where the input
        // is handled, so widgets report it rather than this predicate inferring
        // it.
        self.drag_stopped()
            || clicked_by_any_button(self)
            || self.long_touched()
            || widget_reported_commit(&self.ctx, self.id)
    }
}
