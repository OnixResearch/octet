#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path


def repo_root() -> Path:
    here = Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "openspec/config.yaml").exists() and (parent / "openspec/templates/tasks.md").exists():
            return parent
    raise SystemExit("could not find repo root")


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        raise SystemExit(f"missing {label}: {needle}")


def check_template(root: Path) -> None:
    text = (root / "openspec/templates/tasks.md").read_text()
    for needle, label in [
        ("make each V<n> task name the exact scenario", "V task exact scenario prompt"),
        ("command/helper", "command/helper prompt"),
        ("required input/artifact", "required input/artifact prompt"),
        ("acceptance signal", "acceptance signal prompt"),
        ("broader check that omits the scenario-specific behavior", "broad check warning"),
        ("negative control, or continued-warning case", "negative/continued warning prompt"),
    ]:
        require(text, needle, label)
    print("PASS tasks template includes task obligation fields")


def check_config(root: Path) -> None:
    text = (root / "openspec/config.yaml").read_text()
    for needle, label in [
        ("each `V<n>` task must name the exact scenario", "V task config rule"),
        ("required input/artifact or fixture", "required artifact config rule"),
        ("expected PASS output, diagnostic, schema field, or acceptance signal", "acceptance config rule"),
        ("broad tasks such as `test config parsing` are insufficient", "broad task config warning"),
        ("specific absent-field default, explicit override, required artifact, negative control, or continued-warning case", "specific omitted behavior config rule"),
    ]:
        require(text, needle, label)
    print("PASS config includes tasks-stage obligation guidance")


def check_fixtures(root: Path) -> None:
    base = root / "tests/fixtures/openspec-task-governance"
    missing = base / "task-obligation-missing-blocked"
    covered = base / "task-obligation-covered-pass"
    runner = base / "run-obligation-fixture-gate.sh"
    for path in [missing, covered, runner]:
        if not path.exists():
            raise SystemExit(f"missing fixture path: {path}")
    if (missing / "expected_gate.txt").read_text().strip() != "blocked":
        raise SystemExit("missing fixture expected_gate is not blocked")
    if (covered / "expected_gate.txt").read_text().strip() != "pass":
        raise SystemExit("covered fixture expected_gate is not pass")
    missing_tasks = (missing / "tasks.md").read_text()
    covered_tasks = (covered / "tasks.md").read_text()
    if "expect=`PASS absent disabled_lints and lint_levels default empty`" in missing_tasks:
        raise SystemExit("missing fixture unexpectedly contains exact absent-default acceptance")
    for needle in [
        "Verify scenario \"Task obligations map scenarios to exact proof\"",
        "command=`tests/fixtures/openspec-task-governance/run-obligation-fixture-gate.sh task-obligation-covered-pass`",
        "input=`tests/fixtures/openspec-task-governance/task-obligation-covered-pass/tasks.md`",
        "expect=`PASS expected passing fixture: task-obligation-covered-pass`",
        "evidence=`openspec/changes/task-obligation-covered-pass/evidence/task-gate.txt`",
    ]:
        require(covered_tasks, needle, f"covered task {needle}")
    runner_text = runner.read_text()
    for needle in ["stage=tasks", "omission|missing|misses", "tasks|task|verification|scenario|fixture"]:
        require(runner_text, needle, f"runner {needle}")
    print("PASS task-obligation fixtures encode missing and covered obligations")


def main() -> None:
    if len(sys.argv) != 2 or sys.argv[1] not in {"template", "config", "fixtures"}:
        raise SystemExit("usage: check-tasks-obligation-fixtures.py template|config|fixtures")
    root = repo_root()
    if sys.argv[1] == "template":
        check_template(root)
    elif sys.argv[1] == "config":
        check_config(root)
    else:
        check_fixtures(root)


if __name__ == "__main__":
    main()
