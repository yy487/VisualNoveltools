# -*- coding: utf-8 -*-
from __future__ import annotations

import argparse
import json
import re
import tempfile
from collections import Counter
from pathlib import Path
from typing import Any

import asdis
from common import (
    DEFAULT_ENCODING,
    DEFAULT_FALLBACK_ENCODING,
    disassemble_script_to_bsd,
    iter_sources,
    normalize_rel_path,
)

RE_STRING_LITERAL = re.compile(r'"((?:\\.|[^"\\])*)"')
DIALOG_CONTROL_SUFFIX_CHARS = '<>&.'
DEFAULT_INTERNAL_TEXTS = {
    '指定されたラベルは見つかりませんでした。',
}


def _unescape_text(token: str) -> str:
    try:
        return asdis.unescape(token)
    except Exception:
        return token


def _strip_dialog_suffix(text: str) -> str:
    value = str(text)
    while value and value[-1] in DIALOG_CONTROL_SUFFIX_CHARS:
        value = value[:-1]
    return value


def _normalize_for_compare(text: str) -> str:
    value = _strip_dialog_suffix(str(text))
    if value.startswith('PrintMessage '):
        value = value[len('PrintMessage '):]
    return value


def _has_japanese_or_fullwidth(text: str) -> bool:
    for ch in text:
        o = ord(ch)
        if 0x3040 <= o <= 0x30FF:
            return True
        if 0x3400 <= o <= 0x9FFF:
            return True
        if 0xFF00 <= o <= 0xFFEF:
            return True
        if ch in '「」、。！？・…ー～〜♪♡♥☆（）『』【】':
            return True
    return False


def _looks_like_resource_or_code(text: str) -> bool:
    value = str(text)
    low = value.lower()
    if ':' in value and ('\\' in value or '/' in value):
        return True
    if any(ext in low for ext in ('.bs5', '.bss', '.bsd', '.png', '.bmp', '.jpg', '.jpeg', '.ogg', '.wav', '.avi', '.mpg', '.mp4')):
        return True
    if re.fullmatch(r'[A-Za-z0-9_./\\:\-]+', value):
        return True
    return False


def _is_audit_candidate(text: str, *, include_internal: bool = False) -> bool:
    value = str(text or '')
    if not value:
        return False
    if not include_internal and value in DEFAULT_INTERNAL_TEXTS:
        return False
    if value.startswith('PrintMessage '):
        return bool(_normalize_for_compare(value))
    if _looks_like_resource_or_code(value):
        return False
    return _has_japanese_or_fullwidth(value)


def _iter_bsd_string_literals(path: Path):
    lines = path.read_text(encoding='utf-8').splitlines()
    for line_no, line in enumerate(lines, 1):
        for m in RE_STRING_LITERAL.finditer(line):
            text = _unescape_text(m.group(1))
            yield line_no, text, line.strip()


def _load_json_strings(json_path: Path) -> set[str]:
    if not json_path.exists():
        return set()
    with json_path.open('r', encoding='utf-8') as f:
        data = json.load(f)
    if not isinstance(data, list):
        return set()
    out: set[str] = set()
    for item in data:
        if not isinstance(item, dict):
            continue
        for key in ('scr_msg', 'message', 'name'):
            value = item.get(key)
            if isinstance(value, str):
                out.add(value)
                out.add(_normalize_for_compare(value))
    return out


def _json_path_for_rel(json_root: Path, rel_name: str) -> Path:
    return json_root / (rel_name + '.json')


def _context(path: Path, line_no: int, radius: int) -> list[str]:
    if radius <= 0:
        return []
    lines = path.read_text(encoding='utf-8').splitlines()
    start = max(1, line_no - radius)
    end = min(len(lines), line_no + radius)
    return [f'{i}: {lines[i - 1]}' for i in range(start, end + 1)]


