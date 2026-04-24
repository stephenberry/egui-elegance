//! Pixel snapshot tests for every widget × every built-in theme.
//!
//! Baselines live in `tests/snapshots/{name}_{theme}.png`. On a mismatch the
//! harness writes `{name}_{theme}.new.png` and `{name}_{theme}.diff.png`
//! beside the baseline; inspect those to see what changed.
//!
//! Regenerate after an intentional visual change:
//!
//! ```sh
//! UPDATE_SNAPSHOTS=true cargo test --test visual
//! ```

use eframe::egui;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use elegance::{
    Accent, Badge, BadgeTone, Button, ButtonSize, Callout, CalloutTone, Card, Checkbox,
    CollapsingSection, Indicator, IndicatorState, LogBar, PairItem, Pairing, ProgressBar,
    SegmentedButton, Select, Slider, Spinner, StatusPill, Switch, TabBar, TextArea, TextInput,
    Theme,
};

fn snap(name: &str, theme: Theme, ui_fn: fn(&mut egui::Ui)) {
    snap_with_setup(name, theme, ui_fn, |_| {});
}

fn snap_with_setup(name: &str, theme: Theme, ui_fn: fn(&mut egui::Ui), setup: fn(&Harness<'_>)) {
    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(900.0, 600.0))
        .with_pixels_per_point(2.0)
        .wgpu()
        .build_ui(move |ui| {
            theme.clone().install(ui.ctx());
            let t = Theme::current(ui.ctx());
            egui::Frame::new()
                .fill(t.palette.bg)
                .inner_margin(egui::Margin::same(16))
                .show(ui, ui_fn);
        });
    harness.run();
    harness.fit_contents();
    harness.run();
    setup(&harness);
    harness.run();
    harness.snapshot(name);
}

macro_rules! theme_tests {
    ($name:ident, $ui_fn:expr) => {
        mod $name {
            use super::*;
            #[test]
            fn slate() {
                snap(
                    &format!("{}_slate", stringify!($name)),
                    Theme::slate(),
                    $ui_fn,
                );
            }
            #[test]
            fn charcoal() {
                snap(
                    &format!("{}_charcoal", stringify!($name)),
                    Theme::charcoal(),
                    $ui_fn,
                );
            }
            #[test]
            fn frost() {
                snap(
                    &format!("{}_frost", stringify!($name)),
                    Theme::frost(),
                    $ui_fn,
                );
            }
            #[test]
            fn paper() {
                snap(
                    &format!("{}_paper", stringify!($name)),
                    Theme::paper(),
                    $ui_fn,
                );
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Widget renderers. Each fn is stateless — any mutable state needed by a
// widget is declared locally so the render is deterministic across frames.
// ---------------------------------------------------------------------------

fn buttons_ui(ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        for (lbl, a) in [
            ("Blue", Accent::Blue),
            ("Green", Accent::Green),
            ("Red", Accent::Red),
            ("Purple", Accent::Purple),
            ("Amber", Accent::Amber),
            ("Sky", Accent::Sky),
        ] {
            ui.add(Button::new(lbl).accent(a));
        }
        ui.add(Button::new("Outline").outline());
    });
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        ui.add(
            Button::new("Small")
                .size(ButtonSize::Small)
                .accent(Accent::Blue),
        );
        ui.add(
            Button::new("Medium")
                .size(ButtonSize::Medium)
                .accent(Accent::Blue),
        );
        ui.add(
            Button::new("Large")
                .size(ButtonSize::Large)
                .accent(Accent::Blue),
        );
        ui.add(Button::new("Disabled").accent(Accent::Blue).enabled(false));
    });
}

fn text_inputs_ui(ui: &mut egui::Ui) {
    let mut normal = "steve@example.com".to_string();
    let mut hint = String::new();
    let mut dirty = "3000.0".to_string();
    let mut pw = "hunter2".to_string();
    ui.horizontal(|ui| {
        ui.add(
            TextInput::new(&mut normal)
                .label("Email")
                .desired_width(240.0)
                .id_salt("t_normal"),
        );
        ui.add_space(12.0);
        ui.add(
            TextInput::new(&mut hint)
                .label("With hint")
                .hint("e.g. sensor-01")
                .desired_width(240.0)
                .id_salt("t_hint"),
        );
    });
    ui.horizontal(|ui| {
        ui.add(
            TextInput::new(&mut dirty)
                .label("Dirty (unsaved)")
                .dirty(true)
                .desired_width(240.0)
                .id_salt("t_dirty"),
        );
        ui.add_space(12.0);
        ui.add(
            TextInput::new(&mut pw)
                .label("Password")
                .password(true)
                .desired_width(240.0)
                .id_salt("t_pw"),
        );
    });
}

