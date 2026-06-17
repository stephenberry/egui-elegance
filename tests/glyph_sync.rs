//! Enforces that the bundled glyph set stays internally consistent.
//!
//! A glyph lives in three places that must agree:
//!   * `LUCIDE_GLYPHS` in `scripts/update_lucide_glyphs.py` — the codepoints
//!     actually baked into `assets/elegance-symbols.ttf`.
//!   * the `pub const`s in the `glyphs` module of `src/lib.rs` — the public
//!     API consumers reference.
//!
//! If a const is added without a matching bake entry, that codepoint renders
//! as tofu at runtime; if a bake entry has no const, the glyph is baked but
//! unreachable through the public API. This test fails loudly on either drift
//! so the mismatch is caught at `cargo test` rather than in a downstream app.
//!
//! The two lists use different names on purpose (`key-round` in the bake
//! script, `KEY` in the API), so they are matched by codepoint, which is the
//! invariant that actually matters.

use std::collections::BTreeSet;
use std::path::Path;

/// Parse the `0xNNNN` codepoints from the `LUCIDE_GLYPHS = [ ... ]` literal
/// in the bake script.
fn baked_codepoints(script: &str) -> BTreeSet<u32> {
    let body = script
        .split_once("LUCIDE_GLYPHS = [")
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(list, _)| list)
        .expect("bake script must contain a `LUCIDE_GLYPHS = [ ... ]` literal");

    body.match_indices("0x")
        .map(|(i, _)| {
            let hex: String = body[i + 2..]
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            u32::from_str_radix(&hex, 16)
                .unwrap_or_else(|_| panic!("malformed codepoint near `0x{hex}`"))
        })
        .collect()
}

/// Parse the codepoints from `pub const NAME: char = '\u{NNNN}';` declarations
/// in the `glyphs` module of `src/lib.rs`.
fn const_codepoints(lib: &str) -> BTreeSet<u32> {
    let module = lib
        .split_once("pub mod glyphs {")
        .and_then(|(_, rest)| rest.split_once("\n}"))
        .map(|(body, _)| body)
        .expect("src/lib.rs must contain a `pub mod glyphs { ... }`");

    module
        .match_indices("'\\u{")
        .map(|(i, marker)| {
            let start = i + marker.len();
            let hex: String = module[start..]
                .chars()
                .take_while(|c| c.is_ascii_hexdigit())
                .collect();
            u32::from_str_radix(&hex, 16)
                .unwrap_or_else(|_| panic!("malformed codepoint near `\\u{{{hex}}}`"))
        })
        .collect()
}

#[test]
fn glyph_lists_stay_in_sync() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = std::fs::read_to_string(root.join("scripts/update_lucide_glyphs.py"))
        .expect("read scripts/update_lucide_glyphs.py");
    let lib = std::fs::read_to_string(root.join("src/lib.rs")).expect("read src/lib.rs");

    let baked = baked_codepoints(&script);
    let consts = const_codepoints(&lib);

    // Sanity: each parser must have found the known glyphs, so a parsing
    // failure can't masquerade as "in sync" by finding nothing on both sides.
    assert!(
        baked.len() >= 14,
        "expected to parse the full bake list, found only {} codepoints",
        baked.len(),
    );
    assert_eq!(
        baked.len(),
        consts.len(),
        "bake script defines {} glyphs but the `glyphs` module exposes {}",
        baked.len(),
        consts.len(),
    );

    let baked_only: Vec<_> = baked
        .difference(&consts)
        .map(|c| format!("U+{c:04X}"))
        .collect();
    let const_only: Vec<_> = consts
        .difference(&baked)
        .map(|c| format!("U+{c:04X}"))
        .collect();

    assert!(
        baked_only.is_empty(),
        "baked into the font but missing a `glyphs` const (unreachable glyphs): {baked_only:?}",
    );
    assert!(
        const_only.is_empty(),
        "have a `glyphs` const but not baked into the font (renders as tofu): {const_only:?}",
    );
}
