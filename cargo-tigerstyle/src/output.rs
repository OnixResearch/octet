use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io::Write;

use serde::Serialize;

pub struct LintMeta {
    pub group: &'static str,
    pub principle: &'static str,
    pub guidance: &'static str,
}

#[derive(Serialize)]
pub struct Finding {
    pub version: u32,
    pub index: String,
    pub lint: String,
    pub group: String,
    pub severity: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub suggestion: Option<String>,
    pub applicability: Option<String>,
    pub principle: String,
    pub crate_name: String,
}

impl Finding {
    pub fn is_autofixable(&self) -> bool {
        self.applicability.as_deref() == Some("MachineApplicable")
    }

    pub fn location(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }
}

pub fn lint_meta_map() -> HashMap<&'static str, LintMeta> {
    let entries: &[(&str, &str, &str, &str)] = &[
        ("compound_assertion", "tigerstyle::safety", "Assertion Quality", "split compound assertions so the failing clause is obvious"),
        ("function_length", "tigerstyle::structure", "Function Length", "split long functions into named helpers at invariant or phase boundaries"),
        ("acronym_style", "tigerstyle::naming", "Acronym Capitalization", "capitalize acronyms like ordinary words in type and variant names"),
        ("numeric_units", "tigerstyle::naming", "Unit Suffixes", "put the unit in the name of numeric quantities"),
        ("unbounded_loop", "tigerstyle::safety", "Bounded Control Flow", "bound long-running loops with an exit condition, timeout, or bounded input"),
        ("compound_condition", "tigerstyle::structure", "Condition Decomposition", "break multi-part conditions into named predicates or nested guards"),
        ("verified_purity", "tigerstyle::purity", "Verified Module Purity", "keep verified modules free of runtime-only side effects and shell concerns"),
        ("unjustified_allow", "tigerstyle::structure", "Documented Suppression", "every lint suppression needs a local reason tied to the code shape"),
        ("nested_conditionals", "tigerstyle::structure", "Simple Control Flow", "flatten control flow with guards, early returns, or helper predicates"),
        ("assertion_density", "tigerstyle::safety", "Assertion Density", "add enough assertions to make local invariants visible near the logic they defend"),
        ("multi_lock_ordering", "tigerstyle::concurrency", "Lock Ordering", "document and preserve one lock acquisition order across the codebase"),
        ("explicit_defaults", "tigerstyle::safety", "Explicit Defaults", "construct foreign-crate values explicitly instead of relying on `Default`"),
        ("unbounded_channel", "tigerstyle::safety", "Bounded Queues", "use bounded channels with explicit capacity instead of unbounded queues"),
        ("unbounded_collection_growth", "tigerstyle::safety", "Bounded Resource Growth", "reserve collection capacity or add an explicit local length bound before growing it in a loop"),
        ("sentinel_fallback", "tigerstyle::safety", "safety", "branch explicitly, propagate the missing/error state, or emit tagged fallback text instead of <int>::MAX/<int>::MIN."),
        ("usize_in_public_api", "tigerstyle::structure", "Cross-Platform APIs", "use fixed-width integers at API boundaries instead of `usize` or `isize`"),
        ("too_many_parameters", "tigerstyle::structure", "Simple Interfaces", "group many parameters into a named options struct instead of a long positional signature"),
        ("negated_predicate", "tigerstyle::naming", "Positive Predicates", "prefer positive predicate names over negated forms like `is_not_ready` or `no_capacity`"),
        ("bool_naming", "tigerstyle::naming", "Positive Predicates", "name booleans as predicates so call sites read like questions"),
        ("ignored_result", "tigerstyle::safety", "Error Handling", "handle fallible results explicitly with `?`, `match`, or an intentional logged drop"),
        ("unchecked_narrowing", "tigerstyle::safety", "Safe Narrowing", "narrow integers with checked conversions instead of `as`"),
        ("unchecked_division", "tigerstyle::safety", "Explicit Division", "guard divisors before division so zero is impossible"),
        ("contradictory_time", "tigerstyle::purity", "Functional Core", "choose one source of time for a code path and thread it through consistently"),
        ("ambient_clock", "tigerstyle::purity", "Functional Core", "pass time into core logic or isolate clock reads in a shell-boundary helper with a narrow allow reason"),
        ("catch_all_on_enum", "tigerstyle::safety", "Exhaustive Matching", "match enum variants explicitly so new variants cannot slip through `_`"),
        ("platform_dependent_cast", "tigerstyle::structure", "Explicit Types", "use explicit-width types at boundaries instead of `usize` or `isize` conversions"),
        ("float_for_currency", "tigerstyle::naming", "Integer Currency", "store money as scaled integers, not floats"),
        ("ambiguous_params", "tigerstyle::structure", "Explicit Interfaces", "replace same-type parameter clusters with named fields or an options struct"),
        ("raw_arithmetic_overflow", "tigerstyle::safety", "Overflow Protection", "use checked or saturating arithmetic when overflow would change behavior"),
        ("no_unwrap", "tigerstyle::safety", "Error Handling", "propagate or handle absence explicitly instead of panicking with `unwrap` or `expect`"),
        ("no_panic", "tigerstyle::safety", "Fail Fast Correctly", "return errors or prove exhaustiveness instead of panicking in production code"),
        ("no_todo", "tigerstyle::safety", "Fail Fast Correctly", "replace temporary stubs with implemented code or an explicit error path before release"),
        ("unjustified_no_todo_allow", "tigerstyle::safety", "Documented Todo Escape", "reserve `#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` for temporary `todo!()` stubs with an adjacent `//` justification comment"),
        ("no_recursion", "tigerstyle::safety", "Simple Control Flow", "prefer explicit iteration or work queues over self-recursive control flow"),
    ];

    entries
        .iter()
        .map(|(name, group, principle, guidance)| {
            (
                *name,
                LintMeta {
                    group,
                    principle,
                    guidance,
                },
            )
        })
        .collect()
}

