//! Modal dialog — a centered themed card over a dimmed backdrop.
//!
//! Painted in two layers: a full-viewport dimmed backdrop that swallows
//! clicks (and closes the modal when clicked), and a centered [`Card`]-
//! like window with an optional heading row and a close "×" button.
//! Press `Esc` to dismiss.

use egui::{
    accesskit, Align2, Area, Color32, Context, CornerRadius, Frame, Id, Key, Margin, Order,
    Response, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetText, WidgetType,
};

use crate::{theme::Theme, Button, ButtonSize};

/// A centered modal dialog.
///
/// The `open` flag drives visibility: when it's `false` on entry to
/// [`Modal::show`], nothing is rendered; when the user clicks the backdrop,
/// presses `Esc`, or clicks the "×" button, it's flipped to `false`.
///
/// ```no_run
/// # use elegance::Modal;
/// # let ctx = egui::Context::default();
/// # let mut open = true;
/// Modal::new("stats", &mut open)
///     .heading("Run Summary")
///     .show(&ctx, |ui| {
///         ui.label("…");
///     });
/// ```
#[must_use = "Call `.show(ctx, |ui| { ... })` to render the modal."]
pub struct Modal<'a> {
    id_salt: Id,
    heading: Option<WidgetText>,
    open: &'a mut bool,
    max_width: f32,
    close_on_backdrop: bool,
    close_on_escape: bool,
    alert: bool,
}

impl<'a> std::fmt::Debug for Modal<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Modal")
            .field("id_salt", &self.id_salt)
            .field("heading", &self.heading.as_ref().map(|h| h.text()))
            .field("open", &*self.open)
            .field("max_width", &self.max_width)
            .field("close_on_backdrop", &self.close_on_backdrop)
            .field("close_on_escape", &self.close_on_escape)
            .field("alert", &self.alert)
            .finish()
    }
}

impl<'a> Modal<'a> {
    /// Create a modal keyed by `id_salt` whose visibility is bound to `open`.
    pub fn new(id_salt: impl std::hash::Hash, open: &'a mut bool) -> Self {
        Self {
            id_salt: Id::new(id_salt),
            heading: None,
            open,
            max_width: 440.0,
            close_on_backdrop: true,
            close_on_escape: true,
            alert: false,
        }
    }

    /// Show a strong heading at the top of the modal, alongside the close button.
    pub fn heading(mut self, heading: impl Into<WidgetText>) -> Self {
        self.heading = Some(heading.into());
        self
    }

    /// Override the maximum width of the modal card in points. Default: 440.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Whether clicking the dimmed backdrop dismisses the modal. Default: `true`.
    pub fn close_on_backdrop(mut self, close: bool) -> Self {
        self.close_on_backdrop = close;
        self
    }

    /// Whether pressing `Esc` dismisses the modal. Default: `true`.
    pub fn close_on_escape(mut self, close: bool) -> Self {
        self.close_on_escape = close;
        self
    }

    /// Mark this modal as an *alert dialog* — a dialog that demands the
    /// user's attention to proceed, such as a destructive confirmation or
    /// an unsaved-changes prompt. Screen readers announce alert dialogs
    /// more assertively than ordinary dialogs. Default: `false`.
    ///
    /// Under the hood this exposes `accesskit::Role::AlertDialog` on the
    /// modal's root node instead of the default `Role::Dialog`.
    pub fn alert(mut self, alert: bool) -> Self {
        self.alert = alert;
        self
    }

