#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path


def need(text: str, needle: str, label: str) -> None:
    if needle not in text:
        raise SystemExit(f"missing {label}: {needle!r}")


if len(sys.argv) != 5:
    raise SystemExit(
        "usage: check-nix-ui-test-gate.py <ui-devshell> <nextest> <ui-check> <ui-local-support>"
    )

ui_devshell = Path(sys.argv[1]).read_text()
nextest = Path(sys.argv[2]).read_text()
ui_check = Path(sys.argv[3]).read_text()
ui_local_support = Path(sys.argv[4]).read_text()

need(ui_devshell, "test ui ... ok", "devshell UI success")
need(ui_devshell, "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;", "devshell UI summary")

need(nextest, "tigerstyle-nextest>", "nextest execution")
need(nextest, "Summary [", "nextest summary")
need(nextest, "1 test run: 1 passed, 1 skipped", "nextest pass summary")

need(ui_check, "tigerstyle-ui>", "nix UI execution")
need(ui_check, "test ui ... ok", "nix UI success")
need(ui_check, "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;", "nix UI summary")

need(ui_local_support, "Compiling dylint_internal v5.0.0 (/build/support/dylint/internal)", "local dylint_internal path")
need(ui_local_support, "Compiling dylint_driver v5.0.0 (/build/support/dylint/driver)", "local dylint_driver path")
need(ui_local_support, "Compiling tigerstyle-ui-driver v0.1.0 (/build/support/ui-driver)", "local ui-driver path")

print("ui-devshell: passed")
print("nextest: passed")
print("checks.ui: passed")
print("local-support-driver: passed")
