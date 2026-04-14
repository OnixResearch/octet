#!/usr/bin/env python3
from pathlib import Path

repo_root = Path(__file__).resolve().parents[4]
doc = (repo_root / "docs" / "downstream-rollout.md").read_text()
template = (repo_root / "templates" / "tigerstyle-check.sh").read_text()
archive_notes = (repo_root / "openspec" / "changes" / "archive" / "2026-04-13-aspen-consumer-rollout" / "rollout-notes.md").read_text()
validation_counts = (repo_root / "openspec" / "changes" / "archive" / "2026-04-13-aspen-consumer-rollout" / "evidence" / "validation-counts.txt").read_text()
wrapper_snapshot = (repo_root / "openspec" / "changes" / "archive" / "2026-04-13-custom-clippy-lints" / "evidence" / "aspen-check-path-and-config.txt").read_text()

required_doc = [
    "bash`, `cargo`, `rustc`, `python3`, `cargo-dylint`, `dylint-link`, and\n  `git` are on `PATH`",
    "standard Unix userland is available for `awk`, `basename`, `chmod`, `ln`,\n  `mkdir`, `mktemp`, and `sed`",
    'git -C /path/to/aspen worktree add --detach /tmp/aspen-phase1 909e5e9fa',
    'scripts/tigerstyle-check.sh -p aspen-coordination -- --lib',
    "- 21 `ambient_clock`",
    "- 2 `compound_assertion`",
    "- 4 `no_panic`",
    "TIGERSTYLE_DYLINT_REV=0e0a71eefe6f01563d11acc6e1d7af1d505934a9",
]
for fragment in required_doc:
    assert fragment in doc, fragment

assert "realpath" not in template, "template still depends on realpath"
for fragment in [
    'Reference starter profile from docs/downstream-rollout.md:',
    'starter_lints=(',
    'echo "[tigerstyle] profile:   ${starter_lints[*]}"',
]:
    assert fragment in template, fragment

for fragment in [
    "Phase 1 starter lints:",
    "- `ambient_clock`",
    "- `compound_assertion`",
    "- `contradictory_time`",
    "Total starter-path findings: 6 warnings, all in `ambient_clock`.",
]:
    assert fragment in archive_notes, fragment

for fragment in [
    "default_pilot: aspen-time=2 ambient_clock, aspen-core=4 ambient_clock, aspen-hlc=0 starter findings, total=6",
    "coordination_spotcheck: ambient_clock=21, compound_assertion=2, no_panic=4, total=27",
    "0e0a71eefe6f01563d11acc6e1d7af1d505934a9",
]:
    haystack = validation_counts + "\n" + wrapper_snapshot
    assert fragment in haystack, fragment

print("docs assumptions match template behavior")
print("template no longer depends on realpath")
print("docs reconstruction path still matches archived Aspen phase-1 counts and wrapper facts")
