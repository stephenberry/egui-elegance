//! "Orbit" — a fictional deployment command center.
//!
//! An interactive showcase of the elegance widget set, framed around
//! shipping code.
//!
//! Run with: `cargo run --example orbit`

#![allow(clippy::needless_ifs, clippy::collapsible_if)]

use eframe::egui;
use elegance::{
    request_repaint_at_rate, Accent, Badge, BadgeTone, BuiltInTheme, Button, ButtonSize, Card,
    Checkbox, Indicator, IndicatorState, Menu, MenuItem, ProgressBar, ResponseFlashExt,
    SegmentedButton, Select, Slider, Spinner, StatusPill, Switch, TabBar, TextArea, TextInput,
    Theme, ThemeSwitcher, Toast, Toasts,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Orbit — deployment command center")
            .with_inner_size([1180.0, 820.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Orbit",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

#[derive(Debug)]
struct App {
    theme: BuiltInTheme,
    tab: usize,

    // Release target.
    environment: String,
    region: String,
    image_tag: String,
    replicas: u32,
    release_notes: String,
    drain: bool,
    notify_slack: bool,

    // Gates.
    gate_unit: bool,
    gate_e2e: bool,
    gate_security: bool,
    gate_auto: bool,

    // Services tab.
    service_filter: String,
    auto_heal: bool,
    svc_paused: [bool; 6],

    // Pipelines tab.
    pipeline_name: String,
    pipeline_trigger: String,
    pipeline_schedule: String,
    step_lint: bool,
    step_test: bool,
    step_build: bool,
    step_publish: bool,

    // Secrets tab.
    secret_name: String,
    secret_value: String,
    secret_scope: String,
    secret_ack: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            theme: BuiltInTheme::default(),
            tab: 0,
            environment: "staging".into(),
            region: "us-east-1".into(),
            image_tag: "orbit:v2.14.3".into(),
            replicas: 6,
            release_notes: "• Bump tracing to 0.2\n• Patch oauth callback timeout\n• Swap CDN pool for us-east-2".into(),
            drain: true,
            notify_slack: true,
            gate_unit: true,
            gate_e2e: true,
            gate_security: false,
            gate_auto: false,
            service_filter: String::new(),
            auto_heal: true,
            svc_paused: [false, false, true, false, false, false],
            pipeline_name: "nightly-build".into(),
            pipeline_trigger: "cron".into(),
            pipeline_schedule: "0 3 * * *".into(),
            step_lint: true,
            step_test: true,
            step_build: true,
            step_publish: false,
            secret_name: "STRIPE_API_KEY".into(),
            secret_value: "sk_live_7f3b9a2d5c41e9".into(),
            secret_scope: "production".into(),
            secret_ack: false,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // `ThemeSwitcher` auto-installs the selected theme each frame.
        egui::Panel::top("header")
            .frame(
                egui::Frame::new()
                    .fill(Theme::current(ui.ctx()).palette.bg)
                    .inner_margin(egui::Margin::symmetric(16, 12)),
            )
            .show_inside(ui, |ui| self.header(ui));

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    Card::new().show(ui, |ui| {
                        ui.add(TabBar::new(
                            &mut self.tab,
                            ["Overview", "Services", "Pipelines", "Secrets"],
                        ));
                        ui.add_space(12.0);
                        match self.tab {
                            0 => self.overview_tab(ui),
                            1 => self.services_tab(ui),
                            2 => self.pipelines_tab(ui),
                            _ => self.secrets_tab(ui),
                        }
                    });
                });
        });

        Toasts::new().render(ui.ctx());
    }
}

