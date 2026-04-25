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
    Accent, Badge, BadgeTone, Button, ButtonSize, Callout, CalloutTone, Card, Checkbox,
    CollapsingSection, Indicator, IndicatorState, MenuBar, MenuItem, PairItem, Pairing,
    ProgressBar, ProgressRing, RangeSlider, SegmentedButton, Select, Slider, Spinner, StatusPill,
    Steps, StepsStyle, Switch, TabBar, TextArea, TextInput, Theme,
};

const OUTPUT_DIR: &str = "docs/images";
const PIXELS_PER_POINT: f32 = 2.0;
const TILE_PADDING: i8 = 16;
const INITIAL_SIZE: (f32, f32) = (900.0, 600.0);

fn main() {
    std::fs::create_dir_all(OUTPUT_DIR).expect("create output dir");

    render_hero();
    render_buttons();
    render_text_inputs();
    render_text_areas();
    render_selects();
    render_toggles();
    render_tabs();
    render_status();
    render_feedback();
    render_progress_ring();
    render_steps();
    render_sliders();
    render_range_sliders();
    render_containers();
    render_menu();
    render_menu_bar();
    render_modal();
    render_drawer();
    render_popover();
    render_callout();
    render_toast();
    render_pairing();
    render_glyphs();
    render_theming();

    println!("\nDone. PNGs in {OUTPUT_DIR}/");
}

// ----------------------------------------------------------------------------
// Tiles
// ----------------------------------------------------------------------------

