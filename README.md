# egui-elegance

[![CI](https://github.com/stephenberry/egui-elegance/actions/workflows/ci.yml/badge.svg)](https://github.com/stephenberry/egui-elegance/actions/workflows/ci.yml)

Opinionated widgets for [`egui`]: six-accent rounded buttons, text inputs with a sky focus ring and submit-flash feedback, themed selects and tabs, segmented LED toggles, status pills, and badges — all driven by a single installable `Theme`. Four palettes ship built-in — two dark (`Theme::slate`, `Theme::charcoal`) and two light (`Theme::frost`, `Theme::paper`) — paired so you can toggle without any layout shift.

The design aims to make native apps feel as polished as modern web UIs.

![Buttons](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/buttons.png)

## Install

```sh
cargo add egui-elegance
```

or, in `Cargo.toml`:

```toml
[dependencies]
egui          = "0.34"
egui-elegance = "0.1"
```

The crate is published as `egui-elegance` but the library name is `elegance`, so imports look like `use elegance::Button;`.

MSRV: Rust 1.92.

## Quick start

Install the theme once per `Context`, then drop widgets into any `Ui` the way you would an egui built-in:

```rust
use elegance::{Accent, Button, Card, Checkbox, TextInput, Theme};

struct App {
    email: String,
    remember: bool,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        Theme::slate().install(ui.ctx()); // cheap to call every frame — skips work when unchanged

        egui::CentralPanel::default().show_inside(ui, |ui| {
            Card::new().heading("Account").show(ui, |ui| {
                ui.add(
                    TextInput::new(&mut self.email)
                        .label("Email")
                        .hint("you@example.com"),
                );
                ui.add(Checkbox::new(&mut self.remember, "Keep me signed in"));
                if ui.add(Button::new("Save").accent(Accent::Green)).clicked() {
                    // …
                }
            });
        });
    }
}
```

## Widgets

Every widget follows one of three usage patterns:

- **Leaf widgets** — including stateful ones that take a `&mut T` in their constructor like `TextInput::new(&mut email)` or `Select::new(id, &mut unit)` — implement `egui::Widget` and render with `ui.add(…)`.
- **Container widgets** (`Card`, `CollapsingSection`) take a body closure with `.show(ui, |ui| …)` and return an `InnerResponse<R>`.
- **Overlay widgets** create their own top-level `Area`s and render at `Context` scope: `Modal::new("id", &mut open).show(ctx, |ui| …)` for a dialog, `Toast::new("…").show(ctx)` to enqueue a notification paired with `Toasts::new().render(ctx)` once per frame to draw the stack, and `LogBar` — owned state on your app struct — rendered once per frame with `log.show(ui)`.

Reference for each widget follows. Tiles are rendered headlessly by `cargo render-docs` — see [Regenerating widget screenshots](#regenerating-widget-screenshots).

### Button

![Buttons](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/buttons.png)

Chunky rounded button in six accent colours plus an outline variant, in three sizes.

```rust
use elegance::{Accent, Button, ButtonSize};

if ui.add(Button::new("Save").accent(Accent::Green)).clicked() {
    // …
}
ui.add(Button::new("Cancel").outline().size(ButtonSize::Small));
ui.add(Button::new("Disabled").accent(Accent::Blue).enabled(false));
```

### TextInput

![Text inputs — normal, hint, dirty, password](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/text_inputs.png)

Single-line text input. See also [Submit-flash feedback](#submit-flash-feedback) for success / error tinting on submit.

```rust
use elegance::TextInput;

ui.add(
    TextInput::new(&mut email)
        .label("Email")
        .hint("you@example.com"),
);
ui.add(TextInput::new(&mut secret).label("API key").password(true));
ui.add(TextInput::new(&mut name).label("Name").dirty(true));
```

### TextArea

![Text areas — regular and monospace](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/text_areas.png)

Multi-line counterpart to `TextInput` with a configurable visible row count. Optional monospace for code, JSON, or keys.

```rust
use elegance::TextArea;

ui.add(
    TextArea::new(&mut notes)
        .label("Notes")
        .hint("Jot anything down…")
        .rows(6),
);
ui.add(TextArea::new(&mut json).monospace(true).rows(8));
```

### Select

![Selects](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/selects.png)

Themed combo-box generic over any `PartialEq + Clone` value type.

```rust
use elegance::Select;

#[derive(Clone, PartialEq)]
enum Unit { Us, Ms, S }

ui.add(
    Select::new("unit", &mut unit)
        .options([(Unit::Us, "μs"), (Unit::Ms, "ms"), (Unit::S, "s")]),
);

// Shorthand for string-valued selects:
ui.add(Select::strings("env", &mut env, ["Production", "Staging", "Development"]));
```

### Checkbox · Switch · SegmentedButton

![Toggles — checkbox, switch, segmented](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/toggles.png)

Three flavours of boolean input. Pick `Checkbox` for list-style selection, `Switch` for feature/settings flags, `SegmentedButton` for mode toggles where the on-state should read as a distinct accent pill.

```rust
use elegance::{Accent, Checkbox, SegmentedButton, Switch};

ui.add(Checkbox::new(&mut remember, "Keep me signed in"));
ui.add(Switch::new(&mut notify, "Notify on Slack").accent(Accent::Green));
ui.add(
    SegmentedButton::new(&mut continuous, "Continuous")
        .accent(Accent::Green)
        .min_width(120.0),
);
```

`SegmentedButton` accepts the same `ButtonSize` scale as `Button`, so a mixed row (e.g. `Button::new("Collect")` next to `SegmentedButton::new(&mut continuous, "Continuous")`) stays aligned at any size. Pass matching `.size(ButtonSize::Large)` on both for a chunkier action row without touching the theme.

### TabBar

![TabBar](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/tabs.png)

Horizontal tab strip. The active tab gets a sky underline.

```rust
use elegance::TabBar;

ui.add(TabBar::new(&mut tab, ["Overview", "Settings", "Activity", "Logs"]));
```

### StatusPill · Indicator · Badge

![Status](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/status.png)

`IndicatorState` has three visual modes: `On` (solid green dot), `Off` (red bar), `Connecting` (amber ring). `Badge` carries a `BadgeTone`: `Ok`, `Warning`, `Danger`, `Info`, or `Neutral`.

```rust
use elegance::{Badge, BadgeTone, Indicator, IndicatorState, StatusPill};

ui.add(
    StatusPill::new()
        .item("UI", IndicatorState::On)
        .item("API", IndicatorState::Connecting)
        .item("DB", IndicatorState::Off),
);
ui.add(Indicator::new(IndicatorState::On));
ui.add(Badge::new("Dev build", BadgeTone::Info));
```

### Slider

![Sliders](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/sliders.png)

Pill-track slider generic over `egui::emath::Numeric` — works with any integer or float type. Value readout on the right; `.value_fmt(|v| …)` for custom formatting.

```rust
use elegance::{Accent, Slider};

ui.add(
    Slider::new(&mut cpu, 0.0..=100.0)
        .label("CPU limit")
        .suffix("%")
        .accent(Accent::Green),
);
ui.add(Slider::new(&mut port, 0u16..=65535u16).label("Port"));
```

### Spinner · ProgressBar

![Spinners and progress bars](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/feedback.png)

`Spinner` is the indeterminate loader — an animated sweeping arc. `ProgressBar` is determinate: a pill-shaped bar with an optional inline label.

```rust
use elegance::{Accent, ProgressBar, Spinner};

ui.add(Spinner::new().size(20.0).accent(Accent::Green));

ui.add(ProgressBar::new(0.6));
ui.add(ProgressBar::new(1.0).accent(Accent::Amber).text("Complete"));
```

### Steps

A stepped progress indicator for discrete, countable stages. Three visual styles share the same state model (`total`, `current`, `errored`): `StepsStyle::Cells` paints a segmented bar of uniform rounded cells, suited to compact "N of M" progress. `StepsStyle::Numbered` paints numbered circles connected by thin lines, with a checkmark on completed dots and a glow on the active one. `StepsStyle::Labeled` (via `Steps::labeled`) paints taller pills containing text labels — horizontal by default (a progress bar with readable stage names), or call `.vertical()` for a wizard-sidebar layout. Done cells use the theme's success green, the active one uses sky, and errors use danger red.

```rust
use elegance::{Steps, StepsStyle};

// 4 of 6 release steps complete, step 5 running.
ui.add(Steps::new(6).current(4));

// Migration failed on step 3 of 5.
ui.add(Steps::new(5).current(2).errored(true));

// Onboarding wizard, step 3 of 5.
ui.add(Steps::new(5).current(2).style(StepsStyle::Numbered));

// Labeled horizontal strip — a progress bar with stage names.
ui.add(Steps::labeled(["Plan", "Build", "Test", "Deploy"]).current(2));

// Same data, rendered as a vertical wizard sidebar.
ui.add(
    Steps::labeled(["Plan", "Design", "Build", "Test", "Deploy"])
        .current(2)
        .vertical(),
);
```

### Card · CollapsingSection

![Containers](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/containers.png)

Both take a body closure and return an `InnerResponse<R>`.

```rust
use elegance::{Card, CollapsingSection};

Card::new().heading("Account").show(ui, |ui| {
    ui.label("…card contents…");
});

CollapsingSection::new("advanced", "Show advanced options").show(ui, |ui| {
    ui.label("…hidden until expanded…");
});
```

### Menu · MenuItem

![Menu](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/menu.png)

Click-to-open popup attached to any trigger `Response`. `Esc`, outside-click, or item-click all dismiss.

```rust
use elegance::{Button, ButtonSize, Menu, MenuItem};

let trigger = ui.add(Button::new("⋯").outline().size(ButtonSize::Small));
Menu::new("row_actions").show_below(&trigger, |ui| {
    if ui.add(MenuItem::new("Edit").shortcut("⌘ E")).clicked() { /* … */ }
    if ui.add(MenuItem::new("Duplicate").shortcut("⌘ D")).clicked() { /* … */ }
    ui.separator();
    if ui.add(MenuItem::new("Delete").danger()).clicked() { /* … */ }
});
```

### Modal

![Modal](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/modal.png)

Centered dialog over a dimmed backdrop. `Esc`, backdrop-click, or the built-in × button all flip the bound `open` flag back to `false`.

```rust
use elegance::Modal;

Modal::new("stats", &mut open)
    .heading("Run Summary")
    .show(ctx, |ui| {
        ui.label("…");
    });
```

### Popover

Click-anchored floating panel that points at a trigger. Lighter than `Modal`: no backdrop, no focus trap. Pick a side with `PopoverSide` (top, bottom, left, right), optionally set a `title`, and fill the body closure with whatever you like. `Esc`, outside-click, or a second trigger-click dismiss.

```rust
use elegance::{Accent, Button, ButtonSize, Popover, PopoverSide};

let trigger = ui.add(Button::new("Delete branch").outline());
Popover::new("delete_branch")
    .side(PopoverSide::Bottom)
    .title("Delete feature/snap-baseline?")
    .show(&trigger, |ui| {
        ui.label("This removes the branch from origin too.");
        ui.horizontal(|ui| {
            let _ = ui.add(Button::new("Cancel").outline().size(ButtonSize::Small));
            let _ = ui.add(Button::new("Delete").accent(Accent::Red).size(ButtonSize::Small));
        });
    });
```

### Callout

Full-width inline banner for persistent context: experimental features, unsaved changes, failed builds, maintenance windows. `CalloutTone` picks the accent (`Info`, `Success`, `Warning`, `Danger`, `Neutral`). The closure slot is a right-to-left action area — add primary button first. Opt into a trailing × with `.dismissable(&mut open)`.

```rust
use elegance::{Accent, Button, Callout, CalloutTone};

Callout::new(CalloutTone::Warning)
    .title("Unsaved changes.")
    .body("You have 3 edits that haven't been written to disk.")
    .show(ui, |ui| {
        let _ = ui.add(Button::new("Save now").accent(Accent::Amber));
        let _ = ui.add(Button::new("Discard").outline());
    });
```

Unlike [`Toast`](#toast--toasts) it does not auto-dismiss, and unlike [submit-flash feedback](#submit-flash-feedback) it's a whole surface rather than a pulse on another widget.

### Toast · Toasts

![Toast](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/toast.png)

Non-blocking notifications. `Toast::show(ctx)` enqueues from any callback that has `&Context`; `Toasts::new().render(ctx)` draws the stack once per frame. Auto-dismissed with fade-out after ~4 s (override with `.duration(…)` or `.persistent()`).

```rust
use elegance::{BadgeTone, Toast, Toasts};

// From any callback with `&Context`:
Toast::new("Deploy complete")
    .tone(BadgeTone::Ok)
    .description("Rolled out to us-east-1")
    .show(&ctx);

// In your top-level `ui`:
Toasts::new().render(ctx);
```

### LogBar

Expandable bottom log bar — a monospace console with timestamped rows colour-coded by kind: `Sys`, `Out` (→), `In` (←), `Err`. Owned state — construct once on your app struct, push entries from anywhere with `&mut self`, and render once per frame.

```rust
use elegance::LogBar;

// In App::default, construct once:
let mut log = LogBar::new();

// From a button handler, async callback, completion, etc.:
log.out("reload_config");
log.recv("{\"temp\":42.1}");
log.err("retry budget exceeded");

// Once per frame, inside your top-level `ui`:
log.show(ui);
```

### Pairing

![Pairing](https://raw.githubusercontent.com/stephenberry/egui-elegance/main/docs/images/pairing.png)

One-to-one pairing between two lists, drawn as bezier curves between port circles. Click a port to start a connection, then click an opposite-side port to complete it. Hovering an opposite-side node during selection latches the ghost line to its port. Clicking a paired node breaks its connection *and* starts a new pairing from it — one-click reconnection. Clicking a line unpairs. Optional `.align_left()` / `.align_right()` auto-arranges the chosen side so every pairing renders as a straight horizontal line.

Pairs are stored as `(left_id, right_id)` tuples in a caller-owned `Vec`; transient selection state lives in egui memory keyed by the widget's id salt. Each side supports up to 64 items — layout uses fixed-size stack buffers so there is zero heap allocation per frame.

```rust
use elegance::{PairItem, Pairing};

let clients = vec![
    PairItem::new("c1", "worker-pool-a").detail("24 instances"),
    PairItem::new("c2", "edge-proxy-01").detail("8 instances"),
];
let servers = vec![
    PairItem::new("s1", "api-east-01").detail("10.0.1.5 · us-east"),
    PairItem::new("s2", "api-west-01").detail("10.0.2.4 · us-west"),
];
let mut pairs: Vec<(String, String)> = vec![];

Pairing::new("client-server", &clients, &servers, &mut pairs)
    .left_label("Clients")
    .right_label("Servers")
    .align_right()
    .show(ui);
```

## Submit-flash feedback

`TextInput` can play a short green or red background flash to confirm the outcome of a submit:

```rust
use elegance::{ResponseFlashExt, TextInput};

let resp = ui.add(TextInput::new(&mut port).id_salt("port"));
if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
    match parse_port(&port) {
        Ok(_)  => resp.flash_success(),
        Err(_) => resp.flash_error(),
    }
}
```

The tint fades out over `FLASH_DURATION` (~0.8 s). `resp.clear_flash()` dismisses it early.

## Bundled glyphs

`Theme::install` registers a ~13 KB subset of DejaVu Sans (renamed `Elegance Symbols`) as a Proportional and Monospace fallback, so inline glyphs like `→`, `⋯`, `⌘`, `⌫`, `↩`, `▾` render out of the box without egui's default font missing them.

Covered blocks: arrows, math ellipsis, modifier keys (`⌘ ⌥ ⌃`), delete keys (`⌫ ⌦`), disclosure triangles, check / cross. See [`assets/README.md`](assets/README.md) for the full list and regeneration instructions.

If you need additional fonts (emoji, CJK, a different text face), register them **after** `Theme::install(ctx)` — calling `ctx.set_fonts(...)` before install will be overwritten the first time `install` runs:

```rust
Theme::slate().install(ctx);

let mut fonts = egui::FontDefinitions::default();
fonts.font_data.insert(
    "MyEmoji".into(),
    egui::FontData::from_static(include_bytes!("../assets/NotoEmoji.ttf")).into(),
);
fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
    .push("MyEmoji".into());
ctx.set_fonts(fonts);
```

## Theming

A `Theme` bundles a `Palette` of colours, a `Typography` of font sizes, and a few shape parameters (corner radius, padding). Calling `.install(ctx)` both stores the theme in `ctx` memory so elegance widgets can read it, and updates `egui::Style` so built-in widgets (labels, sliders, scroll bars) inherit the palette.

Four presets are built in, arranged as two dark/light pairs that share shape and typography so you can swap between members of a pair without a layout shift:

| Name | Mode | Flavour |
|---|---|---|
| `Theme::slate()` | dark | cool corporate blue — the default |
| `Theme::frost()` | light | slate-tinted off-white with a sky accent |
| `Theme::charcoal()` | dark | neutral dark grey with a cyan accent |
| `Theme::paper()` | light | warm off-white with a cyan accent |

The `widgets` demo switches between all four live via a header picker. Start from any preset and tweak whatever you like:

```rust
let mut theme = elegance::Theme::charcoal();
theme.palette.sky = egui::Color32::from_rgb(0xa7, 0xf3, 0xd0);
theme.card_radius = 14.0;
theme.install(ctx);
```

For the common case — a header combo-box that lets the user flip between the four presets — drop in `ThemeSwitcher`. It renders the picker and installs the selected theme each frame:

```rust
use elegance::{BuiltInTheme, ThemeSwitcher};
// In your app state:
let mut theme = BuiltInTheme::Slate;
// In your UI:
ui.add(ThemeSwitcher::new(&mut theme));
```

## Demos

An interactive showcase and a widget reference ship with the crate:

```sh
cargo orbit      # a CI/CD deployment command center
cargo widgets    # every widget in one place: a clean reference layout for screenshotting
```

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for regenerating screenshots, running visual regression tests, and adding new widgets.

## License

Dual-licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

[`egui`]: https://github.com/emilk/egui