pub fn emit_json_line(finding: &Finding, out: &mut impl Write) -> Result<(), String> {
    serde_json::to_writer(&mut *out, finding).map_err(|e| format!("write JSON finding: {e}"))?;
    writeln!(out).map_err(|e| format!("write newline: {e}"))?;
    Ok(())
}

pub fn emit_sarif(findings: &[Finding], meta_map: &HashMap<&str, LintMeta>, out: &mut impl Write) -> Result<(), String> {
    let mut seen_rules: Vec<&str> = Vec::new();
    for f in findings {
        if !seen_rules.contains(&f.lint.as_str()) {
            seen_rules.push(&f.lint);
        }
    }

    let rules: Vec<serde_json::Value> = seen_rules
        .iter()
        .map(|lint_name| {
            let guidance = meta_map
                .get(lint_name)
                .map(|m| m.guidance)
                .unwrap_or("unknown");
            let group = meta_map
                .get(lint_name)
                .map(|m| m.group)
                .unwrap_or("tigerstyle::unknown");
            serde_json::json!({
                "id": lint_name,
                "shortDescription": { "text": guidance },
                "properties": { "group": group }
            })
        })
        .collect();

    let results: Vec<serde_json::Value> = findings
        .iter()
        .map(|f| {
            serde_json::json!({
                "ruleId": f.lint,
                "level": f.severity,
                "message": { "text": f.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": f.file },
                        "region": {
                            "startLine": f.line,
                            "startColumn": f.column
                        }
                    }
                }],
                "properties": { "findingIndex": f.index }
            })
        })
        .collect();

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "cargo-tigerstyle",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": rules
                }
            },
            "results": results
        }]
    });

    serde_json::to_writer_pretty(&mut *out, &sarif).map_err(|e| format!("write SARIF: {e}"))?;
    writeln!(out).map_err(|e| format!("write newline: {e}"))?;
    Ok(())
}

pub fn emit_summary(findings: &[Finding], out: &mut impl Write) -> Result<(), String> {
    let w = |e: std::io::Error| format!("write summary: {e}");

    writeln!(out, "--- tigerstyle summary ---").map_err(w)?;

    if findings.is_empty() {
        writeln!(out, "Findings: 0").map_err(w)?;
        writeln!(out, "---").map_err(w)?;
        return Ok(());
    }

    let autofixable = findings.iter().filter(|f| f.is_autofixable()).count();
    writeln!(out, "Findings: {} ({autofixable} autofixable)", findings.len()).map_err(w)?;

    // By crate (sorted by count descending, then name ascending)
    let mut by_crate: BTreeMap<&str, usize> = BTreeMap::new();
    for f in findings {
        *by_crate.entry(&f.crate_name).or_default() += 1;
    }
    let mut by_crate_sorted: Vec<_> = by_crate.into_iter().collect();
    by_crate_sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));

    let max_crate_name = by_crate_sorted.iter().map(|(n, _)| n.len()).max().unwrap_or(0);

    writeln!(out).map_err(w)?;
    writeln!(out, "By crate:").map_err(w)?;
    for (name, count) in &by_crate_sorted {
        writeln!(out, "  {name:<width$} {count}", width = max_crate_name + 6).map_err(w)?;
    }

    // By lint (sorted by count descending, then name ascending)
    let mut by_lint: BTreeMap<&str, usize> = BTreeMap::new();
    for f in findings {
        *by_lint.entry(&f.lint).or_default() += 1;
    }
    let mut by_lint_sorted: Vec<_> = by_lint.into_iter().collect();
    by_lint_sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));

    let max_lint_name = by_lint_sorted.iter().map(|(n, _)| n.len()).max().unwrap_or(0);

    writeln!(out).map_err(w)?;
    writeln!(out, "By lint:").map_err(w)?;
    for (name, count) in &by_lint_sorted {
        writeln!(out, "  {name:<width$} {count}", width = max_lint_name + 6).map_err(w)?;
    }

    // Index table
    let max_idx = findings.iter().map(|f| f.index.len()).max().unwrap_or(0);
    let max_lint = findings.iter().map(|f| f.lint.len()).max().unwrap_or(0);
    let max_crate = findings.iter().map(|f| f.crate_name.len()).max().unwrap_or(0);

    writeln!(out).map_err(w)?;
    writeln!(out, "Index:").map_err(w)?;
    for f in findings {
        writeln!(
            out,
            "  {:<iw$}  {:<lw$} {:<cw$}  {}",
            f.index,
            f.lint,
            f.crate_name,
            f.location(),
            iw = max_idx,
            lw = max_lint,
            cw = max_crate,
        )
        .map_err(w)?;
    }

    writeln!(out, "---").map_err(w)?;
    Ok(())
}
