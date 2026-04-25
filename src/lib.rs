//! Elegance — opinionated, beautiful widgets for egui.
//!
//! Elegance is a small companion crate to [`egui`] that provides a cohesive
//! design system inspired by modern web UIs: chunky rounded buttons in a
//! handful of accent colors, crisp inputs with a focus ring, pill-shaped
//! status indicators, cards, tabs, segmented buttons, and a matching colour
//! palette. Four palettes ship built-in — two dark
//! ([`Theme::slate`] and [`Theme::charcoal`]) and two light
//! ([`Theme::frost`] and [`Theme::paper`]) — paired so you can toggle without
//! a layout shift.
//!
//! # Getting started
//!
//! ```no_run
//! use eframe::egui;
//! use elegance::{Theme, Button, Card, Accent};
//!
//! fn main() -> eframe::Result<()> {
//!     eframe::run_ui_native(
//!         "elegance demo",
//!         eframe::NativeOptions::default(),
//!         |ui, _| {
//!             Theme::slate().install(ui.ctx());
//!             egui::CentralPanel::default().show_inside(ui, |ui| {
//!                 Card::new().heading("Hello").show(ui, |ui| {
//!                     if ui.add(Button::new("Click me").accent(Accent::Blue))
//!                         .clicked()
//!                     {
//!                         println!("clicked!");
//!                     }
//!                 });
//!             });
//!         },
//!     )
//! }
//! ```
//!
//! # Design
//!
//! All visuals are driven by a [`Theme`] value. Calling [`Theme::install`]
//! once at startup configures [`egui::Style`] so that built-in widgets
//! (labels, sliders, etc.) inherit the elegance look, and it stores the
//! theme in `ctx` memory so elegance widgets can pick it up automatically.

#![warn(missing_debug_implementations)]
#![deny(missing_docs)]

mod accordion;
mod badge;
mod browser_tabs;
mod button;
mod callout;
mod card;
mod checkbox;
mod collapsing;
mod color_picker;
mod drawer;
mod file_drop_zone;
mod flash;
mod indicator;
mod input;
mod knob;
mod log_bar;
mod menu;
mod menu_bar;
mod modal;
mod multi_terminal;
mod pairing;
mod pill;
mod popover;
mod progress_bar;
mod progress_ring;
mod range_slider;
mod segmented;
mod select;
mod slider;
mod spinner;
mod steps;
mod switch;
mod tabs;
mod text_area;
mod theme;
mod theme_switcher;
mod toast;
mod tooltip;

pub use accordion::{Accordion, AccordionItem, AccordionUi};
pub use badge::{Badge, BadgeTone};
pub use browser_tabs::{BrowserTab, BrowserTabs, BrowserTabsEvent};
pub use button::{Button, ButtonSize};
pub use callout::{Callout, CalloutTone};
pub use card::Card;
pub use checkbox::Checkbox;
pub use collapsing::CollapsingSection;
pub use color_picker::ColorPicker;
pub use drawer::{Drawer, DrawerSide};
pub use file_drop_zone::{FileDropResponse, FileDropZone};
pub use flash::{flash_error, flash_success, FlashKind, ResponseFlashExt, FLASH_DURATION};
pub use indicator::{Indicator, IndicatorState};
pub use input::TextInput;
pub use knob::{Knob, KnobScale, KnobSize};
pub use log_bar::{LogBar, LogEntry, LogKind};
pub use menu::{Menu, MenuItem, SubMenuItem};
pub use menu_bar::{MenuBar, MenuBarUi};
pub use modal::Modal;
pub use multi_terminal::{
    LineKind, MultiTerminal, TerminalEvent, TerminalLine, TerminalPane, TerminalStatus,
};
pub use pairing::{PairItem, Pairing};
pub use pill::StatusPill;
pub use popover::{Popover, PopoverSide};
pub use progress_bar::ProgressBar;
pub use progress_ring::ProgressRing;
pub use range_slider::RangeSlider;
pub use segmented::SegmentedButton;
pub use select::Select;
pub use slider::Slider;
pub use spinner::Spinner;
pub use steps::{Steps, StepsStyle};
pub use switch::Switch;
pub use tabs::TabBar;
pub use text_area::TextArea;
pub use theme::{Accent, BuiltInTheme, Palette, Theme, Typography};
pub use theme_switcher::ThemeSwitcher;
pub use toast::{Toast, Toasts};
pub use tooltip::{Tooltip, TooltipSide};

