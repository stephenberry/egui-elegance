//! Expandable bottom log bar — a collapsible console for network traffic or event messages.
//!
//! [`LogBar`] owns its own ring buffer of [`LogEntry`] rows and renders a
//! [`egui::Panel::bottom`] with a [`CollapsingSection`](crate::CollapsingSection)
//! trigger, a scrollable monospace log area (capped at 200 pt), and a Clear
//! button. Call [`LogBar::show`] once per frame inside your `App::ui`,
//! before your `CentralPanel`.
//!
//! Entries are prepended (newest at top) and capped at [`LogBar::max_entries`]
//! — older rows are silently dropped once the buffer is full.
//!
//! # Example
//!
//! ```no_run
//! use elegance::{LogBar, Theme};
//!
//! struct App { log: LogBar }
//!
//! impl Default for App {
//!     fn default() -> Self {
//!         let mut log = LogBar::new().heading("Events");
//!         log.sys("Connected");
//!         Self { log }
//!     }
//! }
//!
//! impl eframe::App for App {
//!     fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
//!         Theme::slate().install(ui.ctx());
//!         self.log.show(ui);
//!         egui::CentralPanel::default().show_inside(ui, |ui| {
//!             if ui.button("Reload").clicked() {
//!                 self.log.out("reload_config");
//!                 self.log.recv("ok");
//!             }
//!         });
//!     }
//! }
//! ```
//!
//! The four [`LogKind`] variants map to different colours and arrow prefixes
//! in the spirit of a browser devtools console: outgoing requests, incoming
//! responses, errors, and plain system messages.

use std::collections::VecDeque;
use std::hash::Hash;

use egui::Id;

use crate::theme::Theme;
use crate::{Button, ButtonSize, CollapsingSection};

/// How a single log row is styled and prefixed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogKind {
    /// System or status message — faint text, no arrow.
    Sys,
    /// Outgoing request — muted text with a "→" prefix.
    Out,
    /// Incoming response — success-green text with a "←" prefix.
    In,
    /// Error — danger-red text, no arrow.
    Err,
}

/// A single log row.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp string, `HH:MM:SS` UTC by default.
    pub time: String,
    /// Row style.
    pub kind: LogKind,
    /// Message body.
    pub msg: String,
}

const DEFAULT_CAPACITY: usize = 100;
const SCROLL_MAX_HEIGHT: f32 = 200.0;

/// An expandable log bar anchored to the bottom of the viewport.
///
/// Owns its own entry buffer — construct once, store on your app struct,
/// call [`LogBar::push`] (or one of the shortcuts) from any event handler,
/// and call [`LogBar::show`] once per frame to render.
#[derive(Debug)]
pub struct LogBar {
    entries: VecDeque<LogEntry>,
    open: bool,
    capacity: usize,
    id_salt: Id,
    heading: String,
}

impl Default for LogBar {
    fn default() -> Self {
        Self::new()
    }
}

