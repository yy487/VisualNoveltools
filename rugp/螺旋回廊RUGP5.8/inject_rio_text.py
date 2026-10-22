"""Create and apply fixed-size CRsa text patches to rUGP RIO volumes."""

from __future__ import annotations

import argparse
import json
import shutil
import struct
from pathlib import Path
from typing import Any

import decrypt_rio


PATCH_FORMAT = "rugp-crsa-text-patch-v1"


def _parse_offset(value: str) -> int:
    return int(value, 0)


def _parse_logical_address(value: int | str) -> int:
    if isinstance(value, int):
        return value
    return int(value, 16)


def _encode_rendered_text(text: str, encoding: str) -> bytes:
    characters: list[str] = []
    position = 0
    while position < len(text):
        if text.startswith("{{", position):
            characters.append("{")
            position += 2
            continue
        if text.startswith("}}", position):
            characters.append("}")
            position += 2
            continue
        if text[position] == "{":
            end = text.find("}", position + 1)
            if end >= 0:
                token = text[position + 1 : end]
                if token.isdecimal() and 0 <= int(token) < 0x20:
                    characters.append(chr(int(token)))
                    position = end + 1
                    continue
        characters.append(text[position])
        position += 1
    try:
        return "".join(characters).encode(encoding)
    except UnicodeEncodeError as error:
        raise decrypt_rio.FormatError(
            f"replacement cannot be encoded as {encoding}: {error}"
        ) from error


def _infer_address_shift(volume_offset: int, logical_address: int) -> int:
    for shift in range(32):
        if logical_address << shift == volume_offset:
            return shift
    raise decrypt_rio.FormatError(
        "text metadata has inconsistent logical and byte addresses"
    )


def create_patch_template(
    text_json: Path,
    prefix_offsets: list[int],
) -> dict[str, Any]:
    metadata = json.loads(text_json.read_text(encoding="utf-8"))
    container = metadata["container"]
    messages = {
        int(message["prefix_offset"]): message
        for message in metadata["messages"]
    }
    replacements = []
    for prefix_offset in prefix_offsets:
        message = messages.get(prefix_offset)
        if message is None:
            raise decrypt_rio.FormatError(
                f"message 0x{prefix_offset:X} is not present in {text_json.name}"
            )
        replacements.append(
            {
                "prefix_offset": prefix_offset,
                "content_offset": message["content_offset"],
                "byte_length": message["length"],
                "original_text": message["text"],
                "original_raw_hex": message["raw_hex"],
                "replacement": message["text"],
            }
        )

    logical_address = int(container["logical_address"])
    volume_offset = int(container["volume_offset"])
    return {
        "format": PATCH_FORMAT,
        "encoding": "cp932",
        "volume_name": container["volume_name"],
        "volume_size": (
            text_json.parents[2] / container["volume_name"]
        ).stat().st_size,
        "address_shift": _infer_address_shift(
            volume_offset,
            logical_address,
        ),
        "scripts": [
            {
                "logical_address": f"{logical_address:08X}",
                "volume_offset": volume_offset,
                "container_size": int(container["container_size"]),
                "source_text_json": str(text_json),
                "replacements": replacements,
            }
        ],
    }


def _load_patch_files(paths: list[Path]) -> dict[str, Any]:
    combined: dict[str, Any] = {
        "format": PATCH_FORMAT,
        "encoding": None,
        "volume_name": None,
        "volume_size": None,
        "address_shift": None,
        "scripts": [],
    }
    for path in paths:
        patch = json.loads(path.read_text(encoding="utf-8"))
        if patch.get("format") != PATCH_FORMAT:
            raise decrypt_rio.FormatError(
                f"{path} is not a {PATCH_FORMAT} patch"
            )
        for key in ("encoding", "volume_name", "volume_size", "address_shift"):
            value = patch.get(key)
            if combined[key] is None:
                combined[key] = value
            elif value != combined[key]:
                raise decrypt_rio.FormatError(
                    f"patch files disagree about {key}"
                )
        for script in patch.get("scripts", []):
            combined["scripts"].append({**script, "_patch_file": str(path)})
    if not combined["scripts"]:
        raise decrypt_rio.FormatError("patch contains no scripts")
    return combined


def _read_crsa_container(
    stream: Any,
    volume_offset: int,
) -> bytes:
    stream.seek(volume_offset)
    header = stream.read(decrypt_rio.CRSA_HEADER_SIZE + 8)
    if len(header) != decrypt_rio.CRSA_HEADER_SIZE + 8:
        raise decrypt_rio.FormatError(
            f"CRsa header at 0x{volume_offset:X} is truncated"
        )
    _, encoded_size = decrypt_rio._encrypted_payload_header(
        header,
        decrypt_rio.CRSA_HEADER_SIZE,
    )
    container_size = decrypt_rio.CRSA_HEADER_SIZE + encoded_size
    stream.seek(volume_offset)
    data = stream.read(container_size)
    if len(data) != container_size:
        raise decrypt_rio.FormatError(
            f"CRsa container at 0x{volume_offset:X} is truncated"
        )
    return data


