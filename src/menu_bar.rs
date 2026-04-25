//! Top-of-window menu bar — a horizontal strip of click-to-open menus.
//!
//! [`MenuBar`] paints a desktop-style menu strip with an optional brand on
//! the left, a row of menu triggers (File, Edit, View, …), and an optional
//! status slot on the right. Clicking a trigger opens its dropdown; once
//! one menu is open, hovering a sibling trigger switches to it. That
//! "menu mode" hover-switching matches how native menubars feel.
//!
//! ```no_run
//! # use elegance::{MenuBar, MenuItem};
//! # egui::__run_test_ui(|ui| {
//! MenuBar::new("app_menubar")
//!     .brand("Elegance")
//!     .status("main \u{00b7} up to date")
//!     .show(ui, |bar| {
//!         bar.menu("File", |ui| {
//!             ui.add(MenuItem::new("New").shortcut("\u{2318}N"));
//!             ui.add(MenuItem::new("Open\u{2026}").shortcut("\u{2318}O"));
//!             ui.separator();
//!             ui.add(MenuItem::new("Save").shortcut("\u{2318}S"));
//!         });
//!         bar.menu("Edit", |ui| {
//!             ui.add(MenuItem::new("Undo").shortcut("\u{2318}Z"));
//!         });
//!     });
//! # });
//! ```
//!
//! Dropdowns close on outside-click, `Esc`, or clicking an item.
//!
//! For a single click-to-open menu attached to an arbitrary trigger button,
//! use [`Menu`](crate::Menu) directly.

use std::hash::Hash;

use egui::{
    emath::RectAlign, Align, Color32, CornerRadius, Frame, Id, Layout, Margin, Popup,
    PopupCloseBehavior, Pos2, Rect, Sense, SetOpenCommand, Stroke, Ui, Vec2, WidgetInfo,
    WidgetText, WidgetType,
};

use crate::theme::{mix, with_alpha, Accent, Theme};

const STRIP_PAD_Y: f32 = 4.0;
const STRIP_PAD_X: f32 = 6.0;
const TRIGGER_PAD_X: f32 = 10.0;
const TRIGGER_PAD_Y: f32 = 5.0;
const BRAND_LOGO_SIZE: f32 = 14.0;

#[derive(Debug, Clone)]
struct StatusContent {
    text: WidgetText,
    dot: Option<Color32>,
}

/// A horizontal desktop-style menu bar with click-to-open dropdowns.
///
/// See the module-level docs for an example.
#[derive(Debug, Clone)]
#[must_use = "Call `.show(ui, |bar| ...)` to render the menu bar."]
pub struct MenuBar {
    id_salt: Id,
    brand: Option<WidgetText>,
    status: Option<StatusContent>,
}

impl MenuBar {
    /// Create a new menu bar keyed by `id_salt`. The salt scopes per-menu
    /// open state in egui memory and must be stable across frames.
    pub fn new(id_salt: impl Hash) -> Self {
        Self {
            id_salt: Id::new(("elegance::menu_bar", Id::new(id_salt))),
            brand: None,
            status: None,
        }
    }

    /// Show a brand label on the left, preceded by a small accent square.
    /// Use this for the application name.
    #[inline]
    pub fn brand(mut self, text: impl Into<WidgetText>) -> Self {
        self.brand = Some(text.into());
        self
    }

    /// Show a muted status line on the right (e.g. `"main · up to date"`).
    #[inline]
    pub fn status(mut self, text: impl Into<WidgetText>) -> Self {
        self.status = Some(StatusContent {
            text: text.into(),
            dot: None,
        });
        self
    }

    /// Show a status line preceded by a coloured dot, useful for indicating
    /// connection or run state (green for healthy, amber for running, red
    /// for failing).
    #[inline]
    pub fn status_with_dot(mut self, text: impl Into<WidgetText>, dot: Color32) -> Self {
        self.status = Some(StatusContent {
            text: text.into(),
            dot: Some(dot),
        });
        self
    }

