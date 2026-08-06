//! Text-layout regressions in widgets that paint their own text.

use egui::{Context, RawInput, Rect, Shape, Vec2, pos2};
use elegance::{Theme, Toast, Toasts};

/// Walk a shape tree, since a `Ui`'s output nests painted shapes under
/// `Shape::Vec`. `dyn` rather than `impl Trait` so the recursion monomorphizes.
fn visit(shape: &Shape, f: &mut dyn FnMut(&Shape)) {
    match shape {
        Shape::Vec(shapes) => {
            for shape in shapes {
                visit(shape, f);
            }
        }
        other => f(other),
    }
}

/// Toast text must never be justified.
///
/// `Ui::put` imposes `Layout::centered_and_justified`, which `Label` copies into
/// `LayoutJob::justify`; justification then stretches every row but the last out
/// to the full wrap width, letter-spacing any row that holds a single long word.
/// The description below wraps exactly that way: the path cannot share row one.
#[test]
fn toast_text_is_never_justified() {
    let ctx = Context::default();
    Theme::slate().install(&ctx);

    let input = RawInput {
        screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), Vec2::new(600.0, 400.0))),
        ..Default::default()
    };
    let frame = |ui: &mut egui::Ui| {
        Toast::new("Export finished")
            .description("Saved /var/tmp/exports/quarterly-report-archive.tar.gz")
            .show(ui.ctx());
        Toasts::new().render(ui.ctx());
    };
    // Two frames: the first enqueues and lets the toast Area settle.
    //
    // `TexturesDelta` asserts on drop that its deltas were applied — a real
    // integration uploads them to the GPU. This test only inspects shapes, so
    // discard them explicitly instead.
    let mut output = ctx.run_ui(input.clone(), frame);
    output.textures_delta.clear();
    let mut output = ctx.run_ui(input, frame);
    output.textures_delta.clear();

    let mut wrapped = false;
    for clipped in &output.shapes {
        visit(&clipped.shape, &mut |shape| {
            if let Shape::Text(text) = shape {
                assert!(
                    !text.galley.job.justify,
                    "justified toast text: {:?}",
                    text.galley.job.text
                );
                wrapped |= text.galley.rows.len() > 1;
            }
        });
    }
    // Guards the case above: a description that stopped wrapping would leave
    // the test passing without ever exercising a non-final row.
    assert!(wrapped, "the toast description did not wrap");
}