def prepare_volume_patches(
    volume_path: Path,
    patch_paths: list[Path],
) -> tuple[list[dict[str, Any]], int]:
    patch = _load_patch_files(patch_paths)
    if patch["volume_name"] and volume_path.name != patch["volume_name"]:
        raise decrypt_rio.FormatError(
            f"patch targets {patch['volume_name']}, not {volume_path.name}"
        )
    if patch["volume_size"] is not None:
        actual_size = volume_path.stat().st_size
        if actual_size != int(patch["volume_size"]):
            raise decrypt_rio.FormatError(
                f"volume size mismatch: expected {patch['volume_size']}, "
                f"got {actual_size}"
            )

    encoding = patch["encoding"] or "cp932"
    address_shift = int(patch["address_shift"])
    prepared = []
    changed_count = 0
    occupied: list[tuple[int, int]] = []
    with volume_path.open("rb") as stream:
        for script_spec in patch["scripts"]:
            logical_address = _parse_logical_address(
                script_spec["logical_address"]
            )
            volume_offset = int(script_spec["volume_offset"])
            if logical_address << address_shift != volume_offset:
                raise decrypt_rio.FormatError(
                    f"script {logical_address:08X} has inconsistent addresses"
                )

            container = _read_crsa_container(stream, volume_offset)
            actual_container_size = len(container)
            declared_size = int(
                script_spec.get("container_size", actual_container_size)
            )
            if declared_size not in (
                actual_container_size,
                actual_container_size - 2,
            ):
                raise decrypt_rio.FormatError(
                    f"script {logical_address:08X} container size mismatch: "
                    f"metadata={declared_size}, actual={actual_container_size}"
                )

            right = volume_offset + actual_container_size
            if any(
                volume_offset < old_right and old_left < right
                for old_left, old_right in occupied
            ):
                raise decrypt_rio.FormatError(
                    f"script {logical_address:08X} overlaps another patch"
                )
            occupied.append((volume_offset, right))

            script = decrypt_rio.decrypt_crsa_script(container)
            plaintext = bytearray(script["plaintext"])
            replacements = sorted(
                script_spec.get("replacements", []),
                key=lambda item: int(item["content_offset"]),
            )
            replacement_ranges: list[tuple[int, int]] = []
            script_changes = []
            for replacement in replacements:
                prefix_offset = int(replacement["prefix_offset"])
                content_offset = int(replacement["content_offset"])
                byte_length = int(replacement["byte_length"])
                if not 0 <= prefix_offset < len(plaintext):
                    raise decrypt_rio.FormatError(
                        f"prefix 0x{prefix_offset:X} is outside script "
                        f"{logical_address:08X}"
                    )
                if plaintext[prefix_offset] != byte_length:
                    raise decrypt_rio.FormatError(
                        f"length prefix changed at script "
                        f"{logical_address:08X}:0x{prefix_offset:X}"
                    )
                if content_offset != prefix_offset + 1:
                    raise decrypt_rio.FormatError(
                        "fixed-size injector currently supports one-byte "
                        "CString lengths only"
                    )
                content_end = content_offset + byte_length
                if content_end > len(plaintext):
                    raise decrypt_rio.FormatError("text extends past CRsa data")
                if any(
                    content_offset < old_end and old_start < content_end
                    for old_start, old_end in replacement_ranges
                ):
                    raise decrypt_rio.FormatError(
                        f"overlapping replacements in script "
                        f"{logical_address:08X}"
                    )
                replacement_ranges.append((content_offset, content_end))

                expected = bytes.fromhex(replacement["original_raw_hex"])
                current = bytes(plaintext[content_offset:content_end])
                if len(expected) != byte_length or current != expected:
                    raise decrypt_rio.FormatError(
                        f"original bytes do not match at script "
                        f"{logical_address:08X}:0x{prefix_offset:X}"
                    )
                if "replacement_raw_hex" in replacement:
                    new_value = bytes.fromhex(
                        replacement["replacement_raw_hex"]
                    )
                else:
                    new_value = _encode_rendered_text(
                        replacement["replacement"],
                        encoding,
                    )
                if len(new_value) != byte_length:
                    raise decrypt_rio.FormatError(
                        f"replacement at script {logical_address:08X}:"
                        f"0x{prefix_offset:X} is {len(new_value)} bytes; "
                        f"exactly {byte_length} bytes are required"
                    )
                if new_value == current:
                    continue
                plaintext[content_offset:content_end] = new_value
                changed_count += 1
                script_changes.append(
                    {
                        "prefix_offset": prefix_offset,
                        "byte_length": byte_length,
                        "old_hex": current.hex(),
                        "new_hex": new_value.hex(),
                    }
                )

            rebuilt = decrypt_rio.rebuild_crsa_script(
                container,
                bytes(plaintext),
            )
            if decrypt_rio.decrypt_crsa_script(rebuilt)["plaintext"] != bytes(
                plaintext
            ):
                raise decrypt_rio.FormatError(
                    f"round-trip verification failed for {logical_address:08X}"
                )
            prepared.append(
                {
                    "logical_address": f"{logical_address:08X}",
                    "volume_offset": volume_offset,
                    "container_size": len(container),
                    "original": container,
                    "rebuilt": rebuilt,
                    "changes": script_changes,
                }
            )
    return prepared, changed_count


