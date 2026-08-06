//! Layering and modality contract for the overlay widgets (`Modal`, `Drawer`).
//!
//! These widgets stack two things on top of the app: a full-viewport backdrop
//! that must swallow every interaction behind it, and a content surface that
//! must stay above the backdrop while still letting popups opened *from
//! inside* it float above the content. The assertions below pin that contract
//! down, because every part of it is easy to break by accident when touching
//! `Order` or the areas involved.

use egui::{Order, Pos2, Vec2};
use egui_kittest::{Harness, kittest::Queryable as _};
use elegance::{Drawer, DrawerSide, Modal, Popover, Theme};

/// Ordered list of the layers egui is currently painting, bottom first.
fn layers(h: &Harness<impl Sized>) -> Vec<String> {
    h.ctx
        .memory(|m| m.layer_ids().map(|l| format!("{l:?}")).collect())
}

/// Index of the one layer whose debug formatting contains `needle`.
fn depth(layers: &[String], needle: &str) -> usize {
    layers
        .iter()
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("no layer matching {needle:?} in {layers:#?}"))
}

fn click_at<S>(h: &mut Harness<S>, pos: Pos2) {
    h.hover_at(pos);
    h.run_steps(1);
    h.drag_at(pos);
    h.run_steps(1);
    h.drop_at(pos);
    h.run_steps(2);
}

/// Settle the harness: two passes, so anything tracked one pass behind
/// (the open-overlay registry, the modal-layer claim) has taken effect.
fn settle<S>(h: &mut Harness<S>) {
    h.run();
    h.run();
}

// ---------------------------------------------------------------------------
// An `Order::Foreground` window beneath an overlay
// ---------------------------------------------------------------------------

/// A plain `egui::Window` at `Order::Foreground`, the exact thing PR #13
/// reported an overlay failing to cover.
fn foreground_window(ctx: &egui::Context, pos: Pos2, clicks: &mut usize) {
    egui::Window::new("behind")
        .order(Order::Foreground)
        .fixed_pos(pos)
        .fixed_size(Vec2::new(220.0, 140.0))
        .show(ctx, |ui| {
            if ui.button("behind button").clicked() {
                *clicks += 1;
            }
        });
}

/// A point over the window's button, outside the modal card.
const OVER_WINDOW: Pos2 = Pos2::new(40.0, 60.0);

struct WindowState {
    open: bool,
    window_clicks: usize,
}

/// The window above, with an open modal over it whose button sits under the
/// modal's backdrop.
fn window_harness(close_on_backdrop: bool) -> Harness<'static, WindowState> {
    let mut h = Harness::builder()
        .with_size(Vec2::new(600.0, 400.0))
        .build_ui_state(
            move |ui, s: &mut WindowState| {
                Theme::slate().install(ui.ctx());
                foreground_window(ui.ctx(), Pos2::ZERO, &mut s.window_clicks);
                Modal::new("m", &mut s.open)
                    .heading("Confirm")
                    .close_on_backdrop(close_on_backdrop)
                    .show(ui.ctx(), |ui| {
                        ui.label("Are you sure?");
                    });
            },
            WindowState {
                open: true,
                window_clicks: 0,
            },
        );
    settle(&mut h);
    h
}

#[test]
fn modal_backdrop_blocks_a_foreground_window() {
    let mut h = window_harness(true);
    click_at(&mut h, OVER_WINDOW);

    assert_eq!(
        h.state().window_clicks,
        0,
        "a window behind the backdrop must not receive clicks"
    );
    assert!(
        !h.state().open,
        "the click belongs to the backdrop, which dismisses the modal"
    );
}

/// The original report behind PR #13: clicking outside the modal, over a
/// foreground window, used to raise that window above the modal.
///
/// The modal here ignores backdrop clicks, so it is still open afterwards and
/// the layer order is worth asserting on — dismissing it would leave the
/// window legitimately on top and the check would prove nothing.
#[test]
fn modal_stays_above_a_foreground_window_that_was_clicked() {
    let mut h = window_harness(false);
    click_at(&mut h, OVER_WINDOW);

    assert!(h.state().open, "this modal ignores backdrop clicks");
    assert_eq!(h.state().window_clicks, 0, "the window stays unreachable");

    let l = layers(&h);
    assert!(
        depth(&l, "elegance_modal") > depth(&l, "behind"),
        "the modal must not sink behind a foreground window: {l:#?}"
    );
}

