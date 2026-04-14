#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[5]
ARCHIVE_DIR = REPO_ROOT / "openspec/changes/archive/2026-04-14-per-lint-configuration"
EVIDENCE_DIR = ARCHIVE_DIR / "evidence"
MAIN_SPEC = REPO_ROOT / "openspec/specs/documentation/spec.md"
DELTA_SPEC = ARCHIVE_DIR / "specs/documentation/spec.md"


def read(path: Path) -> str:
    return path.read_text()


def require_contains(path: Path, needle: str) -> None:
    text = read(path)
    if needle not in text:
        raise SystemExit(f"missing expected text in {path.relative_to(REPO_ROOT)}: {needle!r}")


def require_not_contains(path: Path, needle: str) -> None:
    text = read(path)
    if needle in text:
        raise SystemExit(f"unexpected text in {path.relative_to(REPO_ROOT)}: {needle!r}")


def section(title: str) -> None:
    print(f"== {title} ==")


def print_matching_lines(path: Path, prefixes: tuple[str, ...]) -> None:
    rel = path.relative_to(REPO_ROOT)
    print(rel)
    for line in read(path).splitlines():
        if line.startswith(prefixes):
            print(line)
    print()


def print_tail(path: Path, lines: int) -> None:
    rel = path.relative_to(REPO_ROOT)
    print(rel)
    tail = read(path).splitlines()[-lines:]
    for line in tail:
        print(line)
    print()


def main() -> None:
    cargo_test_lib = EVIDENCE_DIR / "cargo-test-lib.txt"
    cargo_test_ui = EVIDENCE_DIR / "cargo-test-ui.txt"
    nix_flake = EVIDENCE_DIR / "nix-flake-check.txt"
    checkpoint = EVIDENCE_DIR / "aspen-validation-checkpoint.txt"
    replay = EVIDENCE_DIR / "aspen-replay.txt"
    smoke = EVIDENCE_DIR / "aspen-smoke.txt"
    boundary = EVIDENCE_DIR / "aspen-boundary-first.txt"
    inventory = EVIDENCE_DIR / "aspen-inventory-only.txt"
    inventory_cfg = EVIDENCE_DIR / "aspen-inventory-only.dylint.toml"

    require_contains(cargo_test_lib, "running 17 tests")
    require_contains(cargo_test_lib, "test tests::lint_inventory_snapshot_matches ... ok")
    require_contains(cargo_test_lib, "test tests::lint_inventory_default_levels_snapshot_matches ... ok")
    require_contains(cargo_test_lib, "test result: ok. 17 passed; 0 failed")

    require_contains(cargo_test_ui, "[ui fixture] no_unwrap_allow_custom/no_unwrap_allow_custom")
    require_contains(cargo_test_ui, "[ui fixture] no_unwrap_warn_custom/no_unwrap_warn_custom")
    require_contains(cargo_test_ui, "[ui fixture] no_unwrap_deny_custom/no_unwrap_deny_custom")
    require_contains(cargo_test_ui, "[ui fixture] no_unwrap_default_custom/no_unwrap_default_custom")
    require_contains(cargo_test_ui, "[ui fixture] no_unwrap_disabled_deny_custom/no_unwrap_disabled_deny_custom")
    require_contains(cargo_test_ui, "[ui fixture] no_unwrap_disabled_source_attr_custom/no_unwrap_disabled_source_attr_custom")
    require_contains(cargo_test_ui, "test result: ok. 1 passed; 0 failed")

    require_contains(nix_flake, "checking derivation checks.x86_64-linux.ui")
    require_contains(nix_flake, "cargo nextest run --cargo-profile release")
    require_contains(nix_flake, "cargo test --release --locked --test ui -- --ignored --nocapture")
    require_contains(nix_flake, "all checks passed!")

    require_contains(checkpoint, "Phase-1 archive: openspec/changes/archive/2026-04-13-aspen-consumer-rollout/")
    require_contains(checkpoint, "Inventory-only config snapshot: openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-inventory-only.dylint.toml")

    require_contains(replay, "Phase-1 archive reference: openspec/changes/archive/2026-04-13-aspen-consumer-rollout/")
    require_contains(replay, "Exact replay command: bash openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/replay-aspen-profiles.sh")

    require_contains(smoke, "warning: `aspen-time` (lib) generated 2 warnings")
    require_contains(boundary, "warning: `aspen-core` (lib) generated 4 warnings")
    require_contains(inventory, "# replay profile: inventory-only")
    require_contains(inventory, "[tigerstyle] profile:   inventory-only no_unwrap")
    require_not_contains(inventory, "production code will panic on failure")
    require_contains(inventory_cfg, "[tigerstyle.lint_levels]")
    require_contains(inventory_cfg, 'no_unwrap = "warn"')

    delta = read(DELTA_SPEC)
    delta_requirement = delta.split("### Requirement: Per-Lint Configuration Support", 1)[1].strip()
    main = read(MAIN_SPEC)
    if delta_requirement not in main:
        raise SystemExit("main documentation spec missing synced per-lint requirement block")

    section("cargo test --lib summary")
    print_tail(cargo_test_lib, 24)

    section("cargo test --test ui fixture proof")
    print_matching_lines(cargo_test_ui, ("[ui fixture]", "test result:"))

    section("nix flake check summary")
    print_matching_lines(nix_flake, ("checking derivation checks.", "all checks passed!", "warning: The check omitted"))

    section("Aspen checkpoint")
    print(read(checkpoint).rstrip())
    print()

    section("Aspen replay header")
    print("\n".join(read(replay).splitlines()[:18]))
    print()

    section("Aspen smoke / boundary / inventory summaries")
    print_tail(smoke, 12)
    print_tail(boundary, 12)
    print("inventory header:")
    for line in read(inventory).splitlines()[:10]:
        print(line)
    print()
    print_tail(inventory, 8)

    section("Spec sync excerpt from main spec")
    marker = "### Requirement: Per-Lint Configuration Support"
    excerpt = main.split(marker, 1)[1].strip()
    print(marker)
    print(excerpt)
    print()
    print("ok")


if __name__ == "__main__":
    main()
