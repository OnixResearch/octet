#!/usr/bin/env python3
"""Capture real source-only Nix consumer checks; Rust bundles remain authoritative."""

import argparse
import collections
import json
import os
from pathlib import Path
import platform
import re
import shlex
import shutil
import subprocess
import sys


VERSION = "0.1.1-rc.1"
VALENCE_URL = "https://github.com/OnixResearch/valence.git"
VALENCE_REV = "e250f498a86ac1e62266b8df749ade021a4b4043"
VALENCE_PACKAGE = "valence-shared-identity-pilot"
PACKAGE = "octet-first-checked-patch"
BAD_SOURCE = """pub fn average_bytes(total_bytes: u64, completed_operations: u64) -> u64 {
    total_bytes / completed_operations
}
"""
FIXED_SOURCE = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AverageBytesError {
    NoCompletedOperations,
}

pub fn average_bytes(
    total_bytes: u64,
    completed_operations: u64,
) -> Result<u64, AverageBytesError> {
    if completed_operations > 0 {
        Ok(total_bytes / completed_operations)
    } else {
        Err(AverageBytesError::NoCompletedOperations)
    }
}
"""
MANIFEST = f"""[package]
name = "{PACKAGE}"
version = "0.1.0"
edition = "2024"
publish = false

[workspace]

[workspace.metadata.octet]
default_scope = ["-p", "{PACKAGE}"]
cargo_check_args = ["--lib", "--locked"]
"""
CONFIG = """# Consumer-owned: this smoke gates division, without disabling other lints.
[octet]

