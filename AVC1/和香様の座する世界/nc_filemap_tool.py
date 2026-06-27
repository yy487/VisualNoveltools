# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import importlib.util
import struct
import sys
from collections import Counter, defaultdict
from pathlib import Path


def load_waka_module():
    tool_path = Path(__file__).with_name("waka_dat_tool.py")
    spec = importlib.util.spec_from_file_location("waka_dat_tool", tool_path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {tool_path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules["waka_dat_tool"] = module
    spec.loader.exec_module(module)
    return module


def read_schemes(idx_path: Path):
    idx = idx_path.read_bytes()
    scheme_count = struct.unpack_from("<I", idx, 0)[0]
    pos = 16
    schemes = []
    for i in range(scheme_count):
        scheme_hash, offset, count = struct.unpack_from("<QII", idx, pos)
        pos += 16
        schemes.append((i, scheme_hash, offset, count))
    return idx, schemes


def iter_scheme_names(idx: bytes, dat: bytes, scheme):
    scheme_id, scheme_hash, offset, count = scheme
    for j in range(count):
        name_hash, name_offset, name_len = struct.unpack_from("<QII", idx, offset + j * 16)
        raw = dat[name_offset : name_offset + name_len]
        yield scheme_id, scheme_hash, j, name_hash, raw.decode("cp932", "replace")


def load_archive_hashes(paths, waka):
    archive_hashes = {}
    for path in paths:
        data = Path(path).read_bytes()
        _, _, _, entries = waka.parse_dat_header(data)
        for e in entries:
            archive_hashes.setdefault(waka.entry_hash(e), []).append((Path(path).name, e.index))
    return archive_hashes


def cmd_summary(args):
    waka = load_waka_module()
    idx, schemes = read_schemes(Path(args.idx))
    dat = Path(args.dat).read_bytes()
    archive_hashes = load_archive_hashes(args.archives, waka)

    for scheme in schemes:
        hits = []
        prefixes = Counter()
        for _, _, _, name_hash, name in iter_scheme_names(idx, dat, scheme):
            if name_hash not in archive_hashes:
                continue
            hits.append((name_hash, name))
            parts = name.split("/")
            prefixes["/".join(parts[:2] if len(parts) >= 2 else parts)] += 1
        if args.min_hits and len(hits) < args.min_hits:
            continue
        scheme_id, scheme_hash, _, count = scheme
        top = ", ".join(f"{k}:{v}" for k, v in prefixes.most_common(8))
        print(f"#{scheme_id:02d} {scheme_hash:016X} count={count} hits={len(hits)} {top}")


def cmd_export(args):
    waka = load_waka_module()
    idx, schemes = read_schemes(Path(args.idx))
    dat = Path(args.dat).read_bytes()
    archive_hashes = load_archive_hashes(args.archives, waka)
    selected = None if args.schemes is None else {int(x, 0) for x in args.schemes.split(",") if x.strip()}
    prefixes = tuple(p.replace("\\", "/") for p in args.prefix)

    by_hash = defaultdict(set)
    for scheme in schemes:
        if selected is not None and scheme[0] not in selected:
            continue
        for _, _, _, name_hash, name in iter_scheme_names(idx, dat, scheme):
            name = name.replace("\\", "/")
            if name_hash not in archive_hashes:
                continue
            if prefixes and not name.startswith(prefixes):
                continue
            by_hash[name_hash].add(name)

    conflicts = {h: names for h, names in by_hash.items() if len(names) > 1}
    names = sorted({name for names in by_hash.values() for name in names})
    out = Path(args.output)
    out.write_text("\n".join(names) + ("\n" if names else ""), encoding="utf-8")

    print(f"[export] names={len(names)} hashes={len(by_hash)} output={out}")
    if conflicts:
        print(f"[export] crc64 conflicts or aliases={len(conflicts)}")
        for h, values in list(conflicts.items())[:10]:
            joined = " | ".join(sorted(values))
            print(f"  {h:016X}: {joined}")


def main():
    ap = argparse.ArgumentParser(description="Extract matching NonColor names from GARbro NCFileMap.")
    ap.add_argument("--idx", default=r"C:\Users\john\OneDrive\Desktop\GARbro\GameData\NCFileMap.idx")
    ap.add_argument("--dat", default=r"C:\Users\john\OneDrive\Desktop\GARbro\GameData\NCFileMap.dat")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("summary")
    p.add_argument("archives", nargs="+")
    p.add_argument("--min-hits", type=int, default=1)
    p.set_defaults(func=cmd_summary)

    p = sub.add_parser("export")
    p.add_argument("archives", nargs="+")
    p.add_argument("-o", "--output", required=True)
    p.add_argument("--schemes", default=None, help="comma separated scheme ids, for example 3,12,13,15")
    p.add_argument("--prefix", action="append", default=[], help="only export names with this prefix")
    p.set_defaults(func=cmd_export)

    args = ap.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
