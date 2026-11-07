#!/usr/bin/env python3
"""Check language IDs and capabilities against the extension's runtime paths."""

import tomllib
import unittest
from pathlib import Path


class ExtensionManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.manifest = tomllib.loads(
            (Path(__file__).resolve().parent.parent / "extension.toml").read_text()
        )

    def test_language_ids_cover_registered_languages(self) -> None:
        server = self.manifest["language_servers"]["css-variables"]
        self.assertEqual(set(server["languages"]), set(server["language_ids"]))
        self.assertEqual(server["language_ids"]["Vue.js"], "vue")
        self.assertEqual(server["language_ids"]["JSX"], "javascriptreact")
        self.assertEqual(server["language_ids"]["TSX"], "typescriptreact")

    def test_installation_and_path_probe_capabilities(self) -> None:
        capabilities = self.manifest["capabilities"]
        self.assertIn(
            {"kind": "npm:install", "package": "css-variable-lsp"}, capabilities
        )
        self.assertIn(
            {
                "kind": "download_file",
                "host": "github.com",
                "path": ["lmn451", "css-lsp-rust", "**"],
            },
            capabilities,
        )
        for flag in ("--version", "-V"):
            self.assertIn(
                {"kind": "process:exec", "command": "*", "args": [flag]}, capabilities
            )


if __name__ == "__main__":
    unittest.main()