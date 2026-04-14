#!/usr/bin/env python3
from pathlib import Path

script_path = Path(__file__).resolve()
repo_root = next(parent for parent in script_path.parents if (parent / ".git").exists())
doc = (repo_root / "docs" / "downstream-rollout.md").read_text()
template = (repo_root / "templates" / "tigerstyle-check.sh").read_text()
archive_notes = (repo_root / "openspec" / "changes" / "archive" / "2026-04-13-aspen-consumer-rollout" / "rollout-notes.md").read_text()
validation_counts = (repo_root / "openspec" / "changes" / "archive" / "2026-04-13-aspen-consumer-rollout" / "evidence" / "validation-counts.txt").read_text()
wrapper_snapshot = (repo_root / "openspec" / "changes" / "archive" / "2026-04-13-custom-clippy-lints" / "evidence" / "aspen-check-path-and-config.txt").read_text()
count_helper = (script_path.parent / "check-aspen-phase1-counts.py").read_text()
record_helper = (script_path.parent / "record-reconstruct-aspen-phase1.sh").read_text()
replay_helper = (script_path.parent / "reconstruct-aspen-phase1.sh").read_text()
transcript = (script_path.parent / "reconstruct-aspen-phase1.txt").read_text()

record_rel = "openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/record-reconstruct-aspen-phase1.sh"
replay_rel = "openspec/changes/archive/2026-04-14-aspen-feedback-hardening-transcript-capture/evidence/reconstruct-aspen-phase1.sh"
record_cmd = f"bash {record_rel}"
replay_cmd = f"bash {replay_rel}"
transcript_prefix = f"$ {record_cmd}\n$ {replay_cmd}\n"

required_doc = [
    "bash`, `cargo`, `rustc`, `python3`, `cargo-dylint`, `dylint-link`, and\n  `git` are on `PATH`",
    "standard Unix userland is available for `awk`, `basename`, `chmod`, `ln`,\n  `mkdir`, `mktemp`, and `sed`",
    "Exact saved-transcript replay path from this repo:",
    record_cmd,
    "It prints the same top-level command preamble saved in that transcript, then",
    "invokes the replay helper below.",
    '`aspen-dns -> /path/to/aspen-dns`',
    '`aspen-wasm-plugin -> /path/to/aspen-wasm-plugin`',
    '`aspen/` worktree at commit `909e5e9fa`',
    '- `aspen-hlc`: 0 starter findings',
    "- 21 `ambient_clock`",
    "- 2 `compound_assertion`",
    "- 4 `no_panic`",
    "TIGERSTYLE_DYLINT_REV=0e0a71eefe6f01563d11acc6e1d7af1d505934a9",
]
for fragment in required_doc:
    assert fragment in doc, fragment

assert "exact-replay/evidence/reconstruct-aspen-phase1.sh" not in doc
assert "realpath" not in template, "template still depends on realpath"
for fragment in [
    'Reference starter profile from docs/downstream-rollout.md:',
    'starter_lints=(',
    'echo "[tigerstyle] profile:   ${starter_lints[*]}"',
]:
    assert fragment in template, fragment

for fragment in [
    'default_hlc_checked = "Checking aspen-hlc v0.1.0" in default_log',
    'default_hlc_warning_summary = "warning: `aspen-hlc` (lib) generated" in default_log',
    'assert default_hlc_checked, "aspen-hlc was not checked"',
    'assert not default_hlc_warning_summary, "aspen-hlc emitted a warning summary"',
]:
    assert fragment in count_helper, fragment

for fragment in [
    f"record_rel='{record_rel}'",
    f"replay_rel='{replay_rel}'",
    "printf '$ bash %s\\n' \"$record_rel\"",
    "printf '$ bash %s\\n' \"$replay_rel\"",
    'bash "$script_dir/reconstruct-aspen-phase1.sh"',
    '| tee "$tmp_transcript"',
    'mv "$tmp_transcript" "$transcript"',
]:
    assert fragment in record_helper, fragment

assert transcript.startswith(transcript_prefix), transcript.splitlines()[:2]
assert "$ bash openspec/changes/archive/2026-04-14-aspen-feedback-hardening-exact-replay/evidence/reconstruct-aspen-phase1.sh" not in transcript
for fragment in [
    "default-run summary:",
    "  aspen-hlc: 0 starter findings",
    "coordination spotcheck summary:",
    "  total: 27 warnings",
]:
    assert fragment in transcript, fragment

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

for fragment in [
    "printf '$ ln -s %q %q\\n'",
    "printf '$ git -C %q worktree add --detach %q %q\\n'",
    'python3 "$script_dir/check-aspen-phase1-counts.py" "$default_log" "$coord_log"',
]:
    assert fragment in replay_helper, fragment

print("docs now name the archive-local transcript capture helper")
print("capture helper emits the saved transcript preamble and calls the replay helper")
print("saved transcript starts with the documented capture-helper preamble")
print("count checker and Aspen phase-1 facts still match archived rollout evidence")
