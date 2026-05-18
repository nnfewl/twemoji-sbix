# twemoji-sbix

Build [Twemoji](https://github.com/jdecked/twemoji) as an sbix font for macOS Core Text.

Produces a `.ttc` font with the family name "Apple Color Emoji" that can shadow
the system emoji font when registered at process scope — useful for patching
terminal emulators (iTerm2, Ghostty) to show Twemoji instead of Apple emoji.

## Requirements

- Python 3.12+
- [uv](https://docs.astral.sh/uv/)
- `rsvg-convert` (from librsvg) for SVG rasterization

## Usage

```bash
# Clone Twemoji assets
git clone --depth 1 https://github.com/jdecked/twemoji

# Rasterize SVGs to PNGs at all strike sizes
uv run rasterize.py

# Build the sbix font
uv run build_font.py

# Validate output
uv run validate.py
```

## Output

`Twemoji.ttc` — an sbix font containing:

- 4009 glyphs (all Twemoji emoji)
- 9 bitmap strikes: 20, 26, 32, 40, 48, 52, 64, 96, 160px
- 1427 single-codepoint emoji mapped in cmap
- Family name: "Apple Color Emoji"

## Limitations

Multi-codepoint sequences (ZWJ families, flags, skin tone modifiers) require a
`morx` (AAT) or `GSUB` (OpenType) ligature table to compose into a single glyph.
Without it, individual base emoji render correctly but sequences show as separate
characters.

## Scripts

| Script | Purpose |
|--------|---------|
| `rasterize.py` | Parallel SVG-to-PNG rasterization at 9 sizes (~36K images) |
| `build_font.py` | Pack PNGs into an sbix font using fonttools FontBuilder |
| `validate.py` | Verify font structure, cmap coverage, and PNG integrity |

## License

Twemoji graphics are licensed under [CC-BY 4.0](https://creativecommons.org/licenses/by/4.0/).
Code in this repo is MIT.
