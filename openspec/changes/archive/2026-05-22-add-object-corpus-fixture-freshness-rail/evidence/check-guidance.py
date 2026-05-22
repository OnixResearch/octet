#!/usr/bin/env python3
from pathlib import Path

root = next(parent for parent in Path(__file__).resolve().parents if (parent / 'flake.nix').exists())
contributor = (root / "docs/contributor-guide.md").read_text()
tasks = (root / "openspec/templates/tasks.md").read_text()

required_contributor = [
    "nix build .#checks.x86_64-linux.object-corpus-fixture-freshness --no-link -L",
    "valence_object_evidence.object_corpus_fixture_freshness_gate: PASS",
    "cargo-octet/src/object.rs",
    "cargo-octet/src/cli.rs",
    "cargo-octet/tests/fingerprint_consumer.rs",
    "THIR/MIR fixture inventories",
    "cargo-octet/tests/fixtures/object-corpus/*",
    "Regenerate stale fixtures",
]
required_tasks = [
    "nix build .#checks.x86_64-linux.object-corpus-fixture-freshness --no-link -L",
    "cargo-octet/src/object.rs",
    "cargo-octet/src/cli.rs",
    "cargo-octet/tests/fingerprint_consumer.rs",
    "THIR/MIR fixture inventories",
    "cargo-octet/tests/fixtures/object-corpus/*",
]

missing = []
for needle in required_contributor:
    if needle not in contributor:
        missing.append(f"docs/contributor-guide.md missing: {needle}")
for needle in required_tasks:
    if needle not in tasks:
        missing.append(f"openspec/templates/tasks.md missing: {needle}")

if missing:
    print("guidance: FAIL")
    for item in missing:
        print(item)
    raise SystemExit(1)

print("guidance: PASS")
print("assert.named_freshness_check_documented: PASS")
print("assert.aggregate_pass_line_documented: PASS")
print("assert_touch_surfaces_require_freshness_vtask: PASS")
