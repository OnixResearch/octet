#!/usr/bin/env python3
from pathlib import Path
import sys

script_path = Path(sys.argv[1])
dylint_path = Path(sys.argv[2])

script = script_path.read_text()
old_scope = "default_scope=(\n  # -p your-core-crate\n  # -p your-supporting-crate\n)\n"
new_scope = "default_scope=(\n  -p aspen-time\n  -p aspen-hlc\n  -p aspen-core\n)\n"
if old_scope not in script:
    raise SystemExit("default_scope block not found in template copy")
script_path.write_text(script.replace(old_scope, new_scope, 1))

configured_dylint = """# Aspen-specific Tiger Style tuning reconstructed from the archived phase-1 rollout.\n#\n# This copy starts from `templates/dylint.toml` and applies the values recorded in\n# `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/rollout-notes.md`.\n\n[tigerstyle]\n\n# Aspen still has larger orchestration handlers. Start at 100 lines while the\n# first rollout proves the workflow, then tighten after the handler cleanup pass.\nfunction_length_max = 100\n\n# Several coordinator and RPC handlers still nest one level deeper than the\n# default tigerstyle threshold. Keep one extra level during phased adoption.\nnesting_max_depth = 4\n\n# Focus assertion-density checks on non-trivial functions. Thin shell wrappers\n# should not need placeholder assertions just to satisfy the lint.\nassertion_density_min_lines = 30\nassertion_density_min_assertions = 2\n\n# Transport and storage code use `IO` pervasively. Keep this single exception\n# explicit; add new acronym exceptions only with rollout-note rationale.\nacronym_allowlist = [\"IO\"]\n"""
dylint_path.write_text(configured_dylint)

print(f"configured {script_path}")
print(f"configured {dylint_path}")
