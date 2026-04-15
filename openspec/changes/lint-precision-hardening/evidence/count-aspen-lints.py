#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

if len(sys.argv) != 10:
    raise SystemExit(
        "usage: count-aspen-lints.py BEFORE_LOG AFTER_LOG BEFORE_COMMIT AFTER_COMMIT ASPEN_COMMIT PHASE1_ARCHIVE PHASE2_ARCHIVE SCRIPT_REL SCOPE_DISPLAY"
    )

before_log = Path(sys.argv[1])
after_log = Path(sys.argv[2])
before_commit = sys.argv[3]
after_commit = sys.argv[4]
aspen_commit = sys.argv[5]
phase1_archive = sys.argv[6]
phase2_archive = sys.argv[7]
script_rel = sys.argv[8]
scope_display = sys.argv[9]

patterns = {
    "ignored_result": re.compile(r"^warning: `Result` value silently discarded with `let _ = \.\.\.`$", re.M),
    "no_unwrap": re.compile(r"^warning: `\.(?:unwrap|expect)\(\)` in production code will panic on failure$", re.M),
    "no_panic": re.compile(r"^warning: `(?:panic!|todo!|unimplemented!|unreachable!)` will abort the process on execution$", re.M),
    "unchecked_narrowing": re.compile(r"^warning: narrowing cast from `.*` to `.*` without bounds check$", re.M),
    "unbounded_loop": re.compile(r"^warning: loop without visible iteration bound$", re.M),
}


def counts(path: Path) -> dict[str, int]:
    text = path.read_text()
    return {name: len(pattern.findall(text)) for name, pattern in patterns.items()}


def note(before: int, after: int) -> str:
    delta = after - before
    if delta < 0:
        return f"reduced by {-delta}"
    if delta > 0:
        return f"regressed by {delta}"
    return "no count change"


before = counts(before_log)
after = counts(after_log)

print("# Aspen lint-precision comparison")
print()
print("- replay helper: `bash {}`".format(script_rel))
print(f"- Aspen commit: `{aspen_commit}`")
print(f"- baseline tigerstyle commit: `{before_commit}`")
print(f"- candidate tigerstyle commit: `{after_commit}`")
print(f"- Aspen comparison scope: `{scope_display}`")
print("- enabled lint families: `ignored_result`, `no_unwrap`, `no_panic`, `unchecked_narrowing`, `unbounded_loop`")
print(f"- phase-1 archive reference: `{phase1_archive}`")
print(f"- phase-2 archive reference: `{phase2_archive}`")
print(f"- baseline transcript: `{before_log.as_posix()}`")
print(f"- candidate transcript: `{after_log.as_posix()}`")
print()
print("| lint family | before | after | delta | note |")
print("|---|---:|---:|---:|---|")
for name in patterns:
    before_count = before[name]
    after_count = after[name]
    delta = after_count - before_count
    print(f"| `{name}` | {before_count} | {after_count} | {delta:+d} | {note(before_count, after_count)} |")
