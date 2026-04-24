//! Automated widget screenshot renderer.
//!
//! Renders every widget category to PNG files under `docs/images/` via
//! `egui_kittest`'s wgpu test renderer. Run with:
//!
//! ```sh
//! cargo render-docs
//! ```
//!
//! Each tile renders standalone in its natural size (thanks to
//! [`Harness::fit_contents`]) against the theme background so the PNGs can
//! be embedded in `README.md` without additional cropping.

use eframe::egui;
use egui_kittest::Harness;
use elegance::{
    Accent, Badge, BadgeTone, Button, ButtonSize, Card, Checkbox, CollapsingSection, Indicator,
    IndicatorState, MenuItem, PairItem, Pairing, ProgressBar, SegmentedButton, Select, Slider,
    Spinner, StatusPill, Switch, TabBar, TextArea, TextInput, Theme,
};

const OUTPUT_DIR: &str = "docs/images";
const PIXELS_PER_POINT: f32 = 2.0;
const TILE_PADDING: i8 = 16;
const INITIAL_SIZE: (f32, f32) = (900.0, 600.0);

fn main() {
    std::fs::create_dir_all(OUTPUT_DIR).expect("create output dir");

    render_buttons();
    render_text_inputs();
    render_text_areas();
    render_selects();
    render_toggles();
    render_tabs();
    render_status();
    render_feedback();
    render_sliders();
    render_containers();
    render_menu();
    render_modal();
    render_toast();
    render_pairing();

    println!("\nDone. PNGs in {OUTPUT_DIR}/");
}

// ----------------------------------------------------------------------------
// Tiles
// ----------------------------------------------------------------------------

fn render_buttons() {
    render("buttons", |ui| {
        background(ui, |ui| {
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
            ui.add_space(6.0);
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
        });
    });
}

fn render_text_inputs() {
    let mut normal = "steve@example.com".to_string();
    let mut hint = String::new();
    let mut dirty = "3000.0".to_string();
    let mut pw = "hunter2".to_string();

    render("text_inputs", move |ui| {
        background(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    TextInput::new(&mut normal)
                        .label("Email")
                        .desired_width(240.0)
                        .id_salt("r_normal"),
                );
                ui.add_space(12.0);
                ui.add(
                    TextInput::new(&mut hint)
                        .label("With hint")
                        .hint("e.g. sensor-01")
                        .desired_width(240.0)
                        .id_salt("r_hint"),
                );
            });
            ui.horizontal(|ui| {
                ui.add(
                    TextInput::new(&mut dirty)
                        .label("Dirty (unsaved)")
                        .dirty(true)
                        .desired_width(240.0)
                        .id_salt("r_dirty"),
                );
                ui.add_space(12.0);
                ui.add(
                    TextInput::new(&mut pw)
                        .label("Password")
                        .password(true)
                        .desired_width(240.0)
                        .id_salt("r_pw"),
                );
            });
        });
    });
}

fn render_text_areas() {
    let mut body = "A multi-line text area.\nDrop notes, logs, or JSON here.".to_string();
    let mut mono = "{\n  \"id\": 42,\n  \"ok\": true\n}".to_string();

    render("text_areas", move |ui| {
        background(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    TextArea::new(&mut body)
                        .label("Text area")
                        .rows(3)
                        .desired_width(240.0)
                        .id_salt("r_area_body"),
                );
                ui.add_space(12.0);
                ui.add(
                    TextArea::new(&mut mono)
                        .label("Monospace")
                        .monospace(true)
                        .rows(3)
                        .desired_width(240.0)
                        .id_salt("r_area_mono"),
                );
            });
        });
    });
}

fn render_selects() {
    let mut unit = "ms".to_string();
    let mut env = "Production".to_string();

    render("selects", move |ui| {
        background(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    Select::strings("r_sel_unit", &mut unit, ["us", "ms", "s"])
                        .label("Unit")
                        .width(120.0),
                );
                ui.add_space(16.0);
                ui.add(
                    Select::strings(
                        "r_sel_env",
                        &mut env,
                        ["Production", "Staging", "Development"],
                    )
                    .label("Environment")
                    .width(180.0),
                );
            });
        });
    });
}