fn text_areas_ui(ui: &mut egui::Ui) {
    let mut body = "A multi-line text area.\nDrop notes, logs, or JSON here.".to_string();
    let mut mono = "{\n  \"id\": 42,\n  \"ok\": true\n}".to_string();
    ui.horizontal(|ui| {
        ui.add(
            TextArea::new(&mut body)
                .label("Text area")
                .rows(3)
                .desired_width(240.0)
                .id_salt("a_body"),
        );
        ui.add_space(12.0);
        ui.add(
            TextArea::new(&mut mono)
                .label("Monospace")
                .monospace(true)
                .rows(3)
                .desired_width(240.0)
                .id_salt("a_mono"),
        );
    });
}

fn selects_ui(ui: &mut egui::Ui) {
    let mut unit = "ms".to_string();
    let mut env = "Production".to_string();
    ui.horizontal(|ui| {
        ui.add(
            Select::strings("s_unit", &mut unit, ["us", "ms", "s"])
                .label("Unit")
                .width(120.0),
        );
        ui.add_space(16.0);
        ui.add(
            Select::strings("s_env", &mut env, ["Production", "Staging", "Development"])
                .label("Environment")
                .width(180.0),
        );
    });
}

fn toggles_ui(ui: &mut egui::Ui) {
    let mut check_on = true;
    let mut check_off = false;
    let mut switch_on = true;
    let mut switch_off = false;
    let mut switch_green = true;
    let mut seg_on = true;
    let mut seg_off = false;

    let theme = Theme::current(ui.ctx());
    ui.label(theme.muted_text("Checkbox"));
    ui.horizontal(|ui| {
        ui.add(Checkbox::new(&mut check_on, "Enabled"));
        ui.add_space(16.0);
        ui.add(Checkbox::new(&mut check_off, "Off"));
    });
    ui.add_space(10.0);

    ui.label(theme.muted_text("Switch"));
    ui.horizontal(|ui| {
        ui.add(Switch::new(&mut switch_on, "On"));
        ui.add_space(16.0);
        ui.add(Switch::new(&mut switch_off, "Off"));
        ui.add_space(16.0);
        ui.add(Switch::new(&mut switch_green, "Green accent").accent(Accent::Green));
    });
    ui.add_space(10.0);

    ui.label(theme.muted_text("Segmented"));
    ui.horizontal(|ui| {
        ui.add(
            SegmentedButton::new(&mut seg_on, "Record")
                .accent(Accent::Green)
                .min_width(140.0),
        );
        ui.add_space(8.0);
        ui.add(
            SegmentedButton::new(&mut seg_off, "Preview")
                .accent(Accent::Blue)
                .min_width(140.0),
        );
    });
}

fn tabs_ui(ui: &mut egui::Ui) {
    let mut tab = 1usize;
    ui.set_min_width(520.0);
    ui.add(TabBar::new(
        &mut tab,
        ["Overview", "Settings", "Activity", "Logs"],
    ));
}

fn status_ui(ui: &mut egui::Ui) {
    let theme = Theme::current(ui.ctx());

    ui.label(theme.muted_text("StatusPill"));
    ui.add(
        StatusPill::new()
            .item("UI", IndicatorState::On)
            .item("API", IndicatorState::Connecting)
            .item("DB", IndicatorState::Off),
    );
    ui.add_space(10.0);

    ui.label(theme.muted_text("Indicators"));
    ui.horizontal(|ui| {
        for (s, text) in [
            (IndicatorState::On, "On"),
            (IndicatorState::Connecting, "Connecting"),
            (IndicatorState::Off, "Off"),
        ] {
            ui.add(Indicator::new(s));
            ui.label(theme.faint_text(text));
            ui.add_space(12.0);
        }
    });
    ui.add_space(10.0);

    ui.label(theme.muted_text("Badges"));
    ui.horizontal_wrapped(|ui| {
        ui.add(Badge::new("OK", BadgeTone::Ok));
        ui.add(Badge::new("Warning", BadgeTone::Warning));
        ui.add(Badge::new("Error", BadgeTone::Danger));
        ui.add(Badge::new("Info", BadgeTone::Info));
        ui.add(Badge::new("Neutral", BadgeTone::Neutral));
    });
}

