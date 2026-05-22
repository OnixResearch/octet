#!/usr/bin/env python3
from pathlib import Path
import sys

ROOT = Path(__file__).resolve()
for parent in ROOT.parents:
    if (parent / "docs" / "contributor-guide.md").exists():
        REPO = parent
        break
else:
    print("doc_boundaries: FAIL")
    print("- could not locate repo root")
    sys.exit(1)

checks = {
    "docs/contributor-guide.md": [
        "THIR: `checked-arithmetic`, `call-return`, and `branch-control-flow`; MIR: `checked-arithmetic` and `call-return`",
        "does not imply general THIR or MIR extraction",
        "thir-spike/branch_control_flow.rs::choose_branch",
        "THIR branch-control-flow does not imply general THIR extraction, correctness, semantic equivalence, or MIR branch-control-flow support",
        "rustc-thir-fixture --fixture-shape branch-control-flow",
    ],
    "docs/current-head-valence-evidence.md": [
        "checked-arithmetic`, `call-return`, and `branch-control-flow`",
        "MIR `branch-control-flow` remains unsupported",
        "positive branch-control-flow replay",
        "bounded local evidence replay",
    ],
}
missing = []
for rel, needles in checks.items():
    text = (REPO / rel).read_text(encoding="utf-8")
    for needle in needles:
        if needle not in text:
            missing.append(f"{rel}: missing {needle!r}")
if missing:
    print("doc_boundaries: FAIL")
    for item in missing:
        print(f"- {item}")
    sys.exit(1)
print("doc_boundaries: PASS")
print("assert.thir_branch_control_flow_selected_fixture_only: PASS")
print("assert.mir_branch_control_flow_unsupported_boundary: PASS")