/// The drawer carried the same defect as the modal in PR #13, so it gets the
/// same guarantee: a foreground window beneath it neither receives the click
/// nor climbs above the panel.
#[test]
fn drawer_stays_above_a_foreground_window_that_was_clicked() {
    let mut h = Harness::builder()
        .with_size(Vec2::new(600.0, 400.0))
        .build_ui_state(
            |ui, s: &mut WindowState| {
                Theme::slate().install(ui.ctx());
                foreground_window(ui.ctx(), Pos2::ZERO, &mut s.window_clicks);
                Drawer::new("d", &mut s.open)
                    .side(DrawerSide::Right)
                    .title("Inspector")
                    .close_on_backdrop(false)
                    .show(ui.ctx(), |ui| {
                        ui.label("body");
                    });
            },
            WindowState {
                open: true,
                window_clicks: 0,
            },
        );
    settle(&mut h);

    click_at(&mut h, OVER_WINDOW);
    assert_eq!(h.state().window_clicks, 0, "the window stays unreachable");

    let l = layers(&h);
    assert!(
        depth(&l, "elegance_drawer") > depth(&l, "behind"),
        "the drawer must not sink behind a foreground window: {l:#?}"
    );
}

/// The far side of a drawer's backdrop, well past where the panel reaches.
///
/// Unlike the modal's, the drawer's area rect grows to cover its own panel, so
/// a window overlapping the panel is occluded by the area alone. Out here there
/// is only backdrop, and nothing but the modal-layer claim keeps a press from
/// being attributed to the window underneath and raising it.
#[test]
fn drawer_backdrop_blocks_a_window_beyond_the_panel() {
    let mut h = Harness::builder()
        .with_size(Vec2::new(900.0, 400.0))
        .build_ui_state(
            |ui, s: &mut WindowState| {
                Theme::slate().install(ui.ctx());
                foreground_window(ui.ctx(), Pos2::new(620.0, 60.0), &mut s.window_clicks);
                Drawer::new("d", &mut s.open)
                    .side(DrawerSide::Left)
                    .title("Nav")
                    .close_on_backdrop(false)
                    .show(ui.ctx(), |ui| {
                        ui.label("body");
                    });
            },
            WindowState {
                open: true,
                window_clicks: 0,
            },
        );
    settle(&mut h);

    click_at(&mut h, Pos2::new(660.0, 120.0));
    assert_eq!(
        h.state().window_clicks,
        0,
        "the window stays unreachable through the backdrop"
    );

    let l = layers(&h);
    assert!(
        depth(&l, "elegance_drawer") > depth(&l, "behind"),
        "pressing the bare backdrop must not raise the window: {l:#?}"
    );
}

// ---------------------------------------------------------------------------
// Popups opened from inside a modal
// ---------------------------------------------------------------------------

#[test]
fn popover_opened_inside_a_modal_floats_above_the_card() {
    struct S {
        open: bool,
        picked: usize,
    }
    let mut h = Harness::builder()
        .with_size(Vec2::new(600.0, 400.0))
        .build_ui_state(
            |ui, s: &mut S| {
                Theme::slate().install(ui.ctx());
                Modal::new("m", &mut s.open)
                    .heading("Filters")
                    .show(ui.ctx(), |ui| {
                        let trigger = ui.button("Open popover");
                        Popover::new("pop").show(&trigger, |ui| {
                            if ui.button("Pick me").clicked() {
                                s.picked += 1;
                            }
                        });
                    });
            },
            S {
                open: true,
                picked: 0,
            },
        );
    settle(&mut h);

    h.get_by_label("Open popover").click();
    settle(&mut h);

    let l = layers(&h);
    assert!(
        depth(&l, "elegance::popover") > depth(&l, "elegance_modal"),
        "a popover opened inside the modal must paint above the card: {l:#?}"
    );

    h.get_by_label("Pick me").click();
    h.run();
    assert_eq!(
        h.state().picked,
        1,
        "the popover's contents must remain clickable inside a modal"
    );
}

// ---------------------------------------------------------------------------
// Backdrop / content ordering within one overlay
// ---------------------------------------------------------------------------

struct CardState {
    open: bool,
    body_clicks: usize,
}

