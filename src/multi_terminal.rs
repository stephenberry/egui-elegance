//! Multi-pane terminal widget with per-pane broadcast toggles.
//!
//! [`MultiTerminal`] renders a responsive grid of [`TerminalPane`]s with a
//! shared keyboard input surface: whatever the user types is broadcast to
//! every pane whose "broadcast" pill is on. Think tmux's synchronized panes
//! or MobaXterm's multi-exec, rendered in the elegance design language.
//!
//! The widget is purely presentational: it captures keystrokes, maintains a
//! pending input buffer, and emits [`TerminalEvent::Command`] when the user
//! presses Enter. The caller is responsible for running the command on each
//! target and pushing response lines back into the corresponding pane via
//! [`MultiTerminal::push_line`].
//!
//! # Interaction
//!
//! * Click a pane header or body to move keyboard focus onto that pane.
//! * Click a pane's broadcast pill to toggle it in or out of the broadcast
//!   set. Every pane with broadcast on will receive input; offline panes
//!   are skipped. If the set is empty, the focused pane receives input as
//!   a fallback, so the buffer always has somewhere to go.
//! * Each pane has a **Solo** target button next to its broadcast pill:
//!   clicking solos that pane (broadcast = `{this}`); clicking again
//!   restores the previously stashed set.
//! * The gridbar has an **All on** toggle: clicking turns broadcast on
//!   for every connected pane, and clicking again turns all of them off.
//! * Keyboard: `Enter` sends, `Backspace` edits, `Esc` clears; `Cmd`/
//!   `Ctrl` + `A` toggles All on/off, `Cmd`/`Ctrl` + `D` solos the focused
//!   pane.
//!
//! # Example
//!
//! ```no_run
//! use elegance::{LineKind, MultiTerminal, TerminalEvent, TerminalLine,
//!                TerminalPane, TerminalStatus};
//!
//! struct App {
//!     terms: MultiTerminal,
//! }
//!
//! impl Default for App {
//!     fn default() -> Self {
//!         let terms = MultiTerminal::new("ssh-multi")
//!             .with_pane(
//!                 TerminalPane::new("api-east", "api-east-01")
//!                     .user("root")
//!                     .cwd("/var/log")
//!                     .status(TerminalStatus::Connected),
//!             )
//!             .with_pane(
//!                 TerminalPane::new("edge", "edge-proxy-01")
//!                     .user("root")
//!                     .status(TerminalStatus::Connected),
//!             );
//!         Self { terms }
//!     }
//! }
//!
//! # impl App {
//! fn ui(&mut self, ui: &mut egui::Ui) {
//!     self.terms.show(ui);
//!     for ev in self.terms.take_events() {
//!         match ev {
//!             TerminalEvent::Command { targets, command } => {
//!                 for id in targets {
//!                     self.terms.push_line(
//!                         &id,
//!                         TerminalLine::new(LineKind::Out, format!("ran: {command}")),
//!                     );
//!                 }
//!             }
//!         }
//!     }
//! }
//! # }
//! ```

use std::collections::HashSet;
use std::hash::Hash;

use egui::epaint::text::{LayoutJob, TextFormat};
use egui::{
    Align2, Color32, CornerRadius, Event, FontFamily, FontId, Id, Key, Modifiers, Pos2, Rect,
    Response, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo, WidgetType,
};

use crate::theme::{Palette, Theme, Typography};

/// Connection status for a [`TerminalPane`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TerminalStatus {
    /// The pane is live and will receive broadcast input.
    Connected,
    /// The pane is temporarily unavailable; shown in amber and excluded
    /// from broadcasts.
    Reconnecting,
    /// The pane is offline; shown in red and excluded from broadcasts.
    Offline,
}

impl TerminalStatus {
    /// Map to the corresponding [`IndicatorState`](crate::IndicatorState) so
    /// the pane header can reuse the library's status-light glyph.
    pub fn indicator_state(self) -> crate::IndicatorState {
        match self {
            Self::Connected => crate::IndicatorState::On,
            Self::Reconnecting => crate::IndicatorState::Connecting,
            Self::Offline => crate::IndicatorState::Off,
        }
    }
}

/// How a [`TerminalLine`] is coloured when rendered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineKind {
    /// Plain output, rendered in the primary text colour.
    Out,
    /// Informational text, rendered faint and italic.
    Info,
    /// Successful output, rendered in the success green.
    Ok,
    /// Warning, rendered in amber.
    Warn,
    /// Error, rendered in danger red.
    Err,
    /// Dimmed secondary output, rendered in muted grey.
    Dim,
    /// A command echo with a full prompt prefix (`user@host:cwd$ cmd`).
    ///
    /// When this variant is used, the `text` field of [`TerminalLine`] is
    /// ignored; the command text is stored inline.
    Command {
        /// Username shown in the prompt.
        user: String,
        /// Hostname shown in the prompt.
        host: String,
        /// Working directory shown in the prompt.
        cwd: String,
        /// The command text the user typed.
        cmd: String,
    },
}

/// A single line in a [`TerminalPane`]'s scrollback buffer.
#[derive(Clone, Debug)]
pub struct TerminalLine {
    /// Colour/style of the line.
    pub kind: LineKind,
    /// The text content. Unused when `kind` is [`LineKind::Command`].
    pub text: String,
}

impl TerminalLine {
    /// Create a line with the given kind and text.
    pub fn new(kind: LineKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }

    /// Plain-output shortcut.
    pub fn out(text: impl Into<String>) -> Self {
        Self::new(LineKind::Out, text)
    }
    /// Informational shortcut.
    pub fn info(text: impl Into<String>) -> Self {
        Self::new(LineKind::Info, text)
    }
    /// Success shortcut.
    pub fn ok(text: impl Into<String>) -> Self {
        Self::new(LineKind::Ok, text)
    }
    /// Warning shortcut.
    pub fn warn(text: impl Into<String>) -> Self {
        Self::new(LineKind::Warn, text)
    }
    /// Error shortcut.
    pub fn err(text: impl Into<String>) -> Self {
        Self::new(LineKind::Err, text)
    }
    /// Dimmed shortcut.
    pub fn dim(text: impl Into<String>) -> Self {
        Self::new(LineKind::Dim, text)
    }

    /// Build a command echo line. Rendered as `user@host:cwd$ cmd` with
    /// elegance's prompt colouring.
    pub fn command(
        user: impl Into<String>,
        host: impl Into<String>,
        cwd: impl Into<String>,
        cmd: impl Into<String>,
    ) -> Self {
        Self {
            kind: LineKind::Command {
                user: user.into(),
                host: host.into(),
                cwd: cwd.into(),
                cmd: cmd.into(),
            },
            text: String::new(),
        }
    }
}

/// A single pane rendered by [`MultiTerminal`].
#[derive(Clone, Debug)]
pub struct TerminalPane {
    /// Stable identifier used as the key in the broadcast set and event
    /// target list. Must be unique across panes in a single `MultiTerminal`.
    pub id: String,
    /// Hostname shown in the header and prompt.
    pub host: String,
    /// Username shown in the prompt. Default: `"user"`.
    pub user: String,
    /// Working directory shown in the prompt. Default: `"~"`.
    pub cwd: String,
    /// Connection status. Default: [`TerminalStatus::Connected`].
    pub status: TerminalStatus,
    /// Scrollback buffer. Oldest line at index 0, newest at the end.
    pub lines: Vec<TerminalLine>,
}

