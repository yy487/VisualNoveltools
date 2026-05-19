# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
from silky_arc_common import extract_archive
import tempfile


def sha1(p: Path) -> str:
    h = hashlib.sha1()
    h.update(p.read_bytes())
    return h.hexdigest()


def collect(root: Path) -> dict[str, str]:
    return {p.relative_to(root).as_posix(): sha1(p) for p in sorted(root.rglob("*")) if p.is_file() and p.name != ".silky_arc_manifest.json"}


def main() -> None:
    ap = argparse.ArgumentParser(description="Compare two ARC archives after decompression")
    ap.add_argument("left", type=Path)
    ap.add_argument("right", type=Path)
    ap.add_argument("--encoding", default="cp932")
    args = ap.parse_args()
    with tempfile.TemporaryDirectory() as a, tempfile.TemporaryDirectory() as b:
        extract_archive(args.left, Path(a), "auto", args.encoding, False)
        extract_archive(args.right, Path(b), "auto", args.encoding, False)
        ca, cb = collect(Path(a)), collect(Path(b))
    left_only = sorted(set(ca) - set(cb))
    right_only = sorted(set(cb) - set(ca))
    changed = sorted(k for k in set(ca) & set(cb) if ca[k] != cb[k])
    print(f"left_only={len(left_only)} right_only={len(right_only)} changed={len(changed)}")
    for title, items in (("left_only", left_only), ("right_only", right_only), ("changed", changed)):
        if items:
            print(f"[{title}]")
            for x in items[:200]:
                print(x)


if __name__ == "__main__":
    main()
