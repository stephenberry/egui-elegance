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

use egui::Response;

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
    /// Applies to any elegance value widget: [`MetricSlider`](crate::MetricSlider),
    /// [`PercentSlider`](crate::PercentSlider), [`Slider`](crate::Slider),
    /// [`RangeSlider`](crate::RangeSlider), [`Knob`](crate::Knob).
    ///
    /// Read the bound value as usual; it is already up to date on the frame
    /// this returns true.
    ///
    /// # This reports a settled interaction, not a different value
    ///
    /// Grabbing a handle and releasing it without moving it still reports
    /// committed once, as does pressing Space or Enter on a focused widget
    /// (egui synthesises a click), or aborting a drag with Escape (egui has no
    /// drag-revert, so the value stays where the pointer left it). Each is one
    /// signal per gesture rather than the one-per-frame `changed()` would give.
    /// If a redundant write is costly, compare against your last-written value.
    fn committed(&self) -> bool;
}

impl ResponseCommitExt for Response {
    fn committed(&self) -> bool {
        // The three terms cover one modality each, and are chosen to fire at
        // most once per gesture:
        //
        // * `drag_stopped` — pointer release that ended a drag.
        // * `clicked` — pointer release that never became a drag. egui decides
        //   between the two on release (a click requires the pointer not be
        //   "decidedly dragging"), so these are mutually exclusive.
        // * `changed && !is_pointer_button_down_on` — a keyboard nudge, which
        //   commits immediately because it is already discrete.
        //
        // The guard on the third term must be `!is_pointer_button_down_on()`
        // and not `!dragged()`. A widget sensing both click and drag (which
        // every elegance value widget does) does not become `dragged` until
        // the pointer is decidedly dragging, so on the press frame of a
        // click-to-set the value has already been written while `dragged()` is
        // still false. Guarding on `!dragged()` would commit on that press
        // frame and then again via `clicked()` on release.
        // `is_pointer_button_down_on()` is true from the press frame onward and
        // is forced false on the release frame, which is exactly the window to
        // exclude.
        self.drag_stopped()
            || self.clicked()
            || (self.changed() && !self.is_pointer_button_down_on())
    }
}