fn sliders_ui(ui: &mut egui::Ui) {
    let mut threshold: u32 = 48;
    let mut gain: f32 = 0.62;
    ui.set_max_width(420.0);
    ui.add(
        Slider::new(&mut threshold, 0u32..=100u32)
            .label("Threshold")
            .suffix("%"),
    );
    ui.add_space(6.0);
    ui.add(
        Slider::new(&mut gain, 0.0..=1.0)
            .label("Gain")
            .accent(Accent::Green),
    );
}

fn feedback_ui(ui: &mut egui::Ui) {
    let theme = Theme::current(ui.ctx());
    ui.set_max_width(500.0);

    ui.label(theme.muted_text("Spinner"));
    ui.horizontal(|ui| {
        ui.add(Spinner::new().size(14.0));
        ui.add_space(10.0);
        ui.add(Spinner::new());
        ui.add_space(10.0);
        ui.add(Spinner::new().size(28.0));
        ui.add_space(20.0);
        ui.add(Spinner::new().accent(Accent::Green));
        ui.add(Spinner::new().accent(Accent::Amber));
        ui.add(Spinner::new().accent(Accent::Red));
        ui.add(Spinner::new().accent(Accent::Purple));
    });
    ui.add_space(10.0);

    ui.label(theme.muted_text("ProgressBar"));
    ui.add(ProgressBar::new(0.25));
    ui.add_space(4.0);
    ui.add(ProgressBar::new(0.6).accent(Accent::Green));
    ui.add_space(4.0);
    ui.add(ProgressBar::new(1.0).accent(Accent::Amber).text("Complete"));
}

fn containers_ui(ui: &mut egui::Ui) {
    let mut open = true;
    let theme = Theme::current(ui.ctx());
    ui.set_min_width(440.0);

    ui.label(theme.muted_text("Card"));
    Card::new().heading("Account").show(ui, |ui| {
        ui.label(theme.body_text("Primary email: steve@example.com"));
        ui.label(theme.muted_text("Two-factor authentication enabled."));
    });
    ui.add_space(8.0);

    ui.label(theme.muted_text("CollapsingSection"));
    CollapsingSection::new("advanced", "Advanced")
        .open(&mut open)
        .show(ui, |ui| {
            ui.label(theme.body_text("Hidden until expanded. Nest anything here."));
        });
}

fn log_bar_ui(ui: &mut egui::Ui) {
    let mut log = LogBar::new();
    log.sys("Ready");
    log.out("probe_status");
    log.recv("{\"temp\":42.1,\"ok\":true}");
    log.err("retry budget exceeded");
    ui.set_min_width(700.0);
    log.show(ui);
}

fn callouts_ui(ui: &mut egui::Ui) {
    let mut dismiss_open = true;
    ui.set_min_width(640.0);

    Callout::new(CalloutTone::Info)
        .title("Node editing is in preview.")
        .body("The wire format may change before 1.0.")
        .show(ui, |_| {});
    ui.add_space(6.0);

    Callout::new(CalloutTone::Warning)
        .title("Unsaved changes.")
        .body("You have 3 edits that haven't been written to disk.")
        .show(ui, |ui| {
            ui.add(Button::new("Save now").accent(Accent::Amber));
            ui.add(Button::new("Discard").outline());
        });
    ui.add_space(6.0);

    Callout::new(CalloutTone::Danger)
        .title("Build failed.")
        .body("cargo returned 2 errors in src/node_editor.rs.")
        .dismissable(&mut dismiss_open)
        .show(ui, |_| {});
    ui.add_space(6.0);

    Callout::new(CalloutTone::Success)
        .title("Deploy complete.")
        .body("Rolled out to us-east-1.")
        .show(ui, |_| {});
    ui.add_space(6.0);

    Callout::new(CalloutTone::Neutral)
        .title("Read-only mode.")
        .body("Database upgrade in progress.")
        .show(ui, |_| {});
}