/// Re-export of [`egui`] for convenience.
pub use egui;

/// Stable codepoints for the icon glyphs bundled in the Elegance Symbols
/// font. All icons are sourced from [Lucide](https://lucide.dev) and are
/// kept in sync via `scripts/update_lucide_glyphs.py`. Use these in
/// [`egui::RichText`] when you want one of elegance's icons in your own
/// UI.
///
/// ```no_run
/// # use elegance::glyphs;
/// # egui::__run_test_ui(|ui| {
/// ui.label(egui::RichText::new(glyphs::UPLOAD).size(24.0));
/// # });
/// ```
pub mod glyphs {
    /// Upload-tray icon. Source: [Lucide `upload`](https://lucide.dev/icons/upload).
    pub const UPLOAD: char = '\u{E000}';
    /// Download-tray icon. Source: [Lucide `download`](https://lucide.dev/icons/download).
    pub const DOWNLOAD: char = '\u{E001}';
    /// Search / magnifier icon. Source: [Lucide `search`](https://lucide.dev/icons/search).
    pub const SEARCH: char = '\u{E002}';
    /// Pin icon. Source: [Lucide `pin`](https://lucide.dev/icons/pin).
    pub const PIN: char = '\u{E003}';
    /// Copy / duplicate icon. Source: [Lucide `copy`](https://lucide.dev/icons/copy).
    pub const COPY: char = '\u{E004}';
    /// Circular alert icon. Source: [Lucide `circle-alert`](https://lucide.dev/icons/circle-alert).
    pub const CIRCLE_ALERT: char = '\u{E005}';
    /// Network / hub icon. Source: [Lucide `network`](https://lucide.dev/icons/network).
    pub const NETWORK: char = '\u{E006}';
    /// Zoom-in (magnifier with `+`) icon. Source: [Lucide `zoom-in`](https://lucide.dev/icons/zoom-in).
    pub const ZOOM_IN: char = '\u{E007}';
    /// Zoom-out (magnifier with `-`) icon. Source: [Lucide `zoom-out`](https://lucide.dev/icons/zoom-out).
    pub const ZOOM_OUT: char = '\u{E008}';
    /// Power icon. Source: [Lucide `power`](https://lucide.dev/icons/power).
    pub const POWER: char = '\u{E009}';
    /// Check / done mark, mapped at standard U+2713 so plain `'✓'` literals
    /// also pick up the elegance treatment.
    /// Source: [Lucide `check`](https://lucide.dev/icons/check).
    pub const CHECK: char = '\u{2713}';
    /// Cross / dismiss mark, mapped at standard U+2717 so plain `'✗'` literals
    /// also pick up the elegance treatment.
    /// Source: [Lucide `x`](https://lucide.dev/icons/x).
    pub const X: char = '\u{2717}';
}

/// Request a repaint such that the next paint comes ~`1/hz` seconds from now,
/// independent of display refresh rate.
///
/// [`egui::Context::request_repaint_after`] internally subtracts `predicted_dt`
/// from the requested delay to budget for the paint taking time. On a 60 Hz
/// integration (egui's default) that subtraction is ~16.7 ms, so a naive
/// `request_repaint_after(1/30 s)` lands on the very next vsync and produces
/// ~60 Hz — double the rate you asked for. This helper adds `predicted_dt`
/// back in so the effective cadence lands near `1/hz` on any refresh rate.
///
/// Typical use: throttle continuously-animating widgets (spinners, progress
/// fills) to 20–30 Hz so they don't burn a full vsync budget on motion the
/// eye can't resolve.
#[track_caller]
pub fn request_repaint_at_rate(ctx: &egui::Context, hz: f32) {
    let pd = ctx.input(|i| i.predicted_dt);
    if let Ok(d) = std::time::Duration::try_from_secs_f32(1.0 / hz + pd) {
        ctx.request_repaint_after(d);
    }
}
