#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import re
import sys

if len(sys.argv) != 2:
    raise SystemExit(f"usage: {Path(sys.argv[0]).name} <evidence-dir>")

evidence_dir = Path(sys.argv[1])
config_path = evidence_dir / "aspen-promoted-starter-config.toml"
default_log = evidence_dir / "aspen-promoted-starter-default.txt"
coordination_log = evidence_dir / "aspen-promoted-starter-coordination.txt"

for path in (config_path, default_log, coordination_log):
    if not path.exists():
        raise SystemExit(f"missing artifact: {path}")

config_text = config_path.read_text()
default_text = default_log.read_text()
coordination_text = coordination_log.read_text()

promoted = [
    "explicit_defaults",
    "unbounded_channel",
    "usize_in_public_api",
    "negated_predicate",
]
deferred = [
    "unbounded_collection_growth",
    "too_many_parameters",
]

for lint in promoted:
    needle = f'"{lint}"'
    if needle in config_text:
        raise SystemExit(f"promoted lint still disabled in config: {lint}")

for lint in deferred:
    needle = f'"{lint}"'
    if needle not in config_text:
        raise SystemExit(f"deferred lint missing from disabled_lints: {lint}")

required_default_crates = ["aspen-time", "aspen-hlc", "aspen-core"]
for crate in required_default_crates:
    if f"Checking {crate} v" not in default_text:
        raise SystemExit(f"default run missing expected starter crate: {crate}")
if "Checking aspen-coordination v" in default_text:
    raise SystemExit("default run unexpectedly checked aspen-coordination")
if "Checking aspen-coordination v" not in coordination_text:
    raise SystemExit("coordination run missing aspen-coordination")

lint_specs = [
    (
        "explicit_defaults",
        re.compile(r"^warning: implicit default on external type", re.MULTILINE),
        1,
        2,
        "promoted",
    ),
    (
        "unbounded_channel",
        re.compile(r"^warning: unbounded channel — use a bounded channel with explicit capacity", re.MULTILINE),
        0,
        0,
        "promoted",
    ),
    (
        "usize_in_public_api",
        re.compile(r"^warning: `(usize|isize)` in public API — use `(?:u|i)32` or `(?:u|i)64` for cross-platform consistency", re.MULTILINE),
        3,
        0,
        "promoted",
    ),
    (
        "negated_predicate",
        re.compile(r"^warning: negated predicate name `[^`]+` — prefer a positive form like `is_ready`", re.MULTILINE),
        0,
        2,
        "promoted",
    ),
    (
        "unbounded_collection_growth",
        re.compile(r"^warning: collection grows in loop without a prior reservation or explicit local bound", re.MULTILINE),
        0,
        0,
        "inventory-first (disabled in promoted starter profile)",
    ),
    (
        "too_many_parameters",
        re.compile(r"^warning: function has \d+ parameters \(limit: \d+\) — consider an options struct", re.MULTILINE),
        0,
        0,
        "inventory-first (disabled in promoted starter profile)",
    ),
]

print("# Aspen promoted starter validation")
print()
print("Phase-1 reference:")
print("- archive: `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`")
print("- Aspen commit: `909e5e9fa`")
print()
print("Starter scope:")
print("- `aspen-time`")
print("- `aspen-hlc`")
print("- `aspen-core`")
print()
print("Focused spotcheck:")
print("- `aspen-coordination --lib`")
print()
print("Promoted starter families:")
for lint in promoted:
    print(f"- `{lint}`")
print()
print("Still deferred:")
for lint in deferred:
    print(f"- `{lint}`")
print()
print("Config assertions:")
print("- promoted families omitted from `disabled_lints`: yes")
print("- `unbounded_collection_growth` stays disabled: yes")
print("- `too_many_parameters` stays disabled: yes")
print()
print("| Family | Starter scope (`aspen-time`, `aspen-hlc`, `aspen-core`) | `aspen-coordination --lib` | Starter status |")
print("| --- | ---: | ---: | --- |")
for lint_name, pattern, expected_default, expected_coordination, status in lint_specs:
    default_count = len(pattern.findall(default_text))
    coordination_count = len(pattern.findall(coordination_text))
    if default_count != expected_default:
        raise SystemExit(
            f"unexpected starter count for {lint_name}: expected {expected_default}, saw {default_count}"
        )
    if coordination_count != expected_coordination:
        raise SystemExit(
            f"unexpected coordination count for {lint_name}: expected {expected_coordination}, saw {coordination_count}"
        )
    print(f"| `{lint_name}` | {default_count} | {coordination_count} | {status} |")

print()
print("All promoted-profile assertions passed.")