impl LogBar {
    /// Create a log bar with default settings: capacity 100, heading
    /// `"Message Log"`, starts collapsed.
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            open: false,
            capacity: DEFAULT_CAPACITY,
            id_salt: Id::new("elegance::log_bar"),
            heading: "Message Log".into(),
        }
    }

    /// Set the label shown in the collapsed header. Default: `"Message Log"`.
    pub fn heading(mut self, heading: impl Into<String>) -> Self {
        self.heading = heading.into();
        self
    }

    /// Set the maximum number of entries kept. When the buffer is full the
    /// oldest entry is dropped on each push. Default: 100. Clamped to a
    /// minimum of 1.
    pub fn max_entries(mut self, n: usize) -> Self {
        self.capacity = n.max(1);
        while self.entries.len() > self.capacity {
            self.entries.pop_back();
        }
        self
    }

    /// Override the id used for the panel and collapse state. Set this if
    /// you want more than one `LogBar` in a single app.
    pub fn id_salt(mut self, salt: impl Hash) -> Self {
        self.id_salt = Id::new(("elegance::log_bar", salt));
        self
    }

    /// Append an entry. The current wall-clock time (`HH:MM:SS` UTC) is
    /// recorded as the row's timestamp.
    pub fn push(&mut self, kind: LogKind, msg: impl Into<String>) {
        self.entries.push_front(LogEntry {
            time: now_hms(),
            kind,
            msg: msg.into(),
        });
        while self.entries.len() > self.capacity {
            self.entries.pop_back();
        }
    }

    /// Shortcut for `push(LogKind::Sys, msg)`.
    pub fn sys(&mut self, msg: impl Into<String>) {
        self.push(LogKind::Sys, msg);
    }

    /// Shortcut for `push(LogKind::Out, msg)`.
    pub fn out(&mut self, msg: impl Into<String>) {
        self.push(LogKind::Out, msg);
    }

    /// Shortcut for `push(LogKind::In, msg)` — named `recv` because `in`
    /// is a Rust keyword.
    pub fn recv(&mut self, msg: impl Into<String>) {
        self.push(LogKind::In, msg);
    }

    /// Shortcut for `push(LogKind::Err, msg)`.
    pub fn err(&mut self, msg: impl Into<String>) {
        self.push(LogKind::Err, msg);
    }

    /// Remove all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of entries currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Whether the bar is currently expanded.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Programmatically set the expanded state.
    pub fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    /// Iterate entries from newest to oldest.
    pub fn entries(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter()
    }

    /// Render the bar. Call once per frame, **before** your `CentralPanel`,
    /// inside your `App::ui` method.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let fill = Theme::current(ui.ctx()).palette.card;
        egui::Panel::bottom(self.id_salt)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(fill)
                    .inner_margin(egui::Margin::symmetric(16, 6)),
            )
            .show_inside(ui, |ui| {
                let theme = Theme::current(ui.ctx());
                let count = self.entries.len();
                let label = if count == 0 {
                    self.heading.clone()
                } else {
                    format!("{}  \u{00b7}  {count}", self.heading)
                };

                let mut clear = false;
                CollapsingSection::new(self.id_salt.with("collapse"), label)
                    .open(&mut self.open)
                    .show(ui, |ui| {
                        ui.add_space(2.0);
                        egui::ScrollArea::vertical()
                            .max_height(SCROLL_MAX_HEIGHT)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                if self.entries.is_empty() {
                                    ui.add(egui::Label::new(theme.faint_text("(no messages)")));
                                } else {
                                    for entry in self.entries.iter() {
                                        log_row(ui, &theme, entry);
                                    }
                                }
                            });
                        ui.add_space(6.0);
                        if ui
                            .add(Button::new("Clear").outline().size(ButtonSize::Small))
                            .clicked()
                        {
                            clear = true;
                        }
                    });
                if clear {
                    self.entries.clear();
                }
            });
    }
}

fn now_hms() -> String {
    let day = now_unix_secs() % 86400;
    format!("{:02}:{:02}:{:02}", day / 3600, (day / 60) % 60, day % 60)
}

// `std::time::SystemTime::now()` panics on `wasm32-unknown-unknown`; use the
// JS `Date` binding there instead.
#[cfg(not(target_family = "wasm"))]
fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(target_family = "wasm")]
fn now_unix_secs() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

fn log_row(ui: &mut egui::Ui, theme: &Theme, entry: &LogEntry) {
    let p = &theme.palette;
    let t = &theme.typography;
    let (color, arrow) = match entry.kind {
        LogKind::Sys => (p.text_faint, ""),
        LogKind::Out => (p.text_muted, "\u{2192} "),
        LogKind::In => (p.success, "\u{2190} "),
        LogKind::Err => (p.danger, ""),
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add(egui::Label::new(
            egui::RichText::new(&entry.time)
                .monospace()
                .color(p.text_faint)
                .size(t.small),
        ));
        ui.add_space(10.0);
        ui.add(egui::Label::new(
            egui::RichText::new(format!("{arrow}{}", entry.msg))
                .monospace()
                .color(color)
                .size(t.small),
        ));
    });
}