impl App {
    fn header(&mut self, ui: &mut egui::Ui) {
        let theme = Theme::current(ui.ctx());
        let p = &theme.palette;
        ui.horizontal(|ui| {
            ui.add(egui::Label::new(
                egui::RichText::new("Orbit")
                    .color(p.sky)
                    .size(22.0)
                    .strong(),
            ));
            ui.add_space(10.0);
            ui.add(egui::Label::new(
                egui::RichText::new("Deployment Command Center")
                    .color(p.text_faint)
                    .size(12.0),
            ));
            ui.add_space(16.0);
            ui.add(
                StatusPill::new()
                    .item("Control", IndicatorState::On)
                    .item("Cluster", IndicatorState::On)
                    .item("Registry", IndicatorState::Connecting)
                    .item("Oncall", IndicatorState::Off),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let tone = env_tone(&self.environment);
                ui.add(Badge::new(upper(&self.environment), tone));
                ui.add_space(6.0);
                ui.add(Badge::new("v2.14.3", BadgeTone::Neutral));
                ui.add_space(6.0);
                ui.add(Badge::new("carmela", BadgeTone::Info));
                ui.add_space(12.0);
                ui.add(ThemeSwitcher::new(&mut self.theme));
            });
        });
    }

    // ---- Overview ----------------------------------------------------------

    fn overview_tab(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |cols| {
            let mut it = cols.iter_mut();
            let left = it.next().unwrap();
            let right = it.next().unwrap();
            self.overview_left(left);
            self.overview_right(right);
        });
    }

    fn overview_left(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Release target").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    Select::strings(
                        "env",
                        &mut self.environment,
                        ["production", "staging", "preview", "local"],
                    )
                    .label("Environment")
                    .width(170.0),
                );
                ui.add_space(12.0);
                ui.add(
                    Select::strings(
                        "region",
                        &mut self.region,
                        ["us-east-1", "us-west-2", "eu-west-1", "ap-south-1"],
                    )
                    .label("Region")
                    .width(170.0),
                );
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let resp = ui.add(
                    TextInput::new(&mut self.image_tag)
                        .label("Image tag (press Enter to validate)")
                        .hint("orbit:vX.Y.Z")
                        .dirty(true)
                        .desired_width(260.0)
                        .id_salt("image_tag"),
                );
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if self.image_tag.trim().is_empty() || !self.image_tag.contains(':') {
                        resp.flash_error();
                    } else {
                        resp.flash_success();
                    }
                }
            });
            ui.add_space(8.0);
            ui.add(
                Slider::new(&mut self.replicas, 1..=20)
                    .label("Replicas")
                    .desired_width(280.0),
            );

            ui.add_space(8.0);
            ui.add(
                TextArea::new(&mut self.release_notes)
                    .label("Release notes")
                    .hint("Summary visible in #release-control…")
                    .rows(4)
                    .id_salt("release_notes"),
            );

            ui.add_space(6.0);
            ui.add(Switch::new(
                &mut self.drain,
                "Drain connections before cutover",
            ));
            ui.add(Switch::new(
                &mut self.notify_slack,
                "Announce in #release-control",
            ));

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui
                    .add(Button::new("Deploy").accent(Accent::Green).min_width(110.0))
                    .clicked()
                {
                    Toast::new("Deploy started")
                        .tone(BadgeTone::Info)
                        .description(format!("{} → {}", self.image_tag, self.environment))
                        .show(ui.ctx());
                }
                if ui
                    .add(Button::new("Rollback").accent(Accent::Red).min_width(110.0))
                    .clicked()
                {
                    Toast::new("Rolling back")
                        .tone(BadgeTone::Warning)
                        .description("Previous revision will be restored")
                        .show(ui.ctx());
                }
                if ui.add(Button::new("Dry run").outline()).clicked() {
                    Toast::new("Dry run complete")
                        .tone(BadgeTone::Ok)
                        .description("No changes would be applied")
                        .show(ui.ctx());
                }
            });
        });

        ui.add_space(10.0);

        Card::new().heading("Approval gates").show(ui, |ui| {
            ui.scope(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.horizontal(|ui| {
                    ui.add(
                        SegmentedButton::new(&mut self.gate_unit, "Unit tests")
                            .accent(Accent::Green)
                            .min_width(140.0),
                    );
                    ui.add(
                        SegmentedButton::new(&mut self.gate_e2e, "E2E suite")
                            .accent(Accent::Blue)
                            .min_width(140.0),
                    );
                    ui.add(
                        SegmentedButton::new(&mut self.gate_security, "Security scan")
                            .accent(Accent::Amber)
                            .min_width(160.0),
                    );
                });
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add(
                    Switch::new(&mut self.gate_auto, "Auto-apply on green").accent(Accent::Green),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(Button::new("Override").outline().size(ButtonSize::Small))
                        .clicked()
                    {}
                    if ui
                        .add(
                            Button::new("Request review")
                                .accent(Accent::Blue)
                                .size(ButtonSize::Small),
                        )
                        .clicked()
                    {}
                });
            });
        });
    }

    fn overview_right(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Services").show(ui, |ui| {
            ui.scope(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                let services: &[(&str, IndicatorState, &str, BadgeTone, &str)] = &[
                    (
                        "api",
                        IndicatorState::On,
                        "healthy",
                        BadgeTone::Ok,
                        "v2.14.3",
                    ),
                    (
                        "worker",
                        IndicatorState::On,
                        "healthy",
                        BadgeTone::Ok,
                        "v2.14.3",
                    ),
                    (
                        "scheduler",
                        IndicatorState::Connecting,
                        "draining",
                        BadgeTone::Warning,
                        "v2.14.2",
                    ),
                    (
                        "ingest",
                        IndicatorState::On,
                        "healthy",
                        BadgeTone::Ok,
                        "v2.14.3",
                    ),
                    (
                        "billing",
                        IndicatorState::Off,
                        "error",
                        BadgeTone::Danger,
                        "v2.13.9",
                    ),
                    (
                        "web",
                        IndicatorState::On,
                        "healthy",
                        BadgeTone::Ok,
                        "v2.14.3",
                    ),
                ];
                for (name, state, status, tone, version) in services {
                    service_row(ui, name, *state, status, *tone, version);
                }
            });
        });

        ui.add_space(10.0);

        Card::new().heading("Recent deployments").show(ui, |ui| {
            ui.scope(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                running_deployment_row(ui, "now", "v2.14.4 → preview", "carmela");
                let items: &[(&str, &str, &str, BadgeTone)] = &[
                    ("3m ago", "v2.14.3 → staging", "carmela", BadgeTone::Ok),
                    ("47m ago", "v2.14.3 → preview", "rollbot", BadgeTone::Ok),
                    ("2h ago", "v2.14.2 → production", "kai", BadgeTone::Warning),
                    ("6h ago", "v2.14.1 → production", "carmela", BadgeTone::Ok),
                    (
                        "1d ago",
                        "v2.14.0 → production",
                        "rollbot",
                        BadgeTone::Danger,
                    ),
                ];
                for (when, desc, who, tone) in items {
                    deployment_row(ui, when, desc, who, *tone);
                }
            });
        });
    }

    // ---- Services ----------------------------------------------------------

    fn services_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add(
                TextInput::new(&mut self.service_filter)
                    .hint("Filter services…")
                    .desired_width(280.0)
                    .id_salt("svc_filter"),
            );
            ui.add_space(10.0);
            ui.add(SegmentedButton::new(&mut self.auto_heal, "Auto-heal").accent(Accent::Green));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(Button::new("Restart all").accent(Accent::Amber))
                    .clicked()
                {}
                if ui.add(Button::new("Scale").accent(Accent::Blue)).clicked() {}
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);

        let names = ["api", "worker", "scheduler", "ingest", "billing", "web"];
        let versions = [
            "v2.14.3", "v2.14.3", "v2.14.2", "v2.14.3", "v2.13.9", "v2.14.3",
        ];
        let states = [
            IndicatorState::On,
            IndicatorState::On,
            IndicatorState::Connecting,
            IndicatorState::On,
            IndicatorState::Off,
            IndicatorState::On,
        ];
        let cpu = ["18%", "42%", "6%", "77%", "—", "22%"];
        let mem = ["210 MB", "1.1 GB", "120 MB", "2.4 GB", "—", "340 MB"];
        let pods = ["6/6", "12/12", "1/1", "8/8", "0/2", "4/4"];

        for i in 0..names.len() {
            if !self.service_filter.is_empty()
                && !names[i].contains(&self.service_filter.to_lowercase())
            {
                continue;
            }
            ui.horizontal(|ui| {
                service_detail_row(
                    ui,
                    names[i],
                    states[i],
                    versions[i],
                    cpu[i],
                    mem[i],
                    pods[i],
                    &mut self.svc_paused[i],
                );
            });
            ui.add_space(4.0);
        }
    }

    // ---- Pipelines ---------------------------------------------------------

    fn pipelines_tab(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |cols| {
            let mut it = cols.iter_mut();
            let left = it.next().unwrap();
            let right = it.next().unwrap();

            Card::new().heading("Edit pipeline").show(left, |ui| {
                ui.add(
                    TextInput::new(&mut self.pipeline_name)
                        .label("Name")
                        .desired_width(f32::INFINITY)
                        .id_salt("pipeline_name"),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add(
                        Select::strings(
                            "trigger",
                            &mut self.pipeline_trigger,
                            ["push", "pull-request", "cron", "webhook", "manual"],
                        )
                        .label("Trigger")
                        .width(170.0),
                    );
                    ui.add_space(12.0);
                    let enabled = self.pipeline_trigger == "cron";
                    ui.add_enabled_ui(enabled, |ui| {
                        ui.add(
                            TextInput::new(&mut self.pipeline_schedule)
                                .label("Schedule (cron)")
                                .hint("* * * * *")
                                .desired_width(180.0)
                                .id_salt("pipeline_schedule"),
                        );
                    });
                });
                ui.add_space(10.0);
                let theme = Theme::current(ui.ctx());
                ui.add(egui::Label::new(theme.muted_text("Steps")));
                ui.add_space(2.0);
                ui.add(Checkbox::new(&mut self.step_lint, "Lint (rust + ts)"));
                ui.add(Checkbox::new(
                    &mut self.step_test,
                    "Unit + integration tests",
                ));
                ui.add(Checkbox::new(&mut self.step_build, "Build container image"));
                ui.add(Checkbox::new(
                    &mut self.step_publish,
                    "Publish to registry on main",
                ));
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(Button::new("Save").accent(Accent::Green).min_width(96.0))
                        .clicked()
                    {}
                    if ui
                        .add(Button::new("Run now").accent(Accent::Blue).min_width(96.0))
                        .clicked()
                    {}
                    if ui.add(Button::new("Discard").outline()).clicked() {}
                });
            });

            Card::new().heading("Recent runs").show(right, |ui| {
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    running_pipeline_row(ui, "#4822", "main @ e01dd7f", "0:14");
                    let runs: &[(&str, &str, &str, BadgeTone, &str)] = &[
                        ("#4821", "main @ a73f1c9", "42s", BadgeTone::Ok, "passed"),
                        ("#4820", "main @ 0b9e7f1", "38s", BadgeTone::Ok, "passed"),
                        (
                            "#4819",
                            "pr-312 @ 9c2ab03",
                            "1m 04s",
                            BadgeTone::Warning,
                            "flaky",
                        ),
                        ("#4818", "main @ 3df91e2", "44s", BadgeTone::Ok, "passed"),
                        (
                            "#4817",
                            "pr-309 @ 71ee4a0",
                            "12s",
                            BadgeTone::Danger,
                            "failed",
                        ),
                        ("#4816", "main @ 8c4f7d8", "41s", BadgeTone::Ok, "passed"),
                    ];
                    for (id, commit, dur, tone, status) in runs {
                        pipeline_run_row(ui, id, commit, dur, *tone, status);
                    }
                });
            });
        });
    }

    // ---- Secrets -----------------------------------------------------------

    fn secrets_tab(&mut self, ui: &mut egui::Ui) {
        Card::new().heading("Rotate secret").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    TextInput::new(&mut self.secret_name)
                        .label("Name")
                        .desired_width(260.0)
                        .id_salt("secret_name"),
                );
                ui.add_space(12.0);
                ui.add(
                    Select::strings(
                        "secret_scope",
                        &mut self.secret_scope,
                        ["production", "staging", "preview", "shared"],
                    )
                    .label("Scope")
                    .width(170.0),
                );
            });
            ui.add_space(8.0);
            let value_resp = ui.add(
                TextInput::new(&mut self.secret_value)
                    .label("Value")
                    .password(true)
                    .dirty(true)
                    .desired_width(f32::INFINITY)
                    .id_salt("secret_value"),
            );

            ui.add_space(10.0);
            ui.add(Checkbox::new(
                &mut self.secret_ack,
                "I understand this will invalidate active sessions",
            ));
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let rotate = ui.add_enabled(
                    self.secret_ack,
                    Button::new("Rotate").accent(Accent::Green).min_width(110.0),
                );
                if rotate.clicked() {
                    self.secret_value = new_secret();
                    value_resp.flash_success();
                }
                let revoke = ui.add_enabled(
                    self.secret_ack,
                    Button::new("Revoke").accent(Accent::Red).min_width(110.0),
                );
                if revoke.clicked() {
                    value_resp.flash_error();
                }
                if ui.add(Button::new("Copy").outline()).clicked() {}
            });
        });

        ui.add_space(10.0);

        Card::new().heading("Active secrets").show(ui, |ui| {
            ui.scope(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                let secrets: &[(&str, &str, &str, BadgeTone)] = &[
                    ("STRIPE_API_KEY", "production", "4 days", BadgeTone::Ok),
                    ("STRIPE_API_KEY", "staging", "4 days", BadgeTone::Ok),
                    ("SMTP_PASSWORD", "shared", "31 days", BadgeTone::Warning),
                    ("SIGNING_KEY", "production", "94 days", BadgeTone::Danger),
                    ("GITHUB_TOKEN", "shared", "12 days", BadgeTone::Ok),
                    ("ANTHROPIC_API_KEY", "production", "2 days", BadgeTone::Ok),
                ];
                for (name, scope, age, tone) in secrets {
                    secret_row(ui, name, scope, age, *tone);
                }
            });
        });
    }
}

