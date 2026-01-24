#!/usr/bin/env python3
"""Bump Zed extension version, update docs, and optionally build/test/tag/push."""

from __future__ import annotations

import argparse
import pathlib
import re
import subprocess
import sys
from typing import Iterable, List, Tuple

ROOT = pathlib.Path.cwd()


def die(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    sys.exit(1)


def read_text(path: pathlib.Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        die(f"missing required file: {path}")


def write_text(path: pathlib.Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")


def run(cmd: List[str]) -> None:
    subprocess.run(cmd, check=True)


def parse_notes(args: argparse.Namespace) -> List[str]:
    notes: List[str] = []
    if args.note:
        notes.extend(args.note)
    if args.notes_file:
        notes_path = pathlib.Path(args.notes_file)
        if not notes_path.exists():
            die(f"notes file not found: {notes_path}")
        for line in notes_path.read_text(encoding="utf-8").splitlines():
            stripped = line.strip()
            if stripped and not stripped.startswith("#"):
                notes.append(stripped)

    cleaned = [note.strip() for note in notes if note.strip()]
    if not cleaned and not args.allow_empty_notes:
        die("provide --note/--notes-file or pass --allow-empty-notes")
    if not cleaned and args.allow_empty_notes:
        cleaned = ["TODO: add release notes"]
    return cleaned


def update_extension_toml(path: pathlib.Path, version: str) -> Tuple[str, str]:
    text = read_text(path)
    match = re.search(r'^version\s*=\s*"([^"]+)"', text, flags=re.M)
    if not match:
        die("extension.toml missing version field")
    current = match.group(1)
    if current == version:
        die(f"extension.toml already at version {version}")
    updated = re.sub(
        r'^version\s*=\s*"([^"]+)"',
        f'version = "{version}"',
        text,
        count=1,
        flags=re.M,
    )
    return text, updated


def insert_changelog(path: pathlib.Path, version: str, notes: Iterable[str]) -> Tuple[str, str]:
    text = read_text(path)
    if re.search(rf"^##\s+{re.escape(version)}\b", text, flags=re.M):
        die(f"CHANGELOG already contains version {version}")
    bullets = "\n".join(f"- {note}" for note in notes)
    section = f"\n## {version}\n\n{bullets}\n"
    match = re.search(r"\n##\s", text)
    if match:
        updated = text[: match.start()] + section + text[match.start() :]
    else:
        updated = text.rstrip() + section + "\n"
    return text, updated


def replace_latest_readme(path: pathlib.Path, version: str) -> Tuple[str, str, bool]:
    text = read_text(path)
    updated, count = re.subn(
        r"(?m)^### Latest:\s*v\d+\.\d+\.\d+\s*$",
        f"### Latest: v{version}",
        text,
        count=1,
    )
    return text, updated, count > 0


def update_publishing(path: pathlib.Path, version: str) -> Tuple[str, str, List[str]]:
    text = read_text(path)
    warnings: List[str] = []

    updated, count = re.subn(
        r"(?m)^- \[x\] `extension\.toml` version: .*?$",
        f"- [x] `extension.toml` version: {version}",
        text,
        count=1,
    )
    if count == 0:
        warnings.append("PUBLISHING.md: extension.toml version line not found")

    updated, count = re.subn(
        r"(?m)^## Release Checklist for v\d+\.\d+\.\d+\s*$",
        f"## Release Checklist for v{version}",
        updated,
        count=1,
    )
    if count == 0:
        warnings.append("PUBLISHING.md: release checklist header not found")

    updated, count = re.subn(
        r"(?m)^- \[ \] Create git tag v\d+\.\d+\.\d+\s*$",
        f"- [ ] Create git tag v{version}",
        updated,
        count=1,
    )
    if count == 0:
        warnings.append("PUBLISHING.md: create git tag line not found")

    return text, updated, warnings


def update_test_version_checks(path: pathlib.Path, version: str) -> Tuple[str, str, bool]:
    text = read_text(path)
    updated, count = re.subn(
        r"(grep\s+-q\s+["\'])(\d+\.\d+\.\d+)(["\']\s+extension\.toml)",
        rf"\1{version}\3",
        text,
    )
    return text, updated, count > 0




def ensure_repo_files() -> None:
    required = [
        ROOT / "extension.toml",
        ROOT / "CHANGELOG.md",
    ]
    for path in required:
        if not path.exists():
            die(f"run from repo root; missing {path.name}")


def build_wasm() -> None:
    run(["cargo", "build", "--release", "--target", "wasm32-wasip1"])
    built = ROOT / "target" / "wasm32-wasip1" / "release" / "zed_css_variables.wasm"
    if not built.exists():
        die("build succeeded but wasm file not found")
    (ROOT / "extension.wasm").write_bytes(built.read_bytes())


def run_tests() -> None:
    run(["cargo", "test", "--lib"])
    run(["./test_extension.sh"])
    run(["./test_clean_install.sh"])


def git_status_files() -> List[str]:
    result = subprocess.run(
        ["git", "status", "--porcelain"],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    lines = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    files = [line[3:] for line in lines if len(line) > 3]
    return files


def git_add(paths: Iterable[pathlib.Path]) -> None:
    for path in paths:
        run(["git", "add", "--", str(path)])


def git_commit(message: str) -> None:
    run(["git", "commit", "-m", message])


def git_tag(tag: str) -> None:
    existing = subprocess.run(
        ["git", "tag", "--list", tag],
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    ).stdout.strip()
    if existing:
        die(f"git tag already exists: {tag}")
    run(["git", "tag", tag])


def git_push(remote: str, tag: str) -> None:
    run(["git", "push", remote, "HEAD"])
    run(["git", "push", remote, tag])


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Bump Zed extension version and optionally release.",
    )
    parser.add_argument("version", help="new version, e.g. 0.0.9")
    parser.add_argument("--note", action="append", help="release note bullet (repeatable)")
    parser.add_argument("--notes-file", help="path to a file with bullet notes (one per line)")
    parser.add_argument(
        "--allow-empty-notes",
        action="store_true",
        help="allow placeholder notes if none are provided",
    )
    parser.add_argument("--build", action="store_true", help="build wasm and copy to extension.wasm")
    parser.add_argument("--run-tests", action="store_true", help="run cargo/test scripts")
    parser.add_argument("--commit", action="store_true", help="commit release changes")
    parser.add_argument("--tag", action="store_true", help="create git tag vX.Y.Z")
    parser.add_argument("--push", action="store_true", help="push commit + tag to remote")
    parser.add_argument("--remote", default="origin", help="git remote to push to")
    parser.add_argument(
        "--allow-dirty",
        action="store_true",
        help="allow unrelated working tree changes when committing",
    )
    args = parser.parse_args()

    ensure_repo_files()
    notes = parse_notes(args)

    changed_files: List[pathlib.Path] = []

    ext_path = ROOT / "extension.toml"
    before, after = update_extension_toml(ext_path, args.version)
    if before != after:
        write_text(ext_path, after)
        changed_files.append(ext_path)

    changelog_path = ROOT / "CHANGELOG.md"
    before, after = insert_changelog(changelog_path, args.version, notes)
    if before != after:
        write_text(changelog_path, after)
        changed_files.append(changelog_path)

    readme_path = ROOT / "README.md"
    if readme_path.exists():
        before, after, updated = replace_latest_readme(readme_path, args.version)
        if updated and before != after:
            write_text(readme_path, after)
            changed_files.append(readme_path)

    publishing_path = ROOT / "PUBLISHING.md"
    if publishing_path.exists():
        before, after, warnings = update_publishing(publishing_path, args.version)
        if before != after:
            write_text(publishing_path, after)
            changed_files.append(publishing_path)
        for warning in warnings:
            print(f"warning: {warning}", file=sys.stderr)

    test_paths = [
        ROOT / "test_extension.sh",
        ROOT / ".github" / "workflows" / "test.yml",
    ]
    for test_path in test_paths:
        if test_path.exists():
            before, after, updated = update_test_version_checks(test_path, args.version)
            if updated and before != after:
                write_text(test_path, after)
                changed_files.append(test_path)

    if args.build:
        build_wasm()
        wasm_path = ROOT / "extension.wasm"
        if wasm_path.exists():
            changed_files.append(wasm_path)

    if args.run_tests:
        run_tests()

    if args.commit:
        changed_set = {str(path) for path in changed_files}
        status_files = git_status_files()
        if status_files and not args.allow_dirty:
            extras = [path for path in status_files if path not in changed_set]
            if extras:
                die(f"working tree has unrelated changes: {', '.join(extras)}")
        git_add(changed_files)
        git_commit(f"Release v{args.version}")

    tag_name = f"v{args.version}"
    if args.tag:
        git_tag(tag_name)

    if args.push:
        if not args.tag:
            existing = subprocess.run(
                ["git", "tag", "--list", tag_name],
                check=True,
                stdout=subprocess.PIPE,
                text=True,
            ).stdout.strip()
            if not existing:
                die("--push requires --tag or an existing tag")
        git_push(args.remote, tag_name)

    print("done")


if __name__ == "__main__":
    main()