[octet.lint_levels]
unchecked_division = "deny"
"""


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def store_executable(value, name):
    path = Path(value).resolve(strict=True)
    require(str(path).startswith("/nix/store/"), f"{name} must be an immutable Nix-store executable: {path}")
    require(path.is_file() and os.access(path, os.X_OK), f"{name} is not executable: {path}")
    return path


def reject_ancestor_cargo_config(root):
    for ancestor in root.parents:
        for name in (".cargo/config", ".cargo/config.toml"):
            require(not (ancestor / name).exists(), f"ambient ancestor Cargo config: {ancestor / name}")


class Capture:
    def __init__(self, root):
        self.root = root
        root.mkdir(parents=True, exist_ok=False)
        (root / "commands").mkdir()
        self.sequence = 0
        self.env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"}
        # Certificates are transport inputs, not credentials. No ambient Rust,
        # Dylint, Cargo, Octet, Git, proxy, or authentication settings survive.
        for key in ("SSL_CERT_FILE", "SSL_CERT_DIR", "NIX_SSL_CERT_FILE"):
            if key in os.environ:
                self.env[key] = os.environ[key]
        for key, relative in (
            ("HOME", "home"), ("CARGO_HOME", "cargo-home"),
            ("RUSTUP_HOME", "rustup-home"), ("XDG_CACHE_HOME", "cache"),
            ("XDG_CONFIG_HOME", "config"), ("XDG_DATA_HOME", "data"),
            ("TMPDIR", "tmp"), ("CARGO_TARGET_DIR", "target"),
        ):
            path = root / relative
            path.mkdir()
            self.env[key] = str(path)
        self.env.update({
            "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_SYSTEM": os.devnull,
            "GIT_CONFIG_GLOBAL": os.devnull, "GIT_TERMINAL_PROMPT": "0",
            "GIT_AUTHOR_NAME": "Octet smoke operator", "GIT_AUTHOR_EMAIL": "smoke@example.invalid",
            "GIT_COMMITTER_NAME": "Octet smoke operator", "GIT_COMMITTER_EMAIL": "smoke@example.invalid",
            "CARGO_TERM_COLOR": "never",
        })
        (root / "environment.txt").write_text(
            "".join(f"{key}={shlex.quote(value)}\n" for key, value in sorted(self.env.items()))
        )
        shutil.copyfile(Path(__file__).resolve(), root / "capture-helper.py")
        self.note("Invocation: " + shlex.join([sys.executable, *sys.argv]))
        self.note("Raw command logs and Rust-emitted bundles are authoritative; this is not a release qualification receipt.")

    def note(self, text):
        with (self.root / "operator.log").open("a") as output:
            output.write(text + "\n")
        print(text, flush=True)

    def assert_that(self, condition, message):
        require(condition, message)
        self.note("ASSERTED: " + message)

    def run(self, label, argv, cwd=None, expected=(0,), extra_env=None):
        self.sequence += 1
        logs = self.root / "commands" / f"{self.sequence:02d}-{label}"
        logs.mkdir()
        cwd = cwd or self.root
        env = self.env | (extra_env or {})
        command = [str(argument) for argument in argv]
        (logs / "command.txt").write_text(
            f"cwd: {shlex.quote(str(cwd))}\n$ " + shlex.join(command) + "\n"
            + "".join(f"env {key}={shlex.quote(value)}\n" for key, value in sorted((extra_env or {}).items()))
        )
        self.note(f"RUN: {logs.relative_to(self.root)}")
        try:
            with (logs / "stdout").open("wb") as stdout, (logs / "stderr").open("wb") as stderr:
                result = subprocess.run(command, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                        stdout=stdout, stderr=stderr, check=False)
        except OSError as error:
            (logs / "launch-error.txt").write_text(f"{type(error).__name__}: {error}\n")
            raise
        (logs / "exit-code.txt").write_text(str(result.returncode) + "\n")
        self.assert_that(result.returncode in expected,
                         f"{label} exit {result.returncode} is in {list(expected)}; logs={logs}")
        return logs

    def git(self, label, repo, *args):
        return self.run(label, ["git", "-c", "core.hooksPath=/dev/null", *args], cwd=repo)

    def verify(self, runner, bundle, label, cwd, valid=True):
        before = tree_bytes(bundle)
        logs = self.run(label, [runner, "artifact", "verify", "--artifact-dir", bundle,
                               "--output-format", "json"], cwd=cwd, expected=(0,) if valid else (1,))
        report = json.loads((logs / "stdout").read_bytes())
        self.assert_that(report["status"] == ("valid" if valid else "invalid"),
                         f"{label} Rust verification status matches expected integrity outcome")
        self.assert_that(tree_bytes(bundle) == before, f"{label} leaves bundle bytes unchanged")
        if valid:
            self.assert_that(not report["diagnostics"], f"{label} has no verification diagnostics")
        else:
            self.assert_that(any("hash mismatch for artifact locator `results.jsonl`" in diagnostic
                                 for diagnostic in report["diagnostics"]),
                             f"{label} rejects the deliberately changed results.jsonl hash")

    def check(self, runner, workspace, label, package, expected):
        bundle = self.root / "bundles" / label
        logs = self.run(label, [runner, "check", "-p", package, "--output-format", "json",
                                "--artifact-dir", bundle, "--", "--lib", "--locked"], cwd=workspace,
                        expected=(expected,), extra_env={"CARGO_TARGET_DIR": str(self.root / "targets" / label)})
        status = json.loads((bundle / "status.json").read_bytes())
        raw_findings = (bundle / "results.jsonl").read_bytes()
        findings = [json.loads(line) for line in raw_findings.splitlines() if line.strip()]
        errors = sum(finding["severity"] == "error" for finding in findings)
        warnings = sum(finding["severity"] == "warning" for finding in findings)
        self.assert_that(raw_findings == (logs / "stdout").read_bytes(), f"{label} retains exact Rust finding stdout")
        self.assert_that(status["exit_code"] == expected, f"{label} Rust status exit matches process exit")
        self.assert_that(status["metadata"]["tool_name"] == "cargo-octet"
                         and status["metadata"]["tool_version"] == VERSION,
                         f"{label} Rust metadata identifies cargo-octet {VERSION}")
        self.assert_that(status["total_findings"] == len(findings)
                         and status["error_findings"] == errors and status["warning_findings"] == warnings
                         and errors + warnings == len(findings), f"{label} counts agree with Rust finding records")
        expected_status = "lint-failure" if errors else "warning-only" if warnings else "clean"
        self.assert_that(status["status"] == expected_status, f"{label} Rust status matches observed severities")
        if expected == 0:
            self.assert_that(errors == 0 and status["cargo_process_exit"]["code"] == 0,
                             f"{label} completed Cargo successfully without error findings")
        self.verify(runner, bundle, label + "-verify", workspace)
        counts = collections.Counter((finding["lint"], finding["severity"]) for finding in findings)
        self.note(f"OBSERVED: {label}: {len(findings)} findings ({warnings} warning, {errors} error)")
        for (lint, severity), count in sorted(counts.items()):
            self.note(f"OBSERVED: {label}: {lint}: {count} {severity}")
        return bundle, findings


def tree_bytes(root):
    return {str(path.relative_to(root)): path.read_bytes() for path in root.rglob("*") if path.is_file()}


def fresh_consumer(capture, runner, cargo):
    repo = capture.root / "fresh-consumer"
    (repo / "src").mkdir(parents=True)
    (repo / "Cargo.toml").write_text(MANIFEST)
    (repo / "dylint.toml").write_text(CONFIG)
    source = repo / "src/lib.rs"
    source.write_text(BAD_SOURCE)
    capture.git("fresh-init", repo, "init", "--initial-branch=smoke")
    capture.run("cargo-version", [cargo, "--version"], cwd=repo)
    capture.run("generate-lockfile", [cargo, "generate-lockfile", "--offline"], cwd=repo)
    lock = (repo / "Cargo.lock").read_bytes()
    capture.assert_that(bool(lock), "fresh consumer has a real Cargo-generated lockfile before Octet runs")
    capture.git("fresh-add", repo, "add", "Cargo.toml", "Cargo.lock", "dylint.toml", "src/lib.rs")
    capture.git("fresh-commit-bad", repo, "-c", "commit.gpgsign=false", "commit", "-m", "Before zero guard")
    original_policy = (repo / "dylint.toml").read_bytes()
    original_manifest = (repo / "Cargo.toml").read_bytes()
    bad_bundle, bad_findings = capture.check(runner, repo, "fresh-bad", PACKAGE, 1)
    selected = [finding for finding in bad_findings if finding["lint"] == "unchecked_division"]
    capture.assert_that(bool(selected) and all(finding["severity"] == "error" for finding in selected),
                        "bad function produces denied unchecked_division findings")
    capture.assert_that(all(finding["lint"] == "unchecked_division" for finding in bad_findings
                            if finding["severity"] == "error"), "bad run fails only for the selected division lint")
    source.write_text(FIXED_SOURCE)
    capture.git("fresh-zero-guard-diff", repo, "diff", "--", "src/lib.rs")
    capture.git("fresh-add-fix", repo, "add", "src/lib.rs")
    capture.git("fresh-commit-fix", repo, "-c", "commit.gpgsign=false", "commit", "-m", "Reject zero completed operations")
    fixed_bundle, fixed_findings = capture.check(runner, repo, "fresh-fixed", PACKAGE, 0)
    capture.assert_that(not any(finding["lint"] == "unchecked_division" for finding in fixed_findings),
                        "zero-guard fix leaves zero selected unchecked_division findings")
    capture.assert_that((repo / "Cargo.lock").read_bytes() == lock
                        and (repo / "dylint.toml").read_bytes() == original_policy
                        and (repo / "Cargo.toml").read_bytes() == original_manifest,
                        "fail/fix keeps the committed lockfile, consumer config, and scope unchanged")
    original_bundles = {bundle: tree_bytes(bundle) for bundle in (bad_bundle, fixed_bundle)}
    tampered = capture.root / "bundles" / "fresh-fixed-tampered"
    shutil.copytree(fixed_bundle, tampered)
    result_path = tampered / "results.jsonl"
    # Valid JSON, changed exact bytes: the negative control tests integrity, not
    # merely rejection of a malformed JSON document.
    result_path.write_bytes(result_path.read_bytes() + b" \n")
    capture.note("NEGATIVE CONTROL: appended JSON whitespace to copied fresh-fixed/results.jsonl; original bundle untouched")
    capture.verify(runner, tampered, "fresh-tamper-verify", repo, valid=False)
    capture.assert_that(all(tree_bytes(bundle) == content for bundle, content in original_bundles.items()),
                        "original fail/fix bundles are retained byte-for-byte after the tamper control")


def downstream_valence(capture, runner, cargo):
    repo = capture.root / "downstream-valence"
    repo.mkdir()
    capture.git("valence-init", repo, "init", "--initial-branch=smoke")
    capture.git("valence-origin", repo, "remote", "add", "origin", VALENCE_URL)
    capture.git("valence-fetch", repo, "fetch", "--depth=1", "origin", VALENCE_REV)
    capture.git("valence-checkout", repo, "checkout", "--detach", "FETCH_HEAD")
    revision_log = capture.git("valence-revision", repo, "rev-parse", "HEAD")
    capture.assert_that((revision_log / "stdout").read_text().strip() == VALENCE_REV,
                        "independent Valence workspace matches the full public source revision")
    capture.note(f"INPUT: downstream={VALENCE_URL}@{VALENCE_REV}; owned package={VALENCE_PACKAGE}; --lib --locked")
    for name in ("Cargo.toml", "Cargo.lock", "dylint.toml", "pilots/shared-identity/Cargo.toml"):
        capture.assert_that((repo / name).is_file(), f"downstream committed input exists: {name}")
    original = {name: (repo / name).read_bytes() for name in ("Cargo.toml", "Cargo.lock", "dylint.toml")}
    capture.note("ACQUISITION: fetching the exact committed downstream dependency graph anonymously before Octet; failure here is acquisition failure, not a lint finding or engine result")
    capture.run("valence-anonymous-dependency-fetch", [cargo, "fetch", "--locked"], cwd=repo)
    capture.check(runner, repo, "valence-warning-inventory", VALENCE_PACKAGE, 0)
    capture.assert_that(all((repo / name).read_bytes() == content for name, content in original.items()),
                        "downstream manifest, lockfile, and consumer lint policy are unchanged")
    dirty = capture.git("valence-tracked-diff", repo, "diff", "--exit-code", "HEAD", "--")
    capture.assert_that(not (dirty / "stdout").read_bytes(), "downstream has no tracked source or policy edits")
    capture.note("BOUNDARY: downstream is a bounded warning inventory, not a strict-profile admission or zero-noise claim")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    producer = parser.add_mutually_exclusive_group(required=True)
    producer.add_argument("--runner", help="packaged /nix/store/.../bin/cargo-octet; does not by itself bind public source")
    producer.add_argument("--flake", help="github:OnixResearch/octet/FULL_40_HEX#cargo-octet (no tags, paths, or branches)")
    parser.add_argument("--cargo", help="Nix-store Cargo for fresh lock generation; defaults to Cargo on PATH")
    parser.add_argument("--output", required=True, type=Path, help="new retained capture directory outside any Cargo-config ancestors")
    parser.add_argument("--downstream", choices=("valence",), help="also check the frozen independent public Valence workspace")
    args = parser.parse_args()
    require(platform.system() == "Linux" and platform.machine() == "x86_64", "initial support is x86_64-linux source-only Nix")
    if args.flake:
        require(re.fullmatch(r"github:OnixResearch/octet/[0-9a-f]{40}#cargo-octet", args.flake) is not None,
                "pinned public proof requires github:OnixResearch/octet/<full lowercase 40-hex revision>#cargo-octet")
    cargo_value = args.cargo or shutil.which("cargo")
    require(cargo_value is not None, "supply --cargo with a Nix-store Cargo executable (or enter the matching Nix devshell)")
    cargo = store_executable(cargo_value, "cargo")
    root = args.output.absolute().resolve()
    reject_ancestor_cargo_config(root / "fresh-consumer")
    runner = store_executable(args.runner, "cargo-octet") if args.runner else None
    capture = Capture(root)
    try:
        if args.flake:
            build = capture.run("pinned-public-build", ["nix", "build", "--no-link", "--print-out-paths", args.flake])
            outputs = (build / "stdout").read_text().splitlines()
            capture.assert_that(len(outputs) == 1, "pinned public build returns exactly one package output")
            runner = store_executable(Path(outputs[0]) / "bin/cargo-octet", "cargo-octet")
            capture.note("INPUT: immutable public flake=" + args.flake)
        else:
            capture.note("BOUNDARY: --runner identifies a local immutable package; source binding needs the parent's build/source receipt")
        capture.note(f"INPUT: runner={runner}")
        capture.run("runner-blake3", ["b3sum", runner])
        capture.note(f"INPUT: lock-generator={cargo}")
        version = capture.run("runner-version", [runner, "--version"])
        capture.assert_that((version / "stdout").read_text().strip() == f"cargo-octet {VERSION}",
                            f"packaged runner reports target version {VERSION}")
        fresh_consumer(capture, runner, cargo)
        if args.downstream:
            downstream_valence(capture, runner, cargo)
        else:
            capture.note("NOT EXERCISED: independent public downstream (request --downstream valence)")
        capture.note("COMPLETE: all requested smoke assertions passed; no formal correctness or release-qualification claim")
    except Exception as error:
        capture.note(f"FAILED: {type(error).__name__}: {error}; retained partial capture={root}")
        raise


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, RuntimeError) as error:
        print(f"public-release-smoke: {error}", file=sys.stderr)
        sys.exit(1)