// ---- Row helpers ----------------------------------------------------------

fn service_row(
    ui: &mut egui::Ui,
    name: &str,
    state: IndicatorState,
    status: &str,
    tone: BadgeTone,
    version: &str,
) {
    let theme = Theme::current(ui.ctx());
    let p = &theme.palette;
    egui::Frame::new()
        .fill(p.input_bg)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(Indicator::new(state));
                ui.add_space(8.0);
                ui.add(egui::Label::new(
                    egui::RichText::new(name).color(p.text).strong(),
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(Badge::new(version, BadgeTone::Neutral));
                    ui.add_space(6.0);
                    ui.add(Badge::new(status, tone));
                });
            });
        });
}

fn deployment_row(ui: &mut egui::Ui, when: &str, desc: &str, who: &str, tone: BadgeTone) {
    let theme = Theme::current(ui.ctx());
    let p = &theme.palette;
    egui::Frame::new()
        .fill(p.input_bg)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(theme.faint_text(when)));
                ui.add_space(10.0);
                ui.add(egui::Label::new(egui::RichText::new(desc).color(p.text)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(Badge::new(
                        match tone {
                            BadgeTone::Ok => "success",
                            BadgeTone::Warning => "manual",
                            BadgeTone::Danger => "failed",
                            _ => "—",
                        },
                        tone,
                    ));
                    ui.add_space(8.0);
                    ui.add(egui::Label::new(theme.muted_text(who)));
                });
            });
        });
}