fn render_toggles() {
    let mut check_on = true;
    let mut check_off = false;
    let mut switch_on = true;
    let mut switch_off = false;
    let mut switch_green = true;
    let mut seg_on = true;
    let mut seg_off = false;

    render("toggles", move |ui| {
        background(ui, |ui| {
            caption(ui, "Checkbox");
            ui.horizontal(|ui| {
                ui.add(Checkbox::new(&mut check_on, "Enabled"));
                ui.add_space(16.0);
                ui.add(Checkbox::new(&mut check_off, "Off"));
            });
            ui.add_space(10.0);

            caption(ui, "Switch");
            ui.horizontal(|ui| {
                ui.add(Switch::new(&mut switch_on, "On"));
                ui.add_space(16.0);
                ui.add(Switch::new(&mut switch_off, "Off"));
                ui.add_space(16.0);
                ui.add(Switch::new(&mut switch_green, "Green accent").accent(Accent::Green));
            });
            ui.add_space(10.0);

            caption(ui, "Segmented");
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
        });
    });
}

fn render_tabs() {
    let mut tab = 1usize;

    render("tabs", move |ui| {
        background(ui, |ui| {
            ui.set_min_width(520.0);
            ui.add(TabBar::new(
                &mut tab,
                ["Overview", "Settings", "Activity", "Logs"],
            ));
        });
    });
}

fn render_status() {
    render("status", |ui| {
        background(ui, |ui| {
            caption(ui, "StatusPill");
            ui.add(
                StatusPill::new()
                    .item("UI", IndicatorState::On)
                    .item("API", IndicatorState::Connecting)
                    .item("DB", IndicatorState::Off),
            );
            ui.add_space(10.0);

            caption(ui, "Indicators");
            ui.horizontal(|ui| {
                for (s, text) in [
                    (IndicatorState::On, "On"),
                    (IndicatorState::Connecting, "Connecting"),
                    (IndicatorState::Off, "Off"),
                ] {
                    ui.add(Indicator::new(s));
                    let theme = Theme::current(ui.ctx());
                    ui.add(egui::Label::new(theme.faint_text(text)));
                    ui.add_space(12.0);
                }
            });
            ui.add_space(10.0);

            caption(ui, "Badges");
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

fn render_feedback() {
    render("feedback", |ui| {
        background(ui, |ui| {
            ui.set_max_width(500.0);

            caption(ui, "Spinner");
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

            caption(ui, "ProgressBar");
            ui.add(ProgressBar::new(0.25));
            ui.add_space(4.0);
            ui.add(ProgressBar::new(0.6).accent(Accent::Green));
            ui.add_space(4.0);
            ui.add(ProgressBar::new(1.0).accent(Accent::Amber).text("Complete"));
        });
    });
}

fn render_sliders() {
    let mut threshold: u32 = 48;
    let mut gain: f32 = 0.62;

    render("sliders", move |ui| {
        background(ui, |ui| {
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
        });
    });
}

fn render_containers() {
    let mut open = true;

    render("containers", move |ui| {
        background(ui, |ui| {
            ui.set_min_width(440.0);

            caption(ui, "Card");
            Card::new().heading("Account").show(ui, |ui| {
                let theme = Theme::current(ui.ctx());
                ui.add(egui::Label::new(
                    theme.muted_text("Card contents — headings, widgets, anything."),
                ));
            });
            ui.add_space(10.0);

            caption(ui, "CollapsingSection");
            CollapsingSection::new("r_collapsing", "Advanced options")
                .open(&mut open)
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    let theme = Theme::current(ui.ctx());
                    ui.add(egui::Label::new(
                        theme.faint_text("…revealed when the header is clicked."),
                    ));
                });
        });
    });
}

fn render_menu() {
    render("menu", |ui| {
        background(ui, |ui| {
            ui.set_max_width(220.0);
            // The actual `Menu` widget paints into its own popup Area, which
            // the Harness render doesn't capture neatly. Re-paint the popup
            // chrome inline so the tile looks identical to the opened menu.
            let theme = Theme::current(ui.ctx());
            let p = &theme.palette;
            egui::Frame::new()
                .fill(p.card)
                .stroke(egui::Stroke::new(1.0, p.border))
                .corner_radius(egui::CornerRadius::same(theme.card_radius as u8))
                .inner_margin(egui::Margin::symmetric(4, 4))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    let _ = ui.add(MenuItem::new("Edit").shortcut("⌘ E"));
                    let _ = ui.add(MenuItem::new("Duplicate").shortcut("⌘ D"));
                    ui.separator();
                    let _ = ui.add(MenuItem::new("Delete").danger());
                });
        });
    });
}

fn render_modal() {
    render("modal", |ui| {
        background(ui, |ui| {
            ui.set_max_width(420.0);
            // Re-paint the modal card inline — Modal proper renders into a
            // top-level centered Area (with a full-viewport backdrop) which
            // doesn't translate well to a tile screenshot.
            let theme = Theme::current(ui.ctx());
            let p = &theme.palette;
            egui::Frame::new()
                .fill(p.card)
                .stroke(egui::Stroke::new(1.0, p.border))
                .corner_radius(egui::CornerRadius::same(theme.card_radius as u8))
                .inner_margin(egui::Margin::same(theme.card_padding as i8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(theme.heading_text("Run Summary")));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let _ = ui.add(Button::new("×").outline().size(ButtonSize::Small));
                        });
                    });
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(10.0);
                    ui.add(egui::Label::new(theme.muted_text(
                        "Centered over a dimmed backdrop. Press Esc or click × to dismiss.",
                    )));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        let _ = ui.add(Button::new("Confirm").accent(Accent::Green));
                        let _ = ui.add(Button::new("Cancel").outline());
                    });
                });
        });
    });
}