    /// Render the modal. Returns `None` if the modal was suppressed because
    /// the bound `open` flag was `false`; otherwise returns `Some(R)` with
    /// the content closure's return value.
    pub fn show<R>(self, ctx: &Context, add_contents: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        // --- Focus lifecycle ------------------------------------------------
        // Track the open/closed transition so we can (a) record which widget
        // had keyboard focus before the modal opened and (b) restore that
        // focus when the modal closes. Without this the user's focus is
        // visually eclipsed by the modal but structurally remains behind it —
        // Tab would navigate widgets on the underlying page.
        let focus_storage = Id::new(("elegance_modal_focus", self.id_salt));
        let mut focus_state: ModalFocusState =
            ctx.data(|d| d.get_temp(focus_storage).unwrap_or_default());
        let is_open = *self.open;

        if focus_state.was_open && !is_open {
            // Just closed this frame — return focus to whatever had it before.
            if let Some(prev) = focus_state.prev_focus {
                ctx.memory_mut(|m| m.request_focus(prev));
            }
            ctx.data_mut(|d| d.insert_temp(focus_storage, ModalFocusState::default()));
            return None;
        }

        if !is_open {
            return None;
        }

        let just_opened = !focus_state.was_open;
        if just_opened {
            focus_state.prev_focus = ctx.memory(|m| m.focused());
            focus_state.was_open = true;
            ctx.data_mut(|d| d.insert_temp(focus_storage, focus_state));
        }

        let theme = Theme::current(ctx);
        let p = &theme.palette;
        let mut should_close = false;
        let mut close_btn_id: Option<Id> = None;

        // --- Backdrop ----------------------------------------------------
        let screen = ctx.content_rect();
        let backdrop_id = Id::new("elegance_modal_backdrop").with(self.id_salt);
        let backdrop = Area::new(backdrop_id)
            .fixed_pos(screen.min)
            .order(Order::Middle)
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    screen,
                    CornerRadius::ZERO,
                    Color32::from_rgba_premultiplied(0, 0, 0, 150),
                );
                ui.allocate_rect(screen, Sense::click())
            });
        if self.close_on_backdrop && backdrop.inner.clicked() {
            should_close = true;
        }

        // --- Content -----------------------------------------------------
        let window_id = Id::new("elegance_modal_window").with(self.id_salt);
        let alert = self.alert;
        let heading_text: Option<String> = self.heading.as_ref().map(|h| h.text().to_string());
        let result = Area::new(window_id)
            .order(Order::Foreground)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                // Upgrade this Ui's accesskit role from `GenericContainer`
                // (set automatically by `Ui::new`) to a dialog role, so
                // screen readers announce the modal correctly and
                // platforms that support dialog focus tracking (AT-SPI)
                // treat it as a window-like surface.
                let role = if alert {
                    accesskit::Role::AlertDialog
                } else {
                    accesskit::Role::Dialog
                };
                let heading_for_label = heading_text.clone();
                ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
                    node.set_role(role);
                    if let Some(label) = heading_for_label {
                        node.set_label(label);
                    }
                });

                ui.set_max_width(self.max_width);
                Frame::new()
                    .fill(p.card)
                    .stroke(Stroke::new(1.0, p.border))
                    .corner_radius(CornerRadius::same(theme.card_radius as u8))
                    .inner_margin(Margin::same(theme.card_padding as i8))
                    .show(ui, |ui| {
                        let has_heading = self.heading.is_some();
                        if has_heading {
                            ui.horizontal(|ui| {
                                if let Some(h) = &self.heading {
                                    ui.add(egui::Label::new(theme.heading_text(h.text())));
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let resp = close_button(ui);
                                        if resp.clicked() {
                                            should_close = true;
                                        }
                                        close_btn_id = Some(resp.id);
                                    },
                                );
                            });
                            ui.add_space(6.0);
                            ui.separator();
                            ui.add_space(10.0);
                        }
                        add_contents(ui)
                    })
            });

        if self.close_on_escape && ctx.input(|i| i.key_pressed(Key::Escape)) {
            should_close = true;
        }

        // On the first frame a modal is open, move keyboard focus into it so
        // Tab navigates within the dialog rather than the background. We
        // target the close button when a heading is present (it has a
        // stable id and is always interactive); without a heading there's
        // no intrinsic focus target, so focus is left to the caller.
        if just_opened {
            if let Some(id) = close_btn_id {
                ctx.memory_mut(|m| m.request_focus(id));
            }
        }

        if should_close {
            *self.open = false;
        }

        Some(result.inner.inner)
    }
}

/// Persistent focus-lifecycle state for a single `Modal`, keyed by the
/// modal's `id_salt`. Stored via `ctx.data_mut`.
#[derive(Clone, Copy, Default, Debug)]
struct ModalFocusState {
    /// Whether the modal was rendered open last frame. Used to detect
    /// open/close transitions.
    was_open: bool,
    /// Which widget (if any) had keyboard focus at the moment the modal
    /// opened. Restored on close.
    prev_focus: Option<Id>,
}

/// Render the modal's close button. Returns its `Response` so the caller
/// can route focus to it and check `clicked()`. The accesskit label is
/// set to `"Close"` explicitly — without this, screen readers announce
/// the "×" glyph literally as "multiplication sign."
///
/// The button is scoped under a stable id (`"elegance_modal_close"`) so
/// focus requests targeting it survive layout changes.
fn close_button(ui: &mut Ui) -> Response {
    let inner = ui
        .push_id("elegance_modal_close", |ui| {
            ui.add(Button::new("×").outline().size(ButtonSize::Small))
        })
        .inner;
    let enabled = inner.enabled();
    inner.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, "Close"));
    inner
}