/// A small outlined button painted with three horizontal dots — the
/// universal "more actions" affordance. Local to this demo; consumers who
/// want a gear / chevron / bell use the same pattern against their own
/// Painter calls.
fn dots_button(ui: &mut egui::Ui) -> egui::Response {
    let resp = ui.add(
        Button::new("")
            .outline()
            .size(ButtonSize::Small)
            .min_width(28.0),
    );
    if ui.is_rect_visible(resp.rect) {
        let theme = Theme::current(ui.ctx());
        let color = if resp.hovered() {
            theme.palette.text
        } else {
            theme.palette.text_muted
        };
        let center = resp.rect.center();
        let spacing = 4.0;
        let radius = 1.4;
        for dx in [-spacing, 0.0, spacing] {
            ui.painter()
                .circle_filled(egui::pos2(center.x + dx, center.y), radius, color);
        }
    }
    resp
}

#[allow(clippy::too_many_arguments)]
fn service_detail_row(
    ui: &mut egui::Ui,
    name: &str,
    state: IndicatorState,
    version: &str,
    cpu: &str,
    mem: &str,
    pods: &str,
    paused: &mut bool,
) {
    let theme = Theme::current(ui.ctx());
    let p = &theme.palette;
    egui::Frame::new()
        .fill(p.input_bg)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(Indicator::new(state));
                ui.add_space(8.0);
                let name_label = egui::RichText::new(name).color(p.text).strong();
                ui.add_sized(
                    egui::vec2(110.0, 20.0),
                    egui::Label::new(name_label).wrap_mode(egui::TextWrapMode::Extend),
                );
                ui.add(Badge::new(version, BadgeTone::Neutral));
                ui.add_space(16.0);
                metric(ui, "cpu", cpu);
                ui.add_space(12.0);
                metric(ui, "mem", mem);
                ui.add_space(12.0);
                metric(ui, "pods", pods);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let more = dots_button(ui);
                    Menu::new(("svc_menu", name)).show_below(&more, |ui| {
                        if ui.add(MenuItem::new("View logs").shortcut("⌘ L")).clicked() {
                            Toast::new(format!("Tailing {name}"))
                                .tone(BadgeTone::Info)
                                .show(ui.ctx());
                        }
                        if ui.add(MenuItem::new("Restart").shortcut("⌘ R")).clicked() {
                            Toast::new(format!("Restarting {name}"))
                                .tone(BadgeTone::Warning)
                                .show(ui.ctx());
                        }
                        if ui.add(MenuItem::new("Duplicate")).clicked() {
                            Toast::new(format!("Duplicated {name}"))
                                .tone(BadgeTone::Ok)
                                .show(ui.ctx());
                        }
                        ui.separator();
                        if ui
                            .add(MenuItem::new("Delete").shortcut("⌫").danger())
                            .clicked()
                        {
                            Toast::new(format!("Deleted {name}"))
                                .tone(BadgeTone::Danger)
                                .show(ui.ctx());
                        }
                    });
                    ui.add_space(4.0);
                    ui.add(
                        SegmentedButton::new(paused, if *paused { "Paused" } else { "Active" })
                            .accent(Accent::Amber)
                            .min_width(96.0),
                    );
                });
            });
        });
}