fn render_toast() {
    render("toast", |ui| {
        background(ui, |ui| {
            // Again, re-paint a single toast card inline. `Toast::show` enqueues
            // into a ctx-scoped Area which doesn't compose into a tile PNG.
            let theme = Theme::current(ui.ctx());
            let p = &theme.palette;
            egui::Frame::new()
                .fill(p.card)
                .stroke(egui::Stroke::new(1.0, p.border))
                .corner_radius(egui::CornerRadius::same(theme.card_radius as u8))
                .inner_margin(egui::Margin::symmetric(14, 10))
                .show(ui, |ui| {
                    ui.set_min_width(320.0);
                    ui.horizontal(|ui| {
                        // Left accent bar — manually painted to match the real toast layout.
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(3.0, 36.0), egui::Sense::hover());
                        ui.painter()
                            .rect_filled(rect, egui::CornerRadius::same(2), p.success);
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new("Deploy complete")
                                    .color(p.text)
                                    .strong()
                                    .size(theme.typography.body),
                            ));
                            ui.add(egui::Label::new(
                                theme.faint_text("Rolled out to us-east-1"),
                            ));
                        });
                    });
                });
        });
    });
}

fn render_pairing() {
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

    render("pairing", move |ui| {
        background(ui, |ui| {
            ui.set_max_width(680.0);
            Pairing::new("r_pairing", &clients, &servers, &mut pairs)
                .left_label("Clients")
                .right_label("Servers")
                .show(ui);
        });
    });
}

// ----------------------------------------------------------------------------
// Rendering helpers
// ----------------------------------------------------------------------------

fn render<F>(name: &str, build_ui: F)
where
    F: FnMut(&mut egui::Ui),
{
    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(INITIAL_SIZE.0, INITIAL_SIZE.1))
        .with_pixels_per_point(PIXELS_PER_POINT)
        .wgpu()
        .build_ui(build_ui);

    // Run once to lay out, shrink to content, re-run, then capture.
    harness.run();
    harness.fit_contents();
    harness.run();

    let image = harness
        .render()
        .unwrap_or_else(|e| panic!("render {name}: {e}"));
    let path = format!("{OUTPUT_DIR}/{name}.png");
    image.save(&path).expect("save png");
    println!("wrote {} ({}×{})", path, image.width(), image.height());
}

fn background<F>(ui: &mut egui::Ui, body: F)
where
    F: FnOnce(&mut egui::Ui),
{
    Theme::slate().install(ui.ctx());
    let theme = Theme::current(ui.ctx());
    egui::Frame::new()
        .fill(theme.palette.bg)
        .inner_margin(egui::Margin::same(TILE_PADDING))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
            body(ui);
        });
}

fn caption(ui: &mut egui::Ui, text: &str) {
    let theme = Theme::current(ui.ctx());
    ui.add(egui::Label::new(theme.muted_text(text)));
    ui.add_space(4.0);
}
