#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import re
import sys

if len(sys.argv) != 2:
    raise SystemExit(f"usage: {Path(sys.argv[0]).name} <evidence-dir>")

evidence_dir = Path(sys.argv[1])

lint_specs = [
    (
        "explicit_defaults",
        re.compile(r"^warning: implicit default on external type", re.MULTILINE),
    ),
    (
        "unbounded_channel",
        re.compile(r"^warning: unbounded channel — use a bounded channel with explicit capacity", re.MULTILINE),
    ),
    (
        "unbounded_collection_growth",
        re.compile(r"^warning: collection grows in loop without a prior reservation or explicit local bound", re.MULTILINE),
    ),
    (
        "usize_in_public_api",
        re.compile(r"^warning: `u?isize` in public API — use `(?:u|i)32` or `(?:u|i)64` for cross-platform consistency", re.MULTILINE),
    ),
    (
        "too_many_parameters",
        re.compile(r"^warning: function has \d+ parameters \(limit: \d+\) — consider an options struct", re.MULTILINE),
    ),
    (
        "negated_predicate",
        re.compile(r"^warning: negated predicate name `[^`]+` — prefer a positive form like `is_ready`", re.MULTILINE),
    ),
]

print("# Aspen new-lint validation counts")
print()
print("Low-noise rule for this change:")
print("- `starter-candidate`: <= 3 findings on the starter scope and <= 5 findings on `aspen-coordination`")
print("- `inventory-first`: anything larger stays out of starter-profile promotion for now")
print()
print("| Lint | Starter scope (`aspen-time`, `aspen-hlc`, `aspen-core`) | `aspen-coordination` | Classification |")
print("| --- | ---: | ---: | --- |")

for lint_name, pattern in lint_specs:
    default_log = evidence_dir / f"aspen-{lint_name}-default.txt"
    coord_log = evidence_dir / f"aspen-{lint_name}-coordination.txt"
    if not default_log.exists():
        raise SystemExit(f"missing log: {default_log}")
    if not coord_log.exists():
        raise SystemExit(f"missing log: {coord_log}")
    default_text = default_log.read_text()
    coord_text = coord_log.read_text()
    default_count = len(pattern.findall(default_text))
    coord_count = len(pattern.findall(coord_text))
    classification = "starter-candidate" if default_count <= 3 and coord_count <= 5 else "inventory-first"
    print(f"| `{lint_name}` | {default_count} | {coord_count} | {classification} |")
