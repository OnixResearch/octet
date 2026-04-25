#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CHANGE = ROOT / "openspec/changes/harden-openspec-design-gate-omissions"
EVIDENCE = CHANGE / "evidence"


def write_lines(path: Path, lines: list[str]) -> None:
    path.write_text("\n".join(lines) + "\n")


def main() -> None:
    design_template = (ROOT / "openspec/templates/design.md").read_text().splitlines()
    coverage_lines = [
        "## Context",
        "## Goals / Non-Goals",
        "## Decisions",
        "## Deterministic Proof Plan",
        "## Risks / Trade-offs",
        "**Choice:** <what was decided>",
        "**Covers:** <exact delta-spec scenario headings or requirement headings this decision satisfies>",
        "**Rationale:** <why this choice wins>",
        "**Alternative considered:** <what was rejected and why>",
        "**Implementation:** <how this will be carried out>",
        "**Boundary cases:** <negative controls, continued-warning cases, or intentional gaps reviewers must see>",
    ]
    headings = [line for line in design_template if line in coverage_lines[:5]]
    labels = [line for line in design_template if line in coverage_lines[5:]]
    if headings + labels != coverage_lines:
        raise SystemExit(f"unexpected design template coverage lines: {headings + labels!r}")
    write_lines(EVIDENCE / "design-template-coverage-fields.txt", coverage_lines)

    proof_lines = [
        "## Deterministic Proof Plan",
        "- scenario or requirement:",
        "- artifact path:",
        "- command / helper:",
        "- exact expected output / acceptance signal:",
        "- ordering / prerequisites:",
        "- cleanup / postcondition:",
    ]
    found_proof = [line for line in design_template if line in proof_lines]
    if found_proof != proof_lines:
        raise SystemExit(f"unexpected design template proof lines: {found_proof!r}")
    write_lines(EVIDENCE / "design-template-proof-fields.txt", proof_lines)

    contributor_guide = (ROOT / "docs/contributor-guide.md").read_text().splitlines()
    contributor_lines = [
        "[OpenSpec proposal template](../openspec/templates/proposal.md)",
        "[OpenSpec delta-spec template](../openspec/templates/spec.md)",
        "[OpenSpec design template](../openspec/templates/design.md)",
    ]
    found_contributor = [line for line in contributor_guide if line in contributor_lines]
    if found_contributor != contributor_lines:
        raise SystemExit(f"unexpected contributor-guide lines: {found_contributor!r}")
    write_lines(EVIDENCE / "contributor-guide-template-links.txt", contributor_lines)

    config_lines = (ROOT / "openspec/config.yaml").read_text().splitlines()
    config_rule_lines = [
        "    - Follow `openspec/templates/proposal.md`; name exact repo-relative files/artifacts, ordering/dependencies, and planned saved evidence or helper/gate runs",
        "    - Follow `openspec/templates/spec.md` for delta-spec authoring; keep change-local specs in delta form, start each added or modified requirement with `The system MUST ...`, and give every added or modified requirement at least one `#### Scenario:` block",
        "    - Follow `openspec/templates/design.md`; every added or modified scenario heading or zero-scenario requirement must appear in a decision's `**Covers:**` field; name boundary cases for non-trigger or continued-warning conditions; every saved proof artifact must state its exact expected output or acceptance signal",
    ]
    found_rules = [line for line in config_lines if line in config_rule_lines]
    if found_rules != config_rule_lines:
        raise SystemExit(f"unexpected config guidance lines: {found_rules!r}")
    write_lines(EVIDENCE / "design-stage-guidance.txt", config_rule_lines)


if __name__ == "__main__":
    main()
