#!/usr/bin/env python3
"""Rasterize Twemoji SVGs to PNGs at multiple strike sizes."""

import os
import subprocess
import sys
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path

SVG_DIR = Path("twemoji/assets/svg")
PNG_DIR = Path("pngs")
SIZES = [20, 26, 32, 40, 48, 52, 64, 96, 160]


def rasterize_one(args):
    svg_path, size, out_path = args
    try:
        subprocess.run(
            ["rsvg-convert", "-w", str(size), "-h", str(size), str(svg_path), "-o", str(out_path)],
            check=True,
            capture_output=True,
        )
        return None
    except subprocess.CalledProcessError as e:
        return f"Failed {svg_path} at {size}px: {e.stderr.decode()}"


def main():
    if not SVG_DIR.exists():
        print(f"Error: {SVG_DIR} not found. Clone twemoji first.")
        sys.exit(1)

    svgs = sorted(SVG_DIR.glob("*.svg"))
    print(f"Found {len(svgs)} SVGs")

    # Create output directories
    for size in SIZES:
        (PNG_DIR / str(size)).mkdir(parents=True, exist_ok=True)

    # Build work items
    tasks = []
    for svg in svgs:
        name = svg.stem
        for size in SIZES:
            out = PNG_DIR / str(size) / f"{name}.png"
            if not out.exists():
                tasks.append((svg, size, out))

    if not tasks:
        print("All PNGs already exist, nothing to do.")
        return

    print(f"Rasterizing {len(tasks)} images ({len(svgs)} SVGs × {len(SIZES)} sizes, skipping existing)...")

    workers = os.cpu_count() or 4
    done = 0
    errors = []

    with ProcessPoolExecutor(max_workers=workers) as pool:
        futures = {pool.submit(rasterize_one, t): t for t in tasks}
        for future in as_completed(futures):
            result = future.result()
            if result:
                errors.append(result)
            done += 1
            if done % 1000 == 0:
                print(f"  {done}/{len(tasks)} done...")

    print(f"Done: {done - len(errors)} succeeded, {len(errors)} failed")
    if errors:
        print("First 10 errors:")
        for e in errors[:10]:
            print(f"  {e}")


if __name__ == "__main__":
    main()
