## Why

Workday H7 cannot accept library-only compiler coverage as all-target evidence. The current planner ignores binary, test, example, benchmark, and proc-macro targets.

## What Changes

Use a bounded Cargo unit plan for explicit target selection. Bind collector selection to source root, test configuration, target triple, and resolved features. Reject ambiguous, missing, or unexpected selected compiler units. Preserve default-off architecture policy and separate lint and compiler verdicts.

## Impact

This changes compiler context compatibility, target planning, collector dispatch, fixtures, and documentation. Existing library policy remains supported. Unsupported Cargo plans fail closed. No consumer pin changes before verified publication.
