# -*- coding: utf-8 -*-
"""Text extraction/injection helpers for Refrain Blue MES files.

The MES files inside MES.ARC contain ordinary CP932 zero-terminated strings.
Opcode 0x01 is a text command followed by a CP932 C string.

v3 ruby/furigana note
---------------------
Some displayed strings are split by ruby helper blocks like::

    01 "『" 00
    0B 15 FF 01 "とう" 00 ...
    01 "東" 00
    0B 15 FF 01 "よう" 00 ...
    01 "陽学園』..." 00

The translation unit should be the visible text "『東陽学園』...".  The ruby
readings are recorded in metadata and are not mixed into ``scr_msg``.
"""
from __future__ import annotations

from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any, Iterable
import json
import re

ENCODING = "cp932"
JAPANESE_RE = re.compile(r"[\u3040-\u30ff\u3400-\u9fff\u3000-\u303f]")
SENTENCE_END = set("。！？!?）」』…")

# 0B 15 FF 01 <ruby> 00 appears to be ruby/furigana helper text.
RUBY_PREFIX = b"\x0B\x15\xFF"
FULLWIDTH_SPACE_CP932 = "　".encode(ENCODING)
HALFWIDTH_SPACE_CP932 = b"\x20"


def _make_space_padding_exact(n: int) -> bytes:
    """Return exactly n bytes of printable CP932 spaces.

    Used only for tiny 1..4 byte gaps where a tail jump cannot fit.
    Keeping the original zero terminator at the original end preserves the
    following opcode boundary without inserting OPX0A into sensitive blocks
    such as choice branch heads.
    """
    if n <= 0:
        return b""
    return FULLWIDTH_SPACE_CP932 * (n // 2) + (HALFWIDTH_SPACE_CP932 if n % 2 else b"")

# Bytes between a ruby string terminator and the visible/base TEXT opcode.
# Current sample often uses: 02 09 A0 00 FF 02 0A A0 00 09 E1 FF 00.
MAX_RUBY_TO_BASE_GAP = 0x40


@dataclass
class TextRecord:
    scr_msg: str
    message: str
    _file: str
    _index: int
    _offset: int          # offset of opcode 01, or first visible part for ruby-composed entries
    _text_offset: int     # offset of first byte of CP932 string
    _end: int             # offset after zero terminator, or last visible part end
    _opcode: str = "01"
    _type: str = "message"
    _raw_hex: str = ""
    # Present only for ruby-composed logical messages.
    scr_msg_parts: list[str] | None = None
    message_parts: list[str] | None = None
    _parts: list[dict[str, Any]] | None = None
    _part_offsets: list[int] | None = None
    _part_ends: list[int] | None = None
    _ruby: list[dict[str, Any]] | None = None

    def to_json_obj(self) -> dict[str, Any]:
        obj = asdict(self)
        return {k: v for k, v in obj.items() if v is not None}


@dataclass
class _TextCandidate:
    text: str
    raw: bytes
    offset: int
    text_offset: int
    end: int
    is_ruby: bool

    @property
    def raw_hex(self) -> str:
        return (b"\x01" + self.raw + b"\x00").hex(" ")


def is_private_use(s: str) -> bool:
    return any(0xE000 <= ord(ch) <= 0xF8FF for ch in s)


def _is_clean_decoded_string(s: str) -> bool:
    if not s:
        return False
    if is_private_use(s):
        return False
    if any((ord(ch) < 32 and ch not in "\t\r\n") for ch in s):
        return False
    if not JAPANESE_RE.search(s):
        return False
    return True


def looks_like_translatable_text(s: str) -> bool:
    """Standalone text filter.

    Short one-character fragments are normally too noisy for the heuristic
    scanner.  Ruby composition handles those fragments separately and does not
    rely on this standalone filter.
    """
    if not s or len(s.strip()) < 2:
        return False
    return _is_clean_decoded_string(s)


def read_cstring(data: bytes, start: int, max_len: int = 500) -> tuple[bytes, int] | None:
    end_limit = min(len(data), start + max_len)
    end = data.find(b"\x00", start, end_limit)
    if end < 0:
        return None
    return data[start:end], end + 1


def decode_cp932(raw: bytes) -> str | None:
    try:
        s = raw.decode(ENCODING)
    except UnicodeDecodeError:
        return None
    if is_private_use(s):
        return None
    return s


def encode_cp932(s: str) -> bytes:
    return s.encode(ENCODING)


def _scan_candidates(data: bytes) -> list[_TextCandidate]:
    """Return all plausible opcode-01 CP932 Japanese strings, including short
    fragments and ruby readings.
    """
    out: list[_TextCandidate] = []
    i = 0
    while i < len(data) - 2:
        if data[i] != 0x01:
            i += 1
            continue
        item = read_cstring(data, i + 1)
        if item is None:
            i += 1
            continue
        raw, end = item
        if not raw:
            i += 1
            continue
        text = decode_cp932(raw)
        if text is None or not _is_clean_decoded_string(text):
            i += 1
            continue
        out.append(
            _TextCandidate(
                text=text,
                raw=raw,
                offset=i,
                text_offset=i + 1,
                end=end,
                is_ruby=(i >= 3 and data[i - 3:i] == RUBY_PREFIX),
            )
        )
        i = end
    return out


def _gap(data: bytes, a: _TextCandidate, b: _TextCandidate) -> bytes:
    if a.end > b.offset:
        return b""
    return data[a.end:b.offset]


def _visible_to_ruby(data: bytes, visible: _TextCandidate, ruby: _TextCandidate) -> bool:
    return (not visible.is_ruby) and ruby.is_ruby and _gap(data, visible, ruby) == RUBY_PREFIX


def _ruby_to_visible(data: bytes, ruby: _TextCandidate, visible: _TextCandidate) -> bool:
    if not ruby.is_ruby or visible.is_ruby:
        return False
    g = _gap(data, ruby, visible)
    if len(g) > MAX_RUBY_TO_BASE_GAP:
        return False
    # 0x13 is a strong display/new-line boundary in this format; a ruby base
    # should appear before the next such boundary.
    if b"\x13" in g:
        return False
    return True


def _parse_ruby_group(data: bytes, cands: list[_TextCandidate], start: int) -> tuple[list[_TextCandidate], list[_TextCandidate], int] | None:
    """Parse one logical visible message containing one or more ruby readings.

    Returns (visible_parts, ruby_parts, next_index).  ``start`` may point to a
    visible prelude immediately followed by ruby, or directly to a ruby reading.
    """
    n = len(cands)
    if start >= n:
        return None

    visible: list[_TextCandidate] = []
    rubies: list[_TextCandidate] = []
    k = start

    if not cands[k].is_ruby:
        if k + 1 >= n or not _visible_to_ruby(data, cands[k], cands[k + 1]):
            return None
        visible.append(cands[k])
        k += 1

    # Now k should be the first ruby candidate.
    if k >= n or not cands[k].is_ruby:
        return None

    while k < n and cands[k].is_ruby:
        ruby = cands[k]
        if k + 1 >= n:
            break
        base = cands[k + 1]
        if not _ruby_to_visible(data, ruby, base):
            break
        rubies.append(ruby)
        visible.append(base)
        k += 2
        # Continue only for the immediate pattern: visible 00 0B 15 FF 01 ruby.
        if k < n and _visible_to_ruby(data, visible[-1], cands[k]):
            continue
        break

    if not rubies or not visible:
        return None
    return visible, rubies, k


def _make_simple_record(c: _TextCandidate, file_name: str, index: int, typ: str = "message") -> TextRecord:
    return TextRecord(
        scr_msg=c.text,
        message=c.text,
        _file=file_name,
        _index=index,
        _offset=c.offset,
        _text_offset=c.text_offset,
        _end=c.end,
        _opcode="01",
        _type=typ,
        _raw_hex=c.raw_hex,
    )


def _make_ruby_group_record(
    data: bytes,
    visible: list[_TextCandidate],
    rubies: list[_TextCandidate],
    file_name: str,
    index: int,
) -> TextRecord:
    parts = []
    for pidx, v in enumerate(visible):
        parts.append(
            {
                "scr_msg": v.text,
                "_part_index": pidx,
                "_offset": v.offset,
                "_text_offset": v.text_offset,
                "_end": v.end,
                "_raw_hex": v.raw_hex,
            }
        )

    # If the group has a visible prelude before the first ruby, ruby[0] maps to
    # visible part 1; otherwise it maps to visible part 0.
    has_prelude = visible and visible[0].offset < rubies[0].offset
    ruby_meta: list[dict[str, Any]] = []
    for ridx, r in enumerate(rubies):
        base_part = ridx + (1 if has_prelude else 0)
        if base_part >= len(visible):
            base_part = len(visible) - 1
        base_text = visible[base_part].text if visible else ""
        ruby_meta.append(
            {
                "rt": r.text,
                "rt_offset": r.offset,
                "rt_text_offset": r.text_offset,
                "rt_end": r.end,
                "base_part": base_part,
                "base": base_text[:1],
            }
        )

    scr_parts = [v.text for v in visible]
    start = min([v.offset for v in visible] + [r.offset - 3 for r in rubies if r.offset >= 3])
    end = max(v.end for v in visible)
    return TextRecord(
        scr_msg="".join(scr_parts),
        message="".join(scr_parts),
        _file=file_name,
        _index=index,
        _offset=visible[0].offset,
        _text_offset=visible[0].text_offset,
        _end=visible[-1].end,
        _opcode="01+ruby",
        _type="message",
        _raw_hex=data[start:end].hex(" "),
        scr_msg_parts=scr_parts,
        message_parts=list(scr_parts),
        _parts=parts,
        _part_offsets=[v.offset for v in visible],
        _part_ends=[v.end for v in visible],
        _ruby=ruby_meta,
    )


def scan_mes_text(data: bytes, file_name: str, include_ruby: bool = False) -> list[TextRecord]:
    """Heuristic but practical scanner for opcode 01 text commands.

    Accepted normal strings keep the old behavior.  Ruby/furigana split strings
    are additionally composed into one logical visible entry, while their ruby
    readings are stored under ``_ruby`` and skipped as standalone translatable
    text by default.
    """
    cands = _scan_candidates(data)
    out: list[TextRecord] = []
    used: set[int] = set()

    i = 0
    while i < len(cands):
        if i in used:
            i += 1
            continue
        c = cands[i]

        group = _parse_ruby_group(data, cands, i)
        if group is not None:
            visible, rubies, next_i = group
            for j in range(i, next_i):
                used.add(j)
            out.append(_make_ruby_group_record(data, visible, rubies, file_name, len(out)))
            if include_ruby:
                for r in rubies:
                    out.append(_make_simple_record(r, file_name, len(out), typ="ruby"))
            i = next_i
            continue

        if c.is_ruby:
            if include_ruby:
                out.append(_make_simple_record(c, file_name, len(out), typ="ruby"))
            i += 1
            continue

        if looks_like_translatable_text(c.text):
            out.append(_make_simple_record(c, file_name, len(out), typ="message"))
        i += 1

    return out


def records_to_json(records: Iterable[TextRecord]) -> list[dict[str, Any]]:
    return [r.to_json_obj() for r in records]


def load_json_entries(path: str | Path) -> list[dict[str, Any]]:
    obj = json.loads(Path(path).read_text(encoding="utf-8"))
    if isinstance(obj, list):
        return obj
    if isinstance(obj, dict):
        for key in ("entries", "texts", "messages"):
            if isinstance(obj.get(key), list):
                return obj[key]
    raise ValueError(f"unsupported JSON format: {path}")


def save_json(path: str | Path, entries: list[dict[str, Any]]) -> None:
    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(entries, ensure_ascii=False, indent=2), encoding="utf-8")


