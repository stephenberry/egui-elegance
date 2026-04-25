//! Widget reference — every elegance widget laid out in labeled tiles,
//! intended for screenshotting for the README and crates.io listing.
//!
//! Run with `cargo widgets`.

#![allow(clippy::collapsible_if)]

use eframe::egui;
use elegance::{
    Accent, Badge, BadgeTone, BuiltInTheme, Button, ButtonSize, Callout, CalloutTone, Card,
    Checkbox, CollapsingSection, Indicator, IndicatorState, LogBar, Menu, MenuItem, Modal,
    MultiTerminal, PairItem, Pairing, Popover, PopoverSide, ProgressBar, ProgressRing,
    SegmentedButton, Select, Slider, Spinner, StatusPill, Steps, StepsStyle, Switch, TabBar,
    TerminalEvent, TerminalLine, TerminalPane, TerminalStatus, TextArea, TextInput, Theme,
    ThemeSwitcher, Toast, Toasts,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Elegance — widget reference")
            .with_inner_size([980.0, 1400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Elegance widgets",
        options,
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
}

#[derive(Debug)]
struct App {
    theme: BuiltInTheme,

    text_normal: String,
    text_hint: String,
    text_dirty: String,
    text_pw: String,
    area_body: String,
    area_mono: String,
    select_unit: String,
    select_env: String,

    check_on: bool,
    check_off: bool,
    switch_on: bool,
    switch_off: bool,
    switch_green: bool,
    seg_on: bool,
    seg_off: bool,
    seg_join_a: bool,
    seg_join_b: bool,
    seg_join_c: bool,
    seg_glow: bool,
    seg_dim: bool,

    category: usize,
    tab_idx: usize,
    collapsing_open: bool,

    slider_int: u32,
    slider_float: f32,

    show_modal: bool,

    pop_filter_open: bool,
    pop_filter_in_review: bool,
    pop_filter_merged: bool,
    pop_filter_closed: bool,
    pop_filter_needs_review: bool,
    pop_filter_ready: bool,
    pop_filter_blocked: bool,

    callout_danger_open: bool,

    pairing_clients: Vec<PairItem>,
    pairing_servers: Vec<PairItem>,
    pairing_pairs: Vec<(String, String)>,
    pairing_align: bool,

    multi_term: MultiTerminal,
    term_pane_count: usize,

    log: LogBar,
}

impl Default for App {
    fn default() -> Self {
        let mut log = LogBar::new();
        log.sys("Ready");
        log.out("probe_status");
        log.recv("{\"temp\":42.1,\"ok\":true}");
        log.err("retry budget exceeded");
        // Long entries to stress-test wrapping / horizontal overflow.
        log.sys(
            "Reconnecting to broker mqtts://gateway.iot.example.com:8883 \
             over TLS 1.3 — credentials accepted, resubscribing to 14 topics, \
             replaying 3 retained messages from the offline queue.",
        );
        log.out(
            "POST https://api.example.com/v2/devices/sensor-7f3a2c1e/telemetry \
             ?since=2026-04-23T18%3A04%3A12Z&fields=pressure,humidity,temp_c,vbat",
        );
        log.recv(
            "{\"id\":\"sensor-7f3a2c1e\",\"ts\":1745434752,\"readings\":{\"pressure\":1013.6,\
             \"humidity\":42.7,\"temp_c\":21.3,\"vbat\":3.842},\"firmware\":\"v2.18.4-rc3\",\
             \"site\":\"warehouse-east-3\",\"flags\":[\"calibrated\",\"online\"]}",
        );
        log.err(
            "panicked at src/pipeline/aggregator.rs:142:21: \
             assertion `left == right` failed (left: 8192, right: 4096) \
             while flushing buffered samples; backtrace truncated, see core.42137 for details",
        );
        log.sys(
            "/Users/stephen/Library/Application Support/elegance-demo/cache/snapshots/\
             2026-04-23T18-04-12Z-warehouse-east-3-sensor-7f3a2c1e.snapshot.json",
        );
        Self {
            theme: BuiltInTheme::default(),
            text_normal: "steve@example.com".into(),
            text_hint: String::new(),
            text_dirty: "3000.0".into(),
            text_pw: "hunter2".into(),
            area_body: "Short note.\nA second line.".into(),
            area_mono: "{\n  \"id\": 42,\n  \"ok\": true\n}".into(),
            select_unit: "ms".into(),
            select_env: "Production".into(),
            check_on: true,
            check_off: false,
            switch_on: true,
            switch_off: false,
            switch_green: true,
            seg_on: true,
            seg_off: false,
            seg_join_a: true,
            seg_join_b: true,
            seg_join_c: false,
            seg_glow: true,
            seg_dim: true,
            category: 0,
            tab_idx: 1,
            collapsing_open: true,
            slider_int: 48,
            slider_float: 0.62,
            show_modal: false,
            pop_filter_open: true,
            pop_filter_in_review: true,
            pop_filter_merged: false,
            pop_filter_closed: false,
            pop_filter_needs_review: true,
            pop_filter_ready: false,
            pop_filter_blocked: false,
            callout_danger_open: true,
            pairing_clients: vec![
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
            ],
            pairing_servers: vec![
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
            ],
            pairing_pairs: vec![
                ("c1".into(), "s3".into()),
                ("c2".into(), "s1".into()),
                ("c4".into(), "s2".into()),
            ],
            pairing_align: false,
            multi_term: build_multi_term(),
            term_pane_count: 4,
            log,
        }
    }
}

/// A pool of up to 16 pre-configured panes the demo draws from when the
/// user adjusts the pane-count slider. Ordering matches typical ops
/// deployment: api first, then workers, edge, caches, a batch host, and
/// a log ingestor.
#[rustfmt::skip]
fn pane_presets() -> Vec<(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    TerminalStatus,
)> {
    vec![
        ("api-east-01",   "api-east-01",    "root",   "/var/log", TerminalStatus::Connected),
        ("api-west-01",   "api-west-01",    "root",   "/var/log", TerminalStatus::Connected),
        ("worker-a",      "worker-pool-a",  "deploy", "~",        TerminalStatus::Reconnecting),
        ("edge-01",       "edge-proxy-01",  "root",   "~",        TerminalStatus::Connected),
        ("api-east-02",   "api-east-02",    "root",   "/var/log", TerminalStatus::Connected),
        ("api-west-02",   "api-west-02",    "root",   "/var/log", TerminalStatus::Connected),
        ("worker-b",      "worker-pool-b",  "deploy", "~",        TerminalStatus::Connected),
        ("edge-02",       "edge-proxy-02",  "root",   "~",        TerminalStatus::Connected),
        ("worker-c",      "worker-pool-c",  "deploy", "~",        TerminalStatus::Offline),
        ("worker-d",      "worker-pool-d",  "deploy", "~",        TerminalStatus::Connected),
        ("edge-03",       "edge-proxy-03",  "root",   "~",        TerminalStatus::Reconnecting),
        ("edge-04",       "edge-proxy-04",  "root",   "~",        TerminalStatus::Connected),
        ("cache-01",      "cache-redis-01", "root",   "/data",    TerminalStatus::Connected),
        ("cache-02",      "cache-redis-02", "root",   "/data",    TerminalStatus::Connected),
        ("warehouse-etl", "warehouse-etl",  "batch",  "/opt/etl", TerminalStatus::Offline),
        ("log-ingest",    "log-ingestor",   "logs",   "/var/kafka", TerminalStatus::Connected),
    ]
}

fn build_preset_pane(idx: usize) -> TerminalPane {
    let (id, host, user, cwd, status) = pane_presets()[idx];
    let mut pane = TerminalPane::new(id, host)
        .user(user)
        .cwd(cwd)
        .status(status)
        .push(TerminalLine::info(format!(
            "Connected via ssh \u{00b7} {host}"
        )));

    // Seed a bit of flavour per pane so the initial view isn't a wall of
    // empty prompts.
    match status {
        TerminalStatus::Connected => {
            pane.push_line(TerminalLine::command(user, host, cwd, "uptime"));
            pane.push_line(TerminalLine::out(
                " 15:54:12 up 18 days, load avg: 0.14 0.22 0.19".to_string(),
            ));
        }
        TerminalStatus::Reconnecting => {
            pane.push_line(TerminalLine::warn(
                "connection degraded, reconnecting\u{2026}".to_string(),
            ));
        }
        TerminalStatus::Offline => {
            pane.push_line(TerminalLine::err(
                "ssh: connect to host — connection refused".to_string(),
            ));
        }
    }
    pane
}

fn build_multi_term() -> MultiTerminal {
    let mut term = MultiTerminal::new("ref_multi_term")
        // Responsive columns. 400 pt keeps each pane wide enough to fit
        // the header row (chevron + hostname + solo + broadcast pill +
        // status indicator) plus a reasonable amount of monospace output
        // before wrapping. Narrower values pack more columns but leave
        // individual panes cramped.
        .columns_auto(400.0)
        .pane_min_height(210.0);
    for i in 0..4 {
        term.add_pane(build_preset_pane(i));
    }
    term
}

/// Add or remove panes so `term` has exactly `target` of them, drawing
/// new ones from the preset pool.
fn sync_pane_count(term: &mut MultiTerminal, target: usize) {
    let target = target.clamp(1, pane_presets().len());
    let current = term.panes().len();
    if current < target {
        for i in current..target {
            term.add_pane(build_preset_pane(i));
        }
    } else if current > target {
        let to_remove: Vec<String> = term.panes()[target..]
            .iter()
            .map(|p| p.id.clone())
            .collect();
        for id in to_remove {
            term.remove_pane(&id);
        }
    }
}

fn simulate_response(pane: &TerminalPane, cmd: &str) -> Vec<TerminalLine> {
    let head = cmd.split_whitespace().next().unwrap_or("");
    match head {
        "uptime" => vec![TerminalLine::out(
            " 15:54:39 up 18 days, 6:12, 1 user, load average: 0.14 0.22 0.19".to_string(),
        )],
        "hostname" => vec![TerminalLine::out(pane.host.clone())],
        "whoami" => vec![TerminalLine::out(pane.user.clone())],
        "pwd" => vec![TerminalLine::out(if pane.cwd == "~" {
            format!("/home/{}", pane.user)
        } else {
            pane.cwd.clone()
        })],
        "date" => vec![TerminalLine::out(
            "Fri Apr 24 15:54:41 UTC 2026".to_string(),
        )],
        "ls" => vec![TerminalLine::out(
            if pane.cwd == "/var/log" {
                "app.log  app.log.1  app.log.2.gz  kernel.log  nginx/"
            } else {
                "deploy.sh  notes.md  scripts/  tmp/"
            }
            .to_string(),
        )],
        "clear" => Vec::new(),
        "" => Vec::new(),
        _ => vec![TerminalLine::err(format!(
            "-bash: {head}: command not found"
        ))],
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The `ThemeSwitcher` in the header installs the selected theme
        // each frame, so no explicit `Theme::install` is needed here.
        let bg = Theme::current(ui.ctx()).palette.bg;

        egui::Panel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(bg)
                    .inner_margin(egui::Margin::symmetric(16, 10)),
            )
            .show_inside(ui, |ui| {
                let theme = Theme::current(ui.ctx());
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("Elegance — Widget reference")
                            .color(theme.palette.text)
                            .size(18.0)
                            .strong(),
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(ThemeSwitcher::new(&mut self.theme));
                    });
                });
            });

        self.log.show(ui);

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add_space(8.0);
            ui.add(TabBar::new(
                &mut self.category,
                ["Inputs", "Display", "Layout", "Overlays", "Tools"],
            ));
            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
                    match self.category {
                        0 => {
                            self.section_buttons(ui);
                            self.section_text(ui);
                            self.section_selects(ui);
                            self.section_toggles(ui);
                            self.section_sliders(ui);
                        }
                        1 => {
                            self.section_tabs(ui);
                            self.section_status(ui);
                            self.section_callouts(ui);
                            self.section_feedback(ui);
                        }
                        2 => {
                            self.section_containers(ui);
                        }
                        3 => {
                            self.section_overlays(ui);
                        }
                        _ => {
                            self.section_pairing(ui);
                            self.section_multi_terminal(ui);
                        }
                    }
                    ui.add_space(12.0);
                });
        });

        self.modal_demo(ui.ctx());
        Toasts::new().render(ui.ctx());
    }
}

