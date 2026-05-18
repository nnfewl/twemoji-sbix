#!/usr/bin/env python3
"""Validate the built Twemoji sbix font."""

import sys
from pathlib import Path

from fontTools.ttLib import TTFont

FONT_PATH = "Twemoji.ttc"


def main():
    if not Path(FONT_PATH).exists():
        print(f"Error: {FONT_PATH} not found. Run build_font.py first.")
        sys.exit(1)

    font = TTFont(FONT_PATH)

    print("=== Font Validation ===\n")

    # Basic info
    family = font["name"].getDebugName(1)
    print(f"Family name: {family}")
    print(f"Tables: {sorted(font.keys())}")
    print(f"Glyph count: {len(font.getGlyphOrder())}")

    # cmap
    cmap = font.getBestCmap()
    if cmap:
        print(f"cmap entries: {len(cmap)}")
        # Sample some well-known emoji
        test_cps = {
            0x1F600: "grinning face",
            0x1F680: "rocket",
            0x2764: "heart",
            0x1F1FA: "regional indicator U",
            0x1F44B: "waving hand",
            0x26A0: "warning",
        }
        print("\nSample cmap lookups:")
        for cp, desc in test_cps.items():
            glyph = cmap.get(cp)
            status = f"✓ → {glyph}" if glyph else "✗ MISSING"
            print(f"  U+{cp:04X} ({desc}): {status}")
    else:
        print("WARNING: No cmap found!")

    # sbix
    if "sbix" in font:
        sbix = font["sbix"]
        print(f"\nsbix strikes: {sorted(sbix.strikes.keys())}")
        for ppem in sorted(sbix.strikes.keys()):
            strike = sbix.strikes[ppem]
            glyph_count = len([g for g in strike.glyphs.values() if g.imageData])
            print(f"  {ppem}px: {glyph_count} glyphs with image data")

        # Check a specific glyph has image data
        largest = max(sbix.strikes.keys())
        strike = sbix.strikes[largest]
        if "u1F600" in strike.glyphs:
            glyph = strike.glyphs["u1F600"]
            print(f"\nSample glyph u1F600 at {largest}px:")
            print(f"  graphicType: {glyph.graphicType}")
            print(f"  imageData size: {len(glyph.imageData)} bytes")
            # Verify PNG header
            if glyph.imageData[:4] == b"\x89PNG":
                print("  Valid PNG header: ✓")
            else:
                print("  WARNING: Invalid PNG header!")
    else:
        print("\nWARNING: No sbix table found!")

    # Check for morx (ligature sequences)
    if "morx" in font:
        print("\nmorx table: present (ZWJ/flag sequences supported)")
    else:
        print("\nmorx table: absent (ZWJ/flag sequences won't render as composed emoji)")

    # Size info
    size_mb = Path(FONT_PATH).stat().st_size / (1024 * 1024)
    print(f"\nFile size: {size_mb:.1f} MB")

    print("\n=== Validation complete ===")
    font.close()


if __name__ == "__main__":
    main()