impl TerminalPane {
    /// Create a pane with the given id and hostname. Defaults: user `"user"`,
    /// cwd `"~"`, status [`TerminalStatus::Connected`], no lines.
    pub fn new(id: impl Into<String>, host: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            host: host.into(),
            user: "user".into(),
            cwd: "~".into(),
            status: TerminalStatus::Connected,
            lines: Vec::new(),
        }
    }

    /// Set the username shown in the prompt.
    #[inline]
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = user.into();
        self
    }

    /// Set the working directory shown in the prompt.
    #[inline]
    pub fn cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = cwd.into();
        self
    }

    /// Set the connection status.
    #[inline]
    pub fn status(mut self, status: TerminalStatus) -> Self {
        self.status = status;
        self
    }

    /// Append a line to the scrollback buffer (builder form).
    #[inline]
    pub fn push(mut self, line: TerminalLine) -> Self {
        self.lines.push(line);
        self
    }

    /// Append a line at runtime.
    pub fn push_line(&mut self, line: TerminalLine) {
        self.lines.push(line);
    }

    /// Replace the connection status at runtime.
    pub fn set_status(&mut self, status: TerminalStatus) {
        self.status = status;
    }

    /// Build a command echo line targeting this pane. Convenience helper:
    /// the prompt pieces are filled from the pane's own user/host/cwd.
    pub fn command_line(&self, cmd: impl Into<String>) -> TerminalLine {
        TerminalLine::command(self.user.clone(), self.host.clone(), self.cwd.clone(), cmd)
    }
}

/// Events emitted by [`MultiTerminal`] that the caller must react to.
#[derive(Clone, Debug)]
pub enum TerminalEvent {
    /// The user pressed Enter with a non-empty buffer. Run `command` on
    /// each pane whose id is in `targets` and push the response lines back
    /// via [`MultiTerminal::push_line`].
    ///
    /// The widget has already echoed the command into each target pane
    /// (as a [`LineKind::Command`] line) before this event is emitted, so
    /// the caller only needs to append the reply.
    Command {
        /// Pane ids that should run this command, in grid order.
        targets: Vec<String>,
        /// The command text as typed by the user.
        command: String,
    },
}

/// Multi-pane terminal with per-pane broadcast toggles.
///
/// See the module-level documentation for the full interaction model.
#[must_use = "Call `.show(ui)` to render the widget."]
pub struct MultiTerminal {
    id_salt: Id,
    panes: Vec<TerminalPane>,
    broadcast: HashSet<String>,
    collapsed: HashSet<String>,
    stashed: Option<HashSet<String>>,
    focused_id: Option<String>,
    pending: String,
    columns_mode: ColumnsMode,
    pane_min_height: f32,
    scrollback_cap: usize,
    events: Vec<TerminalEvent>,
}

/// How [`MultiTerminal`] decides the grid's column count.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnsMode {
    /// Always render exactly `n` columns, regardless of available width.
    Fixed(usize),
    /// Pick the column count each frame from the available width, ensuring
    /// every column is at least `min_col_width` points wide. Scales well
    /// from a narrow sidebar (1 column) up to a wide monitor (3-4+ columns).
    Auto {
        /// Minimum column width before the grid drops a column.
        min_col_width: f32,
    },
}

impl std::fmt::Debug for MultiTerminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiTerminal")
            .field("id_salt", &self.id_salt)
            .field("panes", &self.panes.len())
            .field("broadcast", &self.broadcast)
            .field("collapsed", &self.collapsed)
            .field("focused_id", &self.focused_id)
            .field("pending", &self.pending)
            .field("columns_mode", &self.columns_mode)
            .field("events", &self.events.len())
            .finish()
    }
}

impl MultiTerminal {
    /// Create an empty widget. `id_salt` scopes the widget's memory state;
    /// two `MultiTerminal`s on the same page need distinct salts.
    pub fn new(id_salt: impl Hash) -> Self {
        Self {
            id_salt: Id::new(("elegance_multi_terminal", id_salt)),
            panes: Vec::new(),
            broadcast: HashSet::new(),
            collapsed: HashSet::new(),
            stashed: None,
            focused_id: None,
            pending: String::new(),
            columns_mode: ColumnsMode::Fixed(2),
            pane_min_height: 220.0,
            scrollback_cap: 500,
            events: Vec::new(),
        }
    }

    /// Add a pane at construction time (builder form).
    #[inline]
    pub fn with_pane(mut self, pane: TerminalPane) -> Self {
        self.add_pane(pane);
        self
    }

