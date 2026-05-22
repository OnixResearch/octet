#!/usr/bin/env python3
from pathlib import Path


def find_repo_root() -> Path:
    for parent in Path(__file__).resolve().parents:
        if (parent / "docs").is_dir() and (parent / "openspec").is_dir():
            return parent
    raise SystemExit("could not find repo root for doc boundary check")

ROOT = find_repo_root()
DOCS = {
    "docs/contributor-guide.md": (ROOT / "docs/contributor-guide.md").read_text(),
    "docs/current-head-valence-evidence.md": (ROOT / "docs/current-head-valence-evidence.md").read_text(),
}

REQUIRED = {
    "docs/contributor-guide.md": [
        "nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L",
        "nix build .#checks.x86_64-linux.mir-supplement-inventory-gate --no-link -L",
        "valence_object_evidence.mir_supplement_inventory_gate: PASS",
        "selected-fixture-only (`checked-arithmetic` and `call-return`)",
        "not a correctness proof",
        "does not imply general THIR or MIR extraction",
    ],
    "docs/current-head-valence-evidence.md": [
        "nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L",
        "nix build .#checks.x86_64-linux.mir-supplement-inventory-gate --no-link -L",
        "valence_object_evidence.mir_supplement_inventory_gate: PASS",
        "checked-arithmetic",
        "call-return",
        "unsupported-planned",
        "not a correctness proof",
        "does not embed generated receipts, move any readiness tag, or claim broader MIR support",
    ],
}

failures = []
for path, needles in REQUIRED.items():
    text = DOCS[path]
    for needle in needles:
        if needle not in text:
            failures.append(f"{path}: missing {needle!r}")

if failures:
    print("doc_boundaries: FAIL")
    for failure in failures:
        print(f"- {failure}")
    raise SystemExit(1)

print("doc_boundaries.aggregate_command: PASS")
print("doc_boundaries.mir_inventory_gate: PASS")
print("doc_boundaries.selected_mir_inventory: PASS")
print("doc_boundaries.unsupported_planned_boundary: PASS")
print("doc_boundaries.non_proof_boundaries: PASS")