/// A modal whose body holds one button, so clicks can be attributed to the
/// card or to the backdrop.
fn card_harness(close_on_backdrop: bool) -> Harness<'static, CardState> {
    let mut h = Harness::builder()
        .with_size(Vec2::new(600.0, 400.0))
        .build_ui_state(
            move |ui, s: &mut CardState| {
                Theme::slate().install(ui.ctx());
                Modal::new("m", &mut s.open)
                    .heading("Confirm")
                    .close_on_backdrop(close_on_backdrop)
                    .show(ui.ctx(), |ui| {
                        if ui.button("Body button").clicked() {
                            s.body_clicks += 1;
                        }
                    });
            },
            CardState {
                open: true,
                body_clicks: 0,
            },
        );
    settle(&mut h);
    h
}

/// The card and its backdrop share a layer, so their relationship is settled
/// by hit-testing order within that layer rather than by z-order: a click
/// landing on the card must reach the card's widgets and must *not* also read
/// as a backdrop click.
#[test]
fn clicks_on_the_card_do_not_fall_through_to_the_backdrop() {
    let mut h = card_harness(true);

    h.get_by_label("Body button").click();
    settle(&mut h);

    assert_eq!(
        h.state().body_clicks,
        1,
        "the card's widgets take the click"
    );
    assert!(
        h.state().open,
        "a click inside the card must not dismiss the modal as a backdrop click"
    );
}

/// Pressing the backdrop of a modal that ignores backdrop clicks must not
/// leave the dimmer covering the card it dims.
#[test]
fn pressing_an_inert_backdrop_leaves_the_card_reachable() {
    let mut h = card_harness(false);

    click_at(&mut h, Pos2::new(20.0, 20.0));
    assert!(h.state().open, "close_on_backdrop(false) must keep it open");

    h.get_by_label("Body button").click();
    settle(&mut h);
    assert_eq!(
        h.state().body_clicks,
        1,
        "the card must stay reachable after its backdrop was pressed"
    );
}

// ---------------------------------------------------------------------------
// Two overlays at once
// ---------------------------------------------------------------------------

/// Repeated `Esc` presses must peel a stack of overlays one at a time, in
/// reverse order of opening — never collapse the whole stack at once.
#[test]
fn escape_unwinds_stacked_modals_one_at_a_time() {
    struct S {
        outer: bool,
        inner: bool,
    }
    let mut h = Harness::builder()
        .with_size(Vec2::new(700.0, 460.0))
        .build_ui_state(
            |ui, s: &mut S| {
                Theme::slate().install(ui.ctx());
                Modal::new("outer", &mut s.outer)
                    .heading("Outer")
                    .show(ui.ctx(), |ui| {
                        ui.label("outer body");
                    });
                Modal::new("inner", &mut s.inner)
                    .heading("Inner")
                    .show(ui.ctx(), |ui| {
                        ui.label("inner body");
                    });
            },
            S {
                outer: true,
                inner: true,
            },
        );
    settle(&mut h);

    h.key_press(egui::Key::Escape);
    settle(&mut h);
    assert!(!h.state().inner, "first Esc takes the inner modal");
    assert!(h.state().outer, "and leaves the outer one standing");

    h.key_press(egui::Key::Escape);
    settle(&mut h);
    assert!(
        !h.state().outer,
        "the next Esc takes the outer modal, now topmost"
    );
}

struct StackState {
    drawer: bool,
    modal: bool,
    drawer_clicks: usize,
}

/// A left-anchored drawer with a `nav item` button, and a modal that starts
/// out `modal_open`.
fn stack_harness(modal_open: bool) -> Harness<'static, StackState> {
    let mut h = Harness::builder()
        .with_size(Vec2::new(700.0, 460.0))
        .build_ui_state(
            |ui, s: &mut StackState| {
                Theme::slate().install(ui.ctx());
                Drawer::new("d", &mut s.drawer)
                    .side(DrawerSide::Left)
                    .title("Nav")
                    .show(ui.ctx(), |ui| {
                        if ui.button("nav item").clicked() {
                            s.drawer_clicks += 1;
                        }
                    });
                Modal::new("m", &mut s.modal)
                    .heading("Confirm")
                    .show(ui.ctx(), |ui| {
                        ui.label("body");
                    });
            },
            StackState {
                drawer: true,
                modal: modal_open,
                drawer_clicks: 0,
            },
        );
    settle(&mut h);
    h
}