def _prepare_bsd_sources(input_path: Path, mode: str, encoding: str, fallback_encoding: str, temp_root: Path | None):
    if mode == 'bsd':
        root, sources = iter_sources(input_path, 'bsd')
        return root, [(src, normalize_rel_path(src, root)) for src in sources]
    if temp_root is None:
        raise ValueError('script mode requires temp_root')
    root, sources = iter_sources(input_path, 'script')
    out: list[tuple[Path, str]] = []
    for src in sources:
        rel = normalize_rel_path(src, root)
        bsd_path = temp_root / (rel + '.bsd')
        bsd_path.parent.mkdir(parents=True, exist_ok=True)
        disassemble_script_to_bsd(src, bsd_path, encoding=encoding, fallback_encoding=fallback_encoding)
        out.append((bsd_path, rel))
    return root, out


def audit(input_path: Path, json_root: Path, output_path: Path, *, mode: str, encoding: str, fallback_encoding: str, context_lines: int, include_internal: bool) -> dict[str, Any]:
    with tempfile.TemporaryDirectory(prefix='bgi_v1_audit_') as td:
        temp_root = Path(td)
        _, sources = _prepare_bsd_sources(input_path, mode, encoding, fallback_encoding, temp_root)
        records: list[dict[str, Any]] = []
        scanned_literals = 0
        candidate_literals = 0
        for bsd_path, rel_name in sources:
            json_strings = _load_json_strings(_json_path_for_rel(json_root, rel_name))
            for line_no, text, code in _iter_bsd_string_literals(bsd_path):
                scanned_literals += 1
                if not _is_audit_candidate(text, include_internal=include_internal):
                    continue
                candidate_literals += 1
                norm = _normalize_for_compare(text)
                if text in json_strings or norm in json_strings:
                    continue
                records.append({
                    'file': rel_name,
                    'line': line_no,
                    'text': text,
                    'normalized': norm,
                    'code': code,
                    'context': _context(bsd_path, line_no, context_lines),
                })
        unique_counter = Counter(r['normalized'] for r in records)
        report = {
            'mode': mode,
            'input': str(input_path),
            'json': str(json_root),
            'files': len(sources),
            'scanned_string_literals': scanned_literals,
            'candidate_string_literals': candidate_literals,
            'missing_records': len(records),
            'missing_unique': len(unique_counter),
            'missing_unique_texts': [
                {'text': text, 'count': count}
                for text, count in unique_counter.most_common()
            ],
            'records': records,
        }
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with output_path.open('w', encoding='utf-8', newline='\n') as f:
            json.dump(report, f, ensure_ascii=False, indent=2)
        return report


def main() -> None:
    parser = argparse.ArgumentParser(description='BGI V1 BSD 字符串提取覆盖率审计：扫描 BSD 字符串并与工作流 JSON diff')
    parser.add_argument('input', help='输入脚本目录或 .bsd 目录')
    parser.add_argument('json', help='extract.py 输出的 JSON 目录')
    parser.add_argument('output', help='审计报告 JSON')
    parser.add_argument('--mode', choices=['script', 'bsd'], default='script')
    parser.add_argument('--encoding', default=DEFAULT_ENCODING)
    parser.add_argument('--fallback-encoding', default=DEFAULT_FALLBACK_ENCODING)
    parser.add_argument('--context-lines', type=int, default=8)
    parser.add_argument('--include-internal', action='store_true', help='包含已知内部错误文本等默认忽略项')
    args = parser.parse_args()

    report = audit(
        Path(args.input),
        Path(args.json),
        Path(args.output),
        mode=args.mode,
        encoding=args.encoding,
        fallback_encoding=args.fallback_encoding,
        context_lines=args.context_lines,
        include_internal=args.include_internal,
    )
    print(f"[audit] files={report['files']} scanned_literals={report['scanned_string_literals']} candidates={report['candidate_string_literals']}")
    print(f"[audit] missing_records={report['missing_records']} missing_unique={report['missing_unique']}")
    print(f"[audit] output={args.output}")
    if report['missing_unique_texts']:
        print('[audit] top missing:')
        for item in report['missing_unique_texts'][:20]:
            print(f"  {item['count']}  {item['text']}")


if __name__ == '__main__':
    main()
