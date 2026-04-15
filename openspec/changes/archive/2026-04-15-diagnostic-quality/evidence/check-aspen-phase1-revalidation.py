#!/usr/bin/env python3
from pathlib import Path
import sys

if len(sys.argv) != 3:
    raise SystemExit(f"usage: {sys.argv[0]} DEFAULT_LOG COORD_LOG")

def count(text: str, needle: str) -> int:
    return text.count(needle)

default_log = Path(sys.argv[1]).read_text()
coord_log = Path(sys.argv[2]).read_text()

default_ambient = count(default_log, "warning: direct system clock read in production code")
default_time = count(default_log, "warning: `aspen-time` (lib) generated 2 warnings")
default_core = count(default_log, "warning: `aspen-core` (lib) generated 4 warnings")
default_hlc_checked = "Checking aspen-hlc v0.1.0" in default_log
default_hlc_warning_summary = "warning: `aspen-hlc` (lib) generated" in default_log
coord_ambient = count(coord_log, "warning: direct system clock read in production code")
coord_compound = count(coord_log, "warning: compound assertion should be split into separate assertions")
coord_no_panic = count(coord_log, "will abort the process on execution")
coord_total = count(coord_log, "warning: `aspen-coordination` (lib) generated 22 warnings")

assert default_ambient == 6, default_ambient
assert default_time == 1, default_time
assert default_core == 1, default_core
assert default_hlc_checked, "aspen-hlc was not checked"
assert not default_hlc_warning_summary, "aspen-hlc emitted a warning summary"
assert "generated 0 warnings" not in default_log
assert coord_ambient == 21, coord_ambient
assert coord_compound == 1, coord_compound
assert coord_no_panic == 0, coord_no_panic
assert coord_total == 1, coord_total

print("default-run summary:")
print("  aspen-time: 2 ambient_clock")
print("  aspen-core: 4 ambient_clock")
print("  aspen-hlc: 0 starter findings")
print("  total: 6 warnings")
print("coordination spotcheck summary:")
print("  ambient_clock: 21")
print("  compound_assertion: 1")
print("  no_panic: 0")
print("  total: 22 warnings")
print("note: archived phase-1 coordination evidence was 27 warnings (21 ambient_clock, 2 compound_assertion, 4 no_panic); current tigerstyle keeps the archived starter path working while later lint-precision work lowers that spotcheck baseline.")