    /// Render with a fixed number of columns in the pane grid. Panes
    /// wrap after `columns` per row. Default: 2.
    ///
    /// See also [`columns_auto`](Self::columns_auto) for a width-responsive
    /// mode that's better suited to large pane counts.
    #[inline]
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns_mode = ColumnsMode::Fixed(columns.max(1));
        self
    }

    /// Render with a width-responsive column count. Each frame the grid
    /// picks the largest column count such that every column is at least
    /// `min_col_width` points wide, clamped between 1 and the number of
    /// panes. With 16 panes, this naturally produces 3–4 columns on a
    /// wide monitor and 1–2 on a narrow sidebar.
    ///
    /// `min_col_width` is clamped to a minimum of 240 pt so the pane
    /// header always has room for the chevron, hostname, solo button,
    /// broadcast pill and status indicator.
    #[inline]
    pub fn columns_auto(mut self, min_col_width: f32) -> Self {
        self.columns_mode = ColumnsMode::Auto {
            min_col_width: min_col_width.max(240.0),
        };
        self
    }

    /// Minimum height of a single pane, in points. Default: `220.0`.
    #[inline]
    pub fn pane_min_height(mut self, h: f32) -> Self {
        self.pane_min_height = h.max(80.0);
        self
    }

    /// Cap on the number of lines retained per pane. Older lines are
    /// dropped when the buffer exceeds this count. Default: 500.
    #[inline]
    pub fn scrollback_cap(mut self, n: usize) -> Self {
        self.scrollback_cap = n.max(1);
        self
    }

    /// Append a pane at runtime.
    pub fn add_pane(&mut self, pane: TerminalPane) {
        // If this is the first pane, focus it by default.
        if self.focused_id.is_none() {
            self.focused_id = Some(pane.id.clone());
        }
        // Connected panes start in the broadcast set so the widget has a
        // sensible initial target.
        if pane.status == TerminalStatus::Connected {
            self.broadcast.insert(pane.id.clone());
        }
        self.panes.push(pane);
    }

    /// Remove a pane by id.
    pub fn remove_pane(&mut self, id: &str) {
        self.panes.retain(|p| p.id != id);
        self.broadcast.remove(id);
        if let Some(stash) = self.stashed.as_mut() {
            stash.remove(id);
        }
        if self.focused_id.as_deref() == Some(id) {
            self.focused_id = self.panes.first().map(|p| p.id.clone());
        }
    }

    /// Borrow a pane by id.
    pub fn pane(&self, id: &str) -> Option<&TerminalPane> {
        self.panes.iter().find(|p| p.id == id)
    }

    /// Borrow a pane mutably by id.
    pub fn pane_mut(&mut self, id: &str) -> Option<&mut TerminalPane> {
        self.panes.iter_mut().find(|p| p.id == id)
    }

    /// All panes, in grid order.
    pub fn panes(&self) -> &[TerminalPane] {
        &self.panes
    }

    /// Append a line to the pane with the given id. No-op if not found.
    /// Applies the scrollback cap.
    pub fn push_line(&mut self, id: &str, line: TerminalLine) {
        let cap = self.scrollback_cap;
        if let Some(p) = self.panes.iter_mut().find(|p| p.id == id) {
            p.lines.push(line);
            if p.lines.len() > cap {
                let drop = p.lines.len() - cap;
                p.lines.drain(0..drop);
            }
        }
    }

    /// Change a pane's status at runtime. If the pane leaves the connected
    /// state, it's removed from the broadcast set.
    pub fn set_status(&mut self, id: &str, status: TerminalStatus) {
        if let Some(p) = self.pane_mut(id) {
            p.status = status;
        }
        if status != TerminalStatus::Connected {
            self.broadcast.remove(id);
        }
    }

    /// Id of the currently focused pane, if any.
    pub fn focused(&self) -> Option<&str> {
        self.focused_id.as_deref()
    }

    /// Programmatically set the focused pane.
    pub fn set_focused(&mut self, id: Option<String>) {
        self.focused_id = id;
    }

    /// Current broadcast set (pane ids that will receive input). Does not
    /// include offline panes.
    pub fn broadcast(&self) -> &HashSet<String> {
        &self.broadcast
    }

    /// Replace the broadcast set wholesale. Invalidates the stash used by
    /// the Solo / All-on toggles.
    pub fn set_broadcast(&mut self, set: HashSet<String>) {
        self.broadcast = set;
        self.stashed = None;
    }

    /// Whether a pane is currently collapsed (rendered as a header-only
    /// strip with its scrollback hidden).
    pub fn is_collapsed(&self, id: &str) -> bool {
        self.collapsed.contains(id)
    }

    /// Collapse or expand a pane by id.
    pub fn set_collapsed(&mut self, id: &str, collapsed: bool) {
        if collapsed {
            self.collapsed.insert(id.to_string());
        } else {
            self.collapsed.remove(id);
        }
    }

    /// Flip the collapsed state of a pane.
    pub fn toggle_collapsed(&mut self, id: &str) {
        if self.collapsed.contains(id) {
            self.collapsed.remove(id);
        } else {
            self.collapsed.insert(id.to_string());
        }
    }

    /// Collapse every pane to its header strip.
    pub fn collapse_all(&mut self) {
        for p in &self.panes {
            self.collapsed.insert(p.id.clone());
        }
    }

    /// Expand every pane back to full height.
    pub fn expand_all(&mut self) {
        self.collapsed.clear();
    }

    /// Toggle whether `id` is in the broadcast set. Connected panes only.
    pub fn toggle_broadcast(&mut self, id: &str) {
        if self
            .pane(id)
            .is_some_and(|p| p.status == TerminalStatus::Connected)
        {
            self.stashed = None;
            if self.broadcast.contains(id) {
                self.broadcast.remove(id);
            } else {
                self.broadcast.insert(id.to_string());
            }
        }
    }

    /// Collapse the broadcast set to just the pane with the given id, and
    /// focus that pane. Calling solo on a pane that's already the sole
    /// receiver restores the previously-stashed set (so the button toggles).
    ///
    /// No-op if the id doesn't match a connected pane.
    pub fn solo(&mut self, id: &str) {
        if !self
            .panes
            .iter()
            .any(|p| p.id == id && p.status == TerminalStatus::Connected)
        {
            return;
        }
        let is_solo = self.broadcast.len() == 1 && self.broadcast.contains(id);
        if is_solo {
            self.restore_or_fallback();
        } else {
            self.stashed = Some(self.broadcast.clone());
            self.broadcast.clear();
            self.broadcast.insert(id.to_string());
        }
        self.focused_id = Some(id.to_string());
    }

    /// Solo the currently-focused pane. See [`solo`](Self::solo) for the
    /// toggle semantics. Bound to the `Cmd/Ctrl+D` shortcut.
    pub fn solo_focused(&mut self) {
        if let Some(fid) = self.focused_id.clone() {
            self.solo(&fid);
        }
    }

    /// Toggle broadcast on every connected pane. If every connected pane
    /// is already in the broadcast set, clears it; otherwise fills it with
    /// every connected pane.
    ///
    /// Note: when the set ends up empty, the focused pane still receives
    /// input as a fallback so the buffer always has somewhere to go.
    pub fn broadcast_all(&mut self) {
        let connected: Vec<String> = self
            .panes
            .iter()
            .filter(|p| p.status == TerminalStatus::Connected)
            .map(|p| p.id.clone())
            .collect();
        let all_on =
            !connected.is_empty() && connected.iter().all(|id| self.broadcast.contains(id));
        // All-on is now a plain on/off toggle rather than a stash-and-restore
        // mechanism: an explicit "turn everything off" is cleaner for users
        // than having the button sometimes restore a prior set.
        self.stashed = None;
        if all_on {
            self.broadcast.clear();
        } else {
            self.broadcast = connected.into_iter().collect();
        }
    }

    /// Flip the broadcast state on every connected pane (off becomes on
    /// and vice versa). Clears the stash.
    pub fn invert_broadcast(&mut self) {
        self.stashed = None;
        let mut next = HashSet::new();
        for p in &self.panes {
            if p.status != TerminalStatus::Connected {
                continue;
            }
            if !self.broadcast.contains(&p.id) {
                next.insert(p.id.clone());
            }
        }
        self.broadcast = next;
    }

    /// Current pending input (what the user is typing).
    pub fn pending(&self) -> &str {
        &self.pending
    }

    /// Clear the pending input buffer.
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    /// Drain and return the events accumulated since the previous call.
    /// Call this once per frame after [`show`](Self::show) to react to
    /// user-submitted commands.
    pub fn take_events(&mut self) -> Vec<TerminalEvent> {
        std::mem::take(&mut self.events)
    }

    /// Render the widget. Call once per frame inside a `CentralPanel` or
    /// similar container.
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());
        let focus_id = self.id_salt;

        // Reserve the whole widget region first so we have a rect to make
        // keyboard-focusable. The closure renders the actual content.
        let inner = ui
            .vertical(|ui| {
                self.ui_gridbar(ui, &theme);
                ui.add_space(0.0);
                self.ui_grid(ui, &theme);
            })
            .response;

        // Register the full region as keyboard-focusable *without* claiming
        // pointer clicks. An interactive `Sense::click()` here would sit on
        // top of the children in egui's z-order and swallow their clicks
        // (broadcast pill, quick actions, pane headers). Children call
        // `request_focus(focus_id)` explicitly when clicked.
        let bg = ui.interact(inner.rect, focus_id, Sense::focusable_noninteractive());

        // Auto-claim focus whenever nothing else has it — the widget is a
        // REPL-style typing surface, so keystrokes should land in the panes
        // as soon as the widget is visible, without requiring an initial
        // click. We only take focus when the app isn't focused on something
        // else (a TextEdit elsewhere, for instance).
        let someone_else_has_focus = ui
            .ctx()
            .memory(|m| m.focused().is_some_and(|f| f != focus_id));
        if !someone_else_has_focus {
            ui.ctx().memory_mut(|m| m.request_focus(focus_id));
        }

        if ui.ctx().memory(|m| m.has_focus(focus_id)) {
            self.handle_keys(ui);
        }

        bg.widget_info(|| {
            WidgetInfo::labeled(
                WidgetType::Other,
                true,
                format!(
                    "Multi-terminal, {} pane{}, {} receiving",
                    self.panes.len(),
                    if self.panes.len() == 1 { "" } else { "s" },
                    self.target_ids().len()
                ),
            )
        });
        bg
    }

    // ---- Internal helpers ------------------------------------------------

    /// Restore from the stashed broadcast set or fall back to the focused
    /// pane if nothing is stashed.
    fn restore_or_fallback(&mut self) {
        if let Some(stash) = self.stashed.take() {
            self.broadcast = stash
                .into_iter()
                .filter(|id| {
                    self.panes
                        .iter()
                        .any(|p| p.id == *id && p.status == TerminalStatus::Connected)
                })
                .collect();
        }
        if self.broadcast.is_empty() {
            if let Some(fid) = self.focused_id.clone() {
                self.broadcast.insert(fid);
            }
        }
    }

    /// The set of pane ids that should actually receive input right now.
    /// Falls back to the focused pane when the user-chosen set is empty.
    fn target_ids(&self) -> Vec<String> {
        let alive: Vec<String> = self
            .panes
            .iter()
            .filter(|p| self.broadcast.contains(&p.id) && p.status == TerminalStatus::Connected)
            .map(|p| p.id.clone())
            .collect();
        if !alive.is_empty() {
            return alive;
        }
        if let Some(fid) = &self.focused_id {
            if self
                .panes
                .iter()
                .any(|p| p.id == *fid && p.status == TerminalStatus::Connected)
            {
                return vec![fid.clone()];
            }
        }
        Vec::new()
    }

    fn connected_count(&self) -> usize {
        self.panes
            .iter()
            .filter(|p| p.status == TerminalStatus::Connected)
            .count()
    }

    fn run_pending(&mut self) {
        let cmd = self.pending.trim().to_string();
        if cmd.is_empty() {
            return;
        }
        let targets = self.target_ids();
        if targets.is_empty() {
            return;
        }
        // Echo the command into each target pane before emitting the event,
        // so the caller just appends the response.
        let cap = self.scrollback_cap;
        for id in &targets {
            if let Some(pane) = self.panes.iter_mut().find(|p| p.id == *id) {
                let line = pane.command_line(&cmd);
                pane.lines.push(line);
                if pane.lines.len() > cap {
                    let drop = pane.lines.len() - cap;
                    pane.lines.drain(0..drop);
                }
            }
        }
        self.events.push(TerminalEvent::Command {
            targets,
            command: cmd,
        });
        self.pending.clear();
    }

    fn handle_keys(&mut self, ui: &mut Ui) {
        // Collect events first to release the input borrow; many handlers
        // want `&mut self` which the input closure can't hold.
        let events: Vec<Event> = ui.ctx().input(|i| i.events.clone());
        for event in events {
            match event {
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    if modifiers.matches_exact(Modifiers::COMMAND)
                        || modifiers.matches_exact(Modifiers::CTRL)
                    {
                        match key {
                            Key::A => self.broadcast_all(),
                            Key::D => self.solo_focused(),
                            _ => {}
                        }
                        continue;
                    }
                    if modifiers.any() {
                        // Let other shortcuts fall through untouched.
                        continue;
                    }
                    match key {
                        Key::Enter => self.run_pending(),
                        Key::Escape => self.pending.clear(),
                        Key::Backspace => {
                            self.pending.pop();
                        }
                        _ => {}
                    }
                }
                Event::Text(text) => {
                    for ch in text.chars() {
                        if !ch.is_control() {
                            self.pending.push(ch);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // ---- Painting ------------------------------------------------------

    fn ui_gridbar(&mut self, ui: &mut Ui, theme: &Theme) {
        let palette = &theme.palette;
        let typo = &theme.typography;
        let connected = self.connected_count();
        let targets = self.target_ids();
        let targets_len = targets.len();

        let height = 36.0;
        let (rect, _resp) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
        let painter = ui.painter_at(rect);

        // Fill + top-of-grid rounded corners.
        painter.rect(
            rect,
            CornerRadius {
                nw: theme.card_radius as u8,
                ne: theme.card_radius as u8,
                sw: 0,
                se: 0,
            },
            palette.card,
            Stroke::new(1.0, palette.border),
            StrokeKind::Inside,
        );

        // Broadcast-fraction underline on the bottom edge of the gridbar.
        // Widens with how many panes are receiving; gives a felt sense of
        // reach at a glance.
        if connected > 0 {
            let frac = (targets_len as f32 / connected as f32).clamp(0.0, 1.0);
            let bar_top = rect.bottom() - 1.5;
            let bar_rect = Rect::from_min_max(
                Pos2::new(rect.left(), bar_top),
                Pos2::new(rect.left() + rect.width() * frac, rect.bottom()),
            );
            painter.rect_filled(bar_rect, CornerRadius::ZERO, palette.sky);
        }

        // Mode pill.
        let (mode_label, mode_style) = self.derive_mode(targets_len, connected);
        let mut cursor_x = rect.left() + 14.0;
        let y_mid = rect.center().y;

        cursor_x += self.paint_mode_pill(
            &painter,
            Pos2::new(cursor_x, y_mid),
            mode_label,
            mode_style,
            palette,
            typo,
        );
        cursor_x += 10.0;

        // Target summary (truncated if too many hosts).
        let summary = self.target_summary(&targets, targets_len, connected);
        let summary_color = if targets_len == 0 {
            palette.warning
        } else {
            palette.text_muted
        };
        // Reserve space on the right for buttons so the summary can be
        // clipped without overlapping them.
        let right_reserve = 280.0;
        let max_text_right = (rect.right() - right_reserve).max(cursor_x + 40.0);
        let summary_job = summary_layout(
            &summary,
            palette,
            typo.label,
            summary_color,
            max_text_right - cursor_x,
        );
        let galley = painter.layout_job(summary_job);
        painter.galley(
            Pos2::new(cursor_x, y_mid - galley.size().y * 0.5),
            galley,
            palette.text_muted,
        );

        // Right-aligned "All on" toggle. Solo lives on each pane's header;
        // manual per-pane broadcast toggles cover every other case.
        let mut x = rect.right() - 10.0;
        let all_on = connected > 0 && targets_len == connected;

        let all_w = qa_button(
            ui,
            rect,
            &mut x,
            self.id_salt.with("qa-all"),
            "All on",
            Some("\u{2318}A"),
            all_on,
            theme,
        );
        if all_w.clicked {
            self.broadcast_all();
            // Clicking the button grabs egui's focus; hand it back to the
            // widget so the next keystroke still lands in the panes.
            ui.ctx().memory_mut(|m| m.request_focus(self.id_salt));
        }
    }

    fn target_summary(&self, targets: &[String], n: usize, connected: usize) -> String {
        if n == 0 {
            return "No reachable terminals".into();
        }
        let phrase = if n == 1 {
            "Sending to"
        } else if n == connected {
            "Broadcasting to ALL"
        } else {
            "Broadcasting to"
        };
        let hosts: Vec<&str> = targets
            .iter()
            .filter_map(|id| self.pane(id).map(|p| p.host.as_str()))
            .collect();
        let shown = if hosts.len() <= 3 {
            hosts.join(", ")
        } else {
            format!("{}, +{} more", hosts[..2].join(", "), hosts.len() - 2)
        };
        format!("{phrase} {n} \u{00b7} {shown}")
    }

    fn paint_mode_pill(
        &self,
        painter: &egui::Painter,
        left_center: Pos2,
        label: &str,
        style: ModePillStyle,
        palette: &Palette,
        typo: &Typography,
    ) -> f32 {
        let text_color = match style {
            ModePillStyle::Single => palette.text_muted,
            ModePillStyle::Selected => palette.sky,
            ModePillStyle::All => Color32::from_rgb(0x0f, 0x17, 0x2a),
        };
        let (fill, border) = match style {
            ModePillStyle::Single => (palette.input_bg, palette.border),
            ModePillStyle::Selected => (with_alpha(palette.sky, 22), with_alpha(palette.sky, 90)),
            ModePillStyle::All => (palette.sky, palette.sky),
        };

        let galley = painter.layout_no_wrap(
            label.to_string(),
            FontId::new(typo.small - 1.5, FontFamily::Proportional),
            text_color,
        );
        let pad_x = 7.0;
        let pill_h = galley.size().y + 4.0;
        let pill_w = galley.size().x + pad_x * 2.0;
        let pill_rect = Rect::from_center_size(
            Pos2::new(left_center.x + pill_w * 0.5, left_center.y),
            Vec2::new(pill_w, pill_h),
        );
        painter.rect(
            pill_rect,
            CornerRadius::same((pill_h * 0.5) as u8),
            fill,
            Stroke::new(1.0, border),
            StrokeKind::Inside,
        );
        painter.galley(
            Pos2::new(
                pill_rect.left() + pad_x,
                pill_rect.center().y - galley.size().y * 0.5,
            ),
            galley,
            text_color,
        );
        pill_w
    }

    fn derive_mode(&self, targets: usize, connected: usize) -> (&'static str, ModePillStyle) {
        if targets == 0 {
            ("NO TARGET", ModePillStyle::Single)
        } else if targets == 1 {
            ("SINGLE", ModePillStyle::Single)
        } else if targets == connected {
            ("ALL", ModePillStyle::All)
        } else {
            ("SELECTED", ModePillStyle::Selected)
        }
    }

    fn ui_grid(&mut self, ui: &mut Ui, theme: &Theme) {
        let palette = &theme.palette;
        let full_w = ui.available_width();
        ui.spacing_mut().item_spacing.y = 0.0;

        let inner_pad = 12.0;
        let gap = 12.0;

        // Resolve the column count from the configured mode. Auto picks
        // the largest column count that keeps every column at least
        // `min_col_width` wide.
        let inner_w_for_cols = (full_w - inner_pad * 2.0).max(0.0);
        // Column count: Fixed modes use the caller's number; Auto first
        // finds the cap allowed by the available width, then balances rows
        // by using the *smallest* column count that still fits in that cap.
        // For 4 panes on a 3-col-capable screen this gives 2+2, not 3+1.
        let max_cols_from_width = |min_col_width: f32| -> usize {
            ((inner_w_for_cols + gap) / (min_col_width + gap))
                .floor()
                .max(1.0) as usize
        };
        let pane_count = self.panes.len().max(1);
        let cols_raw = match self.columns_mode {
            ColumnsMode::Fixed(n) => n,
            ColumnsMode::Auto { min_col_width } => {
                let max_cols = max_cols_from_width(min_col_width).min(pane_count);
                let rows = pane_count.div_ceil(max_cols);
                pane_count.div_ceil(rows)
            }
        };
        let cols = cols_raw.max(1).min(pane_count);
        let n_rows = self.panes.len().div_ceil(cols);

        // Per-row heights: a row where every pane is collapsed shrinks
        // to the header height so 16 idle panes don't hog the viewport.
        let header_only_h = PANE_HEADER_HEIGHT;
        let row_heights: Vec<f32> = (0..n_rows)
            .map(|row| {
                let any_expanded = (0..cols).any(|col| {
                    let idx = row * cols + col;
                    idx < self.panes.len() && !self.collapsed.contains(&self.panes[idx].id)
                });
                if any_expanded {
                    self.pane_min_height
                } else {
                    header_only_h
                }
            })
            .collect();
        let total_h = if self.panes.is_empty() {
            60.0
        } else {
            inner_pad * 2.0
                + row_heights.iter().sum::<f32>()
                + (n_rows.saturating_sub(1)) as f32 * gap
        };

        let (outer_rect, _resp) =
            ui.allocate_exact_size(Vec2::new(full_w, total_h), Sense::hover());

        ui.painter().rect(
            outer_rect,
            CornerRadius {
                nw: 0,
                ne: 0,
                sw: theme.card_radius as u8,
                se: theme.card_radius as u8,
            },
            palette.card,
            Stroke::new(1.0, palette.border),
            StrokeKind::Inside,
        );

        if self.panes.is_empty() {
            ui.painter().text(
                outer_rect.center(),
                Align2::CENTER_CENTER,
                "No terminals",
                FontId::proportional(theme.typography.body),
                palette.text_faint,
            );
            return;
        }

        let inner = outer_rect.shrink(inner_pad);
        let cell_w = (inner.width() - gap * (cols as f32 - 1.0)) / cols as f32;

        // Collect click intents across panes so we can apply mutations
        // after the read-only iteration.
        let mut intent_focus: Option<String> = None;
        let mut intent_toggle: Option<String> = None;
        let mut intent_solo: Option<String> = None;
        let mut intent_collapse: Option<String> = None;

        // Rolling vertical cursor so variable-height rows stack tidily.
        let mut y_cursor = inner.top();
        let mut row_top_for = vec![0.0_f32; n_rows];
        for (row, h) in row_heights.iter().enumerate() {
            row_top_for[row] = y_cursor;
            y_cursor += h + gap;
        }

        for (idx, pane) in self.panes.iter().enumerate() {
            let row = idx / cols;
            let col = idx % cols;
            let cell_top = row_top_for[row];
            let cell_left = inner.left() + col as f32 * (cell_w + gap);
            // Collapsed panes render as just the header row at the top of
            // their row-slot — the space below stays empty (and stays the
            // same colour as the grid container so it's invisible).
            let is_collapsed = self.collapsed.contains(&pane.id);
            let cell_h = if is_collapsed {
                header_only_h
            } else {
                row_heights[row]
            };
            let cell_rect =
                Rect::from_min_size(Pos2::new(cell_left, cell_top), Vec2::new(cell_w, cell_h));

            let is_focused = self.focused_id.as_deref() == Some(pane.id.as_str());
            let is_receiving =
                self.broadcast.contains(&pane.id) && pane.status == TerminalStatus::Connected;
            let is_solo = self.broadcast.len() == 1 && self.broadcast.contains(&pane.id);

            let ctx = PaneCtx {
                rect: cell_rect,
                pane,
                is_focused,
                is_receiving,
                is_solo,
                is_collapsed,
                pending: if is_receiving { &self.pending } else { "" },
                theme,
                id_salt: self.id_salt.with(("pane", idx)),
            };
            let actions = draw_pane(ui, &ctx);

            if actions.header_clicked || actions.body_clicked {
                intent_focus = Some(pane.id.clone());
            }
            if actions.toggle_clicked {
                intent_toggle = Some(pane.id.clone());
            }
            if actions.solo_clicked {
                intent_solo = Some(pane.id.clone());
            }
            if actions.collapse_clicked {
                intent_collapse = Some(pane.id.clone());
            }
        }

        if let Some(id) = intent_focus {
            self.focused_id = Some(id);
            ui.ctx().memory_mut(|m| m.request_focus(self.id_salt));
        }
        if let Some(id) = intent_toggle {
            self.toggle_broadcast(&id);
            ui.ctx().memory_mut(|m| m.request_focus(self.id_salt));
        }
        if let Some(id) = intent_solo {
            self.solo(&id);
            ui.ctx().memory_mut(|m| m.request_focus(self.id_salt));
        }
        if let Some(id) = intent_collapse {
            self.toggle_collapsed(&id);
            ui.ctx().memory_mut(|m| m.request_focus(self.id_salt));
        }
    }
}

/// Fixed header height used by pane rendering and by collapsed-row layout.
const PANE_HEADER_HEIGHT: f32 = 34.0;

// ---------------------------------------------------------------------------
// Rendering helpers (free functions, not methods, so the borrow checker
// doesn't get tangled with `&self.panes`).
// ---------------------------------------------------------------------------

struct PaneCtx<'a> {
    rect: Rect,
    pane: &'a TerminalPane,
    is_focused: bool,
    is_receiving: bool,
    /// This pane is the only member of the broadcast set.
    is_solo: bool,
    /// This pane is collapsed to a header-only strip.
    is_collapsed: bool,
    pending: &'a str,
    theme: &'a Theme,
    id_salt: Id,
}

struct PaneActions {
    header_clicked: bool,
    body_clicked: bool,
    toggle_clicked: bool,
    solo_clicked: bool,
    collapse_clicked: bool,
}

fn draw_pane(ui: &mut Ui, ctx: &PaneCtx<'_>) -> PaneActions {
    let palette = &ctx.theme.palette;
    let p = ctx.rect;

    // Background and border.
    let border_color = if ctx.is_focused {
        palette.sky
    } else if ctx.is_receiving {
        with_alpha(palette.sky, 115)
    } else {
        palette.border
    };
    let border_stroke = Stroke::new(if ctx.is_focused { 1.5 } else { 1.0 }, border_color);
    ui.painter().rect(
        p,
        CornerRadius::same((ctx.theme.control_radius + 2.0) as u8),
        palette.card,
        border_stroke,
        StrokeKind::Inside,
    );

    // Focus glow.
    if ctx.is_focused {
        ui.painter().rect_stroke(
            p.expand(2.0),
            CornerRadius::same((ctx.theme.control_radius + 4.0) as u8),
            Stroke::new(1.0, with_alpha(palette.sky, 50)),
            StrokeKind::Outside,
        );
    }

    // Header + (optional) body layout. Collapsed panes don't render a
    // body — the rect is sized to just the header height by the caller.
    let header_rect = Rect::from_min_size(p.min, Vec2::new(p.width(), PANE_HEADER_HEIGHT));
    let (header_clicked, toggle_clicked, solo_clicked, collapse_clicked) =
        draw_pane_header(ui, header_rect, ctx);

    let body_clicked = if ctx.is_collapsed {
        false
    } else {
        let body_rect = Rect::from_min_max(Pos2::new(p.left(), header_rect.bottom()), p.max);
        draw_pane_body(ui, body_rect, ctx)
    };

    PaneActions {
        header_clicked,
        body_clicked,
        toggle_clicked,
        solo_clicked,
        collapse_clicked,
    }
}

fn draw_pane_header(ui: &mut Ui, rect: Rect, ctx: &PaneCtx<'_>) -> (bool, bool, bool, bool) {
    let palette = &ctx.theme.palette;
    let typo = &ctx.theme.typography;

    // Bottom separator under the header — only drawn when the pane is
    // expanded (and therefore has a body below the separator).
    if !ctx.is_collapsed {
        ui.painter().line_segment(
            [
                Pos2::new(rect.left() + 1.0, rect.bottom() - 0.5),
                Pos2::new(rect.right() - 1.0, rect.bottom() - 0.5),
            ],
            Stroke::new(1.0, palette.border),
        );
    }

    // Background click area. Child widgets (chevron, solo, broadcast pill)
    // are drawn afterwards so their clicks take priority via egui's z-order.
    let header_resp = ui.interact(rect, ctx.id_salt.with("header"), Sense::click());

    // Chevron at the far left. Click to collapse / expand this pane.
    let edge_pad = 6.0;
    let (collapse_clicked, chev_w) = draw_chevron_button(ui, ctx, rect, edge_pad);

    // Hostname, offset to the right of the chevron.
    let pad_x = 13.0;
    let host_x = rect.left() + edge_pad + chev_w + 6.0;
    let mut job = LayoutJob::default();
    job.append(
        &ctx.pane.host,
        0.0,
        TextFormat {
            font_id: FontId::monospace(typo.small + 0.5),
            color: palette.text,
            ..Default::default()
        },
    );
    job.append(
        &format!("@{}", ctx.pane.user),
        0.0,
        TextFormat {
            font_id: FontId::monospace(typo.small + 0.5),
            color: palette.text_faint,
            ..Default::default()
        },
    );
    let galley = ui.painter().layout_job(job);
    ui.painter().galley(
        Pos2::new(host_x, rect.center().y - galley.size().y * 0.5),
        galley,
        palette.text,
    );

    // Status indicator on the right — same glyph set as the library's
    // `Indicator` widget (On / Connecting / Off).
    let ind_size = 10.0;
    let ind_center = Pos2::new(rect.right() - pad_x - ind_size * 0.5, rect.center().y);
    paint_status_indicator(ui.painter(), ind_center, ctx.pane.status, palette, ind_size);

    // Broadcast toggle pill, sitting between the hostname and the indicator.
    let bc_rect_right = ind_center.x - ind_size * 0.5 - 8.0;
    let (toggle_clicked, bc_w) = draw_broadcast_pill(ui, ctx, bc_rect_right, rect.center().y);

    // Solo button sits to the left of the broadcast pill.
    let solo_right = bc_rect_right - bc_w - 6.0;
    let (solo_clicked, _solo_w) = draw_solo_button(ui, ctx, solo_right, rect.center().y);

    (
        header_resp.clicked(),
        toggle_clicked,
        solo_clicked,
        collapse_clicked,
    )
}

/// Chevron button at the left edge of a pane header. Triangle pointing
/// down when the pane is expanded, right when it's collapsed.
///
/// Returns `(clicked, width)`.
fn draw_chevron_button(ui: &mut Ui, ctx: &PaneCtx<'_>, header: Rect, edge_pad: f32) -> (bool, f32) {
    let palette = &ctx.theme.palette;
    let size = 18.0;
    let rect = Rect::from_center_size(
        Pos2::new(header.left() + edge_pad + size * 0.5, header.center().y),
        Vec2::splat(size),
    );
    let resp = ui.interact(rect, ctx.id_salt.with("chev"), Sense::click());
    let color = if resp.hovered() {
        palette.text
    } else {
        palette.text_muted
    };

    // Small triangle centred in the button.
    let c = rect.center();
    let h = 3.5; // half-size of the triangle
    let pts = if ctx.is_collapsed {
        // Pointing right: ▸
        vec![
            Pos2::new(c.x - h * 0.7, c.y - h),
            Pos2::new(c.x - h * 0.7, c.y + h),
            Pos2::new(c.x + h, c.y),
        ]
    } else {
        // Pointing down: ▾
        vec![
            Pos2::new(c.x - h, c.y - h * 0.7),
            Pos2::new(c.x + h, c.y - h * 0.7),
            Pos2::new(c.x, c.y + h),
        ]
    };
    ui.painter()
        .add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));

    (resp.clicked(), size)
}

/// Paint the connection indicator glyph at `center`. Mirrors the library's
/// [`Indicator`](crate::Indicator) widget so the pane header shares the
/// same visual vocabulary.
fn paint_status_indicator(
    painter: &egui::Painter,
    center: Pos2,
    status: TerminalStatus,
    palette: &Palette,
    size: f32,
) {
    let r = size * 0.5;
    match status {
        TerminalStatus::Connected => {
            painter.circle_filled(center, r + 1.5, with_alpha(palette.success, 70));
            painter.circle_filled(center, r, palette.success);
        }
        TerminalStatus::Reconnecting => {
            painter.circle_stroke(center, r - 0.5, Stroke::new(1.8, palette.warning));
        }
        TerminalStatus::Offline => {
            painter.circle_stroke(center, r - 0.5, Stroke::new(1.0, palette.danger));
            let bar_w = size * 0.7;
            let bar_h = 2.0;
            let bar = Rect::from_center_size(center, Vec2::new(bar_w, bar_h));
            painter.rect_filled(bar, CornerRadius::same(1), palette.danger);
        }
    }
}

fn draw_broadcast_pill(ui: &mut Ui, ctx: &PaneCtx<'_>, right_edge: f32, y_mid: f32) -> (bool, f32) {
    let palette = &ctx.theme.palette;
    let dim = ctx.pane.status != TerminalStatus::Connected;

    // Compact icon-only toggle: a broadcast-waves glyph (dot with arcs
    // flanking it on both sides) inside a rounded pill.
    let pill_w = 34.0;
    let pill_h = 22.0;
    let rect = Rect::from_min_size(
        Pos2::new(right_edge - pill_w, y_mid - pill_h * 0.5),
        Vec2::new(pill_w, pill_h),
    );

    let resp = ui.interact(rect, ctx.id_salt.with("bcast"), Sense::click());
    let hovered = resp.hovered() && !dim;

    let (fill, border, icon_color) = if ctx.is_receiving {
        // On: sky fill; hover slightly lifts it so the press is felt.
        let fill = if hovered {
            palette.depth_tint(palette.sky, 0.12)
        } else {
            palette.sky
        };
        (fill, palette.sky, Color32::from_rgb(0x0f, 0x17, 0x2a))
    } else if hovered {
        // Off + hovered: preview the "on" state with a faint sky tint so
        // the affordance is obvious — clicking will turn it sky.
        (
            with_alpha(palette.sky, 26),
            with_alpha(palette.sky, 130),
            palette.sky,
        )
    } else {
        (Color32::TRANSPARENT, palette.border, palette.text_faint)
    };

    ui.painter().rect(
        rect,
        CornerRadius::same((pill_h * 0.5) as u8),
        fill,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );

    // Pulse halo behind the centre dot while receiving.
    let center = rect.center();
    if ctx.is_receiving {
        let t = ui.input(|i| i.time);
        let phase = (t.rem_euclid(1.2) / 1.2) as f32;
        let halo_r = 2.0 + phase.min(1.0) * 4.5;
        let halo_a = (70.0 * (1.0 - phase)).clamp(0.0, 255.0) as u8;
        ui.painter()
            .circle_filled(center, halo_r, with_alpha(icon_color, halo_a));
    }

    paint_broadcast_glyph(ui.painter(), center, icon_color);

    (if dim { false } else { resp.clicked() }, pill_w)
}

/// Broadcast-waves glyph: centre dot with two symmetric arcs emanating
/// outward on both sides. Rendered at `center`, roughly 18 pt wide and
/// 8 pt tall so it fits comfortably inside a pill.
fn paint_broadcast_glyph(painter: &egui::Painter, center: Pos2, color: Color32) {
    // Centre source dot.
    painter.circle_filled(center, 1.8, color);

    let stroke = Stroke::new(1.2, color);
    // Inner arcs (radius ~4.5) and outer arcs (radius ~7.5) on each side.
    // Angles are measured in radians; 0 points right, so "right arc" spans
    // roughly [-span, +span] around 0 and "left arc" spans [PI - span, PI + span].
    use std::f32::consts::PI;
    paint_arc(painter, center, 4.5, -0.45, 0.45, stroke);
    paint_arc(painter, center, 4.5, PI - 0.45, PI + 0.45, stroke);
    paint_arc(painter, center, 7.5, -0.32, 0.32, stroke);
    paint_arc(painter, center, 7.5, PI - 0.32, PI + 0.32, stroke);
}

/// Approximate an arc with a short line-segment polyline.
fn paint_arc(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    start: f32,
    end: f32,
    stroke: Stroke,
) {
    const STEPS: usize = 8;
    let mut pts = Vec::with_capacity(STEPS + 1);
    for i in 0..=STEPS {
        let t = i as f32 / STEPS as f32;
        let a = start + (end - start) * t;
        pts.push(Pos2::new(
            center.x + radius * a.cos(),
            center.y + radius * a.sin(),
        ));
    }
    painter.add(egui::Shape::line(pts, stroke));
}

/// Per-pane solo button: a small round target-icon button. Clicking makes
/// this pane the only member of the broadcast set; clicking again (while
/// already solo) restores the prior set.
///
/// Returns `(clicked, width)`.
fn draw_solo_button(ui: &mut Ui, ctx: &PaneCtx<'_>, right_edge: f32, y_mid: f32) -> (bool, f32) {
    let palette = &ctx.theme.palette;
    let dim = ctx.pane.status != TerminalStatus::Connected;

    let size = 22.0;
    let rect = Rect::from_min_size(
        Pos2::new(right_edge - size, y_mid - size * 0.5),
        Vec2::splat(size),
    );

    let resp = ui.interact(rect, ctx.id_salt.with("solo"), Sense::click());
    let hovered = resp.hovered() && !dim;

    let (fill, border, icon_color) = if ctx.is_solo {
        (with_alpha(palette.sky, 28), palette.sky, palette.sky)
    } else if hovered {
        (Color32::TRANSPARENT, palette.text_muted, palette.text)
    } else {
        (Color32::TRANSPARENT, palette.border, palette.text_faint)
    };

    ui.painter().rect(
        rect,
        CornerRadius::same((size * 0.5) as u8),
        fill,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );

    // Solo glyph: 2x2 grid with only the top-left cell filled. Pairs
    // visually with the All-on button's four-cell grid and reads as
    // "just this one of the many".
    paint_solo_icon(ui.painter(), rect.center(), icon_color);

    (if dim { false } else { resp.clicked() }, size)
}

fn paint_solo_icon(painter: &egui::Painter, center: Pos2, color: Color32) {
    let pad = 1.0;
    let cell = 5.5;
    let cells = [
        (-cell - pad, -cell - pad, true),
        (pad, -cell - pad, false),
        (-cell - pad, pad, false),
        (pad, pad, false),
    ];
    for (dx, dy, filled) in cells {
        let r = Rect::from_min_size(Pos2::new(center.x + dx, center.y + dy), Vec2::splat(cell));
        if filled {
            painter.rect_filled(r, CornerRadius::same(1), color);
        } else {
            painter.rect_stroke(
                r,
                CornerRadius::same(1),
                Stroke::new(1.2, color),
                StrokeKind::Inside,
            );
        }
    }
}

/// Returns true if the body area was clicked.
fn draw_pane_body(ui: &mut Ui, rect: Rect, ctx: &PaneCtx<'_>) -> bool {
    let palette = &ctx.theme.palette;
    let typo = &ctx.theme.typography;

    // Terminal-bg fill (darker than the card, like a screen).
    let term_bg = palette.depth_tint(palette.input_bg, 0.015);
    ui.painter().rect_filled(
        rect.shrink2(Vec2::new(1.0, 1.0)),
        CornerRadius {
            nw: 0,
            ne: 0,
            sw: (ctx.theme.control_radius + 1.0) as u8,
            se: (ctx.theme.control_radius + 1.0) as u8,
        },
        term_bg,
    );

    let body_resp = ui.interact(rect, ctx.id_salt.with("body"), Sense::click());

    // Render the lines inside a child UI so we can use ScrollArea.
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink(8.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.spacing_mut().item_spacing.y = 2.0;

    egui::ScrollArea::vertical()
        .id_salt(ctx.id_salt.with("scroll"))
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(&mut child, |ui| {
            for line in &ctx.pane.lines {
                paint_line(ui, line, palette, typo);
            }
            paint_live_prompt(ui, ctx, palette, typo);
        });

    body_resp.clicked()
}

fn paint_line(ui: &mut Ui, line: &TerminalLine, palette: &Palette, typo: &Typography) {
    let size = typo.small + 0.5;
    let font = FontId::monospace(size);
    let wrap_width = ui.available_width();

    match &line.kind {
        LineKind::Command {
            user,
            host,
            cwd,
            cmd,
        } => {
            let mut job = LayoutJob::default();
            // `LayoutJob`s don't wrap by default; constrain them to the
            // pane's current content width so long commands wrap like
            // output lines do. `break_anywhere` lets unbroken tokens
            // (URLs, paths, pasted blobs) wrap mid-character.
            job.wrap.max_width = wrap_width;
            job.wrap.break_anywhere = true;
            job.append(
                &format!("{user}@{host}"),
                0.0,
                TextFormat {
                    font_id: font.clone(),
                    color: palette.success,
                    ..Default::default()
                },
            );
            job.append(
                ":",
                0.0,
                TextFormat {
                    font_id: font.clone(),
                    color: palette.text_muted,
                    ..Default::default()
                },
            );
            job.append(
                cwd,
                0.0,
                TextFormat {
                    font_id: font.clone(),
                    color: palette.purple,
                    ..Default::default()
                },
            );
            job.append(
                "$ ",
                0.0,
                TextFormat {
                    font_id: font.clone(),
                    color: palette.text_muted,
                    ..Default::default()
                },
            );
            job.append(
                cmd,
                0.0,
                TextFormat {
                    font_id: font,
                    color: palette.text,
                    ..Default::default()
                },
            );
            ui.label(job);
        }
        other => {
            let color = color_for_kind(other, palette);
            let italic = matches!(other, LineKind::Info);
            let rich = egui::RichText::new(&line.text).font(font).color(color);
            let rich = if italic { rich.italics() } else { rich };
            ui.add(egui::Label::new(rich).wrap());
        }
    }
}

fn paint_live_prompt(ui: &mut Ui, ctx: &PaneCtx<'_>, palette: &Palette, typo: &Typography) {
    let size = typo.small + 0.5;
    let font = FontId::monospace(size);
    let pane = ctx.pane;

    let mut job = LayoutJob::default();
    // Reserve space for the caret block at the end so the prompt wraps
    // before the caret falls off the right edge.
    job.wrap.max_width = (ui.available_width() - 10.0).max(40.0);
    // Typed text in a terminal is usually one unbroken token (no spaces),
    // so `break_anywhere` is required to wrap it mid-character. Without
    // this the pending buffer overflows past the pane's right edge.
    job.wrap.break_anywhere = true;
    job.append(
        &format!("{}@{}", pane.user, pane.host),
        0.0,
        TextFormat {
            font_id: font.clone(),
            color: palette.success,
            ..Default::default()
        },
    );
    job.append(
        ":",
        0.0,
        TextFormat {
            font_id: font.clone(),
            color: palette.text_muted,
            ..Default::default()
        },
    );
    job.append(
        &pane.cwd,
        0.0,
        TextFormat {
            font_id: font.clone(),
            color: palette.purple,
            ..Default::default()
        },
    );
    job.append(
        "$ ",
        0.0,
        TextFormat {
            font_id: font.clone(),
            color: palette.text_muted,
            ..Default::default()
        },
    );
    if !ctx.pending.is_empty() {
        job.append(
            ctx.pending,
            0.0,
            TextFormat {
                font_id: font.clone(),
                color: palette.sky,
                ..Default::default()
            },
        );
    }

    // Lay out the wrapped prompt ourselves (without a horizontal wrapper,
    // whose effectively-unbounded available width can override the job's
    // wrap cap) and paint the caret at the end of the last wrapped row.
    let galley = ui.painter().layout_job(job);
    let caret_h = size + 2.0;
    let caret_w = 7.0;
    let total_size = Vec2::new(
        galley.size().x + caret_w + 2.0,
        galley.size().y.max(caret_h),
    );
    let (rect, _resp) = ui.allocate_exact_size(total_size, Sense::hover());
    let galley_origin = rect.min;

    // Remember where the last row ends before we move the Arc into painter.galley.
    let last_row = galley.rows.last();
    let caret_x = galley_origin.x + last_row.map(|r| r.rect().right()).unwrap_or(0.0) + 1.0;
    let caret_y = galley_origin.y
        + last_row
            .map(|r| r.rect().center().y)
            .unwrap_or(galley.size().y * 0.5);

    ui.painter().galley(galley_origin, galley, palette.text);

    let caret_rect = Rect::from_min_size(
        Pos2::new(caret_x, caret_y - caret_h * 0.5),
        Vec2::new(caret_w, caret_h),
    );
    let caret_color = if ctx.is_receiving {
        palette.sky
    } else {
        with_alpha(palette.text_faint, 80)
    };
    ui.painter()
        .rect_filled(caret_rect, CornerRadius::ZERO, caret_color);
}

fn color_for_kind(kind: &LineKind, palette: &Palette) -> Color32 {
    match kind {
        LineKind::Out => palette.text,
        LineKind::Info => palette.text_faint,
        LineKind::Ok => palette.success,
        LineKind::Warn => palette.warning,
        LineKind::Err => palette.danger,
        LineKind::Dim => palette.text_muted,
        LineKind::Command { .. } => palette.text,
    }
}

fn summary_layout(
    text: &str,
    palette: &Palette,
    size: f32,
    color: Color32,
    max_width: f32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = max_width;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('\u{2026}');
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::new(size, FontFamily::Proportional),
            color,
            ..Default::default()
        },
    );
    let _ = palette;
    job
}

