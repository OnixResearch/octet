#!/usr/bin/env python3
"""Deterministic checks for design-obligation gate fixtures."""
from __future__ import annotations

import argparse
from pathlib import Path

def find_repo_root(start: Path) -> Path:
    for candidate in [start, *start.parents]:
        if (candidate / "openspec/config.yaml").exists() and (candidate / "openspec/templates/design.md").exists():
            return candidate
    raise SystemExit("could not locate repository root from helper path")


ROOT = find_repo_root(Path(__file__).resolve())
TEMPLATE = ROOT / "openspec/templates/design.md"
CONFIG = ROOT / "openspec/config.yaml"
FIXTURES = ROOT / "tests/fixtures/openspec-design-governance"
OBLIGATION = "same-branch guard suppression"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def line_with(text: str, needle: str) -> str:
    matches = [line for line in text.splitlines() if needle in line]
    require(len(matches) == 1, f"expected exactly one line containing {needle!r}, found {len(matches)}")
    return matches[0]


def run_template() -> None:
    text = TEMPLATE.read_text()
    comment = line_with(text, "Omission-prevention checklist:")
    label = line_with(text, "**Obligation cross-check:**")
    require("proposal verification/evidence promise" in comment, "template comment lacks proposal evidence promise")
    require("delta-spec boundary or negative-control obligation" in comment, "template comment lacks delta-spec obligation")
    require("proposal promises" in label, "cross-check label lacks proposal promises")
    require("negative controls" in label, "cross-check label lacks negative controls")
    print("template obligation prompt: PASS")
    print(comment)
    print(label)


def run_config() -> None:
    text = CONFIG.read_text()
    proposal_line = line_with(text, "Follow `openspec/templates/proposal.md`")
    spec_line = line_with(text, "Follow `openspec/templates/spec.md`")
    design_line = line_with(text, "proposal/spec obligation cross-checks")
    require("planned saved evidence" in proposal_line or "verification/evidence" in proposal_line, "proposal rule lacks verification/evidence expectations")
    require("affected artifacts" in proposal_line or "files/artifacts" in proposal_line, "proposal rule lacks affected artifacts")
    require("boundary cases" in spec_line, "delta-spec rule lacks boundary cases")
    require("negative controls" in spec_line, "delta-spec rule lacks negative controls")
    require("proof output" in spec_line, "delta-spec rule lacks proof output expectations")
    require("proposal verification promises" in design_line, "design rule lacks proposal verification promises")
    require("delta-spec boundary cases" in design_line, "design rule lacks delta-spec boundary cases")
    require("saved proof obligations" in design_line, "design rule lacks saved proof obligations")
    print("config guidance trio: PASS")
    print(proposal_line)
    print(spec_line)
    print(design_line)


def section(text: str, heading: str) -> str:
    start = text.index(heading)
    rest = text[start + len(heading):]
    marker = "\n## "
    end = rest.find(marker)
    return rest if end == -1 else rest[:end]


def run_fixtures() -> None:
    missing = (FIXTURES / "design-obligation-missing-blocked/design.md").read_text()
    passed = (FIXTURES / "design-obligation-covered-pass/design.md").read_text()
    missing_design = section(missing, "## Decisions") + section(missing, "## Deterministic Proof Plan")
    pass_design = section(passed, "## Decisions") + section(passed, "## Deterministic Proof Plan")
    require(OBLIGATION not in missing_design, "missing fixture unexpectedly covers obligation")
    require("**Obligation cross-check:**" not in missing_design, "missing fixture unexpectedly has cross-check label")
    for needle in [
        "**Covers:** `Scenario: Same-branch guard suppression`",
        "**Obligation cross-check:** carries forward the proposal testing promise for same-branch guard suppression",
        "**Boundary cases:** same-branch guard suppression is allowed",
        "`Scenario: Same-branch guard suppression`",
    ]:
        require(needle in pass_design, f"pass fixture missing {needle!r}")
    print("fixture obligation contrast: PASS")
    print("missing fixture: same-branch guard suppression absent from design decisions/proof plan")
    print("pass fixture: same-branch guard suppression present in Covers, Obligation cross-check, Boundary cases, and proof plan")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--template", action="store_true")
    parser.add_argument("--config", action="store_true")
    parser.add_argument("--fixtures", action="store_true")
    args = parser.parse_args()
    selected = [args.template, args.config, args.fixtures]
    require(sum(bool(v) for v in selected) == 1, "select exactly one check")
    if args.template:
        run_template()
    elif args.config:
        run_config()
    else:
        run_fixtures()


if __name__ == "__main__":
    main()
