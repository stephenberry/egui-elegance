# Contributing to egui-elegance

New widgets: see [`dev/adding_a_widget.md`](dev/adding_a_widget.md) for the step-by-step guide.

## Regenerating widget screenshots

The PNGs under `docs/images/` are rendered headlessly by an [`egui_kittest`](https://crates.io/crates/egui_kittest)-based binary. Regenerate them after any visual change with:

```sh
cargo render-docs
```

Each widget category is laid out in its natural size against the theme background at 2× DPI, fitted to its content via `Harness::fit_contents`. Edit `examples/render_docs.rs` to add tiles or adjust layouts.

## Visual regression tests

`tests/visual.rs` renders every widget against every built-in theme (44 combinations) and pixel-diffs the result against baseline PNGs under `tests/snapshots/`. `cargo test` fails if anything drifts, surfacing the exact widget/theme that changed along with a `.new.png` and `.diff.png` beside the baseline.

After an intentional visual change, regenerate the affected baselines:

```sh
UPDATE_SNAPSHOTS=true cargo test --test visual
```

Eyeball the new PNGs under `tests/snapshots/` before committing.