fn render_hero() {
    // The hero is rendered at a fixed wide aspect (no `fit_contents`) so the
    // README banner reads as a real app screen rather than a cropped widget
    // tile. State has to live outside the closure so the stateful widgets
    // (TextInput, Slider, Switch, Select) can take `&mut`.
    let mut env = "staging".to_string();
    let mut region = "us-east-1".to_string();
    let mut image_tag = "orbit:v2.14.3".to_string();
    let mut replicas: u32 = 6;
    let mut drain = true;
    let mut notify = true;

    let hero_size = egui::Vec2::new(1200.0, 510.0);
    let mut harness = Harness::builder()
        .with_size(hero_size)
        .with_pixels_per_point(PIXELS_PER_POINT)
        .wgpu()
        .build_ui(move |ui| {
            Theme::slate().install(ui.ctx());
            let theme = Theme::current(ui.ctx());
            let p = &theme.palette;

            // Paint the full viewport with the theme background — without
            // `fit_contents`, anything not covered by the inner Frame would
            // otherwise show the harness default backdrop.
            let full = ui.max_rect();
            ui.painter()
                .rect_filled(full, egui::CornerRadius::ZERO, p.bg);

            egui::Frame::new()
                .inner_margin(egui::Margin::same(22))
                .show(ui, |ui| {
                    ui.set_min_size(ui.available_size_before_wrap());
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

                    // -- Header strip ------------------------------------------
                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new("Elegance")
                                .color(p.sky)
                                .size(24.0)
                                .strong(),
                        ));
                        ui.add_space(10.0);
                        ui.add(egui::Label::new(
                            egui::RichText::new("Deployment command center")
                                .color(p.text_faint)
                                .size(13.0),
                        ));
                        ui.add_space(18.0);
                        ui.add(
                            StatusPill::new()
                                .item("Control", IndicatorState::On)
                                .item("Cluster", IndicatorState::On)
                                .item("Registry", IndicatorState::Connecting)
                                .item("Oncall", IndicatorState::Off),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add(Badge::new("STAGING", BadgeTone::Warning));
                            ui.add_space(6.0);
                            ui.add(Badge::new("v2.14.3", BadgeTone::Neutral));
                            ui.add_space(6.0);
                            ui.add(Badge::new("@you", BadgeTone::Info));
                        });
                    });

                    ui.add_space(14.0);

                    // -- Body: 2 columns ---------------------------------------
                    ui.columns(2, |cols| {
                        let mut it = cols.iter_mut();
                        let left = it.next().unwrap();
                        let right = it.next().unwrap();

                        Card::new().heading("Release target").show(left, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    Select::strings(
                                        "hero_env",
                                        &mut env,
                                        ["production", "staging", "preview", "local"],
                                    )
                                    .label("Environment")
                                    .width(170.0),
                                );
                                ui.add_space(10.0);
                                ui.add(
                                    Select::strings(
                                        "hero_region",
                                        &mut region,
                                        ["us-east-1", "us-west-2", "eu-west-1", "ap-south-1"],
                                    )
                                    .label("Region")
                                    .width(170.0),
                                );
                            });
                            ui.add_space(8.0);
                            ui.add(
                                TextInput::new(&mut image_tag)
                                    .label("Image tag")
                                    .dirty(true)
                                    .desired_width(f32::INFINITY)
                                    .id_salt("hero_tag"),
                            );
                            ui.add_space(10.0);
                            ui.add(
                                Slider::new(&mut replicas, 1u32..=20u32)
                                    .label("Replicas")
                                    .accent(Accent::Sky),
                            );
                            ui.add_space(6.0);
                            ui.add(Switch::new(&mut drain, "Drain connections before cutover"));
                            ui.add(
                                Switch::new(&mut notify, "Announce in #release-control")
                                    .accent(Accent::Green),
                            );

                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                let _ = ui.add(
                                    Button::new("Deploy").accent(Accent::Green).min_width(110.0),
                                );
                                let _ = ui.add(
                                    Button::new("Rollback").accent(Accent::Red).min_width(110.0),
                                );
                                let _ = ui.add(Button::new("Dry run").outline());
                            });
                        });

                        Card::new().heading("Pipeline").show(right, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    ProgressRing::new(0.6)
                                        .size(76.0)
                                        .accent(Accent::Green)
                                        .text("12 / 20")
                                        .caption("steps"),
                                );
                                ui.add_space(16.0);
                                ui.vertical(|ui| {
                                    ui.add_space(4.0);
                                    ui.add(
                                        Steps::labeled(["Plan", "Build", "Test", "Deploy"])
                                            .current(2)
                                            .desired_width(360.0),
                                    );
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        ui.add(Badge::new("3 / 4 passed", BadgeTone::Ok));
                                        ui.add_space(6.0);
                                        ui.add(Badge::new("ETA 2m", BadgeTone::Info));
                                    });
                                });
                            });
                        });

                        right.add_space(10.0);

                        Card::new().heading("Services").show(right, |ui| {
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
                                        "billing",
                                        IndicatorState::Off,
                                        "error",
                                        BadgeTone::Danger,
                                        "v2.13.9",
                                    ),
                                ];
                                for (name, state, status, tone, version) in services {
                                    hero_service_row(ui, name, *state, status, *tone, version);
                                }
                            });
                        });
                    });
                });
        });

    // No `fit_contents` — the hero stays at the requested aspect.
    harness.run();
    harness.run();

    let image = harness
        .render()
        .unwrap_or_else(|e| panic!("render hero: {e}"));
    let path = format!("{OUTPUT_DIR}/hero.png");
    image.save(&path).expect("save png");
    println!("wrote {} ({}×{})", path, image.width(), image.height());
}

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

