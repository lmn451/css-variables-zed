#!/usr/bin/env python3
"""Compare WASM artifacts while ignoring platform-specific name metadata."""

from __future__ import annotations

import hashlib
import sys
from pathlib import Path
from typing import Iterator, Tuple

WASM_MAGIC = b"\x00asm\x01\x00\x00\x00"


def read_varuint32(data: bytes, offset: int) -> Tuple[int, int]:
    """Read one unsigned 32-bit LEB128 value and return value plus next offset."""

    value = 0
    for shift in range(0, 35, 7):
        if offset >= len(data):
            raise ValueError("truncated unsigned LEB128 value")

        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift

        if byte & 0x80 == 0:
            if shift == 28 and byte > 0x0F:
                raise ValueError("unsigned LEB128 value exceeds 32 bits")
            return value, offset

    raise ValueError("unsigned LEB128 value exceeds 32 bits")


def iter_sections(data: bytes) -> Iterator[Tuple[int, int, int, int]]:
    """Yield section start, payload start, payload end, and section id."""

    if not data.startswith(WASM_MAGIC):
        raise ValueError("not a WebAssembly binary")

    offset = len(WASM_MAGIC)
    while offset < len(data):
        section_start = offset
        section_id = data[offset]
        offset += 1
        payload_size, payload_start = read_varuint32(data, offset)
        payload_end = payload_start + payload_size
        if payload_end > len(data):
            raise ValueError("WASM section extends past the end of the file")

        yield section_start, payload_start, payload_end, section_id
        offset = payload_end


def custom_section_name(data: bytes, payload_start: int, payload_end: int) -> bytes:
    """Return the raw name of a custom section."""

    name_length, name_start = read_varuint32(data, payload_start)
    name_end = name_start + name_length
    if name_end > payload_end:
        raise ValueError("WASM custom-section name extends past its payload")
    return data[name_start:name_end]


def canonicalize_wasm(data: bytes) -> bytes:
    """Remove only the platform-sensitive ``name`` custom section.

    All executable, data, component, and other custom sections remain byte-for-byte
    significant. The ``name`` section contains compiler/debug names and can include
    absolute source paths that differ between macOS and Linux builds.
    """

    canonical = bytearray(WASM_MAGIC)
    for section_start, payload_start, payload_end, section_id in iter_sections(data):
        if section_id == 0 and custom_section_name(data, payload_start, payload_end) == b"name":
            continue
        canonical.extend(data[section_start:payload_end])
    return bytes(canonical)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def compare_paths(left_path: Path, right_path: Path) -> int:
    try:
        left = left_path.read_bytes()
        right = right_path.read_bytes()
        left_canonical = canonicalize_wasm(left)
        right_canonical = canonicalize_wasm(right)
    except (OSError, ValueError) as error:
        print(f"WASM verification error: {error}", file=sys.stderr)
        return 2

    if left_canonical == right_canonical:
        print("WASM artifacts match after canonicalizing platform-specific name metadata.")
        return 0

    print("WASM artifacts differ after canonicalization.", file=sys.stderr)
    print(
        f"  {left_path}: raw={len(left)} canonical={len(left_canonical)} "
        f"sha256={sha256(left_canonical)}",
        file=sys.stderr,
    )
    print(
        f"  {right_path}: raw={len(right)} canonical={len(right_canonical)} "
        f"sha256={sha256(right_canonical)}",
        file=sys.stderr,
    )
    return 1


def main() -> int:
    if len(sys.argv) != 3:
        print(f"usage: {Path(sys.argv[0]).name} BUILT_WASM TRACKED_WASM", file=sys.stderr)
        return 2
    return compare_paths(Path(sys.argv[1]), Path(sys.argv[2]))


if __name__ == "__main__":
    raise SystemExit(main())