    /// Render the menu bar. The closure receives a [`MenuBarUi`] used to
    /// declare each menu's trigger label and dropdown body.
    pub fn show<R>(self, ui: &mut Ui, body: impl FnOnce(&mut MenuBarUi<'_>) -> R) -> R {
        let theme = Theme::current(ui.ctx());
        let p = &theme.palette;

        // The strip sits between the page background and the elevated card
        // tone — close to body bg on dark themes, close to card on light
        // themes. Mixing keeps the same visual relationship across all four
        // built-in palettes.
        let menubar_fill = mix(p.bg, p.card, 0.45);

        // Read the previous frame's snapshot: which menus existed, where
        // their triggers were, and whether any was open.
        let state_id = self.id_salt.with("__state");
        let prev_state: MenuBarFrameState = ui
            .ctx()
            .data(|d| d.get_temp::<MenuBarFrameState>(state_id))
            .unwrap_or_default();

        // Hover arbitrator. If a sibling trigger is being hovered while
        // another menu of ours is currently open, close that other menu
        // *before* any popup renders this frame. The new menu's normal
        // hover-switch logic then opens itself in its own paint, and only
        // one popup ends up visible. Without this step, the previously
        // open menu would render its area for one extra frame because its
        // `Popup::show` runs (and renders) before the new menu's call to
        // `open_id` overwrites the memory slot.
        if prev_state.any_open {
            if let Some(pointer) = ui.ctx().pointer_hover_pos() {
                let open_idx = prev_state
                    .triggers
                    .iter()
                    .position(|(id, _)| Popup::is_id_open(ui.ctx(), *id));
                if let Some(open_idx) = open_idx {
                    let on_sibling = prev_state
                        .triggers
                        .iter()
                        .enumerate()
                        .any(|(i, (_, rect))| i != open_idx && rect.contains(pointer));
                    if on_sibling {
                        Popup::close_id(ui.ctx(), prev_state.triggers[open_idx].0);
                    }
                }
            }
        }

        let frame = Frame::new()
            .fill(menubar_fill)
            .inner_margin(Margin::symmetric(STRIP_PAD_X as i8, STRIP_PAD_Y as i8));

        let outer = frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.set_min_height(theme.typography.body + TRIGGER_PAD_Y * 2.0);

                if let Some(brand) = self.brand.as_ref() {
                    paint_brand(ui, &theme, brand.clone());
                }

                let mut bar = MenuBarUi {
                    ui,
                    base_id: self.id_salt,
                    next_idx: 0,
                    any_open_prev: prev_state.any_open,
                    any_open_now: false,
                    triggers: Vec::with_capacity(prev_state.triggers.len()),
                };
                let r = body(&mut bar);
                let any_open_now = bar.any_open_now;
                let triggers = std::mem::take(&mut bar.triggers);

                if let Some(status) = self.status.as_ref() {
                    bar.ui
                        .with_layout(Layout::right_to_left(Align::Center), |ui| {
                            paint_status(ui, &theme, status);
                        });
                }

                bar.ui.ctx().data_mut(|d| {
                    d.insert_temp(
                        state_id,
                        MenuBarFrameState {
                            triggers,
                            any_open: any_open_now,
                        },
                    )
                });

                r
            })
            .inner
        });

        // Bottom border separates the strip from the body content below.
        let strip_rect = outer.response.rect;
        ui.painter().line_segment(
            [
                Pos2::new(strip_rect.min.x, strip_rect.max.y - 0.5),
                Pos2::new(strip_rect.max.x, strip_rect.max.y - 0.5),
            ],
            Stroke::new(1.0, p.border),
        );

        outer.inner
    }
}

/// Per-frame state stored in [`egui::Memory`] so the next frame can know
/// where each trigger sat and whether any of our menus was open.
#[derive(Clone, Default, Debug)]
struct MenuBarFrameState {
    triggers: Vec<(Id, Rect)>,
    any_open: bool,
}

/// The handle passed to a [`MenuBar::show`] closure for declaring menu
/// triggers. Each call to [`MenuBarUi::menu`] paints one trigger and its
/// dropdown.
pub struct MenuBarUi<'u> {
    ui: &'u mut Ui,
    base_id: Id,
    next_idx: usize,
    any_open_prev: bool,
    any_open_now: bool,
    triggers: Vec<(Id, Rect)>,
}

impl<'u> std::fmt::Debug for MenuBarUi<'u> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuBarUi")
            .field("base_id", &self.base_id)
            .field("next_idx", &self.next_idx)
            .field("any_open_prev", &self.any_open_prev)
            .field("any_open_now", &self.any_open_now)
            .finish()
    }
}

impl<'u> MenuBarUi<'u> {
    /// Paint a single menu trigger with `label` and attach a dropdown
    /// populated by `body`. Clicking an item inside the dropdown dismisses
    /// the menu — the standard pattern for action-style menus (File / Edit
    /// / etc.). For settings-style menus that should stay open while the
    /// user toggles items, use [`MenuBarUi::menu_keep_open`].
    ///
    /// Returns `Some` with the body closure's return value while the
    /// dropdown is open, `None` while it's closed.
    pub fn menu<R>(
        &mut self,
        label: impl Into<WidgetText>,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        self.menu_inner(label, PopupCloseBehavior::CloseOnClick, body)
    }

    /// Like [`MenuBarUi::menu`], but the dropdown stays open while the
    /// user clicks items inside it. Useful for menus full of toggles
    /// (checkboxes, radio groups) where the user expects to see the state
    /// change without the menu vanishing. The menu still closes on click
    /// outside, on `Esc`, or when the user clicks the trigger again.
    pub fn menu_keep_open<R>(
        &mut self,
        label: impl Into<WidgetText>,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        self.menu_inner(label, PopupCloseBehavior::CloseOnClickOutside, body)
    }