def validate_text_span(data: bytes, off: int, end: int, scr_msg: str) -> tuple[int, int, bytes]:
    """Return (offset, end, old_raw) after validating a 01 string span."""
    if off < 0 or end > len(data) or off >= end:
        raise ValueError(f"invalid offset/end: off=0x{off:X}, end=0x{end:X}")
    if data[off] != 0x01:
        raise ValueError(f"offset 0x{off:X} is no longer opcode 01; inject from original MES/ARC")
    old_raw = encode_cp932(str(scr_msg))
    actual = data[off + 1:end - 1]
    if actual != old_raw:
        try:
            actual_s = actual.decode(ENCODING)
        except Exception:
            actual_s = actual.hex(" ")
        raise ValueError(f"scr_msg mismatch at 0x{off:X}: json={scr_msg!r}, file={actual_s!r}")
    return off, end, old_raw


def validate_entry_at(data: bytes, entry: dict[str, Any]) -> tuple[int, int, bytes]:
    """Return (offset, end, old_raw) after validating scr_msg at original offset."""
    return validate_text_span(data, int(entry["_offset"]), int(entry["_end"]), str(entry["scr_msg"]))


def _split_message_by_original_parts(message: str, scr_parts: list[str]) -> list[str]:
    """Fallback split when a ruby-composed entry has only flat ``message`` edited.

    It preserves the number of visible TEXT slots.  For all but the last slot,
    the original character count is reused; the last slot receives the rest.
    This is deliberately simple and stable.  Translators can edit
    ``message_parts`` manually for exact ruby/base control.
    """
    if not scr_parts:
        return [message]
    result: list[str] = []
    pos = 0
    chars = list(message)
    for part in scr_parts[:-1]:
        take = len(part)
        result.append("".join(chars[pos:pos + take]))
        pos += take
    result.append("".join(chars[pos:]))
    return result


