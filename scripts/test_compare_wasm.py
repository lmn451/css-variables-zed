#!/usr/bin/env python3

from __future__ import annotations

import unittest

from compare_wasm import COMPONENT_MAGIC, WASM_MAGIC, canonicalize_wasm


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

    def test_component_nested_names_are_ignored(self) -> None:
        def component(path: bytes) -> bytes:
            core = module(custom_section(b"name", path), section(10, b"code"))
            nested = COMPONENT_MAGIC + section(1, core)
            return COMPONENT_MAGIC + section(4, nested)

        self.assertEqual(
            canonicalize_wasm(component(b"/Users/runner/" + b"a" * 160)),
            canonicalize_wasm(component(b"/home/runner/work")),
        )

    def test_nested_code_and_component_metadata_remain_significant(self) -> None:
        left = COMPONENT_MAGIC + section(1, module(section(10, b"code-a")))
        right = COMPONENT_MAGIC + section(1, module(section(10, b"code-b")))
        self.assertNotEqual(canonicalize_wasm(left), canonicalize_wasm(right))
        self.assertNotEqual(
            canonicalize_wasm(left + custom_section(b"zed:api-version", b"0.7")),
            canonicalize_wasm(left + custom_section(b"zed:api-version", b"0.8")),
        )

    def test_component_sections_are_preserved(self) -> None:
        component = COMPONENT_MAGIC + section(2, b"instance") + custom_section(
            b"component-name", b"identity"
        )
        self.assertEqual(canonicalize_wasm(component), component)
        self.assertNotEqual(canonicalize_wasm(COMPONENT_MAGIC), canonicalize_wasm(WASM_MAGIC))

    def test_malformed_components_are_rejected(self) -> None:
        for invalid in (
            COMPONENT_MAGIC + section(1, b"invalid"),
            COMPONENT_MAGIC + section(4, WASM_MAGIC),
            COMPONENT_MAGIC + section(1, WASM_MAGIC + b"\x01\x80"),
            b"\x00asm\x02\x00\x00\x00",
        ):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                canonicalize_wasm(invalid)


if __name__ == "__main__":
    unittest.main()
