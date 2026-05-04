use rustc_errors::Applicability;
use rustc_errors::DiagDecorator;
use rustc_lint::Lint;
use rustc_lint::LintContext;
use rustc_span::Span;
use serde::Serialize;

pub(crate) const GROUP_SAFETY: &str = "tigerstyle::safety";
pub(crate) const GROUP_NAMING: &str = "tigerstyle::naming";
pub(crate) const GROUP_PERFORMANCE: &str = "tigerstyle::performance";
pub(crate) const GROUP_STRUCTURE: &str = "tigerstyle::structure";
pub(crate) const GROUP_PURITY: &str = "tigerstyle::purity";
pub(crate) const GROUP_CONCURRENCY: &str = "tigerstyle::concurrency";
pub(crate) const GROUP_NAMES: &[&str] = &[
    GROUP_SAFETY,
    GROUP_NAMING,
    GROUP_PERFORMANCE,
    GROUP_STRUCTURE,
    GROUP_PURITY,
    GROUP_CONCURRENCY,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExampleMode {
    Required,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LintDiagnosticMeta {
    pub lint_name: &'static str,
    pub principle: &'static str,
    pub guide_section: &'static str,
    pub domain_group: &'static str,
    pub guidance: &'static str,
    pub example_mode: ExampleMode,
    pub example_text: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LintGroupSnapshot {
    pub group_name: &'static str,
    pub lint_names: Vec<&'static str>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct CompatibilityAliasSnapshot {
    pub alias_name: &'static str,
    pub canonical_lint_name: String,
}

impl LintDiagnosticMeta {
    fn help_text(self) -> String {
        match self.example_mode {
            ExampleMode::Required => format!(
                "Tiger Style ({}): {} Example: {}",
                self.principle,
                self.guidance,
                self.example_text.expect("missing required example text"),
            ),
            ExampleMode::None => format!("Tiger Style ({}): {}", self.principle, self.guidance),
        }
    }
}

pub(crate) const DIAGNOSTIC_METADATA: &[LintDiagnosticMeta] = &[
    LintDiagnosticMeta {
        lint_name: "compound_assertion",
        principle: "Assertion Quality",
        guide_section: "Assertion Density",
        domain_group: GROUP_SAFETY,
        guidance: "split compound assertions so the failing clause is obvious",
        example_mode: ExampleMode::Required,
        example_text: Some("assert!(a > 0); assert!(b > 0);"),
    },
    LintDiagnosticMeta {
        lint_name: "function_length",
        principle: "Function Length",
        guide_section: "Function Length",
        domain_group: GROUP_STRUCTURE,
        guidance: "split long functions into named helpers at invariant or phase boundaries",
        example_mode: ExampleMode::None,
        example_text: None,
    },
    LintDiagnosticMeta {
        lint_name: "acronym_style",
        principle: "Acronym Capitalization",
        guide_section: "Acronym Capitalization",
        domain_group: GROUP_NAMING,
        guidance: "capitalize acronyms like ordinary words in type and variant names",
        example_mode: ExampleMode::Required,
        example_text: Some("RpcHandler"),
    },
    LintDiagnosticMeta {
        lint_name: "numeric_units",
        principle: "Unit Suffixes",
        guide_section: "Unit Suffixes",
        domain_group: GROUP_NAMING,
        guidance: "put the unit in the name of numeric quantities",
        example_mode: ExampleMode::Required,
        example_text: Some("timeout_ms"),
    },
    LintDiagnosticMeta {
        lint_name: "unbounded_loop",
        principle: "Bounded Control Flow",
        guide_section: "Bounded Control Flow",
        domain_group: GROUP_SAFETY,
        guidance: "bound long-running loops with an exit condition, timeout, or bounded input",
        example_mode: ExampleMode::Required,
        example_text: Some("while let Some(msg) = rx.recv().await { ... }"),
    },
    LintDiagnosticMeta {
        lint_name: "compound_condition",
        principle: "Condition Decomposition",
        guide_section: "Condition Decomposition",
        domain_group: GROUP_STRUCTURE,
        guidance: "break multi-part conditions into named predicates or nested guards",
        example_mode: ExampleMode::Required,
        example_text: Some("if is_leader { if has_quorum { ... } }"),
    },
    LintDiagnosticMeta {
        lint_name: "verified_purity",
        principle: "Verified Module Purity",
        guide_section: "Verified Module Purity",
        domain_group: GROUP_PURITY,
        guidance: "keep verified modules free of runtime-only side effects and shell concerns",
        example_mode: ExampleMode::Required,
        example_text: Some("pub fn next_state(input: Input) -> Output"),
    },
    LintDiagnosticMeta {
        lint_name: "unjustified_allow",
        principle: "Documented Suppression",
        guide_section: "Documented Suppression",
        domain_group: GROUP_STRUCTURE,
        guidance: "every lint suppression needs a local reason tied to the code shape",
        example_mode: ExampleMode::Required,
        example_text: Some(
            "// Verified casts: Verus checks every narrowing in this module.\n#[allow(clippy::cast_possible_truncation)]",
        ),
    },
    LintDiagnosticMeta {
        lint_name: "nested_conditionals",
        principle: "Simple Control Flow",
        guide_section: "Simple Control Flow",
        domain_group: GROUP_STRUCTURE,
        guidance: "flatten control flow with guards, early returns, or helper predicates",
        example_mode: ExampleMode::Required,
        example_text: Some("if !is_ready { return; }"),
    },
    LintDiagnosticMeta {
        lint_name: "assertion_density",
        principle: "Assertion Density",
        guide_section: "Assertion Density",
        domain_group: GROUP_SAFETY,
        guidance: "add enough assertions to make local invariants visible near the logic they defend",
        example_mode: ExampleMode::None,
        example_text: None,
    },
    LintDiagnosticMeta {
        lint_name: "multi_lock_ordering",
        principle: "Lock Ordering",
        guide_section: "Lock Ordering",
        domain_group: GROUP_CONCURRENCY,
        guidance: "document and preserve one lock acquisition order across the codebase",
        example_mode: ExampleMode::Required,
        example_text: Some("// Lock order: state -> peer -> metrics"),
    },
    LintDiagnosticMeta {
        lint_name: "explicit_defaults",
        principle: "Explicit Defaults",
        guide_section: "Explicit Defaults",
        domain_group: GROUP_SAFETY,
        guidance: "construct foreign-crate values explicitly instead of relying on `Default`",
        example_mode: ExampleMode::Required,
        example_text: Some("let client = reqwest::ClientBuilder::new();"),
    },
    LintDiagnosticMeta {
        lint_name: "unbounded_channel",
        principle: "Bounded Queues",
        guide_section: "Bounded Queues",
        domain_group: GROUP_SAFETY,
        guidance: "use bounded channels with explicit capacity instead of unbounded queues",
        example_mode: ExampleMode::Required,
        example_text: Some("let (tx, rx) = tokio::sync::mpsc::channel(128);"),
    },
    LintDiagnosticMeta {
        lint_name: "unbounded_collection_growth",
        principle: "Bounded Resource Growth",
        guide_section: "Bounded Resource Growth",
        domain_group: GROUP_SAFETY,
        guidance: "reserve collection capacity or add an explicit local length bound before growing it in a loop",
        example_mode: ExampleMode::Required,
        example_text: Some("let mut items = Vec::with_capacity(limit);"),
    },
    LintDiagnosticMeta {
        lint_name: "sentinel_fallback",
        principle: "safety",
        guide_section: "make invalid state explicit",
        domain_group: GROUP_SAFETY,
        guidance: "branch explicitly, propagate the missing/error state, or emit tagged fallback text instead of <int>::MAX/<int>::MIN.",
        example_mode: ExampleMode::Required,
        example_text: Some(
            "match u64::try_from(command_index) { Ok(i) => format!(\"recent_{i}\"), Err(_) => \"recent_missing_index\".to_string() }",
        ),
    },
    LintDiagnosticMeta {
        lint_name: "usize_in_public_api",
        principle: "Cross-Platform APIs",
        guide_section: "Explicit Types",
        domain_group: GROUP_STRUCTURE,
        guidance: "use fixed-width integers at API boundaries instead of `usize` or `isize`",
        example_mode: ExampleMode::Required,
        example_text: Some("pub fn get(index: u32) -> Option<&Item>"),
    },
    LintDiagnosticMeta {
        lint_name: "borrowed_argument_types",
        principle: "Borrowed Interfaces",
        guide_section: "Explicit Interfaces",
        domain_group: GROUP_STRUCTURE,
        guidance: "accept general borrowed argument types such as `&str`, `&[T]`, and `&Path` instead of references to owned containers",
        example_mode: ExampleMode::Required,
        example_text: Some("fn parse_name(name: &str) -> Result<Name, Error>"),
    },
    LintDiagnosticMeta {
        lint_name: "clone_then_borrow",
        principle: "Clone Avoidance",
        guide_section: "Performance / Ownership Clarity",
        domain_group: GROUP_PERFORMANCE,
        guidance: "borrow the original value or use an explicit borrowed adapter instead of cloning only to take a shared borrow",
        example_mode: ExampleMode::Required,
        example_text: Some("emit(&name);"),
    },
    LintDiagnosticMeta {
        lint_name: "too_many_parameters",
        principle: "Simple Interfaces",
        guide_section: "Explicit Interfaces",
        domain_group: GROUP_STRUCTURE,
        guidance: "group many parameters into a named options struct instead of a long positional signature",
        example_mode: ExampleMode::Required,
        example_text: Some("fn connect(opts: ConnectOptions) -> ..."),
    },
    LintDiagnosticMeta {
        lint_name: "negated_predicate",
        principle: "Positive Predicates",
        guide_section: "Positive Predicates",
        domain_group: GROUP_NAMING,
        guidance: "prefer positive predicate names over negated forms like `is_not_ready` or `no_capacity`",
        example_mode: ExampleMode::Required,
        example_text: Some("fn is_ready(state: &State) -> bool"),
    },
    LintDiagnosticMeta {
        lint_name: "bool_naming",
        principle: "Positive Predicates",
        guide_section: "Positive Predicates",
        domain_group: GROUP_NAMING,
        guidance: "name booleans as predicates so call sites read like questions",
        example_mode: ExampleMode::Required,
        example_text: Some("let is_ready = status == Ready;"),
    },
    LintDiagnosticMeta {
        lint_name: "ignored_result",
        principle: "Error Handling",
        guide_section: "Error Handling",
        domain_group: GROUP_SAFETY,
        guidance: "handle fallible results explicitly with `?`, `match`, or an intentional logged drop",
        example_mode: ExampleMode::Required,
        example_text: Some("tx.send(msg)?;"),
    },
    LintDiagnosticMeta {
        lint_name: "unchecked_narrowing",
        principle: "Safe Narrowing",
        guide_section: "Safe Narrowing / Conversions",
        domain_group: GROUP_SAFETY,
        guidance: "narrow integers with checked conversions instead of `as`",
        example_mode: ExampleMode::Required,
        example_text: Some("let small = u32::try_from(value)?;"),
    },
    LintDiagnosticMeta {
        lint_name: "unchecked_division",
        principle: "Explicit Division",
        guide_section: "Explicit Division",
        domain_group: GROUP_SAFETY,
        guidance: "guard divisors before division so zero is impossible",
        example_mode: ExampleMode::Required,
        example_text: Some("assert!(divisor != 0); let q = value / divisor;"),
    },
    LintDiagnosticMeta {
        lint_name: "contradictory_time",
        principle: "Functional Core",
        guide_section: "Functional Core / Imperative Shell",
        domain_group: GROUP_PURITY,
        guidance: "choose one source of time for a code path and thread it through consistently",
        example_mode: ExampleMode::Required,
        example_text: Some("fn expires_at(now: Instant, ttl: Duration) -> Instant { now + ttl }"),
    },
    LintDiagnosticMeta {
        lint_name: "ambient_clock",
        principle: "Functional Core",
        guide_section: "Functional Core / Imperative Shell",
        domain_group: GROUP_PURITY,
        guidance: "pass time into core logic or isolate clock reads in a shell-boundary helper with a narrow allow reason",
        example_mode: ExampleMode::Required,
        example_text: Some(
            "#[allow(tigerstyle::ambient_clock, reason = \"shell boundary clock read\")] fn read_now() -> Instant { Instant::now() }",
        ),
    },
    LintDiagnosticMeta {
        lint_name: "catch_all_on_enum",
        principle: "Exhaustive Matching",
        guide_section: "Exhaustive Matching",
        domain_group: GROUP_SAFETY,
        guidance: "match enum variants explicitly so new variants cannot slip through `_`",
        example_mode: ExampleMode::Required,
        example_text: Some("match state { State::Open => ..., State::Closed => ... }"),
    },
    LintDiagnosticMeta {
        lint_name: "platform_dependent_cast",
        principle: "Explicit Types",
        guide_section: "Explicit Types",
        domain_group: GROUP_STRUCTURE,
        guidance: "use explicit-width types at boundaries instead of `usize` or `isize` conversions",
        example_mode: ExampleMode::Required,
        example_text: Some("let len: u32 = value.try_into()?;"),
    },
    LintDiagnosticMeta {
        lint_name: "float_for_currency",
        principle: "Integer Currency",
        guide_section: "Integer Currency",
        domain_group: GROUP_NAMING,
        guidance: "store money as scaled integers, not floats",
        example_mode: ExampleMode::Required,
        example_text: Some("let price_micros: i64 = 125_000;"),
    },
    LintDiagnosticMeta {
        lint_name: "ambiguous_params",
        principle: "Explicit Interfaces",
        guide_section: "Explicit Interfaces",
        domain_group: GROUP_STRUCTURE,
        guidance: "replace same-type parameter clusters with named fields or an options struct",
        example_mode: ExampleMode::Required,
        example_text: Some("fn connect(opts: ConnectOptions) -> ..."),
    },
    LintDiagnosticMeta {
        lint_name: "raw_arithmetic_overflow",
        principle: "Overflow Protection",
        guide_section: "Overflow Protection",
        domain_group: GROUP_SAFETY,
        guidance: "use checked or saturating arithmetic when overflow would change behavior",
        example_mode: ExampleMode::Required,
        example_text: Some("let total = a.checked_add(b)?;"),
    },
    LintDiagnosticMeta {
        lint_name: "no_unwrap",
        principle: "Error Handling",
        guide_section: "Error Handling",
        domain_group: GROUP_SAFETY,
        guidance: "propagate or handle absence explicitly instead of panicking with `unwrap` or `expect`",
        example_mode: ExampleMode::Required,
        example_text: Some("let value = maybe?;"),
    },
    LintDiagnosticMeta {
        lint_name: "no_panic",
        principle: "Fail Fast Correctly",
        guide_section: "Error Handling / Panics",
        domain_group: GROUP_SAFETY,
        guidance: "return errors or prove exhaustiveness instead of panicking in production code",
        example_mode: ExampleMode::Required,
        example_text: Some("return Err(Error::invalid_state());"),
    },
    LintDiagnosticMeta {
        lint_name: "no_todo",
        principle: "Fail Fast Correctly",
        guide_section: "Error Handling / Panics",
        domain_group: GROUP_SAFETY,
        guidance: "replace temporary stubs with implemented code or an explicit error path before release",
        example_mode: ExampleMode::Required,
        example_text: Some("return Err(Error::not_ready());"),
    },
    LintDiagnosticMeta {
        lint_name: "unjustified_no_todo_allow",
        principle: "Documented Todo Escape",
        guide_section: "Error Handling / Panics",
        domain_group: GROUP_SAFETY,
        guidance: "reserve `#[allow(tigerstyle::no_todo)]` or `#[allow(no_todo)]` for temporary `todo!()` stubs with an adjacent `//` justification comment",
        example_mode: ExampleMode::Required,
        example_text: Some("// justified stub while API lands\n#[allow(tigerstyle::no_todo)] fn stub() { todo!() }"),
    },
    LintDiagnosticMeta {
        lint_name: "no_recursion",
        principle: "Simple Control Flow",
        guide_section: "Simple Control Flow",
        domain_group: GROUP_SAFETY,
        guidance: "prefer explicit iteration or work queues over self-recursive control flow",
        example_mode: ExampleMode::Required,
        example_text: Some("while let Some(item) = stack.pop() { ... }"),
    },
];

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn all_metadata() -> &'static [LintDiagnosticMeta] {
    DIAGNOSTIC_METADATA
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn canonical_lint_name(lint_name: &str) -> String {
    format!("tigerstyle::{lint_name}")
}

pub(crate) fn lint_names_for_group(group_name: &str) -> Vec<&'static str> {
    DIAGNOSTIC_METADATA
        .iter()
        .filter(|meta| meta.domain_group == group_name)
        .map(|meta| meta.lint_name)
        .collect()
}

pub(crate) fn group_snapshots() -> Vec<LintGroupSnapshot> {
    GROUP_NAMES
        .iter()
        .map(|group_name| LintGroupSnapshot {
            group_name,
            lint_names: lint_names_for_group(group_name),
        })
        .collect()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn compatibility_alias_snapshots() -> Vec<CompatibilityAliasSnapshot> {
    DIAGNOSTIC_METADATA
        .iter()
        .map(|meta| CompatibilityAliasSnapshot {
            alias_name: meta.lint_name,
            canonical_lint_name: canonical_lint_name(meta.lint_name),
        })
        .collect()
}

pub(crate) fn diagnostic_meta(lint_name: &str) -> &'static LintDiagnosticMeta {
    DIAGNOSTIC_METADATA
        .iter()
        .find(|meta| meta.lint_name == lint_name)
        .unwrap_or_else(|| panic!("missing diagnostic metadata for lint `{lint_name}`"))
}

pub(crate) enum LintSuggestion {
    Single {
        span: Span,
        message: String,
        replacement: String,
        applicability: Applicability,
    },
    Multipart {
        message: String,
        parts: Vec<(Span, String)>,
        applicability: Applicability,
    },
}

fn applicability_label(applicability: Applicability) -> &'static str {
    match applicability {
        Applicability::MachineApplicable => "MachineApplicable",
        Applicability::MaybeIncorrect => "MaybeIncorrect",
        Applicability::HasPlaceholders => "HasPlaceholders",
        Applicability::Unspecified => "Unspecified",
    }
}

pub(crate) fn emit_tiger_style_lint(
    cx: &impl LintContext,
    lint: &'static Lint,
    span: Span,
    message: impl Into<String>,
    lint_name: &'static str,
) {
    let meta = diagnostic_meta(lint_name);
    let message = message.into();

    cx.opt_span_lint(
        lint,
        Some(span),
        DiagDecorator(|diag| {
            diag.primary_message(message.clone());
            diag.help(meta.help_text());
            diag.note(format!("see Tiger Style guide: {}", meta.guide_section));
        }),
    );
}

pub(crate) fn emit_tiger_style_lint_with_suggestion(
    cx: &impl LintContext,
    lint: &'static Lint,
    span: Span,
    message: impl Into<String>,
    lint_name: &'static str,
    suggestion: LintSuggestion,
) {
    let meta = diagnostic_meta(lint_name);
    let message = message.into();
    let applicability = match &suggestion {
        LintSuggestion::Single { applicability, .. } | LintSuggestion::Multipart { applicability, .. } => {
            *applicability
        }
    };

    cx.opt_span_lint(
        lint,
        Some(span),
        DiagDecorator(|diag| {
            diag.primary_message(message.clone());
            match suggestion {
                LintSuggestion::Single {
                    span,
                    message,
                    replacement,
                    applicability,
                } => {
                    diag.span_suggestion_verbose(span, message, replacement, applicability);
                }
                LintSuggestion::Multipart {
                    message,
                    parts,
                    applicability,
                } => {
                    diag.multipart_suggestion(message, parts, applicability);
                }
            }
            diag.help(meta.help_text());
            diag.note(format!("structured suggestion applicability: {}", applicability_label(applicability)));
            diag.note(format!("see Tiger Style guide: {}", meta.guide_section));
        }),
    );
}