fn running_deployment_row(ui: &mut egui::Ui, when: &str, desc: &str, who: &str) {
    let theme = Theme::current(ui.ctx());
    let p = &theme.palette;
    egui::Frame::new()
        .fill(p.input_bg)
        .stroke(egui::Stroke::new(1.0, p.sky))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(Spinner::new().size(14.0));
                ui.add_space(6.0);
                ui.add(egui::Label::new(
                    egui::RichText::new(when).color(p.sky).strong(),
                ));
                ui.add_space(10.0);
                ui.add(egui::Label::new(egui::RichText::new(desc).color(p.text)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(Badge::new("running", BadgeTone::Info));
                    ui.add_space(8.0);
                    ui.add(egui::Label::new(theme.muted_text(who)));
                });
            });
            ui.add_space(6.0);
            // Rollout progress: looped fill from 0 to 100% every ~7s.
            let t = ui.input(|i| i.time) as f32;
            let frac = (t * 0.14) % 1.0;
            request_repaint_at_rate(ui.ctx(), 30.0);
            let pods = (frac * 12.0).floor() as u32;
            ui.add(
                ProgressBar::new(frac)
                    .accent(Accent::Sky)
                    .height(18.0)
                    .text(format!("{pods}/12 pods")),
            );
        });
}

fn running_pipeline_row(ui: &mut egui::Ui, id: &str, commit: &str, elapsed: &str) {
    let theme = Theme::current(ui.ctx());
    let p = &theme.palette;
    egui::Frame::new()
        .fill(p.input_bg)
        .stroke(egui::Stroke::new(1.0, p.sky))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(id).color(p.sky).strong(),
                ));
                ui.add_space(10.0);
                ui.add(egui::Label::new(
                    egui::RichText::new(commit).color(p.text).monospace(),
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(Badge::new("running", BadgeTone::Info));
                    ui.add_space(8.0);
                    ui.add(Spinner::new().size(14.0));
                    ui.add_space(4.0);
                    ui.add(egui::Label::new(theme.faint_text(elapsed)));
                });
            });
        });
}