#[test]
fn escape_closes_only_the_topmost_overlay() {
    let mut h = stack_harness(true);

    h.key_press(egui::Key::Escape);
    settle(&mut h);

    assert!(!h.state().modal, "Esc dismisses the modal on top");
    assert!(
        h.state().drawer,
        "Esc must not also dismiss the drawer underneath"
    );
}

/// A modal raised over an already-open drawer must take `Esc` itself, leaving
/// the drawer beneath untouched.
#[test]
fn modal_opened_over_a_drawer_takes_escape_from_it() {
    let mut h = stack_harness(false);
    // The modal arrives over an already-settled drawer.
    h.state_mut().modal = true;
    settle(&mut h);

    h.key_press(egui::Key::Escape);
    settle(&mut h);
    assert!(!h.state().modal, "the newly raised modal takes Esc");
    assert!(h.state().drawer, "the drawer beneath must survive");
}

#[test]
fn modal_blocks_an_open_drawer_underneath() {
    let mut h = stack_harness(true);

    let l = layers(&h);
    assert!(
        depth(&l, "elegance_modal") > depth(&l, "elegance_drawer"),
        "the modal must sit above an open drawer: {l:#?}"
    );

    // Over the drawer's `nav item` button, which is reachable when the modal
    // does not cover it.
    click_at(&mut h, Pos2::new(58.0, 80.0));
    assert_eq!(
        h.state().drawer_clicks,
        0,
        "drawer contents are inert while a modal covers them"
    );
}

// ---------------------------------------------------------------------------
// Focus restoration vs. draw order
// ---------------------------------------------------------------------------

/// Focus must return to the pre-overlay widget on close even when the app
/// draws the overlay *before* the content behind it.
///
/// This ordering is the one that catches focus bugs: anything the overlay does
/// to focus happens before the background widgets register for it, so a
/// mistake there is silently undone by egui later in the same frame.
#[test]
fn focus_returns_to_background_drawn_after_the_modal() {
    struct S {
        open: bool,
    }
    let mut h = Harness::builder()
        .with_size(Vec2::new(600.0, 400.0))
        .build_ui_state(
            |ui, s: &mut S| {
                Theme::slate().install(ui.ctx());
                Modal::new("m", &mut s.open)
                    .heading("Confirm")
                    .show(ui.ctx(), |ui| {
                        ui.label("body");
                    });
                // Drawn AFTER the overlay, on purpose.
                let _ = ui.button("background button");
            },
            S { open: false },
        );
    settle(&mut h);
    h.get_by_label("background button").focus();
    h.run();
    let bg = h.ctx.memory(|m| m.focused());
    assert!(bg.is_some(), "background button should be focusable");

    h.state_mut().open = true;
    settle(&mut h);
    assert_ne!(
        h.ctx.memory(|m| m.focused()),
        bg,
        "opening the modal moves focus into the dialog"
    );

    h.key_press(egui::Key::Escape);
    settle(&mut h);
    assert_eq!(
        h.ctx.memory(|m| m.focused()),
        bg,
        "closing must hand focus back to the background button"
    );
}

/// The same ordering hazard, for the drawer. Closed from code rather than with
/// `Esc`: that is the case where the previous frame's modal-layer claim is
/// still in force on the frame the close is noticed, so only the second stage
/// of the restore survives.
#[test]
fn focus_returns_to_background_drawn_after_the_drawer() {
    struct S {
        open: bool,
    }
    let mut h = Harness::builder()
        .with_size(Vec2::new(600.0, 400.0))
        .build_ui_state(
            |ui, s: &mut S| {
                Theme::slate().install(ui.ctx());
                Drawer::new("d", &mut s.open)
                    .title("Inspector")
                    .show(ui.ctx(), |ui| {
                        ui.label("body");
                    });
                let _ = ui.button("background button");
            },
            S { open: false },
        );
    settle(&mut h);
    h.get_by_label("background button").focus();
    h.run();
    let bg = h.ctx.memory(|m| m.focused());
    assert!(bg.is_some());

    h.state_mut().open = true;
    settle(&mut h);
    h.state_mut().open = false;
    settle(&mut h);
    assert_eq!(
        h.ctx.memory(|m| m.focused()),
        bg,
        "closing must hand focus back to the background button"
    );
}
