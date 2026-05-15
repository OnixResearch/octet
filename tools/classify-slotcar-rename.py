#!/usr/bin/env python3
"""Classify remaining old-name strings during the slotcar rename."""
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKIP_PARTS = {".git", ".direnv", "target", ".pi", ".agent"}
BINARY_SUFFIXES = {".png", ".jpg", ".jpeg", ".gif", ".pdf", ".wasm", ".rlib", ".so", ".dylib", ".a"}


def classify(rel: str, line: str) -> str:
    lower = line.lower()
    if rel == "AGENTS.md" or rel.startswith("openspec/specs/"):
        return "compatibility"
    if rel.startswith("openspec/changes/archive/"):
        return "archived"
    if "global `tigerstyle` skill" in line or "/home/brittonr/git/tigerstyle" in line:
        return "compatibility"
    if "classify remaining non-archived tigerstyle strings" in line:
        return "compatibility"
    if "tiger style" in lower or "tigerbeetle" in lower:
        return "philosophy"
    if (
        "tigerstyle::" in line
        or "register_tool(tigerstyle)" in line
        or "tool lint" in lower
        or "libtigerstyle" in line
        or "tigerstyle dylint" in lower
        or "tigerstyle${" in line
        or "tigerstyle@" in line
    ):
        return "namespace"
    compatibility_tokens = [
        "cargo-tigerstyle",
        "cargo tigerstyle",
        "tigerstyle\"",
        "\"tigerstyle",
        "tigerstyle-",
        "-tigerstyle",
        "tigerstyle_",
        "_tigerstyle",
        "TIGERSTYLE_",
        "TIGERSTYLE ",
        "tigerstyle.",
        ".tigerstyle",
        "tigerstyle = slotcar",
        "[tigerstyle]",
        "workspace.metadata.tigerstyle",
        "workspace tigerstyle metadata",
        "tigerstyle nix parity",
        "compatibility",
        "alias",
        "historical",
        "old command",
        "previous command",
    ]
    if any(token in line for token in compatibility_tokens):
        return "compatibility"
    return "needs-fix"


def iter_text_files() -> list[Path]:
    paths: list[Path] = []
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        rel = path.relative_to(ROOT)
        if any(part in SKIP_PARTS for part in rel.parts):
            continue
        if rel.as_posix().endswith("/evidence/name-classification.txt"):
            continue
        if path.suffix.lower() in BINARY_SUFFIXES:
            continue
        paths.append(path)
    return sorted(paths, key=lambda p: p.relative_to(ROOT).as_posix())


def main() -> int:
    rows: list[tuple[str, str, int, str]] = []
    for path in iter_text_files():
        rel = path.relative_to(ROOT).as_posix()
        try:
            text = path.read_text()
        except UnicodeDecodeError:
            continue
        for line_number, line in enumerate(text.splitlines(), start=1):
            if "tigerstyle" not in line and "TIGERSTYLE" not in line and "Tiger Style" not in line:
                continue
            rows.append((classify(rel, line), rel, line_number, line.strip()))

    counts: dict[str, int] = {}
    for category, *_ in rows:
        counts[category] = counts.get(category, 0) + 1

    print("slotcar rename remaining-name classification")
    print(f"root: {ROOT}")
    for category in ["philosophy", "compatibility", "namespace", "archived", "needs-fix"]:
        print(f"count.{category}: {counts.get(category, 0)}")
    print("columns: category<TAB>path<TAB>line<TAB>text")
    for category, rel, line_number, text in rows:
        print(f"{category}\t{rel}\t{line_number}\t{text}")

    return 1 if counts.get("needs-fix", 0) else 0


if __name__ == "__main__":
    raise SystemExit(main())