def apply_volume_patches(
    volume_path: Path,
    patch_paths: list[Path],
    output_path: Path,
    *,
    force: bool,
) -> dict[str, Any]:
    prepared, changed_count = prepare_volume_patches(volume_path, patch_paths)
    if changed_count == 0:
        raise decrypt_rio.FormatError(
            "patch contains no changed replacement text"
        )
    if output_path.resolve() == volume_path.resolve():
        raise decrypt_rio.FormatError("output must be a new RIO file")
    if output_path.exists() and not force:
        raise decrypt_rio.FormatError(
            f"output already exists: {output_path} (use --force to replace)"
        )

    output_path.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(volume_path, output_path)
    with output_path.open("r+b") as stream:
        for item in prepared:
            stream.seek(item["volume_offset"])
            current = stream.read(item["container_size"])
            if current != item["original"]:
                raise decrypt_rio.FormatError(
                    f"output changed before patching "
                    f"{item['logical_address']}"
                )
            stream.seek(item["volume_offset"])
            stream.write(item["rebuilt"])

    with output_path.open("rb") as stream:
        for item in prepared:
            stream.seek(item["volume_offset"])
            written = stream.read(item["container_size"])
            if written != item["rebuilt"]:
                raise decrypt_rio.FormatError(
                    f"output verification failed for "
                    f"{item['logical_address']}"
                )
            decrypt_rio.decrypt_crsa_script(written)

    report = {
        "format": PATCH_FORMAT,
        "source_volume": str(volume_path),
        "output_volume": str(output_path),
        "patched_script_count": sum(bool(item["changes"]) for item in prepared),
        "replacement_count": changed_count,
        "scripts": [
            {
                key: value
                for key, value in item.items()
                if key not in ("original", "rebuilt")
            }
            for item in prepared
            if item["changes"]
        ],
    }
    report_path = output_path.with_name(output_path.name + ".textpatch.json")
    report_path.write_text(
        json.dumps(report, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    return report


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Create and apply fixed-size CRsa text patches."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    template_parser = subparsers.add_parser(
        "template",
        help="create a patch template from one exported text JSON",
    )
    template_parser.add_argument("text_json", type=Path)
    template_parser.add_argument(
        "prefix_offsets",
        nargs="+",
        type=_parse_offset,
        help="message prefix offsets, preferably written as 0xHEX",
    )
    template_parser.add_argument("-o", "--output", required=True, type=Path)

    check_parser = subparsers.add_parser(
        "check",
        help="validate patches against an original RIO volume",
    )
    check_parser.add_argument("volume", type=Path)
    check_parser.add_argument("patches", nargs="+", type=Path)

    apply_parser = subparsers.add_parser(
        "apply",
        help="write patched CRsa containers to a new RIO volume",
    )
    apply_parser.add_argument("volume", type=Path)
    apply_parser.add_argument("patches", nargs="+", type=Path)
    apply_parser.add_argument("-o", "--output", required=True, type=Path)
    apply_parser.add_argument("--force", action="store_true")

    args = parser.parse_args()
    try:
        if args.command == "template":
            patch = create_patch_template(
                args.text_json.resolve(),
                args.prefix_offsets,
            )
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(
                json.dumps(patch, ensure_ascii=False, indent=2),
                encoding="utf-8",
            )
            print(f"Wrote patch template: {args.output}")
        elif args.command == "check":
            scripts, changed = prepare_volume_patches(
                args.volume.resolve(),
                [path.resolve() for path in args.patches],
            )
            print(
                f"Validated {len(scripts)} script(s); "
                f"{changed} replacement(s) would change bytes."
            )
        else:
            report = apply_volume_patches(
                args.volume.resolve(),
                [path.resolve() for path in args.patches],
                args.output.resolve(),
                force=args.force,
            )
            print(
                f"Patched {report['replacement_count']} text(s) in "
                f"{report['patched_script_count']} script(s): "
                f"{report['output_volume']}"
            )
    except (OSError, KeyError, ValueError, decrypt_rio.FormatError) as error:
        print(f"ERROR: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