fn render_range_sliders() {
    let (mut price_lo, mut price_hi): (u32, u32) = (24, 118);
    let (mut latency_lo, mut latency_hi): (u32, u32) = (120, 340);
    let (mut volume_lo, mut volume_hi): (u32, u32) = (18, 62);

    render("range_sliders", move |ui| {
        background(ui, |ui| {
            let w = 520.0_f32;
            Card::new().show(ui, |ui| {
                ui.set_max_width(w);
                ui.add(
                    RangeSlider::new(&mut price_lo, &mut price_hi, 0u32..=200u32)
                        .label("Price")
                        .value_fmt(|v| format!("${v:.0}"))
                        .desired_width(w)
                        .id_salt("doc_range_price"),
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
                        .id_salt("doc_range_latency"),
                );
                ui.add_space(8.0);
                ui.add(
                    RangeSlider::new(&mut volume_lo, &mut volume_hi, 0u32..=100u32)
                        .label("Volume")
                        .suffix(" dB")
                        .accent(Accent::Green)
                        .desired_width(w)
                        .id_salt("doc_range_volume"),
                );
            });
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
            ui.set_max_width(440.0);
            // Re-paint the modal card inline — Modal proper renders into a
            // top-level centered Area (with a full-viewport backdrop) which
            // doesn't translate well to a tile screenshot.
            let theme = Theme::current(ui.ctx());
            let p = &theme.palette;
            let pad = theme.card_padding;
            egui::Frame::new()
                .fill(p.card)
                .stroke(egui::Stroke::new(1.0, p.border))
                .corner_radius(egui::CornerRadius::same(theme.card_radius as u8))
                .show(ui, |ui| {
                    // Header band with icon halo + title stack + close.
                    egui::Frame::new()
                        .inner_margin(egui::Margin {
                            left: pad as i8,
                            right: pad as i8,
                            top: pad as i8,
                            bottom: 0,
                        })
                        .show(ui, |ui| {
                            ui.horizontal_top(|ui| {
                                let halo_size = 32.0;
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(halo_size, halo_size),
                                    egui::Sense::hover(),
                                );
                                let fg = theme.palette.accent_fill(Accent::Red);
                                let bg = egui::Color32::from_rgba_unmultiplied(
                                    fg.r(),
                                    fg.g(),
                                    fg.b(),
                                    36,
                                );
                                ui.painter()
                                    .circle_filled(rect.center(), halo_size * 0.5, bg);
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "!",
                                    egui::FontId::proportional(theme.typography.heading + 2.0),
                                    fg,
                                );
                                ui.add_space(10.0);
                                ui.vertical(|ui| {
                                    ui.add(egui::Label::new(theme.heading_text("Delete project?")));
                                    ui.add(egui::Label::new(
                                        theme.muted_text("This action cannot be undone."),
                                    ));
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        let _ = ui.add(
                                            Button::new("×").outline().size(ButtonSize::Small),
                                        );
                                    },
                                );
                            });
                        });
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(10.0);
                    egui::Frame::new()
                        .inner_margin(egui::Margin {
                            left: pad as i8,
                            right: pad as i8,
                            top: 0,
                            bottom: pad as i8 / 2,
                        })
                        .show(ui, |ui| {
                            ui.add(egui::Label::new(theme.muted_text(
                                "All dashboards, alerts, and 3 active deployments will be \
                                 permanently removed. Members will lose access immediately.",
                            )));
                        });
                    ui.separator();
                    egui::Frame::new()
                        .fill(theme.palette.depth_tint(p.card, 0.04))
                        .inner_margin(egui::Margin::symmetric(pad as i8, pad as i8 * 3 / 4))
                        .show(ui, |ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let _ =
                                        ui.add(Button::new("Delete project").accent(Accent::Red));
                                    let _ = ui.add(Button::new("Cancel").outline());
                                },
                            );
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

fn render_progress_ring() {
    render("progress_ring", |ui| {
        background(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add(ProgressRing::new(0.42));
                ui.add_space(20.0);
                ui.add(
                    ProgressRing::new(0.6)
                        .size(88.0)
                        .accent(Accent::Green)
                        .text("12 / 20")
                        .caption("files"),
                );
                ui.add_space(20.0);
                ui.add(ProgressRing::new(0.75).size(48.0).accent(Accent::Amber));
                ui.add_space(20.0);
                ui.add(ProgressRing::new(1.0).size(48.0).accent(Accent::Purple));
            });
        });
    });
}

fn render_steps() {
    render("steps", |ui| {
        background(ui, |ui| {
            ui.set_min_width(440.0);

            caption(ui, "Cells");
            ui.add(Steps::new(6).current(4).desired_width(420.0));
            ui.add_space(10.0);

            caption(ui, "Numbered");
            ui.add(
                Steps::new(5)
                    .current(2)
                    .style(StepsStyle::Numbered)
                    .desired_width(420.0),
            );
            ui.add_space(10.0);

            caption(ui, "Labeled");
            ui.add(
                Steps::labeled(["Plan", "Build", "Test", "Deploy"])
                    .current(2)
                    .desired_width(420.0),
            );
        });
    });
}

fn render_menu_bar() {
    render("menu_bar", |ui| {
        background(ui, |ui| {
            let theme = Theme::current(ui.ctx());
            let p = &theme.palette;

            // Allocate a fixed-size scaffold rect so the tile renders at a
            // predictable width regardless of the harness viewport.
            let scaffold_w = 820.0_f32;
            let scaffold_h = 110.0_f32;
            let (scaffold_rect, _) =
                ui.allocate_exact_size(egui::vec2(scaffold_w, scaffold_h), egui::Sense::hover());
            ui.painter().rect_filled(
                scaffold_rect,
                egui::CornerRadius::same(theme.card_radius as u8),
                p.bg,
            );
            ui.painter().rect_stroke(
                scaffold_rect,
                egui::CornerRadius::same(theme.card_radius as u8),
                egui::Stroke::new(1.0, p.border),
                egui::StrokeKind::Inside,
            );

            // Paint the bar into the top portion of the scaffold using a
            // child UI bounded to the scaffold's rect.
            let mut bar_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(scaffold_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            MenuBar::new("docs_menu_bar")
                .brand("Elegance")
                .status_with_dot("main \u{00b7} up to date", p.green)
                .show(&mut bar_ui, |bar| {
                    bar.menu("File", |_| {});
                    bar.menu("Edit", |_| {});
                    bar.menu("View", |_| {});
                    bar.menu("Window", |_| {});
                    bar.menu("Help", |_| {});
                });

            // Faux body placeholder bars under the strip.
            let body_top = bar_ui.min_rect().bottom() + 14.0;
            let pad = 16.0;
            let avail_w = scaffold_w - pad * 2.0;
            for (i, w_frac) in [0.78, 0.55, 0.68].iter().enumerate() {
                let bar = egui::Rect::from_min_size(
                    egui::pos2(scaffold_rect.left() + pad, body_top + i as f32 * 12.0),
                    egui::vec2(avail_w * w_frac, 8.0),
                );
                ui.painter().rect_filled(
                    bar,
                    egui::CornerRadius::same(3),
                    p.depth_tint(p.card, 0.18),
                );
            }
        });
    });
}

fn render_drawer() {
    render("drawer", |ui| {
        // Drawer paints into top-level Areas (backdrop + panel), which the
        // Harness can't compose into a tile. Mock the scaffold (page +
        // backdrop + drawer panel) inline so the tile reads as the open
        // state from the right side of an app window.
        let theme = Theme::current(ui.ctx());
        let p = &theme.palette;

        let scaffold_w = 800.0_f32;
        let scaffold_h = 360.0_f32;
        let panel_w = 360.0_f32;

        // Allocate the scaffold rect.
        let (scaffold_rect, _) =
            ui.allocate_exact_size(egui::vec2(scaffold_w, scaffold_h), egui::Sense::hover());
        let painter = ui.painter_at(scaffold_rect);

        // Page background.
        painter.rect_filled(
            scaffold_rect,
            egui::CornerRadius::same(theme.card_radius as u8),
            p.bg,
        );

        // Faux page content — a couple of placeholder bars + a tile grid.
        let content_pad = 18.0;
        let mut y = scaffold_rect.top() + content_pad;
        for w in [scaffold_w - panel_w - 60.0, scaffold_w - panel_w - 140.0] {
            let bar = egui::Rect::from_min_size(
                egui::pos2(scaffold_rect.left() + content_pad, y),
                egui::vec2(w, 8.0),
            );
            painter.rect_filled(bar, egui::CornerRadius::same(3), p.depth_tint(p.card, 0.18));
            y += 14.0;
        }
        y += 8.0;
        let cell_w = (scaffold_w - panel_w - content_pad * 2.0 - 24.0) / 4.0;
        let cell_h = 50.0;
        for col in 0..4 {
            for row in 0..2 {
                let r = egui::Rect::from_min_size(
                    egui::pos2(
                        scaffold_rect.left() + content_pad + col as f32 * (cell_w + 8.0),
                        y + row as f32 * (cell_h + 8.0),
                    ),
                    egui::vec2(cell_w, cell_h),
                );
                let fill = if col == 1 && row == 0 {
                    egui::Color32::from_rgba_unmultiplied(p.sky.r(), p.sky.g(), p.sky.b(), 30)
                } else {
                    p.depth_tint(p.card, 0.18)
                };
                painter.rect_filled(r, egui::CornerRadius::same(5), fill);
                if col == 1 && row == 0 {
                    painter.rect_stroke(
                        r,
                        egui::CornerRadius::same(5),
                        egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgba_unmultiplied(
                                p.sky.r(),
                                p.sky.g(),
                                p.sky.b(),
                                115,
                            ),
                        ),
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }

        // Backdrop dimming.
        painter.rect_filled(
            scaffold_rect,
            egui::CornerRadius::same(theme.card_radius as u8),
            egui::Color32::from_black_alpha(120),
        );

        // Drawer panel.
        let panel_rect = egui::Rect::from_min_max(
            egui::pos2(scaffold_rect.right() - panel_w, scaffold_rect.top()),
            scaffold_rect.right_bottom(),
        );

        // Soft shadow on the panel's leading (left) edge — paint a few
        // alpha-decreasing strips so the rendered tile shows the depth that
        // the live `Frame::shadow` produces.
        for i in 0..16 {
            let t = i as f32;
            let alpha = ((1.0 - t / 16.0) * 50.0) as u8;
            let strip = egui::Rect::from_min_max(
                egui::pos2(panel_rect.left() - t - 1.0, panel_rect.top()),
                egui::pos2(panel_rect.left() - t, panel_rect.bottom()),
            );
            painter.rect_filled(
                strip,
                egui::CornerRadius::ZERO,
                egui::Color32::from_black_alpha(alpha),
            );
        }

        painter.rect_filled(panel_rect, egui::CornerRadius::ZERO, p.card);
        painter.line_segment(
            [panel_rect.left_top(), panel_rect.left_bottom()],
            egui::Stroke::new(1.0, p.border),
        );

        // Header band.
        let header_h = 64.0;
        let header_rect = egui::Rect::from_min_max(
            panel_rect.left_top(),
            egui::pos2(panel_rect.right(), panel_rect.top() + header_h),
        );
        painter.line_segment(
            [
                egui::pos2(header_rect.left(), header_rect.bottom()),
                egui::pos2(header_rect.right(), header_rect.bottom()),
            ],
            egui::Stroke::new(1.0, p.border),
        );

        // Header text.
        painter.text(
            egui::pos2(header_rect.left() + 18.0, header_rect.top() + 14.0),
            egui::Align2::LEFT_TOP,
            "INC-2187 — api-west-02",
            egui::FontId::proportional(theme.typography.heading),
            p.text,
        );
        painter.text(
            egui::pos2(header_rect.left() + 18.0, header_rect.top() + 36.0),
            egui::Align2::LEFT_TOP,
            "Latency spike · 18 min ago",
            egui::FontId::proportional(theme.typography.label),
            p.text_muted,
        );

        // Close × glyph (mocked, like the live Drawer's outline button).
        let close_size = 24.0;
        let close_rect = egui::Rect::from_center_size(
            egui::pos2(
                header_rect.right() - 18.0 - close_size * 0.5,
                header_rect.top() + 18.0,
            ),
            egui::vec2(close_size, close_size),
        );
        painter.rect_stroke(
            close_rect,
            egui::CornerRadius::same(theme.control_radius as u8),
            egui::Stroke::new(1.0, p.border),
            egui::StrokeKind::Inside,
        );
        painter.text(
            close_rect.center(),
            egui::Align2::CENTER_CENTER,
            "×",
            egui::FontId::proportional(theme.typography.body + 1.0),
            p.text_muted,
        );

        // Body — a couple of section labels + KV rows + status pill row.
        let mut by = header_rect.bottom() + 14.0;
        let body_left = panel_rect.left() + 18.0;
        painter.text(
            egui::pos2(body_left, by),
            egui::Align2::LEFT_TOP,
            "STATUS",
            egui::FontId::proportional(theme.typography.small),
            p.text_faint,
        );
        by += 18.0;
        let pill = egui::Rect::from_min_size(egui::pos2(body_left, by), egui::vec2(110.0, 22.0));
        painter.rect_filled(
            pill,
            egui::CornerRadius::same(11),
            egui::Color32::from_rgba_unmultiplied(p.warning.r(), p.warning.g(), p.warning.b(), 30),
        );
        painter.rect_stroke(
            pill,
            egui::CornerRadius::same(11),
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(
                    p.warning.r(),
                    p.warning.g(),
                    p.warning.b(),
                    80,
                ),
            ),
            egui::StrokeKind::Inside,
        );
        painter.circle_filled(
            egui::pos2(pill.left() + 12.0, pill.center().y),
            3.5,
            p.warning,
        );
        painter.text(
            egui::pos2(pill.left() + 22.0, pill.center().y),
            egui::Align2::LEFT_CENTER,
            "Investigating",
            egui::FontId::proportional(theme.typography.small),
            p.warning,
        );
        by += 32.0;

        painter.text(
            egui::pos2(body_left, by),
            egui::Align2::LEFT_TOP,
            "DETAILS",
            egui::FontId::proportional(theme.typography.small),
            p.text_faint,
        );
        by += 18.0;
        for (k, v) in [
            ("Service", "api-web"),
            ("Region", "us-west-2"),
            ("Signal", "p95 > 300 ms · 3 m"),
            ("Owner", "Platform Edge"),
        ] {
            painter.text(
                egui::pos2(body_left, by),
                egui::Align2::LEFT_TOP,
                k,
                egui::FontId::proportional(theme.typography.label),
                p.text_muted,
            );
            painter.text(
                egui::pos2(body_left + 100.0, by),
                egui::Align2::LEFT_TOP,
                v,
                egui::FontId::monospace(theme.typography.label),
                p.text,
            );
            painter.line_segment(
                [
                    egui::pos2(body_left, by + 22.0),
                    egui::pos2(panel_rect.right() - 18.0, by + 22.0),
                ],
                egui::Stroke::new(1.0, p.border),
            );
            by += 26.0;
        }

        // Footer band with two mock buttons.
        let foot_h = 56.0;
        let foot_rect = egui::Rect::from_min_max(
            egui::pos2(panel_rect.left(), panel_rect.bottom() - foot_h),
            panel_rect.right_bottom(),
        );
        painter.rect_filled(
            foot_rect,
            egui::CornerRadius::ZERO,
            p.depth_tint(p.card, 0.04),
        );
        painter.line_segment(
            [foot_rect.left_top(), foot_rect.right_top()],
            egui::Stroke::new(1.0, p.border),
        );

        let btn_y = foot_rect.center().y;
        let primary = egui::Rect::from_center_size(
            egui::pos2(panel_rect.right() - 18.0 - 56.0, btn_y),
            egui::vec2(112.0, 30.0),
        );
        painter.rect_filled(
            primary,
            egui::CornerRadius::same(theme.control_radius as u8),
            p.blue,
        );
        painter.text(
            primary.center(),
            egui::Align2::CENTER_CENTER,
            "Acknowledge",
            egui::FontId::proportional(theme.typography.button),
            p.bg,
        );

        let secondary = egui::Rect::from_center_size(
            egui::pos2(primary.left() - 8.0 - 40.0, btn_y),
            egui::vec2(80.0, 30.0),
        );
        painter.rect_stroke(
            secondary,
            egui::CornerRadius::same(theme.control_radius as u8),
            egui::Stroke::new(1.0, p.border),
            egui::StrokeKind::Inside,
        );
        painter.text(
            secondary.center(),
            egui::Align2::CENTER_CENTER,
            "Snooze",
            egui::FontId::proportional(theme.typography.button),
            p.text,
        );
    });
}

fn render_popover() {
    render("popover", |ui| {
        background(ui, |ui| {
            ui.set_max_width(380.0);
            // The real `Popover` widget paints into a top-level `Popup` Area
            // anchored at a trigger Response, which the Harness can't compose
            // into a tile. Re-paint the trigger button + popover panel + arrow
            // inline so the tile reads as the open state.
            let trigger = ui.add(Button::new("Delete branch").outline());
            ui.add_space(10.0);

            let theme = Theme::current(ui.ctx());
            let p = &theme.palette;

            let frame_response = egui::Frame::new()
                .fill(p.card)
                .stroke(egui::Stroke::new(1.0, p.border))
                .corner_radius(egui::CornerRadius::same(theme.card_radius as u8))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_min_width(320.0);
                    ui.add(egui::Label::new(
                        egui::RichText::new("Delete feature/snap-baseline?")
                            .color(p.text)
                            .strong()
                            .size(theme.typography.body),
                    ));
                    ui.add_space(4.0);
                    ui.add(egui::Label::new(
                        theme.muted_text("This removes the branch from origin too."),
                    ));
                    ui.add_space(10.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let _ = ui.add(
                            Button::new("Delete")
                                .accent(Accent::Red)
                                .size(ButtonSize::Small),
                        );
                        let _ = ui.add(Button::new("Cancel").outline().size(ButtonSize::Small));
                    });
                });

            // Upward arrow centered on the trigger.
            let panel = frame_response.response.rect;
            let cx = trigger
                .rect
                .center()
                .x
                .clamp(panel.left() + 14.0, panel.right() - 14.0);
            let painter = ui.painter();
            let half_base = 6.0;
            let depth = 6.0;
            let base_y = panel.top();
            let tip = egui::pos2(cx, base_y - depth);
            painter.add(egui::Shape::convex_polygon(
                vec![
                    egui::pos2(cx - half_base, base_y),
                    tip,
                    egui::pos2(cx + half_base, base_y),
                ],
                p.card,
                egui::Stroke::NONE,
            ));
            let stroke = egui::Stroke::new(1.0, p.border);
            painter.line_segment([egui::pos2(cx - half_base, base_y), tip], stroke);
            painter.line_segment([tip, egui::pos2(cx + half_base, base_y)], stroke);
            // Hide the panel's top border under the arrow base.
            painter.line_segment(
                [
                    egui::pos2(cx - half_base + 0.5, base_y),
                    egui::pos2(cx + half_base - 0.5, base_y),
                ],
                egui::Stroke::new(1.0, p.card),
            );
        });
    });
}