fn pipeline_run_row(
    ui: &mut egui::Ui,
    id: &str,
    commit: &str,
    duration: &str,
    tone: BadgeTone,
    status: &str,
) {
    let theme = Theme::current(ui.ctx());
    let p = &theme.palette;
    egui::Frame::new()
        .fill(p.input_bg)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(id).color(p.sky).strong(),
                ));
                ui.add_space(10.0);
                ui.add(egui::Label::new(
                    egui::RichText::new(commit).color(p.text).monospace(),
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(Badge::new(status, tone));
                    ui.add_space(8.0);
                    ui.add(egui::Label::new(theme.faint_text(duration)));
                });
            });
        });
}

fn secret_row(ui: &mut egui::Ui, name: &str, scope: &str, age: &str, tone: BadgeTone) {
    let theme = Theme::current(ui.ctx());
    let p = &theme.palette;
    egui::Frame::new()
        .fill(p.input_bg)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(name).color(p.text).monospace().strong(),
                ));
                ui.add_space(12.0);
                ui.add(Badge::new(scope, BadgeTone::Neutral));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            Button::new("Rotate")
                                .accent(Accent::Blue)
                                .size(ButtonSize::Small),
                        )
                        .clicked()
                    {}
                    ui.add_space(6.0);
                    ui.add(Badge::new(format!("age {age}"), tone));
                });
            });
        });
}

fn metric(ui: &mut egui::Ui, label: &str, value: &str) {
    let theme = Theme::current(ui.ctx());
    let p = &theme.palette;
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        ui.add(egui::Label::new(theme.faint_text(label)));
        ui.add(egui::Label::new(
            egui::RichText::new(value).color(p.text).size(13.0).strong(),
        ));
    });
}

// ---- Utilities -----------------------------------------------------------

fn env_tone(env: &str) -> BadgeTone {
    match env {
        "production" => BadgeTone::Danger,
        "staging" => BadgeTone::Warning,
        "preview" => BadgeTone::Info,
        _ => BadgeTone::Neutral,
    }
}

fn upper(s: &str) -> String {
    s.to_uppercase()
}

fn new_secret() -> String {
    // Just a stable-looking fake rotation for the demo.
    let tail: u32 = 0xC0FFEE;
    format!("sk_live_{tail:08x}")
}
