#!/usr/bin/env python3
from pathlib import Path

HERE = Path(__file__).resolve()
CHANGE_DIR = HERE.parents[1]
ROOT = next(parent for parent in HERE.parents if (parent / "docs").is_dir() and (parent / "openspec").is_dir())
paths = [
    ROOT / "docs/contributor-guide.md",
    ROOT / "docs/content-addressed-rust-evidence-boundaries.md",
    ROOT / "docs/current-head-valence-evidence.md",
    CHANGE_DIR / "specs/standards-codification/spec.md",
]
texts = {str(path.relative_to(ROOT)): path.read_text() for path in paths}
all_text = "\n".join(texts.values())
required = [
    "does not imply general THIR or MIR extraction",
    "selected-fixture-only",
    "unsupported-planned",
    "branch-control-flow",
    "semantic-equivalence",
]
for needle in required:
    if needle not in all_text:
        raise SystemExit(f"boundary: FAIL missing {needle!r}")
if "general MIR extraction" in all_text and "does not imply general THIR or MIR extraction" not in all_text:
    raise SystemExit("boundary: FAIL unbounded MIR extraction wording")
if "`checked-arithmetic`, `call-return`, and `branch-control-flow`" not in all_text:
    raise SystemExit("boundary: FAIL MIR selected inventory not explicit")
if "THIR selected fixture inventory remains unchanged" not in all_text:
    raise SystemExit("boundary: FAIL THIR preservation not explicit")
print("boundary: PASS")
print("assert.selected_fixture_only: PASS")
print("assert.default_non_fixture_unsupported_planned: PASS")
print("assert.no_general_mir_or_thir_extraction_claim: PASS")
print("assert.no_correctness_or_semantic_equivalence_claim: PASS")
print("assert.thir_inventory_preserved: PASS")
