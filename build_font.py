#!/usr/bin/env python3
"""Build a Twemoji sbix font from rasterized PNGs."""

import argparse
import os
import sys
from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.ttLib import getTableClass
from fontTools.ttLib.tables._g_l_y_f import Glyph as GlyfGlyph
from fontTools.ttLib.tables.sbixStrike import Strike as SbixStrike
from fontTools.ttLib.tables.sbixGlyph import Glyph as SbixGlyph

from strikes import PRESETS, DEFAULT_PRESET, get_sizes, list_presets

PNG_DIR = Path("pngs")
FAMILY_NAME = "Apple Color Emoji"
UNITS_PER_EM = 2048
ADVANCE_WIDTH = 2550
ASCENT = 1900
DESCENT = 500


def codepoints_from_filename(name: str) -> list[int]:
    """Parse '1f600' or '1f1fa-1f1f8' into list of ints."""
    return [int(cp, 16) for cp in name.split("-")]


def glyph_name_from_filename(name: str) -> str:
    """Convert filename to glyph name: '1f600' → 'u1F600'."""
    parts = name.upper().split("-")
    return "u" + "_".join(parts)


def build_font(sizes: list[int], output: str):
    largest = max(sizes)
    largest_dir = PNG_DIR / str(largest)
    if not largest_dir.exists():
        print(f"Error: {largest_dir} not found. Run rasterize.py with matching preset first.")
        sys.exit(1)

    png_files = sorted(largest_dir.glob("*.png"))
    if not png_files:
        print(f"Error: No PNGs found in {largest_dir}")
        sys.exit(1)

    print(f"Found {len(png_files)} glyphs in {largest_dir}")

    # Separate single-codepoint and multi-codepoint emoji
    single_cp_glyphs = []
    multi_cp_glyphs = []

    for png in png_files:
        name = png.stem
        cps = codepoints_from_filename(name)
        glyph_name = glyph_name_from_filename(name)
        if len(cps) == 1:
            single_cp_glyphs.append((cps[0], glyph_name, name))
        else:
            multi_cp_glyphs.append((glyph_name, name))

    glyph_names = [".notdef"] + [g[1] for g in single_cp_glyphs] + [g[0] for g in multi_cp_glyphs]
    print(f"Total glyphs: {len(glyph_names)}")
    print(f"  Single-codepoint (in cmap): {len(single_cp_glyphs)}")
    print(f"  Multi-codepoint (need morx/GSUB): {len(multi_cp_glyphs)}")

    cmap_dict = {cp: glyph_name for cp, glyph_name, _ in single_cp_glyphs}

    fb = FontBuilder(UNITS_PER_EM, isTTF=True)
    fb.setupGlyphOrder(glyph_names)
    fb.setupCharacterMap(cmap_dict)

    metrics = {name: (ADVANCE_WIDTH, 0) for name in glyph_names}
    fb.setupHorizontalMetrics(metrics)
    fb.setupHorizontalHeader(ascent=ASCENT, descent=-DESCENT)

    fb.setupGlyf({})
    glyf = fb.font["glyf"]
    for name in glyph_names:
        glyf[name] = GlyfGlyph()

    fb.setupNameTable({
        "familyName": FAMILY_NAME,
        "styleName": "Regular",
    })

    fb.setupOS2(
        sTypoAscender=ASCENT,
        sTypoDescender=-DESCENT,
        sTypoLineGap=0,
        usWinAscent=ASCENT,
        usWinDescent=DESCENT,
        sxHeight=0,
        sCapHeight=0,
        usWeightClass=400,
        usWidthClass=5,
    )

    fb.setupPost()
    fb.font["head"].flags = 0x000B

    # --- sbix table ---
    print(f"Building sbix table ({len(sizes)} strikes)...")
    sbix = getTableClass("sbix")("sbix")
    sbix.version = 1
    sbix.flags = 1
    sbix.strikes = {}

    glyph_order_set = set(glyph_names)

    for ppem in sizes:
        size_dir = PNG_DIR / str(ppem)
        if not size_dir.exists():
            print(f"  Warning: {size_dir} not found, skipping strike {ppem}")
            continue

        strike_glyphs = {}
        for png_path in sorted(size_dir.glob("*.png")):
            name = png_path.stem
            glyph_name = glyph_name_from_filename(name)
            if glyph_name not in glyph_order_set:
                continue
            with open(png_path, "rb") as f:
                glyph = SbixGlyph(
                    glyphName=glyph_name,
                    graphicType="png ",
                    originOffsetX=0,
                    originOffsetY=0,
                    imageData=f.read(),
                )
                strike_glyphs[glyph_name] = glyph

        strike = SbixStrike()
        strike.ppem = ppem
        strike.resolution = 72
        strike.glyphs = strike_glyphs
        sbix.strikes[ppem] = strike
        print(f"  Strike {ppem}px: {len(strike_glyphs)} glyphs")

    fb.font["sbix"] = sbix

    fb.font.save(output)
    file_size = os.path.getsize(output) / (1024 * 1024)
    print(f"\nSaved: {output} ({file_size:.1f} MB)")
    print(f"  Family: {FAMILY_NAME}")
    print(f"  Strikes: {sizes}")
    print(f"  cmap entries: {len(cmap_dict)}")


def main():
    parser = argparse.ArgumentParser(description="Build Twemoji sbix font.")
    parser.add_argument(
        "--preset", default=DEFAULT_PRESET, choices=PRESETS.keys(),
        help=f"Strike size preset (default: {DEFAULT_PRESET})",
    )
    parser.add_argument(
        "--sizes", type=int, nargs="+",
        help="Custom strike sizes (overrides --preset)",
    )
    parser.add_argument(
        "--output", "-o", default="Twemoji.ttc",
        help="Output file path (default: Twemoji.ttc)",
    )
    parser.add_argument(
        "--list-presets", action="store_true",
        help="Show available presets and exit",
    )
    args = parser.parse_args()

    if args.list_presets:
        print("Available presets:\n")
        print(list_presets())
        return

    sizes = args.sizes if args.sizes else get_sizes(args.preset)
    print(f"Preset: {args.preset} → strikes {sizes}")
    build_font(sizes, args.output)


if __name__ == "__main__":
    main()
