#!/usr/bin/env python3
"""Write deterministic evidence extracts for roi-01 omission-prevention rails."""
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CHANGE = ROOT / "openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails"
EVIDENCE = CHANGE / "evidence"


def read(rel: str) -> str:
    return (ROOT / rel).read_text(encoding="utf-8")


def require(text: str, needle: str, label: str) -> str:
    if needle not in text:
        raise SystemExit(f"missing {label}: {needle}")
    return needle


def write_artifact(name: str, title: str, lines: list[str]) -> None:
    path = EVIDENCE / name
    path.write_text("\n".join([f"# {title}", "", *lines, ""]), encoding="utf-8")
    print(f"wrote {path.relative_to(ROOT)}")


def main() -> None:
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    proposal = read("openspec/templates/proposal.md")
    spec = read("openspec/templates/spec.md")
    design = read("openspec/templates/design.md")
    tasks = read("openspec/templates/tasks.md")
    config = read("openspec/config.yaml")

    write_artifact(
        "proposal-template-omission-prompts.txt",
        "Proposal Template Omission Prompts",
        [
            require(
                proposal,
                "<!-- Omission-prevention checklist: name planned verification/evidence expectations, exact affected files/artifacts, and explicit non-goal boundaries before writing specs or tasks. -->",
                "proposal omission checklist",
            ),
            require(proposal, "- **Files / artifacts**: <exact repo-relative paths touched by this change>", "proposal files prompt"),
            require(proposal, "- **Verification / evidence**: <saved artifacts or helper/gate runs planned for review>", "proposal evidence prompt"),
            "preserved heading order: ## Why -> ## What Changes -> ## Non-Goals -> ## Capabilities -> ## Impact",
        ],
    )

    write_artifact(
        "spec-template-omission-prompts.txt",
        "Delta Spec Template Omission Prompts",
        [
            require(
                spec,
                "<!-- Omission-prevention checklist: enumerate boundary cases, negative controls, and continued-warning or continued-blocking scenarios when the requirement has non-trigger or regression-catching behavior. -->",
                "spec omission checklist",
            ),
            require(spec, "<!-- If acceptance depends on a saved artifact, describe the artifact's exact expected output or schema in the requirement body, not only the path. -->", "spec proof artifact prompt"),
            require(spec, "#### Scenario: <scenario heading>", "spec scenario prompt"),
            "preserved repo-specific guidance comments: exact proof output/schema and scenario reminder",
        ],
    )

    write_artifact(
        "design-template-omission-prompts.txt",
        "Design Template Omission Prompts",
        [
            require(
                design,
                "<!-- Omission-prevention checklist: check every added/modified scenario heading and every zero-scenario requirement against decision coverage, boundary cases, continued-warning cases, and exact proof acceptance signals before implementation starts. -->",
                "design omission checklist",
            ),
            require(design, "**Covers:** <exact delta-spec scenario headings or requirement headings this decision satisfies>", "design coverage prompt"),
            require(design, "**Boundary cases:** <negative controls, continued-warning cases, or intentional gaps reviewers must see>", "design boundary prompt"),
            require(design, "- exact expected output / acceptance signal:", "design proof signal prompt"),
            "preserved design contract: heading order, **Covers:** labels, boundary labels, deterministic proof-plan labels",
        ],
    )

    config_lines = [
        "    - \"Follow `openspec/templates/proposal.md`; name exact repo-relative files/artifacts, ordering/dependencies, explicit non-goal boundaries, and planned saved evidence or helper/gate runs before drafting specs or tasks\"",
        "    - \"Follow `openspec/templates/spec.md` for delta-spec authoring; keep change-local specs in delta form, start each added or modified requirement with `The system MUST ...`, give every added or modified requirement at least one `#### Scenario:` block, enumerate boundary cases/negative controls/continued-warning or continued-blocking cases when behavior has non-trigger or regression-catching paths, and describe exact proof output/schema when artifacts matter\"",
        "    - \"Follow `openspec/templates/design.md`; every added or modified scenario heading or zero-scenario requirement must appear in a decision's `**Covers:**` field; name boundary cases, continued-warning cases, and intentional gaps reviewers must see; every saved proof artifact must state its exact expected output or acceptance signal\"",
        "    - \"Keep implementation, deterministic verification, and human/oracle checkpoints in separate checkbox units; an `I<n>` task becomes invalid if it includes markers like `verify`, `save`, `capture`, `evidence=`, or a repo-relative `evidence/` path\"",
        "    - \"Compare `tasks.md` against the delta-spec scenario headings and keep one `## Verification Coverage` row per added or modified scenario, even when multiple rows share one artifact\"",
        "    - \"`H<n>` tasks are invalid unless they include exact `command=`, pinned `input=` or prerequisite artifact, one `asserts=` claim, and saved `evidence=` path; broad workflows, implicit inputs, bundled assertions, or missing evidence paths are underspecified\"",
    ]
    write_artifact(
        "config-omission-guidance.txt",
        "Config Omission Guidance",
        [require(config, line, f"config line {idx}") for idx, line in enumerate(config_lines, start=1)],
    )

    write_artifact(
        "tasks-template-omission-prompts.txt",
        "Tasks Template Omission Prompts",
        [
            require(
                tasks,
                "<!-- Omission-prevention checklist: keep implementation separate from verification/evidence, give every added or modified scenario its own coverage row, and make each H<n> oracle checkpoint self-verifiable with exact command=, input=, asserts=, and evidence= fields. -->",
                "tasks omission checklist",
            ),
            require(tasks, "- `I<n>` — one implementation claim only. Do not combine code changes with verification runs, saved evidence, or transcript capture in the same checkbox. Split that work into a separate `V<n>` or `H<n>` task.", "task split prompt"),
            require(tasks, "- Before marking a change ready, read the delta specs and ensure every added or modified `Scenario:` heading appears in `## Verification Coverage` exactly once. Shared artifacts still need separate scenario rows.", "coverage prompt"),
            require(tasks, "- Treat an `H<n>` checkbox as underspecified if `command=` is a broad workflow instead of an exact command/helper, `input=` omits the pinned fixture or prerequisite artifact, `asserts=` bundles multiple human claims, or `evidence=` omits the saved review artifact path.", "oracle underspec prompt"),
            require(tasks, "- [ ] H1 Human/oracle checkpoint: command=`<exact command or helper path>`; input=`<pinned revision, fixture, or prerequisite artifact>`; asserts=`<single human claim>`; evidence=`openspec/changes/<change>/evidence/<artifact>.txt`", "oracle task example"),
            "preserved status layout: ## Change Status immediately before ## Verification Coverage with required status/open-task lines",
        ],
    )


if __name__ == "__main__":
    main()
