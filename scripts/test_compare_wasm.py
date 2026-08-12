#!/usr/bin/env python3

from __future__ import annotations

import unittest

from compare_wasm import WASM_MAGIC, canonicalize_wasm


def encode_varuint32(value: int) -> bytes:
    encoded = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            encoded.append(byte | 0x80)
        else:
            encoded.append(byte)
            return bytes(encoded)


def section(section_id: int, payload: bytes) -> bytes:
    return bytes([section_id]) + encode_varuint32(len(payload)) + payload


def custom_section(name: bytes, payload: bytes) -> bytes:
    return section(0, encode_varuint32(len(name)) + name + payload)


def module(*sections: bytes) -> bytes:
    return WASM_MAGIC + b"".join(sections)


class CompareWasmTests(unittest.TestCase):
    def test_name_section_is_ignored_for_cross_platform_paths(self) -> None:
        mac = module(
            custom_section(b"name", b"/Users/runner/work/css-variables/src/lib.rs"),
            section(1, b"core-section"),
        )
        linux = module(
            custom_section(b"name", b"/home/runner/work/css-variables/src/lib.rs"),
            section(1, b"core-section"),
        )

        self.assertEqual(canonicalize_wasm(mac), canonicalize_wasm(linux))

    def test_other_custom_sections_remain_significant(self) -> None:
        left = module(custom_section(b"zed:api-version", b"0.7"))
        right = module(custom_section(b"zed:api-version", b"0.8"))

        self.assertNotEqual(canonicalize_wasm(left), canonicalize_wasm(right))

    def test_core_sections_remain_significant(self) -> None:
        left = module(section(1, b"core-section-a"))
        right = module(section(1, b"core-section-b"))

        self.assertNotEqual(canonicalize_wasm(left), canonicalize_wasm(right))

    def test_malformed_modules_are_rejected(self) -> None:
        with self.assertRaises(ValueError):
            canonicalize_wasm(WASM_MAGIC + b"\x01\x80")

        with self.assertRaises(ValueError):
            canonicalize_wasm(WASM_MAGIC + b"\x00\x05\x01")


if __name__ == "__main__":
    unittest.main()
