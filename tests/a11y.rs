//! Accessibility tests — verify that widgets expose the right accesskit
//! roles so screen readers and automation tools can find and classify them.
//!
//! Unlike `visual.rs`, these don't compare pixels; they run a harness long
//! enough to populate the accesskit tree and then assert on it.

use eframe::egui;
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::Harness;
use elegance::{
    BadgeTone, Drawer, DrawerSide, MenuBar, MenuItem, Modal, TextInput, Theme, Toast, Toasts,
};

fn new_harness<'a>(app: impl FnMut(&mut egui::Ui) + 'a) -> Harness<'a> {
    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(600.0, 400.0))
        .build_ui(app);
    // Two runs to let any deferred layout settle (Modal's Area caches size
    // on the first frame).
    harness.run();
    harness.run();
    harness
}

#[test]
fn modal_exposes_dialog_role() {
    let harness = new_harness(|ui| {
        Theme::slate().install(ui.ctx());
        let mut open = true;
        Modal::new("a11y_dialog", &mut open)
            .heading("Confirm")
            .show(ui.ctx(), |ui| {
                ui.label("Are you sure?");
            });
    });

    // The modal's root Ui should carry Role::Dialog with the heading as label.
    let _ = harness.get_by_role_and_label(egui::accesskit::Role::Dialog, "Confirm");
}

#[test]
fn modal_alert_exposes_alert_dialog_role() {
    let harness = new_harness(|ui| {
        Theme::slate().install(ui.ctx());
        let mut open = true;
        Modal::new("a11y_alert", &mut open)
            .heading("Delete project?")
            .alert(true)
            .show(ui.ctx(), |ui| {
                ui.label("This cannot be undone.");
            });
    });

    let _ = harness.get_by_role_and_label(egui::accesskit::Role::AlertDialog, "Delete project?");
}

#[test]
fn toast_info_exposes_status_role() {
    let mut enqueued = false;
    let harness = new_harness(move |ui| {
        Theme::slate().install(ui.ctx());
        if !enqueued {
            Toast::new("Saved")
                .tone(BadgeTone::Ok)
                .description("Changes written to disk")
                .show(ui.ctx());
            enqueued = true;
        }
        Toasts::new().render(ui.ctx());
    });

    let _ = harness.get_by_role_and_label(egui::accesskit::Role::Status, "Saved");
}

#[test]
fn toast_danger_exposes_alert_role() {
    let mut enqueued = false;
    let harness = new_harness(move |ui| {
        Theme::slate().install(ui.ctx());
        if !enqueued {
            Toast::new("Upload failed")
                .tone(BadgeTone::Danger)
                .show(ui.ctx());
            enqueued = true;
        }
        Toasts::new().render(ui.ctx());
    });

    let _ = harness.get_by_role_and_label(egui::accesskit::Role::Alert, "Upload failed");
}

#[test]
fn modal_close_button_has_close_label() {
    // Without an explicit override the button's label would be "×",
    // which screen readers announce as "multiplication sign."
    let harness = new_harness(|ui| {
        Theme::slate().install(ui.ctx());
        let mut open = true;
        Modal::new("close_label", &mut open)
            .heading("Dialog")
            .show(ui.ctx(), |ui| {
                ui.label("Body");
            });
    });

    let _ = harness.get_by_role_and_label(egui::accesskit::Role::Button, "Close");
}

#[derive(Clone)]
struct FocusTestState {
    open: bool,
    email: String,
}

fn focus_harness<'a>() -> Harness<'a, FocusTestState> {
    Harness::builder()
        .with_size(egui::Vec2::new(600.0, 400.0))
        .build_ui_state(
            |ui, state: &mut FocusTestState| {
                Theme::slate().install(ui.ctx());
                ui.add(
                    TextInput::new(&mut state.email)
                        .label("Email")
                        .id_salt("focus_email"),
                );
                Modal::new("focus_modal", &mut state.open)
                    .heading("Dialog")
                    .show(ui.ctx(), |ui| {
                        ui.label("Body");
                    });
            },
            FocusTestState {
                open: false,
                email: "hi".into(),
            },
        )
}

#[test]
fn modal_focuses_close_button_on_open() {
    let mut harness = focus_harness();

    // Populate the a11y tree, then focus the Email input.
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Email")
        .focus();
    harness.run();

    // Flip the modal open. Modal::show's just-opened branch fires on the
    // first frame is_open is true; accesskit publishes focus on the
    // following frame.
    harness.state_mut().open = true;
    harness.run();
    harness.run();

    let close = harness.get_by_role_and_label(egui::accesskit::Role::Button, "Close");
    assert!(
        close.accesskit_node().is_focused(),
        "close button should be focused on modal open"
    );
}

#[test]
fn modal_restores_focus_on_close() {
    let mut harness = focus_harness();

    // Focus Email, then open the modal.
    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Email")
        .focus();
    harness.run();
    harness.state_mut().open = true;
    harness.run();
    harness.run();

    // Close the modal — focus should return to Email.
    harness.state_mut().open = false;
    harness.run();
    harness.run();

    let email = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Email");
    assert!(
        email.accesskit_node().is_focused(),
        "focus should be restored to the widget that had it before the modal opened"
    );
}

