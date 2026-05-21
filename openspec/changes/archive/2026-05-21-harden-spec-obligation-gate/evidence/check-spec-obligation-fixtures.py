#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path


def repo_root() -> Path:
    for parent in Path(__file__).resolve().parents:
        if (parent / "openspec/config.yaml").exists() and (parent / "openspec/templates/spec.md").exists():
            return parent
    raise SystemExit("repo root not found")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def check_template(root: Path) -> None:
    text = (root / "openspec/templates/spec.md").read_text()
    required = [
        "Obligation cross-check",
        "proposal verification/evidence promise",
        "affected artifact",
        "non-goal boundary",
        "design decision/proof obligation",
        "explicit delta-spec requirement or named scenario",
        "proposal-stage gate review",
    ]
    for needle in required:
        require(needle in text, f"template missing {needle!r}")
    print("PASS template includes spec obligation cross-check fields")


def check_config(root: Path) -> None:
    text = (root / "openspec/config.yaml").read_text()
    required = [
        "Follow `openspec/templates/spec.md`",
        "cross-check proposal/design obligations",
        "explicit requirements or named scenarios",
        "boundary cases/negative controls/continued-warning",
        "exact proof output/schema",
    ]
    for needle in required:
        require(needle in text, f"config missing {needle!r}")
    print("PASS config includes spec-stage proposal/design obligation guidance")


def check_fixtures(root: Path) -> None:
    base = root / "tests/fixtures/openspec-spec-governance"
    missing = base / "spec-obligation-missing-blocked"
    covered = base / "spec-obligation-covered-pass"
    for path in [missing, covered]:
        require((path / ".openspec.yaml").exists(), f"fixture missing metadata: {path}")
        require((path / "proposal.md").exists(), f"fixture missing proposal: {path}")
        require((path / "design.md").exists(), f"fixture missing design: {path}")
        require((path / "specs/openspec-stage-authoring/spec.md").exists(), f"fixture missing spec: {path}")
    missing_spec = (missing / "specs/openspec-stage-authoring/spec.md").read_text()
    covered_spec = (covered / "specs/openspec-stage-authoring/spec.md").read_text()
    require("historical archived examples are not normalized" not in missing_spec, "missing fixture unexpectedly covers archive boundary")
    require("PASS archive untouched" not in missing_spec, "missing fixture unexpectedly covers evidence promise")
    require("historical archived examples are not normalized" in (missing / "proposal.md").read_text(), "missing fixture proposal lacks archive boundary")
    require("Archive non-goal and evidence promise are specified" in covered_spec, "covered fixture lacks obligation scenario")
    require("PASS archive untouched" in covered_spec, "covered fixture lacks exact evidence signal")
    runner = base / "run-spec-fixture-gate.sh"
    require(runner.exists(), "fixture runner missing")
    require(runner.stat().st_mode & 0o111, "fixture runner is not executable")
    print("PASS spec-obligation fixtures include missing and covered cases")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", choices=["template", "config", "fixtures"], required=True)
    args = parser.parse_args()
    root = repo_root()
    if args.check == "template":
        check_template(root)
    elif args.check == "config":
        check_config(root)
    else:
        check_fixtures(root)


if __name__ == "__main__":
    main()
