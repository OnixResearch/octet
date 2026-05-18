#!/usr/bin/env python3
"""Run Octet's archived indexable-policy lint family against Octet itself.

The helper is intentionally change-local evidence machinery. It copies the
current checkout to a temporary workspace, writes dogfood-only config there,
runs the canonical cargo-octet JSON path, then normalizes artifacts back into
this change's evidence directory. The repository root is left without temporary
`dylint.toml` or manifest edits.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
from collections import Counter
from pathlib import Path

CHANGE = "roi-02-dogfood-indexable-policy-lints"
EVIDENCE_REL = Path("openspec/changes") / CHANGE / "evidence"
RUN_ARTIFACT = "dogfood-run.txt"
FINDINGS_ARTIFACT = "dogfood-findings.jsonl"
SUMMARY_ARTIFACT = "dogfood-summary.txt"

TARGET_LINTS = [
    "drop_must_use",
    "fallible_int_fallback",
    "magic_number",
    "excessive_file_length",
    "module_file_count",
    "underscore_in_module_filename",
    "non_trait_imports",
    "renamed_imports",
    "path_segment_repetition",
]

ALL_LINTS = [
    "compound_assertion",
    "function_length",
    "acronym_style",
    "numeric_units",
    "magic_number",
    "excessive_file_length",
    "module_file_count",
    "underscore_in_module_filename",
    "non_trait_imports",
    "renamed_imports",
    "path_segment_repetition",
    "unbounded_loop",
    "compound_condition",
    "verified_purity",
    "unjustified_allow",
    "nested_conditionals",
    "assertion_density",
    "multi_lock_ordering",
    "explicit_defaults",
    "unbounded_channel",
    "unbounded_collection_growth",
    "sentinel_fallback",
    "usize_in_public_api",
    "borrowed_argument_types",
    "deref_polymorphism",
    "clone_then_borrow",
    "too_many_parameters",
    "negated_predicate",
    "bool_naming",
    "ignored_result",
    "drop_must_use",
    "fallible_int_fallback",
    "unchecked_narrowing",
    "unchecked_division",
    "contradictory_time",
    "ambient_clock",
    "catch_all_on_enum",
    "platform_dependent_cast",
    "float_for_currency",
    "ambiguous_params",
    "raw_arithmetic_overflow",
    "no_unwrap",
    "no_panic",
    "public_unsafe_api",
    "no_todo",
    "unjustified_no_todo_allow",
    "no_recursion",
]

COPY_IGNORE_NAMES = {
    ".git",
    ".direnv",
    ".jj",
    "target",
    "result",
    "result-bin",
    "result-doc",
    "result-man",
}


def repo_root() -> Path:
    completed = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return Path(completed.stdout.strip())


def ignore_copy_names(_directory: str, names: list[str]) -> set[str]:
    ignored = {name for name in names if name in COPY_IGNORE_NAMES}
    ignored.update(name for name in names if name.startswith("result-"))
    return ignored


def append_workspace_metadata(manifest: Path) -> None:
    source = manifest.read_text()
    if "[workspace.metadata.octet]" in source:
        return
    source = source.rstrip() + "\n\n[workspace.metadata.octet]\ndefault_scope = [\"-p\", \"octet\"]\ncargo_check_args = [\"--lib\"]\n"
    manifest.write_text(source)


def write_dogfood_config(workspace: Path) -> list[str]:
    target = set(TARGET_LINTS)
    disabled = [lint for lint in ALL_LINTS if lint not in target]
    quoted = ", ".join(json.dumps(lint) for lint in disabled)
    config = "[octet]\n" f"disabled_lints = [{quoted}]\n"
    (workspace / "dylint.toml").write_text(config)
    return disabled


def parse_jsonl(stdout: str) -> list[dict[str, object]]:
    findings: list[dict[str, object]] = []
    for line in stdout.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        try:
            value = json.loads(stripped)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict) and "lint" in value and "file" in value:
            findings.append(value)
    return findings


def normalize_path(repo: Path, value: object) -> str | None:
    raw = str(value or "")
    if not raw:
        return raw
    path = Path(raw)
    if path.is_absolute():
        try:
            return path.relative_to(repo).as_posix()
        except ValueError:
            return None
    return raw


def normalize_findings(repo: Path, findings: list[dict[str, object]]) -> tuple[list[dict[str, object]], int]:
    normalized = []
    excluded_external = 0
    for index, finding in enumerate(findings, start=1):
        item = dict(finding)
        normalized_file = normalize_path(repo, item.get("file"))
        if normalized_file is None:
            excluded_external += 1
            continue
        item["index"] = f"F{index}"
        item["file"] = normalized_file
        normalized.append(item)
    normalized.sort(key=lambda item: (str(item.get("lint", "")), str(item.get("file", "")), int(item.get("line") or 0), str(item.get("message", ""))))
    for index, item in enumerate(normalized, start=1):
        item["index"] = f"F{index}"
    return normalized, excluded_external


def write_findings(path: Path, findings: list[dict[str, object]]) -> None:
    with path.open("w") as handle:
        for finding in findings:
            handle.write(json.dumps(finding, sort_keys=True, separators=(",", ":")))
            handle.write("\n")


def summary_text(
    command: list[str],
    exit_code: int,
    disabled_lints: list[str],
    findings: list[dict[str, object]],
    root: Path,
    excluded_external_findings: int,
) -> str:
    by_lint = Counter(str(item.get("lint", "unknown")) for item in findings)
    by_file = Counter(str(item.get("file", "unknown")) for item in findings)
    enabled_set = set(ALL_LINTS) - set(disabled_lints)
    enabled = [lint for lint in TARGET_LINTS if lint in enabled_set]
    root_config_absent = not (root / "dylint.toml").exists()

    lines = [
        "# Indexable-policy dogfood summary",
        f"command: {' '.join(command)}",
        f"exit_code: {exit_code}",
        f"target_lints: {', '.join(TARGET_LINTS)}",
        f"enabled_lints: {', '.join(enabled)}",
        f"disabled_lints_count: {len(disabled_lints)}",
        f"findings_total: {len(findings)}",
        f"external_findings_excluded: {excluded_external_findings}",
        f"assert.uses_canonical_cargo_octet: {'PASS' if 'cargo-octet' in ' '.join(command) else 'FAIL'}",
        f"assert.enabled_exactly_nine_archived_lints: {'PASS' if enabled == TARGET_LINTS else 'FAIL'}",
        f"assert.non_target_lints_disabled: {'PASS' if set(disabled_lints) == (set(ALL_LINTS) - set(TARGET_LINTS)) else 'FAIL'}",
        f"assert.no_repo_root_temporary_dylint_toml: {'PASS' if root_config_absent else 'FAIL'}",
        "",
        "## Counts by lint",
    ]
    if by_lint:
        for lint, count in sorted(by_lint.items()):
            lines.append(f"- {lint}: {count}")
    else:
        lines.append("- none: 0")

    lines.extend(["", "## Counts by file"])
    if by_file:
        for file_name, count in sorted(by_file.items()):
            lines.append(f"- {file_name}: {count}")
    else:
        lines.append("- none: 0")

    lines.extend(["", "## Finding index"])
    if findings:
        for item in findings:
            lines.append(
                f"- {item.get('index')}: {item.get('lint')} {item.get('file')}:{item.get('line')} {item.get('message')}"
            )
    else:
        lines.append("- none")
    return "\n".join(lines) + "\n"


def main() -> int:
    root = repo_root()
    evidence = root / EVIDENCE_REL
    evidence.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory(prefix="octet-indexable-dogfood-") as tmp:
        workspace = Path(tmp) / "source"
        shutil.copytree(root, workspace, ignore=ignore_copy_names)
        append_workspace_metadata(workspace / "Cargo.toml")
        disabled_lints = write_dogfood_config(workspace)

        command = [
            "nix",
            "develop",
            f"path:{root}",
            "-c",
            "nix",
            "run",
            f"path:{root}#cargo-octet",
            "--",
            "check",
            "-p",
            "octet",
            "--output-format",
            "json",
            "--",
            "--lib",
        ]
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(Path(tmp) / "target")
        completed = subprocess.run(
            command,
            cwd=workspace,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
        )

        parsed_findings = parse_jsonl(completed.stdout)
        findings, excluded_external_findings = normalize_findings(workspace, parsed_findings)
        write_findings(evidence / FINDINGS_ARTIFACT, findings)
        (evidence / RUN_ARTIFACT).write_text(
            "$ " + " ".join(command) + "\n"
            + "cwd: <dogfood-temp>/source\n"
            + f"exit_code: {completed.returncode}\n"
            + "\n## stdout\n"
            + completed.stdout
            + "\n## stderr\n"
            + completed.stderr
        )
        (evidence / SUMMARY_ARTIFACT).write_text(
            summary_text(command, completed.returncode, disabled_lints, findings, root, excluded_external_findings)
        )

    summary = (evidence / SUMMARY_ARTIFACT).read_text()
    required_asserts = [
        "assert.uses_canonical_cargo_octet: PASS",
        "assert.enabled_exactly_nine_archived_lints: PASS",
        "assert.non_target_lints_disabled: PASS",
        "assert.no_repo_root_temporary_dylint_toml: PASS",
    ]
    missing = [line for line in required_asserts if line not in summary]
    if missing:
        for line in missing:
            print(f"missing required summary assertion: {line}", file=sys.stderr)
        return 1

    print(summary, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