fn render_callout() {
    render("callout", |ui| {
        background(ui, |ui| {
            ui.set_min_width(720.0);
            ui.set_max_width(720.0);

            Callout::new(CalloutTone::Info)
                .title("Node editing is in preview.")
                .body("The wire format may change before 1.0.")
                .show(ui, |_| {});
            ui.add_space(8.0);

            Callout::new(CalloutTone::Warning)
                .title("Unsaved changes.")
                .body("You have 3 edits that haven't been written to disk.")
                .show(ui, |ui| {
                    let _ = ui.add(Button::new("Save now").accent(Accent::Amber));
                    let _ = ui.add(Button::new("Discard").outline());
                });
            ui.add_space(8.0);

            Callout::new(CalloutTone::Success)
                .title("Deploy complete.")
                .body("Rolled out to us-east-1.")
                .show(ui, |_| {});
        });
    });
}

fn render_glyphs() {
    render("glyphs", |ui| {
        background(ui, |ui| {
            let theme = Theme::current(ui.ctx());
            let p = &theme.palette;
            egui::Grid::new("r_glyphs")
                .spacing([20.0, 6.0])
                .show(ui, |ui| {
                    use elegance::glyphs as g;
                    let icons: String = [
                        g::UPLOAD,
                        g::DOWNLOAD,
                        g::SEARCH,
                        g::PIN,
                        g::COPY,
                        g::CIRCLE_ALERT,
                        g::NETWORK,
                        g::ZOOM_IN,
                        g::ZOOM_OUT,
                        g::POWER,
                    ]
                    .iter()
                    .map(|c| format!("{c} "))
                    .collect();
                    for (label, glyphs) in [
                        ("Arrows", "← ↑ → ↓ ↩ ↲ ↵"),
                        ("Ellipsis", "⋮ ⋯"),
                        ("Modifier keys", "⌃ ⌘ ⌥ ⇧ ⇪"),
                        ("Editing keys", "⌫ ⌦ ⌧ ⏎ ⇥"),
                        ("Triangles", "▴ ▸ ▾ ◂"),
                        ("Status", "✓ ✗"),
                        ("Icons", icons.trim_end()),
                    ] {
                        ui.add(egui::Label::new(theme.muted_text(label)));
                        ui.add(egui::Label::new(
                            egui::RichText::new(glyphs)
                                .color(p.text)
                                .size(theme.typography.body + 4.0)
                                .monospace(),
                        ));
                        ui.end_row();
                    }
                });
        });
    });
}

