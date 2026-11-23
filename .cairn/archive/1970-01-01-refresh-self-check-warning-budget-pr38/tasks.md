# Tasks: Refresh the self-check warning budget for PR #38

## Phase 1: Remove the avoidable new findings

- [x] [serial] Rename the three boolean bindings and qualify the retention type at its uses. r[octet.self_check.warning_budget_refresh]
- [x] [serial] Replace discarded and unwrapped summary writes with direct string appends, reserving both marker collections. r[octet.self_check.warning_budget_refresh]
- [x] [serial] Split the three over-length functions and bundle the proof-card path inputs. r[octet.self_check.warning_budget_refresh]

## Phase 2: Reviewed refresh

- [x] [serial] Regenerate the baseline with the packaged runner and record every added and removed identity with its location and reason. r[octet.self_check.warning_budget_refresh]
- [x] [serial] Update both accepted totals and the documented stored-budget paragraph. r[octet.self_check.warning_budget_refresh]

## Phase 3: Verification

- [x] [serial] Run the canonical self-check gate, the touched module tests, rustfmt, Cairn validation and the three gates; record the transcripts in `evidence/`. r[octet.self_check.warning_budget_refresh]

## Verification Coverage

- `Scenario: Refreshed baseline has no stale or new findings` -> canonical self-check gate transcript
- `Scenario: Counts match the baseline` -> same gate transcript (structured status)
- `Scenario: Added identities are recorded` -> `evidence/finding-delta.md`
- `Scenario: Warning budget remains advisory` -> proposal and design non-claims
