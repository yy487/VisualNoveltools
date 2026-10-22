#!/usr/bin/env python
"""
Batch decoder for COptimizedObs resources.

Usage:
  python run_obs_decode.py [--dir unpacked] [--output png_output] [--files file1 file2 ...]
"""

import sys
import argparse
from pathlib import Path

# Ensure obs_decoder package is importable
sys.path.insert(0, str(Path(__file__).resolve().parent))

from obs_decoder import decode_obs_file, save_png
from obs_decoder.pipeline import ObsPipeline


def main():
    parser = argparse.ArgumentParser(
        description="Decode rUGP COptimizedObs resources to PNG"
    )
    parser.add_argument(
        "--dir", default="unpacked",
        help="Directory containing extracted .optimizedobs files"
    )
    parser.add_argument(
        "--output", default="png_output",
        help="Output directory for PNG files"
    )
    parser.add_argument(
        "--files", nargs="*",
        help="Specific files to decode"
    )
    parser.add_argument(
        "--width", type=int, default=640,
        help="Output image width (default: 640)"
    )
    parser.add_argument(
        "--height", type=int, default=480,
        help="Output image height (default: 480)"
    )
    parser.add_argument(
        "--limit", type=int, default=0,
        help="Limit number of files to process (0=all)"
    )
    parser.add_argument(
        "--stats", action="store_true",
        help="Show detailed statistics for each file"
    )
    args = parser.parse_args()

    # Find files
    base = Path(args.dir)
    if not base.is_absolute():
        base = Path(__file__).resolve().parent / base

    if args.files:
        obs_files = [Path(f) for f in args.files]
    else:
        obs_files = sorted(
            f for f in base.rglob("*")
            if f.is_file() and "optimizedobs" in f.name.lower()
        )

    if not obs_files:
        print(f"No COptimizedObs files found in {base}")
        return 1

    # Setup output
    out_dir = Path(args.output)
    if not out_dir.is_absolute():
        out_dir = Path(__file__).resolve().parent / out_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    # Process
    total = len(obs_files)
    if args.limit > 0:
        obs_files = obs_files[:args.limit]
        print(f"Processing {len(obs_files)} of {total} files")
    else:
        print(f"Processing {len(obs_files)} files")

    success = 0
    failed = 0
    empty = 0

    for i, f in enumerate(obs_files):
        name = f.name
        size = f.stat().st_size

        if args.stats or (i < 3):
            print(f"[{i+1}/{len(obs_files)}] {name} ({size:,} bytes)...", end=" ")

        try:
            pixels, w, h = decode_obs_file(str(f), args.width, args.height)

            if pixels and len(pixels) > 0:
                out_path = out_dir / f"{f.stem}_decoded.png"
                save_png(str(out_path), pixels, w, h)
                if args.stats or (i < 3):
                    print(f"OK ({w}x{h}, {len(pixels):,} bytes)")
                success += 1
            else:
                if args.stats or (i < 3):
                    print("EMPTY")
                empty += 1
        except Exception as e:
            if args.stats or (i < 3):
                print(f"ERROR: {e}")
            failed += 1

    print(f"\n{'='*50}")
    print(f"Results: {success} decoded, {empty} empty, {failed} failed")
    print(f"Output: {out_dir}")

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