fn render_theming() {
    render("theming", |ui| {
        // Skip the standard `background()` helper — it installs Slate, but we
        // want each cell to render against its own theme's palette.
        Theme::slate().install(ui.ctx());
        let outer = Theme::current(ui.ctx());
        egui::Frame::new()
            .fill(outer.palette.bg)
            .inner_margin(egui::Margin::same(TILE_PADDING))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 8.0);
                ui.horizontal(|ui| {
                    for (name, theme) in [
                        ("Slate", Theme::slate()),
                        ("Frost", Theme::frost()),
                        ("Charcoal", Theme::charcoal()),
                        ("Paper", Theme::paper()),
                    ] {
                        theme.clone().install(ui.ctx());
                        let p = theme.palette;
                        egui::Frame::new()
                            .fill(p.bg)
                            .stroke(egui::Stroke::new(1.0, p.border))
                            .corner_radius(egui::CornerRadius::same(theme.card_radius as u8))
                            .inner_margin(egui::Margin::same(14))
                            .show(ui, |ui| {
                                // Frame.show inherits the parent's horizontal
                                // layout — wrap in vertical so widgets stack.
                                ui.vertical(|ui| {
                                    ui.set_min_width(170.0);
                                    ui.set_max_width(170.0);
                                    ui.spacing_mut().item_spacing.y = 8.0;
                                    ui.add(egui::Label::new(
                                        egui::RichText::new(name)
                                            .color(p.text)
                                            .strong()
                                            .size(theme.typography.heading),
                                    ));
                                    ui.horizontal(|ui| {
                                        let _ = ui.add(Button::new("Save").accent(Accent::Blue));
                                        let _ = ui.add(Button::new("Cancel").outline());
                                    });
                                    ui.add(
                                        StatusPill::new()
                                            .item("UI", IndicatorState::On)
                                            .item("API", IndicatorState::Connecting),
                                    );
                                    ui.add(Badge::new("v1.0", BadgeTone::Info));
                                });
                            });
                    }
                });
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

fn hero_service_row(
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
