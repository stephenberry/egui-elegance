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
    Accent, Accordion, Badge, BadgeTone, BrowserTab, BrowserTabs, Button, ButtonSize, Callout,
    CalloutTone, Card, Checkbox, CollapsingSection, ColorPicker, FileDropZone, Indicator,
    IndicatorState, Knob, KnobSize, LogBar, MenuBar, MenuItem, PairItem, Pairing, Popover,
    PopoverSide, ProgressBar, ProgressRing, RangeSlider, SegmentedButton, Select, Slider, Spinner,
    StatusPill, Steps, StepsStyle, Switch, TabBar, TextArea, TextInput, Theme, Tooltip,
    TooltipSide,
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
    ui.add_space(10.0);

    // Mixed-control row: Button + SegmentedButton share theme.control_padding_y
    // so they align cleanly at any shared ButtonSize.
    ui.label(theme.muted_text("Mixed action row (Medium, Large)"));
    ui.horizontal(|ui| {
        ui.add(Button::new("Collect").accent(Accent::Green).min_width(96.0));
        ui.add_space(8.0);
        ui.add(
            SegmentedButton::new(&mut seg_on, "Continuous")
                .accent(Accent::Green)
                .min_width(140.0),
        );
    });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add(
            Button::new("Collect")
                .accent(Accent::Green)
                .size(ButtonSize::Large)
                .min_width(120.0),
        );
        ui.add_space(8.0);
        ui.add(
            SegmentedButton::new(&mut seg_on, "Continuous")
                .accent(Accent::Green)
                .size(ButtonSize::Large)
                .min_width(180.0),
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

fn browser_tabs_ui(ui: &mut egui::Ui) {
    let mut tabs = BrowserTabs::new("vis_browser_tabs")
        .with_tab(BrowserTab::new("readme", "README.md"))
        .with_tab(BrowserTab::new("theme", "theme.rs").dirty(true))
        .with_tab(BrowserTab::new("button", "widgets/button.rs"))
        .with_tab(BrowserTab::new(
            "cargo",
            "cargo output \u{2014} a longer title that truncates",
        ));
    tabs.set_selected("theme");
    ui.set_min_width(720.0);
    tabs.show(ui);
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

fn knobs_ui(ui: &mut egui::Ui) {
    let theme = Theme::current(ui.ctx());
    ui.set_max_width(560.0);

    // Row of small instrument-panel knobs.
    ui.label(theme.muted_text("Instrument panel"));
    ui.horizontal_wrapped(|ui| {
        let mut gain = -12.0_f32;
        let mut cutoff = 1000.0_f32;
        let mut q = 2.4_f32;
        let mut mix = 35_u32;
        ui.spacing_mut().item_spacing.x = 14.0;
        ui.add(
            Knob::new(&mut gain, -60.0..=12.0)
                .label("Gain")
                .size(KnobSize::Small)
                .default(0.0_f32)
                .show_value(true)
                .value_fmt(|v| format!("{v:.0} dB")),
        );
        ui.add(
            Knob::new(&mut cutoff, 20.0..=20000.0)
                .label("Cutoff")
                .size(KnobSize::Small)
                .log_scale()
                .default(1000.0_f32)
                .show_value(true)
                .value_fmt(|v| {
                    if v >= 1000.0 {
                        format!("{:.1} kHz", v / 1000.0)
                    } else {
                        format!("{v:.0} Hz")
                    }
                }),
        );
        ui.add(
            Knob::new(&mut q, 0.1..=10.0)
                .label("Q")
                .size(KnobSize::Small)
                .log_scale()
                .default(0.707_f32)
                .show_value(true),
        );
        ui.add(
            Knob::new(&mut mix, 0u32..=100u32)
                .label("Mix")
                .size(KnobSize::Small)
                .default(50_u32)
                .show_value(true)
                .value_fmt(|v| format!("{v:.0}%"))
                .accent(Accent::Green),
        );
    });
    ui.add_space(14.0);

    // Stepped knob with labeled detents.
    ui.label(theme.muted_text("Stepped Timebase"));
    let mut idx: u32 = 3;
    ui.add(
        Knob::new(&mut idx, 0u32..=8u32)
            .size(KnobSize::Large)
            .step(1.0)
            .detents([
                (0u32, "1µ"),
                (1u32, "2µ"),
                (2u32, "5µ"),
                (3u32, "10µ"),
                (4u32, "20µ"),
                (5u32, "50µ"),
                (6u32, "100µ"),
                (7u32, "200µ"),
                (8u32, "500µ"),
            ])
            .default(3_u32),
    );
    ui.add_space(14.0);

    // Bipolar knob.
    ui.label(theme.muted_text("Bipolar DC offset"));
    let mut dc = -1.4_f32;
    ui.add(
        Knob::new(&mut dc, -5.0..=5.0)
            .label("DC offset")
            .bipolar()
            .accent(Accent::Purple)
            .default(0.0_f32)
            .show_value(true)
            .value_fmt(|v| format!("{:+.2} V", v)),
    );
}

fn range_sliders_ui(ui: &mut egui::Ui) {
    let (mut price_lo, mut price_hi): (u32, u32) = (24, 118);
    let (mut latency_lo, mut latency_hi): (u32, u32) = (120, 340);
    let (mut volume_lo, mut volume_hi): (u32, u32) = (18, 62);
    let (mut retention_lo, mut retention_hi): (u32, u32) = (7, 30);
    let w = 520.0_f32;
    Card::new().show(ui, |ui| {
        ui.set_max_width(w);
        ui.add(
            RangeSlider::new(&mut price_lo, &mut price_hi, 0u32..=200u32)
                .label("Price")
                .value_fmt(|v| format!("${v:.0}"))
                .desired_width(w)
                .id_salt("rs_price"),
        );
        ui.add_space(8.0);
        ui.add(
            RangeSlider::new(&mut latency_lo, &mut latency_hi, 0u32..=500u32)
                .label("Latency target")
                .suffix(" ms")
                .step(10.0)
                .ticks(6)
                .show_tick_labels(true)
                .desired_width(w)
                .id_salt("rs_latency"),
        );
        ui.add_space(8.0);
        ui.add(
            RangeSlider::new(&mut volume_lo, &mut volume_hi, 0u32..=100u32)
                .label("Volume")
                .suffix(" dB")
                .accent(Accent::Green)
                .desired_width(w)
                .id_salt("rs_volume"),
        );
        ui.add_space(8.0);
        ui.add(
            RangeSlider::new(&mut retention_lo, &mut retention_hi, 1u32..=90u32)
                .label("Retention window (locked)")
                .suffix(" days")
                .enabled(false)
                .desired_width(w)
                .id_salt("rs_retention"),
        );
    });
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

fn progress_rings_ui(ui: &mut egui::Ui) {
    let theme = Theme::current(ui.ctx());
    ui.set_max_width(560.0);

    ui.label(theme.muted_text("Progression"));
    ui.horizontal(|ui| {
        for f in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            ui.add(ProgressRing::new(f));
            ui.add_space(8.0);
        }
    });
    ui.add_space(14.0);

    ui.label(theme.muted_text("Sizes"));
    ui.horizontal(|ui| {
        ui.add(ProgressRing::new(0.62).size(36.0));
        ui.add_space(12.0);
        ui.add(ProgressRing::new(0.62).size(56.0));
        ui.add_space(12.0);
        ui.add(ProgressRing::new(0.62).size(88.0));
    });
    ui.add_space(14.0);

    ui.label(theme.muted_text("Accents"));
    ui.horizontal(|ui| {
        for a in [
            Accent::Sky,
            Accent::Green,
            Accent::Amber,
            Accent::Red,
            Accent::Purple,
        ] {
            ui.add(ProgressRing::new(0.62).accent(a));
            ui.add_space(8.0);
        }
    });
    ui.add_space(14.0);

    ui.label(theme.muted_text("Custom centre"));
    ui.horizontal(|ui| {
        ui.add(
            ProgressRing::new(0.6)
                .size(88.0)
                .text("12 / 20")
                .caption("files"),
        );
        ui.add_space(16.0);
        ui.add(
            ProgressRing::new(0.83)
                .size(88.0)
                .accent(Accent::Amber)
                .text("3.4s")
                .caption("remaining"),
        );
        ui.add_space(16.0);
        ui.add(ProgressRing::new(1.0).size(72.0).accent(Accent::Green));
    });
}

fn steps_ui(ui: &mut egui::Ui) {
    let theme = Theme::current(ui.ctx());
    ui.set_max_width(520.0);

    ui.label(theme.muted_text("Cells — in progress"));
    ui.add(Steps::new(6).current(4));
    ui.add_space(10.0);

    ui.label(theme.muted_text("Cells — errored"));
    ui.add(Steps::new(5).current(2).errored(true));
    ui.add_space(10.0);

    ui.label(theme.muted_text("Cells — complete"));
    ui.add(Steps::new(4).current(4));
    ui.add_space(14.0);

    ui.label(theme.muted_text("Numbered — in progress"));
    ui.add(Steps::new(5).current(2).style(StepsStyle::Numbered));
    ui.add_space(10.0);

    ui.label(theme.muted_text("Numbered — complete"));
    ui.add(Steps::new(4).current(4).style(StepsStyle::Numbered));
    ui.add_space(10.0);

    ui.label(theme.muted_text("Numbered — errored"));
    ui.add(
        Steps::new(5)
            .current(2)
            .errored(true)
            .style(StepsStyle::Numbered),
    );
    ui.add_space(14.0);

    ui.label(theme.muted_text("Numbered — labeled with active sublabel"));
    ui.add(
        Steps::labeled(["Account", "Workspace", "Billing", "Integrations", "Review"])
            .style(StepsStyle::Numbered)
            .current(2)
            .active_sublabel("In progress"),
    );
    ui.add_space(10.0);

    ui.label(theme.muted_text("Numbered — labeled, no sublabel"));
    ui.add(
        Steps::labeled(["Details", "Payment", "Confirm"])
            .style(StepsStyle::Numbered)
            .current(0),
    );
    ui.add_space(14.0);

    ui.label(theme.muted_text("Labeled — horizontal"));
    ui.add(Steps::labeled(["Plan", "Build", "Test", "Deploy"]).current(2));
    ui.add_space(6.0);
    ui.add(
        Steps::labeled(["Schema", "Backfill", "Reindex", "Finalize"])
            .current(2)
            .errored(true),
    );
    ui.add_space(14.0);

    ui.label(theme.muted_text("Labeled — vertical"));
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_max_width(200.0);
            ui.add(
                Steps::labeled(["Plan", "Design", "Build", "Test", "Deploy"])
                    .current(2)
                    .vertical(),
            );
        });
        ui.add_space(16.0);
        ui.vertical(|ui| {
            ui.set_max_width(220.0);
            ui.add(
                Steps::labeled(["Schema validated", "Backfill", "Index rebuild", "Finalize"])
                    .current(2)
                    .errored(true)
                    .vertical(),
            );
        });
    });
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