def _get_part_messages(entry: dict[str, Any], report: dict[str, Any], idx: int) -> tuple[list[dict[str, Any]], list[str]] | None:
    parts = entry.get("_parts")
    if not isinstance(parts, list) or not parts:
        return None
    norm_parts: list[dict[str, Any]] = []
    for pidx, p in enumerate(parts):
        if not isinstance(p, dict):
            report["errors"].append(f"entry[{idx}] _parts[{pidx}] is not an object")
            return None
        if "scr_msg" not in p or "_offset" not in p or "_end" not in p:
            report["errors"].append(f"entry[{idx}] _parts[{pidx}] missing scr_msg/_offset/_end")
            return None
        norm_parts.append(p)

    scr_parts_obj = entry.get("scr_msg_parts")
    if isinstance(scr_parts_obj, list) and len(scr_parts_obj) == len(norm_parts):
        scr_parts = [str(x) for x in scr_parts_obj]
    else:
        scr_parts = [str(p["scr_msg"]) for p in norm_parts]

    msg = entry.get("message", entry.get("scr_msg"))
    scr = entry.get("scr_msg")
    msg_parts_obj = entry.get("message_parts")

    if isinstance(msg_parts_obj, list) and len(msg_parts_obj) == len(norm_parts):
        msg_parts = [str(x) for x in msg_parts_obj]
        # Common translation workflow edits only flat ``message`` and leaves
        # initial message_parts untouched.  In that case, split the flat message.
        if isinstance(msg, str) and isinstance(scr, str) and msg != scr and msg_parts == scr_parts:
            msg_parts = _split_message_by_original_parts(msg, scr_parts)
            report["warnings"].append(
                f"entry[{idx}] ruby-composed entry has edited flat message; split into message_parts automatically"
            )
    elif isinstance(msg, str) and isinstance(scr, str) and msg != scr:
        msg_parts = _split_message_by_original_parts(msg, scr_parts)
        report["warnings"].append(
            f"entry[{idx}] ruby-composed entry lacks valid message_parts; split flat message automatically"
        )
    else:
        msg_parts = scr_parts

    if len(msg_parts) != len(norm_parts):
        report["errors"].append(f"entry[{idx}] message_parts length mismatch")
        return None
    return norm_parts, msg_parts