#[test]
fn drawer_exposes_dialog_role() {
    let harness = new_harness(|ui| {
        Theme::slate().install(ui.ctx());
        let mut open = true;
        Drawer::new("a11y_drawer", &mut open)
            .side(DrawerSide::Right)
            .title("Inspector")
            .subtitle("api-west-02 · INC-2187")
            .show(ui.ctx(), |ui| {
                ui.label("Body");
            });
    });

    let _ = harness.get_by_role_and_label(egui::accesskit::Role::Dialog, "Inspector");
}

#[test]
fn drawer_close_button_has_close_label() {
    let harness = new_harness(|ui| {
        Theme::slate().install(ui.ctx());
        let mut open = true;
        Drawer::new("a11y_drawer_close", &mut open)
            .title("Inspector")
            .show(ui.ctx(), |ui| {
                ui.label("Body");
            });
    });

    let _ = harness.get_by_role_and_label(egui::accesskit::Role::Button, "Close");
}

fn drawer_focus_harness<'a>() -> Harness<'a, FocusTestState> {
    Harness::builder()
        .with_size(egui::Vec2::new(600.0, 400.0))
        .build_ui_state(
            |ui, state: &mut FocusTestState| {
                Theme::slate().install(ui.ctx());
                ui.add(
                    TextInput::new(&mut state.email)
                        .label("Email")
                        .id_salt("focus_email"),
                );
                Drawer::new("focus_drawer", &mut state.open)
                    .title("Inspector")
                    .show(ui.ctx(), |ui| {
                        ui.label("Body");
                    });
            },
            FocusTestState {
                open: false,
                email: "hi".into(),
            },
        )
}

#[test]
fn drawer_focuses_close_button_on_open() {
    let mut harness = drawer_focus_harness();

    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Email")
        .focus();
    harness.run();

    harness.state_mut().open = true;
    // Two extra frames: one for the open transition + accesskit publish, one
    // more so the focus request settles into the new node.
    harness.run();
    harness.run();
    harness.run();

    let close = harness.get_by_role_and_label(egui::accesskit::Role::Button, "Close");
    assert!(
        close.accesskit_node().is_focused(),
        "close button should be focused on drawer open"
    );
}

#[test]
fn drawer_restores_focus_on_close() {
    let mut harness = drawer_focus_harness();

    harness.run();
    harness
        .get_by_role_and_label(egui::accesskit::Role::TextInput, "Email")
        .focus();
    harness.run();
    harness.state_mut().open = true;
    harness.run();
    harness.run();

    harness.state_mut().open = false;
    harness.run();
    harness.run();

    let email = harness.get_by_role_and_label(egui::accesskit::Role::TextInput, "Email");
    assert!(
        email.accesskit_node().is_focused(),
        "focus should be restored to the widget that had it before the drawer opened"
    );
}

#[test]
fn menu_bar_triggers_expose_button_role() {
    // Each MenuBar trigger should be reachable by its label so screen
    // readers can announce them and automation tools can drive the bar.
    let harness = new_harness(|ui| {
        Theme::slate().install(ui.ctx());
        MenuBar::new("a11y_menu_bar").brand("App").show(ui, |bar| {
            bar.menu("File", |_| {});
            bar.menu("Edit", |_| {});
            bar.menu("View", |_| {});
        });
    });

    let _ = harness.get_by_role_and_label(egui::accesskit::Role::Button, "File");
    let _ = harness.get_by_role_and_label(egui::accesskit::Role::Button, "Edit");
    let _ = harness.get_by_role_and_label(egui::accesskit::Role::Button, "View");
}

#[test]
fn menu_item_checked_reports_selected_state() {
    // Toggle items should announce as checkboxes (with selected=true/false)
    // rather than as plain buttons, so screen readers say "checked" / "not
    // checked" instead of just "button".
    let harness = new_harness(|ui| {
        Theme::slate().install(ui.ctx());
        ui.add(MenuItem::new("Show sidebar").checked(true));
        ui.add(MenuItem::new("Show minimap").checked(false));
    });

    let on = harness.get_by_role_and_label(egui::accesskit::Role::CheckBox, "Show sidebar");
    let off = harness.get_by_role_and_label(egui::accesskit::Role::CheckBox, "Show minimap");
    assert!(
        on.accesskit_node()
            .toggled()
            .map(|t| t == egui::accesskit::Toggled::True)
            .unwrap_or(false),
        "checked menu item should report toggled=True"
    );
    assert!(
        off.accesskit_node()
            .toggled()
            .map(|t| t == egui::accesskit::Toggled::False)
            .unwrap_or(false),
        "unchecked menu item should report toggled=False"
    );
}

#[test]
fn menu_item_radio_reports_selected_state() {
    // Radio items should announce as radio buttons. accesskit reports the
    // selected state via `Toggled::True`/`False`.
    let harness = new_harness(|ui| {
        Theme::slate().install(ui.ctx());
        ui.add(MenuItem::new("Compact").radio(false));
        ui.add(MenuItem::new("Comfortable").radio(true));
        ui.add(MenuItem::new("Spacious").radio(false));
    });

    let chosen = harness.get_by_role_and_label(egui::accesskit::Role::RadioButton, "Comfortable");
    assert!(
        chosen
            .accesskit_node()
            .toggled()
            .map(|t| t == egui::accesskit::Toggled::True)
            .unwrap_or(false),
        "selected radio menu item should report toggled=True"
    );
}
