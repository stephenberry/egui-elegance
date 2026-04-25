# Bundled assets

## `elegance-symbols.ttf`

A ~13 KB subset of DejaVu Sans containing just the glyphs elegance widgets
and consumers are likely to inline in text: arrows, math ellipsis, modifier
keys, delete keys, return arrows, disclosure triangles, check / cross.

The source font, DejaVu Sans, is derived from Bitstream Vera. Its license
permits redistribution of modified/subset fonts provided the font is renamed
to not contain "Bitstream" or "Vera" — this subset is renamed
`Elegance Symbols`. The full license text (covering both Bitstream Vera and
Arev contributions) is preserved in `elegance-symbols-LICENSE.txt`.

### Glyph set

| Codepoint | Glyph | Description               |
| --------- | ----- | ------------------------- |
| U+2190    | ←     | leftwards arrow           |
| U+2191    | ↑     | upwards arrow             |
| U+2192    | →     | rightwards arrow          |
| U+2193    | ↓     | downwards arrow           |
| U+21A9    | ↩     | leftwards arrow with hook |
| U+21B2    | ↲     | downwards arrow with corner left |
| U+21B5    | ↵     | downwards arrow with corner leftwards |
| U+21E5    | ⇥     | rightwards arrow to bar (tab key) |
| U+21E7    | ⇧     | upwards white arrow (shift key) |
| U+21EA    | ⇪     | upwards white arrow from bar (caps lock) |
| U+22EE    | ⋮     | vertical ellipsis         |
| U+22EF    | ⋯     | midline horizontal ellipsis |
| U+2303    | ⌃     | up arrowhead (control key) |
| U+2318    | ⌘     | place of interest (command key) |
| U+2325    | ⌥     | option key                |
| U+2326    | ⌦     | erase to the right (forward delete) |
| U+2327    | ⌧     | x in rectangle (clear key) |
| U+232B    | ⌫     | erase to the left (backspace) |
| U+23CE    | ⏎     | return symbol             |
| U+25B4    | ▴     | small up-pointing triangle |
| U+25B8    | ▸     | small right-pointing triangle |
| U+25BE    | ▾     | small down-pointing triangle |
| U+25C2    | ◂     | small left-pointing triangle |
| U+2713    | ✓     | check mark                |
| U+2717    | ✗     | ballot x                  |

### Regenerating the subset

To add more glyphs or update the source font, re-run the subset step with
the extended codepoint list:

```sh
# Download source
curl -LO https://github.com/dejavu-fonts/dejavu-fonts/releases/download/version_2_37/dejavu-fonts-ttf-2.37.tar.bz2
tar -xjf dejavu-fonts-ttf-2.37.tar.bz2

# Subset
pip install fonttools
pyftsubset dejavu-fonts-ttf-2.37/ttf/DejaVuSans.ttf \
    --output-file=assets/elegance-symbols.ttf \
    --unicodes='U+2190-2193,U+21A9,U+21B2,U+21B5,U+21E5,U+21E7,U+21EA,U+22EE,U+22EF,U+2303,U+2318,U+2325,U+2327,U+232B,U+2326,U+23CE,U+25B4,U+25B8,U+25BE,U+25C2,U+2713,U+2717' \
    --no-hinting --desubroutinize --name-IDs='*'

# Then rename via fontTools (see scripts/rename-symbol-font.py if present).
```

Any new codepoints must also be added to the table above and any consumer-
facing documentation that lists the bundled glyphs.