def _blank_one_ruby_span(
    *,
    original: bytes,
    buf: bytearray,
    ruby: dict[str, Any],
    label: str,
    report: dict[str, Any],
    seen_offsets: set[int],
) -> bool:
    """Overwrite one ruby/furigana C-string with full-width spaces in place.

    The 0B 15 FF 01 structure and the zero terminator are left exactly where
    they were.  Only the bytes inside the ruby text span are modified, so this
    debug policy cannot affect jump/choice offsets.
    """
    try:
        rt_offset = int(ruby.get("rt_offset"))
        rt_text_offset = int(ruby.get("rt_text_offset"))
        rt_end = int(ruby.get("rt_end"))
        rt = str(ruby.get("rt", ""))
    except Exception as ex:
        report["errors"].append(f"{label} invalid ruby metadata: {ex}")
        return False

    if rt_text_offset in seen_offsets:
        return False
    if rt_offset < 0 or rt_text_offset <= rt_offset or rt_end > len(original) or rt_text_offset >= rt_end:
        report["errors"].append(
            f"{label} invalid ruby span: rt_offset=0x{rt_offset:X}, rt_text_offset=0x{rt_text_offset:X}, rt_end=0x{rt_end:X}"
        )
        return False
    if original[rt_offset] != 0x01:
        report["errors"].append(f"{label} ruby offset 0x{rt_offset:X} is not opcode 01")
        return False
    if original[rt_end - 1] != 0x00:
        report["errors"].append(f"{label} ruby span does not end with 00 at 0x{rt_end - 1:X}")
        return False

    old_raw = original[rt_text_offset:rt_end - 1]
    try:
        expected = encode_cp932(rt)
    except Exception as ex:
        report["warnings"].append(f"{label} ruby rt cannot be encoded for validation: {ex}")
        expected = None
    if expected is not None and old_raw != expected:
        try:
            actual_s = old_raw.decode(ENCODING)
        except Exception:
            actual_s = old_raw.hex(" ")
        report["errors"].append(f"{label} ruby mismatch at 0x{rt_offset:X}: json={rt!r}, file={actual_s!r}")
        return False

    raw_len = len(old_raw)
    if raw_len == 0:
        seen_offsets.add(rt_text_offset)
        return False

    if raw_len % len(FULLWIDTH_SPACE_CP932) != 0:
        # This should not happen for ordinary CP932 kana ruby readings.  Keep the
        # span length exact anyway; the final ASCII space is a debug fallback.
        fill = FULLWIDTH_SPACE_CP932 * (raw_len // 2) + b" "
        report["warnings"].append(
            f"{label} ruby byte length is odd ({raw_len}); used one ASCII space as tail padding"
        )
    else:
        fill = FULLWIDTH_SPACE_CP932 * (raw_len // len(FULLWIDTH_SPACE_CP932))

    buf[rt_text_offset:rt_end - 1] = fill
    seen_offsets.add(rt_text_offset)
    report["ruby_blank_fullwidth"] += 1
    report["ruby_blank_bytes"] += raw_len
    return True


def _apply_ruby_policy(
    *,
    original: bytes,
    buf: bytearray,
    entry: dict[str, Any],
    idx: int,
    ruby_policy: str,
    report: dict[str, Any],
    seen_offsets: set[int],
) -> None:
    if ruby_policy in ("keep", "none", ""):
        return
    if ruby_policy not in ("blank-fullwidth", "fullwidth-space", "blank"):
        report["errors"].append(f"entry[{idx}] unsupported ruby_policy={ruby_policy!r}")
        return

    rubies = entry.get("_ruby")
    if not isinstance(rubies, list):
        # Also support JSONs extracted with --include-ruby, where ruby entries may
        # appear as standalone _type=ruby records.
        if entry.get("_type") == "ruby" and all(k in entry for k in ("_offset", "_text_offset", "_end", "scr_msg")):
            rubies = [
                {
                    "rt": entry.get("scr_msg", ""),
                    "rt_offset": entry.get("_offset"),
                    "rt_text_offset": entry.get("_text_offset"),
                    "rt_end": entry.get("_end"),
                }
            ]
        else:
            return

    for ridx, r in enumerate(rubies):
        if not isinstance(r, dict):
            report["errors"].append(f"entry[{idx}]._ruby[{ridx}] is not an object")
            continue
        _blank_one_ruby_span(
            original=original,
            buf=buf,
            ruby=r,
            label=f"entry[{idx}]._ruby[{ridx}]",
            report=report,
            seen_offsets=seen_offsets,
        )

def patch_mes_non_equal(data: bytes, entries: list[dict[str, Any]], *, force_jump: bool = False, ruby_policy: str = "keep") -> tuple[bytes, dict[str, Any]]:
    """Patch MES text with safe non-equal-length support.

    MES text command format is::

        01 <cp932 zero-terminated string> 00

    The established non-equal-length policy is preserved:
      * unchanged entries are skipped;
      * exactly equal command spans are patched in place;
      * shorter/equal-fitting replacements with at least 5 spare bytes use an
        inline tail jump: ``01 new 00 0A old_end``;
      * v6 fix: shorter replacements with only 1..4 spare bytes are patched
        in-place by padding printable spaces before the original terminator,
        instead of replacing opcode 01 with an EOF trampoline;
      * longer replacements use an EOF trampoline: original position becomes
        ``0A append_off`` and EOF receives ``01 new 00 0A old_end``.

    v3 also accepts ruby-composed JSON entries with ``_parts``.  Each visible
    part is validated and patched independently, so the already-verified jump
    and choice logic is not changed.

    ``ruby_policy="blank-fullwidth"`` is a debug/QA switch: every ruby reading
    recorded under ``_ruby`` is overwritten in place with CP932 full-width
    spaces of the same byte length.  Ruby opcodes and zero terminators are kept
    unchanged.
    """
    buf = bytearray(data)
    appended = bytearray()
    report: dict[str, Any] = {
        "json_entries": len(entries),
        "patched": 0,
        "skipped_same": 0,
        "inplace_equal": 0,
        "inline_tail_jump": 0,
        "inplace_spacepad": 0,
        "eof_trampoline": 0,
        "parts_patched": 0,
        "ruby_policy": ruby_policy,
        "ruby_blank_fullwidth": 0,
        "ruby_blank_bytes": 0,
        # Compatibility counters for older scripts/log readers.
        "inplace": 0,
        "jump": 0,
        "errors": [],
        "warnings": [],
    }

    def is_choice_branch_head(off: int) -> bool:
        """TEXT immediately after 0E 02 <idx> FF 00 <end:u32>.

        These locations are sensitive because the menu/choice logic appears to
        inspect branch bodies at their physical address.  Replacing the leading
        01 with OPX0A can make the choice wait state fail or become invisible.
        """
        return (
            off >= 9
            and data[off - 9:off - 7] == b"\x0E\x02"
            and data[off - 6:off - 4] == b"\xFF\x00"
        )

    def patch_unit(off: int, end: int, scr_msg: str, msg: str, label: str) -> bool:
        nonlocal appended
        if msg == scr_msg:
            return False
        try:
            off2, end2, _old_raw = validate_text_span(data, off, end, scr_msg)
            new_raw = encode_cp932(msg)
        except Exception as ex:
            report["errors"].append(f"{label} {ex}")
            return False

        old_span = end2 - off2
        new_command = b"\x01" + new_raw + b"\x00"
        return_jump = b"\x0A" + end2.to_bytes(4, "little")

        if not force_jump and len(new_command) == old_span:
            buf[off2:end2] = new_command
            report["patched"] += 1
            report["inplace_equal"] += 1
            report["inplace"] += 1
            return True

        if not force_jump and len(new_command) + 5 <= old_span:
            used = len(new_command) + 5
            buf[off2:off2 + len(new_command)] = new_command
            buf[off2 + len(new_command):off2 + used] = return_jump
            if used < old_span:
                buf[off2 + used:end2] = b"\x00" * (old_span - used)
            report["patched"] += 1
            report["inline_tail_jump"] += 1
            report["jump"] += 1
            return True

        # v6: If the translation is shorter/equal but there are only 1..4 spare
        # bytes, do not fall back to an EOF trampoline.  Replacing the original
        # opcode 01 with OPX0A is unsafe for some choice/display units.  Instead
        # keep the whole command span in place and put printable spaces before
        # the original zero terminator.
        if not force_jump and len(new_command) < old_span:
            spare = old_span - len(new_command)
            if 0 < spare < 5:
                padded_command = b"\x01" + new_raw + _make_space_padding_exact(spare) + b"\x00"
                if len(padded_command) != old_span:
                    report["errors"].append(
                        f"{label} internal padding size mismatch: padded={len(padded_command)}, old_span={old_span}"
                    )
                    return False
                buf[off2:end2] = padded_command
                report["patched"] += 1
                report["inplace_spacepad"] += 1
                report["inplace"] += 1
                if is_choice_branch_head(off2):
                    report["warnings"].append(
                        f"{label} choice-branch-head patched with in-place space padding at 0x{off2:X}"
                    )
                return True

        if old_span < 5:
            report["errors"].append(
                f"{label} original span too small for EOF trampoline: span={old_span}, off=0x{off2:X}"
            )
            return False

        if is_choice_branch_head(off2):
            report["warnings"].append(
                f"{label} uses EOF trampoline at choice-branch-head 0x{off2:X}; consider shortening this line if menu/choice freezes"
            )
        append_off = len(buf) + len(appended)
        buf[off2:off2 + 5] = b"\x0A" + append_off.to_bytes(4, "little")
        if old_span > 5:
            buf[off2 + 5:end2] = b"\x00" * (old_span - 5)
        appended += new_command + return_jump
        report["patched"] += 1
        report["eof_trampoline"] += 1
        report["jump"] += 1
        return True

    seen_ruby_offsets: set[int] = set()

    # Offsets are from the original file and remain valid because we never
    # insert in the middle of the existing region.  We only modify bytes inside
    # the original command span or append new code at EOF.
    for idx, e in enumerate(entries):
        _apply_ruby_policy(
            original=data,
            buf=buf,
            entry=e,
            idx=idx,
            ruby_policy=ruby_policy,
            report=report,
            seen_offsets=seen_ruby_offsets,
        )

        # Ruby-composed logical message: patch visible 01 slots by parts.
        part_info = _get_part_messages(e, report, idx)
        if part_info is not None:
            parts, msg_parts = part_info
            changed = False
            for pidx, (part, part_msg) in enumerate(zip(parts, msg_parts)):
                scr_part = str(part["scr_msg"])
                if part_msg == scr_part:
                    continue
                ok = patch_unit(
                    int(part["_offset"]),
                    int(part["_end"]),
                    scr_part,
                    part_msg,
                    f"entry[{idx}].part[{pidx}]",
                )
                if ok:
                    report["parts_patched"] += 1
                    changed = True
            if not changed:
                report["skipped_same"] += 1
            continue

        msg = e.get("message", e.get("scr_msg"))
        scr = e.get("scr_msg")
        if not isinstance(scr, str) or not isinstance(msg, str):
            report["warnings"].append(f"entry[{idx}] missing string scr_msg/message; skipped")
            continue
        if msg == scr:
            report["skipped_same"] += 1
            continue
        patch_unit(int(e["_offset"]), int(e["_end"]), scr, msg, f"entry[{idx}]")

    if appended:
        buf += appended
    report["old_size"] = len(data)
    report["new_size"] = len(buf)
    report["appended_bytes"] = len(appended)
    return bytes(buf), report
