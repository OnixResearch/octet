#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[5]
UI_RS = REPO_ROOT / "tests" / "ui.rs"
SNAPSHOTS = [
    REPO_ROOT / "tests" / "fixtures" / "lint_inventory_names.txt",
    REPO_ROOT / "tests" / "fixtures" / "lint_inventory_default_levels.txt",
]
PER_LINT_FIXTURES = [
    "no_unwrap_disabled_custom/no_unwrap_disabled_custom",
    "no_unwrap_allow_custom/no_unwrap_allow_custom",
    "no_unwrap_warn_custom/no_unwrap_warn_custom",
    "no_unwrap_deny_custom/no_unwrap_deny_custom",
    "no_unwrap_default_custom/no_unwrap_default_custom",
    "no_unwrap_disabled_deny_custom/no_unwrap_disabled_deny_custom",
    "no_unwrap_disabled_source_attr_custom/no_unwrap_disabled_source_attr_custom",
]


def parse_fixtures() -> list[str]:
    text = UI_RS.read_text()
    match = re.search(r"const FIXTURES: &\[&str\] = &\[(.*?)\];", text, re.S)
    if not match:
        raise SystemExit(f"FIXTURES block not found in {UI_RS}")
    return re.findall(r'"([^"]+)"', match.group(1))


def main() -> None:
    fixtures = parse_fixtures()
    missing = [fixture for fixture in PER_LINT_FIXTURES if fixture not in fixtures]
    if missing:
        raise SystemExit(f"missing per-lint fixtures in tests/ui.rs: {missing}")

    print(f"tests/ui.rs fixture count: {len(fixtures)}")
    print("per-lint fixtures required by per-lint-configuration:")
    for fixture in PER_LINT_FIXTURES:
        print(f"- {fixture}")
    print()

    print("all tests/ui.rs fixtures:")
    for fixture in fixtures:
        print(f"- {fixture}")
    print()

    for snapshot in SNAPSHOTS:
        if not snapshot.exists():
            raise SystemExit(f"missing snapshot: {snapshot}")
        rel = snapshot.relative_to(REPO_ROOT)
        contents = snapshot.read_text().rstrip("\n")
        if not contents:
            raise SystemExit(f"empty snapshot: {rel}")
        print(f"== {rel} ==")
        print(contents)
        print()

    print("ok")


if __name__ == "__main__":
    main()
