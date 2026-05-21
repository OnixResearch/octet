#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path


def repo_root() -> Path:
    here = Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "openspec/config.yaml").exists() and (parent / "openspec/templates/proposal.md").exists():
            return parent
    raise SystemExit("could not find repo root")


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        raise SystemExit(f"missing {label}: {needle}")


def check_template(root: Path) -> None:
    text = (root / "openspec/templates/proposal.md").read_text()
    for needle, label in [
        ("planned verification/evidence expectations", "verification/evidence prompt"),
        ("exact affected files/artifacts", "affected artifacts prompt"),
        ("concrete package/profile/fixture scope", "concrete scope prompt"),
        ("downstream validation boundaries", "downstream boundary prompt"),
        ("explicit non-goal boundaries", "non-goal boundary prompt"),
        ("exact expected output/schema", "exact proof output prompt"),
    ]:
        require(text, needle, label)
    print("PASS proposal template includes obligation fields")


def check_config(root: Path) -> None:
    text = (root / "openspec/config.yaml").read_text()
    for needle, label in [
        ("Follow `openspec/templates/proposal.md`", "proposal template config rule"),
        ("exact repo-relative files/artifacts", "exact artifacts config rule"),
        ("concrete package/profile/fixture or downstream target scope", "concrete scope config rule"),
        ("downstream validation boundaries", "downstream config rule"),
        ("planned saved evidence or helper/gate runs with exact expected output/schema", "evidence config rule"),
    ]:
        require(text, needle, label)
    print("PASS config includes proposal-stage obligation guidance")


def check_fixtures(root: Path) -> None:
    base = root / "tests/fixtures/openspec-proposal-governance"
    missing = base / "proposal-obligation-missing-blocked"
    covered = base / "proposal-obligation-covered-pass"
    for path in [missing, covered, base / "run-proposal-fixture-gate.sh"]:
        if not path.exists():
            raise SystemExit(f"missing fixture path: {path}")
    if (missing / "expected_gate.txt").read_text().strip() != "blocked":
        raise SystemExit("missing fixture expected_gate is not blocked")
    if (covered / "expected_gate.txt").read_text().strip() != "pass":
        raise SystemExit("covered fixture expected_gate is not pass")
    missing_text = (missing / "proposal.md").read_text()
    covered_text = (covered / "proposal.md").read_text()
    if "Concrete scope" in missing_text or "Downstream validation boundaries" in missing_text:
        raise SystemExit("missing fixture unexpectedly names concrete scope/downstream fields")
    for needle in [
        "**Concrete scope**",
        "**Downstream validation boundaries**",
        "PASS expected passing fixture: proposal-obligation-covered-pass",
        "PASS proposal template includes obligation fields",
    ]:
        require(covered_text, needle, f"covered proposal {needle}")
    runner = (base / "run-proposal-fixture-gate.sh").read_text()
    for needle in ["stage=proposal", "omission", "proposal|missing|scope|evidence|artifact"]:
        require(runner, needle, f"runner {needle}")
    print("PASS proposal-obligation fixtures encode missing and covered obligations")


def main() -> None:
    if len(sys.argv) != 2 or sys.argv[1] not in {"template", "config", "fixtures"}:
        raise SystemExit("usage: check-proposal-obligation-fixtures.py template|config|fixtures")
    root = repo_root()
    if sys.argv[1] == "template":
        check_template(root)
    elif sys.argv[1] == "config":
        check_config(root)
    else:
        check_fixtures(root)


if __name__ == "__main__":
    main()