// ---------------------------------------------------------------------------
// "All on" toggle button in the gridbar.
// ---------------------------------------------------------------------------

struct QaResult {
    clicked: bool,
}

#[allow(clippy::too_many_arguments)]
fn qa_button(
    ui: &mut Ui,
    bar_rect: Rect,
    x_right: &mut f32,
    id: Id,
    label: &str,
    shortcut: Option<&str>,
    active: bool,
    theme: &Theme,
) -> QaResult {
    let palette = &theme.palette;
    let typo = &theme.typography;
    let font = FontId::new(typo.small, FontFamily::Proportional);
    let label_galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), palette.text);

    let kbd_font = FontId::monospace(typo.small - 1.5);
    let kbd_galley = shortcut.map(|s| {
        ui.painter()
            .layout_no_wrap(s.to_string(), kbd_font.clone(), palette.text_faint)
    });

    let icon_w = 16.0;
    let pad_x = 8.0;
    let label_w = label_galley.size().x;
    let kbd_w = kbd_galley.as_ref().map(|g| g.size().x + 8.0).unwrap_or(0.0);
    let btn_w = icon_w + 6.0 + label_w + kbd_w + pad_x * 2.0;
    let btn_h = bar_rect.height() - 10.0;
    let btn_rect = Rect::from_min_size(
        Pos2::new(*x_right - btn_w, bar_rect.center().y - btn_h * 0.5),
        Vec2::new(btn_w, btn_h),
    );
    *x_right = btn_rect.left() - 4.0;

    let resp = ui.interact(btn_rect, id, Sense::click());
    let hover = resp.hovered();

    let (fg, border, fill) = if active {
        (
            palette.sky,
            with_alpha(palette.sky, 110),
            with_alpha(palette.sky, 22),
        )
    } else if hover {
        (palette.text, palette.text_muted, Color32::TRANSPARENT)
    } else {
        (palette.text_muted, palette.border, Color32::TRANSPARENT)
    };

    ui.painter().rect(
        btn_rect,
        CornerRadius::same(theme.control_radius as u8),
        fill,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );

    // Icon: 2x2 grid of small squares matching the pane-grid metaphor.
    let icon_center = Pos2::new(btn_rect.left() + pad_x + icon_w * 0.5, btn_rect.center().y);
    paint_grid_icon(ui.painter(), icon_center, fg);

    // Label.
    let label_x = btn_rect.left() + pad_x + icon_w + 6.0;
    let label_galley2 = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), fg);
    ui.painter().galley(
        Pos2::new(label_x, btn_rect.center().y - label_galley2.size().y * 0.5),
        label_galley2,
        fg,
    );

    // Shortcut pill (right-aligned).
    if let Some(kbd) = shortcut {
        let kbd_galley2 =
            ui.painter()
                .layout_no_wrap(kbd.to_string(), kbd_font.clone(), palette.text_faint);
        let kbd_rect = Rect::from_min_size(
            Pos2::new(
                btn_rect.right() - pad_x - kbd_galley2.size().x - 8.0,
                btn_rect.center().y - (kbd_galley2.size().y + 2.0) * 0.5,
            ),
            Vec2::new(kbd_galley2.size().x + 8.0, kbd_galley2.size().y + 2.0),
        );
        ui.painter().rect(
            kbd_rect,
            CornerRadius::same(3),
            palette.input_bg,
            Stroke::new(1.0, palette.border),
            StrokeKind::Inside,
        );
        ui.painter().galley(
            Pos2::new(
                kbd_rect.left() + 4.0,
                kbd_rect.center().y - kbd_galley2.size().y * 0.5,
            ),
            kbd_galley2,
            palette.text_faint,
        );
    }

    QaResult {
        clicked: resp.clicked(),
    }
}

/// 2x2 grid glyph drawn at `center`. Used as the "All on" button's icon.
fn paint_grid_icon(painter: &egui::Painter, center: Pos2, color: Color32) {
    let pad = 1.0;
    let size = 5.5;
    for (dx, dy) in &[
        (-size - pad, -size - pad),
        (pad, -size - pad),
        (-size - pad, pad),
        (pad, pad),
    ] {
        let r = Rect::from_min_size(Pos2::new(center.x + dx, center.y + dy), Vec2::splat(size));
        painter.rect_stroke(
            r,
            CornerRadius::same(1),
            Stroke::new(1.2, color),
            StrokeKind::Inside,
        );
    }
}

#[derive(Clone, Copy)]
enum ModePillStyle {
    Single,
    Selected,
    All,
}

fn with_alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}
