//! Action menus — a popup list of items attached to a trigger [`Response`].
//!
//! [`Menu`] opens a themed popup below a trigger widget when the trigger is
//! clicked. [`MenuItem`] is the styled leaf inside the popup: label on the
//! left, optional keyboard-shortcut hint on the right, optional `danger`
//! tint for destructive actions. Separators between groups use the stock
//! `ui.separator()`.
//!
//! ```no_run
//! # use elegance::{Button, ButtonSize, Menu, MenuItem};
//! # egui::__run_test_ui(|ui| {
//! let trigger = ui.add(Button::new("⋯").outline().size(ButtonSize::Small));
//! Menu::new("row_actions").show_below(&trigger, |ui| {
//!     if ui.add(MenuItem::new("Edit").shortcut("⌘ E")).clicked() {
//!         // …
//!     }
//!     if ui.add(MenuItem::new("Duplicate")).clicked() { /* … */ }
//!     ui.separator();
//!     if ui.add(MenuItem::new("Delete").danger()).clicked() { /* … */ }
//! });
//! # });
//! ```
//!
//! The popup is dismissed by clicking any item, clicking outside, or
//! pressing `Esc`. Keyboard navigation (arrows + Enter) is not implemented
//! in this version.

use std::hash::Hash;

use egui::{
    CornerRadius, Id, Popup, PopupCloseBehavior, Pos2, Response, Sense, Ui, Vec2, Widget,
    WidgetInfo, WidgetText, WidgetType,
};

use crate::theme::{with_alpha, Theme};

/// A click-to-open popup menu anchored below a trigger [`Response`].
///
/// Call [`Menu::show_below`] after painting the trigger; it opens on
/// trigger clicks and closes on item click, outside-click, or `Esc`.
#[derive(Debug, Clone)]
#[must_use = "Call `.show_below(&trigger, |ui| ...)` to render the menu."]
pub struct Menu {
    id_salt: Id,
    min_width: f32,
}

impl Menu {
    /// Create a menu keyed by `id_salt`. The salt is used to persist the
    /// open/closed state across frames and must be stable for the trigger
    /// it's attached to.
    pub fn new(id_salt: impl Hash) -> Self {
        Self {
            id_salt: Id::new(("elegance::menu", Id::new(id_salt))),
            min_width: 180.0,
        }
    }

    /// Minimum width of the popup in points. Default: 180.
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = min_width;
        self
    }

    /// Render the menu below `trigger`. Returns `Some(R)` with the body
    /// closure's return value while the menu is open, `None` while closed.
    pub fn show_below<R>(
        self,
        trigger: &Response,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        let popup_id = Id::new(self.id_salt);
        Popup::menu(trigger)
            .id(popup_id)
            .close_behavior(PopupCloseBehavior::CloseOnClick)
            .show(|ui| {
                ui.set_min_width(self.min_width);
                // Tight stacking — MenuItem has its own interior padding.
                ui.spacing_mut().item_spacing.y = 2.0;
                add_contents(ui)
            })
            .map(|r| r.inner)
    }
}

/// A single selectable row inside a [`Menu`].
///
/// Add with `ui.add(MenuItem::new("…"))` inside a menu body. The returned
/// [`Response`]'s `.clicked()` fires on activation.
#[must_use = "Add with `ui.add(...)`."]
pub struct MenuItem {
    label: WidgetText,
    shortcut: Option<String>,
    danger: bool,
    enabled: bool,
}

impl std::fmt::Debug for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuItem")
            .field("label", &self.label.text())
            .field("shortcut", &self.shortcut)
            .field("danger", &self.danger)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl MenuItem {
    /// Create a menu item with the given label.
    pub fn new(label: impl Into<WidgetText>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            danger: false,
            enabled: true,
        }
    }

    /// Display a keyboard-shortcut hint on the right (informational only —
    /// the actual shortcut is not bound).
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Render the item in the danger tone — red label, red hover highlight.
    /// Use for destructive actions.
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    /// Disable the item. Disabled items do not fire `clicked()` and render
    /// with muted text.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Widget for MenuItem {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());
        let p = &theme.palette;
        let t = &theme.typography;

        let pad_x = 10.0;
        let pad_y = 6.0;
        let gap_x = 16.0;

        let label_color = if !self.enabled {
            p.text_faint
        } else if self.danger {
            p.danger
        } else {
            p.text
        };

        let label_galley =
            crate::theme::placeholder_galley(ui, self.label.text(), t.body, false, f32::INFINITY);

        let shortcut_galley = self
            .shortcut
            .as_deref()
            .map(|s| crate::theme::placeholder_galley(ui, s, t.small, false, f32::INFINITY));

        let content_w =
            label_galley.size().x + shortcut_galley.as_ref().map_or(0.0, |g| g.size().x + gap_x);
        let desired = Vec2::new(
            ui.available_width().max(content_w + pad_x * 2.0),
            label_galley.size().y.max(t.body) + pad_y * 2.0,
        );

        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(desired, sense);

        if ui.is_rect_visible(rect) {
            let is_hovered = response.hovered() && self.enabled;
            if is_hovered {
                let bg = if self.danger {
                    with_alpha(p.red, 40)
                } else {
                    with_alpha(p.sky, 28)
                };
                let radius = CornerRadius::same((theme.control_radius as u8).saturating_sub(2));
                ui.painter().rect_filled(rect, radius, bg);
            }

            let label_pos = Pos2::new(
                rect.min.x + pad_x,
                rect.center().y - label_galley.size().y * 0.5,
            );
            ui.painter().galley(label_pos, label_galley, label_color);

            if let Some(galley) = shortcut_galley {
                let pos = Pos2::new(
                    rect.max.x - pad_x - galley.size().x,
                    rect.center().y - galley.size().y * 0.5,
                );
                let color = if !self.enabled {
                    p.text_faint
                } else if self.danger {
                    with_alpha(p.danger, 200)
                } else {
                    p.text_muted
                };
                ui.painter().galley(pos, galley, color);
            }
        }

        response.widget_info(|| {
            WidgetInfo::labeled(WidgetType::Button, self.enabled, self.label.text())
        });
        response
    }
}