fn pairing_ui(ui: &mut egui::Ui) {
    let clients = vec![
        PairItem::new("c1", "worker-pool-a")
            .detail("24 instances")
            .icon("▸"),
        PairItem::new("c2", "edge-proxy-01")
            .detail("8 instances")
            .icon("▸"),
        PairItem::new("c3", "cache-layer")
            .detail("4 instances")
            .icon("▸"),
        PairItem::new("c4", "batch-workers")
            .detail("12 instances")
            .icon("▸"),
    ];
    let servers = vec![
        PairItem::new("s1", "api-east-01")
            .detail("10.0.1.5 · us-east")
            .icon("◂"),
        PairItem::new("s2", "api-east-02")
            .detail("10.0.1.6 · us-east")
            .icon("◂"),
        PairItem::new("s3", "api-west-01")
            .detail("10.0.2.4 · us-west")
            .icon("◂"),
        PairItem::new("s4", "api-eu-01")
            .detail("10.0.3.2 · eu-west")
            .icon("◂"),
    ];
    let mut pairs = vec![
        ("c1".to_string(), "s3".to_string()),
        ("c2".to_string(), "s1".to_string()),
        ("c4".to_string(), "s2".to_string()),
    ];
    ui.set_max_width(680.0);
    Pairing::new("vis_pairing", &clients, &servers, &mut pairs)
        .left_label("Clients")
        .right_label("Servers")
        .show(ui);
}

theme_tests!(buttons, buttons_ui);
theme_tests!(text_inputs, text_inputs_ui);
theme_tests!(text_areas, text_areas_ui);
theme_tests!(selects, selects_ui);
theme_tests!(toggles, toggles_ui);
theme_tests!(tabs, tabs_ui);
theme_tests!(status, status_ui);
theme_tests!(sliders, sliders_ui);
theme_tests!(feedback, feedback_ui);
theme_tests!(containers, containers_ui);
theme_tests!(callouts, callouts_ui);
theme_tests!(log_bar, log_bar_ui);
theme_tests!(pairing, pairing_ui);

// ---------------------------------------------------------------------------
// Interaction-state tests. Each renders a single widget, injects a mouse /
// keyboard event after the initial layout, and snapshots the resulting
// hover / focus / pressed visual.
// ---------------------------------------------------------------------------

fn single_button_ui(ui: &mut egui::Ui) {
    ui.add(Button::new("Deploy").accent(Accent::Green));
}

fn single_switch_off_ui(ui: &mut egui::Ui) {
    let mut off = false;
    ui.add(Switch::new(&mut off, "Notify"));
}

fn single_text_input_ui(ui: &mut egui::Ui) {
    let mut email = "steve@example.com".to_string();
    ui.add(
        TextInput::new(&mut email)
            .label("Email")
            .desired_width(240.0)
            .id_salt("focus_email"),
    );
}

fn dirty_text_input_ui(ui: &mut egui::Ui) {
    let mut val = "3000.0".to_string();
    ui.add(
        TextInput::new(&mut val)
            .label("Dirty")
            .dirty(true)
            .desired_width(240.0)
            .id_salt("dirty_focus"),
    );
}

fn hover_deploy(h: &Harness) {
    h.get_by_label("Deploy").hover();
}

fn focus_deploy(h: &Harness) {
    h.get_by_label("Deploy").focus();
}

fn hover_notify(h: &Harness) {
    h.get_by_label("Notify").hover();
}

fn focus_email(h: &Harness) {
    h.get_by_role_and_label(egui::accesskit::Role::TextInput, "Email")
        .focus();
}

fn focus_dirty(h: &Harness) {
    h.get_by_role_and_label(egui::accesskit::Role::TextInput, "Dirty")
        .focus();
}

macro_rules! interact_tests {
    ($name:ident, $ui_fn:expr, $setup:expr) => {
        mod $name {
            use super::*;
            #[test]
            fn slate() {
                snap_with_setup(
                    &format!("{}_slate", stringify!($name)),
                    Theme::slate(),
                    $ui_fn,
                    $setup,
                );
            }
            #[test]
            fn charcoal() {
                snap_with_setup(
                    &format!("{}_charcoal", stringify!($name)),
                    Theme::charcoal(),
                    $ui_fn,
                    $setup,
                );
            }
            #[test]
            fn frost() {
                snap_with_setup(
                    &format!("{}_frost", stringify!($name)),
                    Theme::frost(),
                    $ui_fn,
                    $setup,
                );
            }
            #[test]
            fn paper() {
                snap_with_setup(
                    &format!("{}_paper", stringify!($name)),
                    Theme::paper(),
                    $ui_fn,
                    $setup,
                );
            }
        }
    };
}

interact_tests!(button_hovered, single_button_ui, hover_deploy);
interact_tests!(button_focused, single_button_ui, focus_deploy);
interact_tests!(switch_hovered_off, single_switch_off_ui, hover_notify);
interact_tests!(text_input_focused, single_text_input_ui, focus_email);
interact_tests!(text_input_dirty_focused, dirty_text_input_ui, focus_dirty);