    fn menu_inner<R>(
        &mut self,
        label: impl Into<WidgetText>,
        close_behavior: PopupCloseBehavior,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        let label: WidgetText = label.into();
        let theme = Theme::current(self.ui.ctx());
        let p = &theme.palette;
        let t = &theme.typography;

        let idx = self.next_idx;
        self.next_idx += 1;
        let popup_id = self.base_id.with("__menu").with(idx);

        let galley =
            crate::theme::placeholder_galley(self.ui, label.text(), t.body, false, f32::INFINITY);
        let trigger_size = Vec2::new(
            galley.size().x + TRIGGER_PAD_X * 2.0,
            galley.size().y + TRIGGER_PAD_Y * 2.0,
        );
        let (rect, response) = self.ui.allocate_exact_size(trigger_size, Sense::click());
        self.triggers.push((popup_id, rect));

        let was_open = Popup::is_id_open(self.ui.ctx(), popup_id);
        let hovered = response.hovered();
        let clicked = response.clicked();

        // Decide what to do this frame.
        //   1. Click: toggle this menu open/closed.
        //   2. Hover while another menu of ours is already open: switch
        //      to this one. `Bool(true)` opens this id and closes others
        //      (egui memory only tracks one open popup at a time).
        //   3. Otherwise: leave the state as-is.
        let intent: Option<SetOpenCommand> = if clicked {
            Some(SetOpenCommand::Bool(!was_open))
        } else if self.any_open_prev && hovered && !was_open {
            Some(SetOpenCommand::Bool(true))
        } else {
            None
        };

        let will_be_open = matches!(intent, Some(SetOpenCommand::Bool(true)))
            || (was_open && !matches!(intent, Some(SetOpenCommand::Bool(false))));
        self.any_open_now |= will_be_open;

        if self.ui.is_rect_visible(rect) {
            let bg = if will_be_open {
                p.card
            } else if hovered {
                with_alpha(p.text, 14)
            } else {
                Color32::TRANSPARENT
            };
            if bg.a() > 0 {
                self.ui.painter().rect_filled(rect, CornerRadius::ZERO, bg);
            }
            let text_color = if will_be_open || hovered {
                p.text
            } else {
                p.text_muted
            };
            let pos = Pos2::new(
                rect.min.x + TRIGGER_PAD_X,
                rect.center().y - galley.size().y * 0.5,
            );
            self.ui.painter().galley(pos, galley, text_color);
        }

        // Dropdown panel. Top-left corner is square so the panel reads
        // visually flush with the trigger above it.
        let r = theme.card_radius as u8;
        let frame = Frame::new()
            .fill(p.card)
            .stroke(Stroke::new(1.0, p.border))
            .corner_radius(CornerRadius {
                nw: 0,
                ne: r,
                sw: r,
                se: r,
            })
            .inner_margin(Margin::same(4));

        let label_text = label.text().to_string();
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, &label_text));

        let result = Popup::menu(&response)
            .id(popup_id)
            .open_memory(intent)
            .align(RectAlign::BOTTOM_START)
            .gap(0.0)
            .frame(frame)
            .close_behavior(close_behavior)
            .show(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                body(ui)
            });

        result.map(|r| r.inner)
    }
}

fn paint_brand(ui: &mut Ui, theme: &Theme, text: WidgetText) {
    let p = &theme.palette;
    let t = &theme.typography;

    let logo_size = Vec2::splat(BRAND_LOGO_SIZE);
    let (logo_rect, _) = ui.allocate_exact_size(logo_size, Sense::hover());
    ui.painter()
        .rect_filled(logo_rect, CornerRadius::same(3), p.accent_fill(Accent::Sky));
    ui.add_space(8.0);

    let galley = crate::theme::placeholder_galley(ui, text.text(), t.body, true, f32::INFINITY);
    let label_size = Vec2::new(galley.size().x, galley.size().y + 4.0);
    let (rect, _) = ui.allocate_exact_size(label_size, Sense::hover());
    let pos = Pos2::new(rect.min.x, rect.center().y - galley.size().y * 0.5);
    ui.painter().galley(pos, galley, p.text);

    ui.add_space(14.0);
}

fn paint_status(ui: &mut Ui, theme: &Theme, status: &StatusContent) {
    let p = &theme.palette;
    let t = &theme.typography;

    // Layout is right-to-left here, so allocations come from the right edge
    // inward — paint text first, then the dot to the left of it.
    ui.add_space(4.0);
    let galley =
        crate::theme::placeholder_galley(ui, status.text.text(), t.small, false, f32::INFINITY);
    let label_size = Vec2::new(galley.size().x, galley.size().y + 4.0);
    let (rect, _) = ui.allocate_exact_size(label_size, Sense::hover());
    let pos = Pos2::new(rect.min.x, rect.center().y - galley.size().y * 0.5);
    ui.painter().galley(pos, galley, p.text_faint);

    if let Some(color) = status.dot {
        ui.add_space(6.0);
        let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(7.0), Sense::hover());
        ui.painter().circle_filled(dot_rect.center(), 3.5, color);
    }
}
