//! Keyboard operation of `ColorPicker`'s three continuous surfaces.
//!
//! The SV plane, hue strip, and alpha strip were pointer-only: focusable, so
//! `Tab` landed on them, but bound to no keys, so the arrows navigated straight
//! back off and a keyboard user could not pick a colour at all. These tests
//! drive each surface by key and assert the bound colour moves.

use eframe::egui;
use egui::accesskit::Role;
use egui::{Color32, Key, ecolor::HsvaGamma};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use elegance::{ColorPicker, Theme};

const ID: &str = "cp_keys";

struct Probe {
    color: Color32,
}

/// A picker with its popover forced open, so the surfaces are live every frame.
fn picker_harness(initial: Color32) -> Harness<'static, Probe> {
    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(400.0, 460.0))
        .build_ui_state(
            |ui, probe: &mut Probe| {
                Theme::slate().install(ui.ctx());
                egui::Popup::open_id(
                    &ui.ctx().clone(),
                    elegance::Popover::popup_id(("elegance::color_picker", egui::Id::new(ID))),
                );
                ui.set_min_size(egui::vec2(360.0, 400.0));
                ui.add(ColorPicker::new(ID, &mut probe.color));
            },
            Probe { color: initial },
        );
    harness.run();
    harness.run();
    harness
}

fn focus(harness: &Harness<'_, Probe>, label: &str) {
    harness.get_by_role_and_label(Role::Slider, label).focus();
}

fn hsv(probe: &Probe) -> HsvaGamma {
    HsvaGamma::from(probe.color)
}

/// A mid-range colour: every channel has room to move in both directions.
fn mid() -> Color32 {
    Color32::from_rgba_unmultiplied(0x38, 0xbd, 0xf8, 0x80)
}

#[test]
fn sv_plane_arrows_move_saturation_and_value() {
    let mut harness = picker_harness(mid());
    focus(&harness, "Saturation and value");
    harness.run();

    let before = hsv(harness.state());

    harness.key_press(Key::ArrowRight);
    harness.step();
    let after_right = hsv(harness.state());
    assert!(
        after_right.s > before.s,
        "ArrowRight should raise saturation, {} -> {}",
        before.s,
        after_right.s
    );

    harness.key_press(Key::ArrowUp);
    harness.step();
    let after_up = hsv(harness.state());
    assert!(
        after_up.v > after_right.v,
        "ArrowUp should raise value, {} -> {}",
        after_right.v,
        after_up.v
    );

    harness.key_press(Key::ArrowLeft);
    harness.step();
    assert!(
        hsv(harness.state()).s < after_up.s,
        "ArrowLeft should lower saturation"
    );
}

#[test]
fn hue_strip_arrows_move_hue() {
    let mut harness = picker_harness(mid());
    focus(&harness, "Hue");
    harness.run();

    let before = hsv(harness.state()).h;

    harness.key_press(Key::ArrowRight);
    harness.step();
    assert!(
        hsv(harness.state()).h > before,
        "ArrowRight should advance hue, {} -> {}",
        before,
        hsv(harness.state()).h
    );

    harness.key_press(Key::Home);
    harness.step();
    assert!(
        hsv(harness.state()).h < 0.001,
        "Home should jump hue to the start of the strip, got {}",
        hsv(harness.state()).h
    );
}

#[test]
fn alpha_strip_arrows_move_alpha() {
    let mut harness = picker_harness(mid());
    focus(&harness, "Alpha");
    harness.run();

    let before = harness.state().color.a();

    harness.key_press(Key::ArrowRight);
    harness.step();
    assert!(
        harness.state().color.a() > before,
        "ArrowRight should raise alpha, {} -> {}",
        before,
        harness.state().color.a()
    );

    harness.key_press(Key::End);
    harness.step();
    assert_eq!(
        harness.state().color.a(),
        255,
        "End should take alpha to fully opaque"
    );

    harness.key_press(Key::Home);
    harness.step();
    assert_eq!(
        harness.state().color.a(),
        0,
        "Home should take alpha to fully transparent"
    );
}

/// `Shift` is a 10x nudge on these strips, matching the crate's sliders. Alpha
/// is the channel where that arithmetic is legible in the bound value: one step
/// is 1% of 255, so ten of them clear a byte's worth of rounding either way.
#[test]
fn shift_is_a_ten_times_nudge() {
    let mut harness = picker_harness(Color32::from_rgba_unmultiplied(0x38, 0xbd, 0xf8, 0x00));

    focus(&harness, "Alpha");
    harness.run();
    harness.key_press(Key::ArrowRight);
    harness.step();
    let one_step = harness.state().color.a();

    // Back to zero, then a single shifted press.
    harness.key_press(Key::Home);
    harness.step();
    harness.event(egui::Event::Key {
        key: Key::ArrowRight,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::SHIFT,
    });
    harness.step();
    let shifted = harness.state().color.a();

    assert!(
        shifted > one_step * 5,
        "Shift+ArrowRight should be far larger than a plain nudge: {one_step} vs {shifted}"
    );
}

/// The plane binds all four arrows, so it has to claim both axes. Without that,
/// the first `ArrowDown` nudges value *and* hands focus to the strip below,
/// leaving the rest of the presses to drive something else — so the tell is not
/// where focus went, it is that only one of three presses landed.
#[test]
fn plane_takes_every_vertical_nudge_instead_of_leaking_focus() {
    let mut harness = picker_harness(mid());
    focus(&harness, "Saturation and value");
    harness.run();

    let before = hsv(harness.state()).v;

    for _ in 0..3 {
        harness.key_press(Key::ArrowDown);
        harness.step();
    }

    // Three steps of 0.01 each. One step, the leaky outcome, cannot clear 0.02
    // even with the rounding of a Color32 round-trip.
    let moved = before - hsv(harness.state()).v;
    assert!(
        moved > 0.02,
        "expected three nudges of value, got {moved} — focus leaked after the first"
    );
}

/// A keyboard adjustment is a settled one, so it reports a commit the same way
/// a pointer release does — which is what pushes the colour to the recents row.
/// Without that, a keyboard user could pick a colour and never build a history.
#[test]
fn a_keyboard_adjustment_reaches_the_recents_row() {
    let mut harness = picker_harness(mid());
    focus(&harness, "Hue");
    harness.run();

    let recents_before = harness
        .ctx
        .data(|d| d.get_temp::<Vec<Color32>>(egui::Id::new(ID).with("color_picker::recents")))
        .unwrap_or_default();
    assert!(
        recents_before.is_empty(),
        "nothing should be recorded before any adjustment"
    );

    harness.key_press(Key::ArrowRight);
    harness.step();
    harness.run();

    let recents_after = harness
        .ctx
        .data(|d| d.get_temp::<Vec<Color32>>(egui::Id::new(ID).with("color_picker::recents")))
        .unwrap_or_default();
    assert_eq!(
        recents_after.first().copied(),
        Some(harness.state().color),
        "the nudged colour should head the recents row, got {recents_after:?}"
    );
}