fn accordion_ui(ui: &mut egui::Ui) {
    let theme = Theme::current(ui.ctx());
    ui.set_min_width(620.0);

    ui.label(theme.muted_text("FAQ — bordered, default"));
    Accordion::new("vis_acc_faq").show(ui, |acc| {
        acc.item("How do I invite teammates to my workspace?")
            .default_open(true)
            .show(|ui| {
                ui.label(theme.muted_text(
                    "Open Settings ▸ Members and click Invite. Paste a list of emails or share a role-scoped signup link.",
                ));
            });
        acc.item("What happens when I archive a project?")
            .show(|_| {});
        acc.item("Is there an API for bulk imports?").show(|_| {});
    });
    ui.add_space(14.0);

    ui.label(theme.muted_text("Settings — exclusive, with icon and meta"));
    Accordion::new("vis_acc_settings")
        .exclusive(true)
        .show(ui, |acc| {
            acc.item("Notifications")
                .icon("\u{1F514}")
                .accent(Accent::Sky)
                .subtitle("Email, Slack, and in-app alerts")
                .meta(|ui| {
                    ui.add(Badge::new("3 channels", BadgeTone::Ok));
                })
                .default_open(true)
                .show(|ui| {
                    ui.label(
                        theme
                            .muted_text("Three channels enabled. Tap Manage channels for details."),
                    );
                });
            acc.item("Security")
                .icon("\u{1F512}")
                .accent(Accent::Green)
                .subtitle("2FA, sessions, and trusted devices")
                .meta(|ui| {
                    ui.add(Badge::new("Strong", BadgeTone::Ok));
                })
                .show(|_| {});
            acc.item("Integrations")
                .icon("\u{2731}")
                .accent(Accent::Amber)
                .subtitle("GitHub, Linear, PagerDuty, and 2 more")
                .meta(|ui| {
                    ui.add(Badge::new("1 needs auth", BadgeTone::Warning));
                })
                .show(|_| {});
            acc.item("Billing (owner-only)")
                .icon("\u{1F3E0}")
                .subtitle("Invoices, plan, and seats")
                .meta(|ui| {
                    ui.label(theme.faint_text("Admin required"));
                })
                .disabled(true)
                .show(|_| {});
        });
    ui.add_space(14.0);

    ui.label(theme.muted_text("Flush — inline, no outer card"));
    Accordion::new("vis_acc_flush").flush(true).show(ui, |acc| {
        acc.item("Advanced options")
            .subtitle("(rarely needed)")
            .default_open(true)
            .show(|ui| {
                ui.label(
                    theme.muted_text("Override the default request timeout and retry behavior."),
                );
            });
        acc.item("Experimental features").show(|_| {});
        acc.item("Danger zone").show(|_| {});
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

fn file_drop_zone_ui(ui: &mut egui::Ui) {
    ui.set_max_width(560.0);
    let _ = FileDropZone::new()
        .hint("up to 10 MB \u{00b7} PNG, JPG, CSV, PDF")
        .show(ui);
    ui.add_space(10.0);
    let _ = FileDropZone::new()
        .prompt("Drop a CSV to import")
        .min_height(96.0)
        .enabled(false)
        .show(ui);
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

fn color_picker_triggers_ui(ui: &mut egui::Ui) {
    let mut a = egui::Color32::from_rgb(0x38, 0xbd, 0xf8);
    let mut b = egui::Color32::from_rgb(0x4a, 0xde, 0x80);
    let mut c = egui::Color32::from_rgba_unmultiplied(0xc0, 0x84, 0xfc, 0xa6);
    ui.set_min_width(360.0);
    ui.horizontal(|ui| {
        ui.add(ColorPicker::new("cp_a", &mut a).label("Brand"));
        ui.add_space(12.0);
        ui.add(ColorPicker::new("cp_b", &mut b).label("Success"));
        ui.add_space(12.0);
        ui.add(
            ColorPicker::new("cp_c", &mut c)
                .label("Overlay")
                .alpha(true),
        );
    });
}

fn color_picker_palette_open_ui(ui: &mut egui::Ui) {
    let id = "cp_open_palette";
    egui::Popup::open_id(
        &ui.ctx().clone(),
        elegance::Popover::popup_id(("elegance::color_picker", egui::Id::new(id))),
    );
    let mut color = egui::Color32::from_rgb(0x38, 0xbd, 0xf8);
    ui.set_min_size(egui::vec2(360.0, 540.0));
    ui.add_space(8.0);
    ui.add(
        ColorPicker::new(id, &mut color)
            .palette(ColorPicker::default_palette())
            .continuous(false)
            .recents(false),
    );
}

fn color_picker_continuous_open_ui(ui: &mut egui::Ui) {
    let id = "cp_open_continuous";
    egui::Popup::open_id(
        &ui.ctx().clone(),
        elegance::Popover::popup_id(("elegance::color_picker", egui::Id::new(id))),
    );
    let mut color = egui::Color32::from_rgba_unmultiplied(0x38, 0xbd, 0xf8, 0xd9);
    ui.set_min_size(egui::vec2(360.0, 380.0));
    ui.add_space(8.0);
    ui.add(ColorPicker::new(id, &mut color).recents(false));
}

// egui only allows one popup open at a time per viewport, so each side
// gets its own snapshot test.

fn menu_bar_ui(ui: &mut egui::Ui) {
    // Closed-bar snapshot — the strip itself is what we care about pinning.
    // Live dropdowns paint into a top-level Area, which the kittest harness
    // doesn't compose into the same surface, so we don't open one here.
    let theme = Theme::current(ui.ctx());
    MenuBar::new("snap_menu_bar")
        .brand("Elegance")
        .status_with_dot("main \u{00b7} up to date", theme.palette.green)
        .show(ui, |bar| {
            bar.menu("File", |ui| {
                ui.add(MenuItem::new("New").shortcut("\u{2318}N"));
            });
            bar.menu("Edit", |ui| {
                ui.add(MenuItem::new("Undo").shortcut("\u{2318}Z"));
            });
            bar.menu("View", |_| {});
            bar.menu("Window", |_| {});
            bar.menu("Help", |_| {});
        });
}

fn popover_bottom_ui(ui: &mut egui::Ui) {
    let theme = Theme::current(ui.ctx());
    egui::Popup::open_id(&ui.ctx().clone(), Popover::popup_id("pop_bottom"));
    ui.set_min_size(egui::vec2(320.0, 260.0));
    ui.add_space(40.0);
    ui.horizontal(|ui| {
        ui.add_space(60.0);
        let bottom = ui.add(Button::new("Delete branch").outline());
        Popover::new("pop_bottom")
            .side(PopoverSide::Bottom)
            .title("Delete feature/snap-baseline?")
            .show(&bottom, |ui| {
                ui.add(egui::Label::new(
                    theme.muted_text("This removes the branch from origin too."),
                ));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let _ = ui.add(Button::new("Cancel").outline().size(ButtonSize::Small));
                    let _ = ui.add(
                        Button::new("Delete")
                            .accent(Accent::Red)
                            .size(ButtonSize::Small),
                    );
                });
            });
    });
}

fn popover_top_ui(ui: &mut egui::Ui) {
    let theme = Theme::current(ui.ctx());
    egui::Popup::open_id(&ui.ctx().clone(), Popover::popup_id("pop_top"));
    ui.set_min_size(egui::vec2(260.0, 180.0));
    ui.add_space(100.0);
    ui.horizontal(|ui| {
        ui.add_space(40.0);
        let top = ui.add(Button::new("What's a baseline?").outline());
        Popover::new("pop_top")
            .side(PopoverSide::Top)
            .title("Baselines")
            .width(260.0)
            .show(&top, |ui| {
                ui.add(egui::Label::new(theme.muted_text(
                    "Accepted reference image a widget is compared against.",
                )));
            });
    });
}

fn popover_left_ui(ui: &mut egui::Ui) {
    let theme = Theme::current(ui.ctx());
    egui::Popup::open_id(&ui.ctx().clone(), Popover::popup_id("pop_left"));
    ui.set_min_size(egui::vec2(420.0, 120.0));
    ui.add_space(40.0);
    ui.horizontal(|ui| {
        ui.add_space(260.0);
        let left = ui.add(Button::new("Details").outline());
        Popover::new("pop_left")
            .side(PopoverSide::Left)
            .show(&left, |ui| {
                ui.add(egui::Label::new(theme.muted_text("Opens to the left.")));
            });
    });
}

fn popover_right_ui(ui: &mut egui::Ui) {
    let theme = Theme::current(ui.ctx());
    egui::Popup::open_id(&ui.ctx().clone(), Popover::popup_id("pop_right"));
    ui.set_min_size(egui::vec2(420.0, 120.0));
    ui.add_space(40.0);
    ui.horizontal(|ui| {
        ui.add_space(40.0);
        let right = ui.add(Button::new("Details").outline());
        Popover::new("pop_right")
            .side(PopoverSide::Right)
            .show(&right, |ui| {
                ui.add(egui::Label::new(theme.muted_text("Opens to the right.")));
            });
    });
}

// Tooltips piggy-back on egui's hover-driven tooltip system; force them
// open via memory().set_everything_is_visible(true) so the snapshot
// captures the rendered card without needing a pointer-hover fixture.
fn tooltip_label_ui(ui: &mut egui::Ui) {
    ui.ctx().memory_mut(|m| m.set_everything_is_visible(true));
    ui.set_min_size(egui::vec2(280.0, 120.0));
    ui.add_space(80.0);
    ui.horizontal(|ui| {
        ui.add_space(80.0);
        let trigger = ui.add(Button::new("Share").outline().size(ButtonSize::Small));
        Tooltip::new("Copy share link").show(&trigger);
    });
}

fn tooltip_rich_ui(ui: &mut egui::Ui) {
    ui.ctx().memory_mut(|m| m.set_everything_is_visible(true));
    ui.set_min_size(egui::vec2(360.0, 180.0));
    ui.add_space(120.0);
    ui.horizontal(|ui| {
        ui.add_space(80.0);
        let trigger = ui.add(Button::new("Save").outline());
        Tooltip::new("Write the working tree to disk. Remote sync runs in the background.")
            .heading("Save changes")
            .shortcut("\u{2318} S")
            .show(&trigger);
    });
}

fn tooltip_below_ui(ui: &mut egui::Ui) {
    ui.ctx().memory_mut(|m| m.set_everything_is_visible(true));
    ui.set_min_size(egui::vec2(360.0, 180.0));
    ui.add_space(20.0);
    ui.horizontal(|ui| {
        ui.add_space(80.0);
        let trigger = ui.add(
            Button::new("degraded")
                .accent(Accent::Amber)
                .size(ButtonSize::Small),
        );
        Tooltip::new("api-west-01 is returning elevated 5xx. Other regions healthy.")
            .heading("Partial outage")
            .side(TooltipSide::Bottom)
            .show(&trigger);
    });
}

theme_tests!(buttons, buttons_ui);
theme_tests!(text_inputs, text_inputs_ui);
theme_tests!(text_areas, text_areas_ui);
theme_tests!(selects, selects_ui);
theme_tests!(toggles, toggles_ui);
theme_tests!(tabs, tabs_ui);
theme_tests!(browser_tabs, browser_tabs_ui);
theme_tests!(status, status_ui);
theme_tests!(sliders, sliders_ui);
theme_tests!(range_sliders, range_sliders_ui);
theme_tests!(knobs, knobs_ui);
theme_tests!(feedback, feedback_ui);
theme_tests!(progress_rings, progress_rings_ui);
theme_tests!(steps, steps_ui);
theme_tests!(containers, containers_ui);
theme_tests!(accordion, accordion_ui);
theme_tests!(callouts, callouts_ui);
theme_tests!(file_drop_zone, file_drop_zone_ui);
theme_tests!(log_bar, log_bar_ui);
theme_tests!(pairing, pairing_ui);
theme_tests!(popover_bottom, popover_bottom_ui);
theme_tests!(popover_top, popover_top_ui);
theme_tests!(popover_left, popover_left_ui);
theme_tests!(popover_right, popover_right_ui);
theme_tests!(menu_bar, menu_bar_ui);
theme_tests!(tooltip_label, tooltip_label_ui);
theme_tests!(tooltip_rich, tooltip_rich_ui);
theme_tests!(tooltip_below, tooltip_below_ui);
theme_tests!(color_picker_triggers, color_picker_triggers_ui);
theme_tests!(color_picker_palette_open, color_picker_palette_open_ui);
theme_tests!(
    color_picker_continuous_open,
    color_picker_continuous_open_ui
);

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