impl App {
    fn section_buttons(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Buttons").show(ui, |ui| {
            labeled(ui, "Accents", |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (lbl, a) in [
                        ("Blue", Accent::Blue),
                        ("Green", Accent::Green),
                        ("Red", Accent::Red),
                        ("Purple", Accent::Purple),
                        ("Amber", Accent::Amber),
                        ("Sky", Accent::Sky),
                    ] {
                        let _ = ui.add(Button::new(lbl).accent(a));
                    }
                    let _ = ui.add(Button::new("Outline").outline());
                });
            });

            labeled(ui, "Sizes", |ui| {
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
                });
            });

            labeled(ui, "States", |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.add(Button::new("Default").accent(Accent::Blue));
                    ui.add(Button::new("Disabled").accent(Accent::Blue).enabled(false));
                });
            });
        });
    }

    fn section_text(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Text inputs").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    TextInput::new(&mut self.text_normal)
                        .label("Email")
                        .desired_width(220.0)
                        .id_salt("ref_t_normal"),
                );
                ui.add_space(12.0);
                ui.add(
                    TextInput::new(&mut self.text_hint)
                        .label("With hint")
                        .hint("e.g. sensor-01")
                        .desired_width(220.0)
                        .id_salt("ref_t_hint"),
                );
            });
            ui.horizontal(|ui| {
                ui.add(
                    TextInput::new(&mut self.text_dirty)
                        .label("Dirty (unsaved)")
                        .dirty(true)
                        .desired_width(220.0)
                        .id_salt("ref_t_dirty"),
                );
                ui.add_space(12.0);
                ui.add(
                    TextInput::new(&mut self.text_pw)
                        .label("Password")
                        .password(true)
                        .desired_width(220.0)
                        .id_salt("ref_t_pw"),
                );
            });

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.add(
                    TextArea::new(&mut self.area_body)
                        .label("Text area")
                        .rows(3)
                        .desired_width(220.0)
                        .id_salt("ref_a_body"),
                );
                ui.add_space(12.0);
                ui.add(
                    TextArea::new(&mut self.area_mono)
                        .label("Monospace")
                        .monospace(true)
                        .rows(3)
                        .desired_width(220.0)
                        .id_salt("ref_a_mono"),
                );
            });
        });
    }

    fn section_selects(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Select").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    Select::strings("ref_sel_unit", &mut self.select_unit, ["us", "ms", "s"])
                        .label("Unit")
                        .width(120.0),
                );
                ui.add_space(16.0);
                ui.add(
                    Select::strings(
                        "ref_sel_env",
                        &mut self.select_env,
                        ["Production", "Staging", "Development"],
                    )
                    .label("Environment")
                    .width(180.0),
                );
            });
        });
    }

    fn section_toggles(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Toggles").show(ui, |ui| {
            labeled(ui, "Checkbox", |ui| {
                ui.horizontal(|ui| {
                    ui.add(Checkbox::new(&mut self.check_on, "Enabled"));
                    ui.add_space(16.0);
                    ui.add(Checkbox::new(&mut self.check_off, "Off"));
                });
            });

            labeled(ui, "Switch", |ui| {
                ui.horizontal(|ui| {
                    ui.add(Switch::new(&mut self.switch_on, "On"));
                    ui.add_space(16.0);
                    ui.add(Switch::new(&mut self.switch_off, "Off"));
                    ui.add_space(16.0);
                    ui.add(
                        Switch::new(&mut self.switch_green, "Green accent").accent(Accent::Green),
                    );
                });
            });

            labeled(ui, "Segmented", |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        SegmentedButton::new(&mut self.seg_on, "Record")
                            .accent(Accent::Green)
                            .min_width(130.0),
                    );
                    ui.add_space(8.0);
                    ui.add(
                        SegmentedButton::new(&mut self.seg_off, "Preview")
                            .accent(Accent::Blue)
                            .min_width(130.0),
                    );
                });
            });

            labeled(
                ui,
                "Mixed action row — Button and SegmentedButton share control height at any size",
                |ui| {
                    ui.horizontal(|ui| {
                        let _ =
                            ui.add(Button::new("Collect").accent(Accent::Green).min_width(96.0));
                        ui.add_space(8.0);
                        ui.add(
                            SegmentedButton::new(&mut self.seg_on, "Continuous")
                                .accent(Accent::Green)
                                .min_width(140.0),
                        );
                    });
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let _ = ui.add(
                            Button::new("Collect")
                                .accent(Accent::Green)
                                .size(ButtonSize::Large)
                                .min_width(120.0),
                        );
                        ui.add_space(8.0);
                        ui.add(
                            SegmentedButton::new(&mut self.seg_on, "Continuous")
                                .accent(Accent::Green)
                                .size(ButtonSize::Large)
                                .min_width(180.0),
                        );
                    });
                },
            );

            labeled(ui, "Joined (shared rounding)", |ui| {
                let radius = 8;
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    ui.horizontal(|ui| {
                        ui.add(
                            SegmentedButton::new(&mut self.seg_join_a, "Lint")
                                .accent(Accent::Green)
                                .corner_radius(egui::CornerRadius {
                                    nw: radius,
                                    sw: radius,
                                    ne: 0,
                                    se: 0,
                                })
                                .min_width(110.0),
                        );
                        ui.add(
                            SegmentedButton::new(&mut self.seg_join_b, "Test")
                                .accent(Accent::Blue)
                                .corner_radius(egui::CornerRadius::ZERO)
                                .min_width(110.0),
                        );
                        ui.add(
                            SegmentedButton::new(&mut self.seg_join_c, "Deploy")
                                .accent(Accent::Purple)
                                .corner_radius(egui::CornerRadius {
                                    nw: 0,
                                    sw: 0,
                                    ne: radius,
                                    se: radius,
                                })
                                .min_width(110.0),
                        );
                    });
                });
            });

            labeled(ui, "Dim when on", |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        SegmentedButton::new(&mut self.seg_glow, "Glow (default)")
                            .accent(Accent::Green)
                            .min_width(160.0),
                    );
                    ui.add_space(8.0);
                    ui.add(
                        SegmentedButton::new(&mut self.seg_dim, "Subdued")
                            .accent(Accent::Green)
                            .dim_when_on(true)
                            .min_width(160.0),
                    );
                });
            });
        });
    }

    fn section_tabs(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Tabs").show(ui, |ui| {
            ui.add(TabBar::new(
                &mut self.tab_idx,
                ["Overview", "Settings", "Activity", "Logs"],
            ));
        });
    }

    fn section_status(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Status").show(ui, |ui| {
            labeled(ui, "StatusPill", |ui| {
                ui.add(
                    StatusPill::new()
                        .item("UI", IndicatorState::On)
                        .item("API", IndicatorState::Connecting)
                        .item("DB", IndicatorState::Off),
                );
            });

            labeled(ui, "Indicators", |ui| {
                ui.horizontal(|ui| {
                    for (s, caption) in [
                        (IndicatorState::On, "On"),
                        (IndicatorState::Connecting, "Connecting"),
                        (IndicatorState::Off, "Off"),
                    ] {
                        ui.add(Indicator::new(s));
                        ui.add(egui::Label::new(
                            Theme::current(ui.ctx()).faint_text(caption),
                        ));
                        ui.add_space(12.0);
                    }
                });
            });

            labeled(ui, "Badges", |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.add(Badge::new("OK", BadgeTone::Ok));
                    ui.add(Badge::new("Warning", BadgeTone::Warning));
                    ui.add(Badge::new("Error", BadgeTone::Danger));
                    ui.add(Badge::new("Info", BadgeTone::Info));
                    ui.add(Badge::new("Neutral", BadgeTone::Neutral));
                });
            });
        });
    }

    fn section_callouts(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Callouts").show(ui, |ui| {
            labeled(ui, "Info", |ui| {
                Callout::new(CalloutTone::Info)
                    .title("Node editing is in preview.")
                    .body("The wire format may change before 1.0.")
                    .show(ui, |_| {});
            });

            labeled(ui, "Warning with actions", |ui| {
                Callout::new(CalloutTone::Warning)
                    .title("Unsaved changes.")
                    .body("You have 3 edits that haven't been written to disk.")
                    .show(ui, |ui| {
                        let _ = ui.add(Button::new("Save now").accent(Accent::Amber));
                        let _ = ui.add(Button::new("Discard").outline());
                    });
            });

            labeled(ui, "Danger, dismissable", |ui| {
                if self.callout_danger_open {
                    Callout::new(CalloutTone::Danger)
                        .title("Build failed.")
                        .body("cargo returned 2 errors in src/node_editor.rs.")
                        .dismissable(&mut self.callout_danger_open)
                        .show(ui, |_| {});
                } else if ui.add(Button::new("Restore banner").outline()).clicked() {
                    self.callout_danger_open = true;
                }
            });

            labeled(ui, "Success", |ui| {
                Callout::new(CalloutTone::Success)
                    .title("Deploy complete.")
                    .body("Rolled out to us-east-1.")
                    .show(ui, |_| {});
            });

            labeled(ui, "Neutral", |ui| {
                Callout::new(CalloutTone::Neutral)
                    .title("Read-only mode.")
                    .body("Database upgrade in progress.")
                    .show(ui, |_| {});
            });
        });
    }

    fn section_feedback(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Spinners & progress").show(ui, |ui| {
            labeled(ui, "Spinner sizes", |ui| {
                ui.horizontal(|ui| {
                    ui.add(Spinner::new().size(14.0));
                    ui.add_space(10.0);
                    ui.add(Spinner::new());
                    ui.add_space(10.0);
                    ui.add(Spinner::new().size(28.0));
                });
            });
            labeled(ui, "Spinner accents", |ui| {
                ui.horizontal(|ui| {
                    ui.add(Spinner::new().accent(Accent::Blue));
                    ui.add(Spinner::new().accent(Accent::Green));
                    ui.add(Spinner::new().accent(Accent::Amber));
                    ui.add(Spinner::new().accent(Accent::Red));
                    ui.add(Spinner::new().accent(Accent::Purple));
                });
            });
            labeled(ui, "Progress", |ui| {
                ui.add(ProgressBar::new(0.25));
                ui.add_space(4.0);
                ui.add(ProgressBar::new(0.6).accent(Accent::Green));
                ui.add_space(4.0);
                ui.add(ProgressBar::new(1.0).accent(Accent::Amber).text("Complete"));
            });
            // Drive every ProgressRing tile from a shared clock: 5 s of
            // linear fill, 1.5 s hold at 100%, then snap back to 0.
            let time = ui.ctx().input(|i| i.time) as f32;
            let cycle_len = 6.5_f32;
            let rise_len = 5.0_f32;
            let progress_at = |offset: f32| -> f32 {
                let t = (time + offset).rem_euclid(cycle_len);
                (t / rise_len).min(1.0)
            };
            let ring_fraction = progress_at(0.0);
            elegance::request_repaint_at_rate(ui.ctx(), 30.0);

            labeled(ui, "ProgressRing — progression", |ui| {
                // Five rings phase-offset through the cycle, so at any
                // instant the row reads as a left-to-right "progression".
                ui.horizontal(|ui| {
                    for i in 0..5 {
                        let offset = (i as f32 / 5.0) * cycle_len;
                        ui.add(ProgressRing::new(progress_at(offset)));
                        ui.add_space(8.0);
                    }
                });
            });
            labeled(ui, "ProgressRing — sizes & centre text", |ui| {
                ui.horizontal(|ui| {
                    ui.add(ProgressRing::new(ring_fraction).size(36.0));
                    ui.add_space(12.0);
                    ui.add(ProgressRing::new(ring_fraction).size(56.0));
                    ui.add_space(12.0);
                    let done = (ring_fraction * 20.0).round() as u32;
                    ui.add(
                        ProgressRing::new(ring_fraction)
                            .size(88.0)
                            .text(format!("{} / 20", done))
                            .caption("files"),
                    );
                    ui.add_space(12.0);
                    let remaining = (1.0 - ring_fraction) * 4.0;
                    ui.add(
                        ProgressRing::new(ring_fraction)
                            .size(72.0)
                            .accent(Accent::Amber)
                            .text(format!("{:.1}s", remaining))
                            .caption("remaining"),
                    );
                });
            });
            labeled(ui, "ProgressRing — accents", |ui| {
                ui.horizontal(|ui| {
                    for a in [
                        Accent::Sky,
                        Accent::Green,
                        Accent::Amber,
                        Accent::Red,
                        Accent::Purple,
                    ] {
                        ui.add(ProgressRing::new(ring_fraction).accent(a));
                        ui.add_space(8.0);
                    }
                });
            });
            labeled(ui, "Stepped — cells", |ui| {
                ui.add(Steps::new(6).current(4));
                ui.add_space(6.0);
                ui.add(Steps::new(5).current(2).errored(true));
            });
            labeled(ui, "Stepped — numbered", |ui| {
                ui.add(Steps::new(5).current(2).style(StepsStyle::Numbered));
            });
            labeled(ui, "Stepped — labeled (horizontal)", |ui| {
                ui.add(Steps::labeled(["Plan", "Build", "Test", "Deploy"]).current(2));
                ui.add_space(6.0);
                ui.add(
                    Steps::labeled(["Schema", "Backfill", "Reindex", "Finalize"])
                        .current(2)
                        .errored(true),
                );
            });
            labeled(ui, "Stepped — labeled (vertical)", |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_max_width(200.0);
                        ui.add(
                            Steps::labeled(["Plan", "Design", "Build", "Test", "Deploy"])
                                .current(2)
                                .vertical(),
                        );
                    });
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        ui.set_max_width(220.0);
                        ui.add(
                            Steps::labeled([
                                "Schema validated",
                                "Backfill complete",
                                "Index rebuild",
                                "Finalize",
                            ])
                            .current(2)
                            .errored(true)
                            .vertical(),
                        );
                    });
                });
            });
        });
    }

    fn section_sliders(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Slider").show(ui, |ui| {
            ui.add(
                Slider::new(&mut self.slider_int, 0u32..=100u32)
                    .label("Threshold")
                    .suffix("%"),
            );
            ui.add_space(6.0);
            ui.add(
                Slider::new(&mut self.slider_float, 0.0..=1.0)
                    .label("Gain")
                    .accent(Accent::Green),
            );
        });
    }

    fn section_containers(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Containers").show(ui, |ui| {
            labeled(ui, "Card (inner)", |ui| {
                let theme = Theme::current(ui.ctx());
                Card::new()
                    .padding(12.0)
                    .fill(theme.palette.input_bg)
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(theme.muted_text("An inner card.")));
                    });
            });

            labeled(ui, "CollapsingSection", |ui| {
                CollapsingSection::new("ref_collapsing", "Advanced options")
                    .open(&mut self.collapsing_open)
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        ui.add(egui::Label::new(
                            Theme::current(ui.ctx())
                                .faint_text("…hidden until the header is clicked."),
                        ));
                    });
            });
        });
    }

    fn section_pairing(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Pairing").show(ui, |ui| {
            ui.add(Switch::new(
                &mut self.pairing_align,
                "Align servers to clients (straight lines)",
            ));
            ui.add_space(8.0);
            let mut pairing = Pairing::new(
                "ref_pairing",
                &self.pairing_clients,
                &self.pairing_servers,
                &mut self.pairing_pairs,
            )
            .left_label("Clients")
            .right_label("Servers");
            if self.pairing_align {
                pairing = pairing.align_right();
            }
            pairing.show(ui);
        });
    }

    fn section_multi_terminal(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Multi-terminal").show(ui, |ui| {
            let theme = Theme::current(ui.ctx());
            ui.add(egui::Label::new(theme.faint_text(
                "Click a broadcast pill to include a pane. Type anywhere; Enter runs on every pane with broadcast on.",
            )));
            ui.add_space(8.0);

            // Demo controls: adjust pane count and bulk-toggle collapse.
            ui.horizontal(|ui| {
                ui.add(
                    Slider::new(&mut self.term_pane_count, 1..=16)
                        .label("Terminals")
                        .desired_width(220.0),
                );
                ui.add_space(12.0);
                if ui
                    .add(
                        Button::new("Collapse all")
                            .outline()
                            .size(ButtonSize::Small),
                    )
                    .clicked()
                {
                    self.multi_term.collapse_all();
                }
                if ui
                    .add(
                        Button::new("Expand all")
                            .outline()
                            .size(ButtonSize::Small),
                    )
                    .clicked()
                {
                    self.multi_term.expand_all();
                }
            });
            ui.add_space(6.0);

            // Apply the slider value by adding or removing panes.
            sync_pane_count(&mut self.multi_term, self.term_pane_count);

            let _ = self.multi_term.show(ui);
            for event in self.multi_term.take_events() {
                match event {
                    TerminalEvent::Command { targets, command } => {
                        for id in &targets {
                            let reply = match self.multi_term.pane(id) {
                                Some(pane) => simulate_response(pane, &command),
                                None => continue,
                            };
                            if command.trim() == "clear" {
                                if let Some(pane) = self.multi_term.pane_mut(id) {
                                    pane.lines.clear();
                                    pane.push_line(TerminalLine::info("screen cleared"));
                                }
                            } else {
                                for line in reply {
                                    self.multi_term.push_line(id, line);
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    fn section_overlays(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Overlays").show(ui, |ui| {
            let theme = Theme::current(ui.ctx());
            ui.add(egui::Label::new(theme.faint_text(
                "Click to open. Modal, Menu, Popover, and Toast render over everything else.",
            )));
            ui.add_space(6.0);

            labeled(ui, "Modal, Menu, Toast", |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(Button::new("Open modal").accent(Accent::Blue))
                        .clicked()
                    {
                        self.show_modal = true;
                    }

                    let menu_trigger =
                        ui.add(Button::new("Open menu").outline().size(ButtonSize::Medium));
                    Menu::new("ref_menu").show_below(&menu_trigger, |ui| {
                        let _ = ui.add(MenuItem::new("Edit").shortcut("⌘ E"));
                        let _ = ui.add(MenuItem::new("Duplicate").shortcut("⌘ D"));
                        ui.separator();
                        let _ = ui.add(MenuItem::new("Delete").danger());
                    });

                    if ui.add(Button::new("Toast").accent(Accent::Green)).clicked() {
                        Toast::new("Saved")
                            .description("All changes have been persisted.")
                            .tone(BadgeTone::Ok)
                            .show(ui.ctx());
                    }
                });
            });

            self.popover_examples(ui);
        });
    }

    fn popover_examples(&mut self, ui: &mut egui::Ui) {
        labeled(ui, "Popover · placements", |ui| {
            ui.horizontal(|ui| {
                for (label, side) in [
                    ("Top", PopoverSide::Top),
                    ("Bottom", PopoverSide::Bottom),
                    ("Left", PopoverSide::Left),
                    ("Right", PopoverSide::Right),
                ] {
                    let trigger = ui.add(Button::new(label).outline().size(ButtonSize::Small));
                    Popover::new(("placement", label))
                        .side(side)
                        .show(&trigger, |ui| {
                            let theme = Theme::current(ui.ctx());
                            ui.add(egui::Label::new(theme.muted_text(format!(
                                "Opens on the {} side.",
                                label.to_lowercase()
                            ))));
                        });
                }
            });
        });

        labeled(ui, "Popover · title + body + destructive footer", |ui| {
            let trigger = ui.add(Button::new("Delete branch").outline());
            Popover::new("pop_confirm")
                .side(PopoverSide::Bottom)
                .title("Delete feature/snap-baseline?")
                .show(&trigger, |ui| {
                    let theme = Theme::current(ui.ctx());
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

        labeled(ui, "Popover · info card, no footer, fixed width", |ui| {
            let trigger = ui.add(Button::new("What's a baseline?").outline());
            Popover::new("pop_info")
                .side(PopoverSide::Top)
                .title("Baselines")
                .width(300.0)
                .show(&trigger, |ui| {
                    let theme = Theme::current(ui.ctx());
                    ui.add(egui::Label::new(theme.muted_text(
                        "A baseline is the accepted reference image for a widget. \
                         Tests compare each render against it pixel by pixel.",
                    )));
                });
        });

        labeled(ui, "Popover · multi-select filter", |ui| {
            ui.horizontal(|ui| {
                let trigger = ui.add(Button::new("Filter ▾").outline());
                Popover::new("pop_filter")
                    .side(PopoverSide::Bottom)
                    .width(220.0)
                    .show(&trigger, |ui| {
                        let theme = Theme::current(ui.ctx());
                        ui.add(egui::Label::new(theme.faint_text("STATUS")));
                        ui.add(Checkbox::new(&mut self.pop_filter_open, "Open"));
                        ui.add(Checkbox::new(&mut self.pop_filter_in_review, "In review"));
                        ui.add(Checkbox::new(&mut self.pop_filter_merged, "Merged"));
                        ui.add(Checkbox::new(&mut self.pop_filter_closed, "Closed"));
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(4.0);
                        ui.add(egui::Label::new(theme.faint_text("LABELS")));
                        ui.add(Checkbox::new(
                            &mut self.pop_filter_needs_review,
                            "Needs review",
                        ));
                        ui.add(Checkbox::new(&mut self.pop_filter_ready, "Ready to merge"));
                        ui.add(Checkbox::new(&mut self.pop_filter_blocked, "Blocked"));
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            let _ = ui.add(Button::new("Clear").outline().size(ButtonSize::Small));
                            let _ = ui.add(
                                Button::new("Apply")
                                    .accent(Accent::Blue)
                                    .size(ButtonSize::Small),
                            );
                        });
                    });
            });
        });

        labeled(
            ui,
            "Popover · no arrow, custom gap — dropdown surface",
            |ui| {
                let trigger = ui.add(Button::new("Quick actions ▾").outline());
                Popover::new("pop_plain")
                    .side(PopoverSide::Bottom)
                    .arrow(false)
                    .gap(4.0)
                    .show(&trigger, |ui| {
                        ui.add(MenuItem::new("Rename"));
                        ui.add(MenuItem::new("Duplicate"));
                        ui.add(MenuItem::new("Archive"));
                        ui.separator();
                        ui.add(MenuItem::new("Delete").danger());
                    });
            },
        );
    }

    fn modal_demo(&mut self, ctx: &egui::Context) {
        if !self.show_modal {
            return;
        }
        Modal::new("ref_modal", &mut self.show_modal)
            .heading("Modal dialog")
            .max_width(420.0)
            .show(ctx, |ui| {
                let theme = Theme::current(ui.ctx());
                ui.add(egui::Label::new(theme.muted_text(
                    "Centered over a dimmed backdrop. Press Esc or click the × to dismiss.",
                )));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    let _ = ui.add(Button::new("Confirm").accent(Accent::Green));
                    let _ = ui.add(Button::new("Cancel").outline());
                });
            });
    }
}

fn labeled(ui: &mut egui::Ui, label: &str, body: impl FnOnce(&mut egui::Ui)) {
    let theme = Theme::current(ui.ctx());
    ui.add_space(4.0);
    ui.add(egui::Label::new(theme.muted_text(label)));
    ui.add_space(4.0);
    body(ui);
    ui.add_space(8.0);
}
