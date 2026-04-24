#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
//! Functional core for the Tiger Style policy checker.

pub mod architecture;
pub mod diagnostic;
pub mod fixture;
pub mod model;
pub mod report;
pub mod resource;
pub mod runner;
pub mod source;
pub mod suppression;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::architecture::check_architecture;
    use crate::diagnostic::Diagnostic;
    use crate::diagnostic::DiagnosticInput;
    use crate::fixture::FixtureManifest;
    use crate::fixture::PolicyFixtureCoverage;
    use crate::fixture::validate_hard_policy_fixture_coverage;
    use crate::model::Classification;
    use crate::model::EnforcementLevel;
    use crate::model::FamilySelection;
    use crate::model::FixtureCoverage;
    use crate::model::FixtureStatus;
    use crate::model::PolicyEntry;
    use crate::model::PolicyRegistry;
    use crate::model::Severity;
    use crate::model::SuppressionRequirements;
    use crate::report::CheckReport;
    use crate::report::render_json;
    use crate::report::render_markdown;
    use crate::resource::check_resources;
    use crate::runner::CheckInput;
    use crate::runner::run_checks;
    use crate::source::SourceFile;

    const DEVTOOLS_POLICY_ID: &str = "tigerstyle.devtools.lint-suppressions";
    const EMBEDDED_POLICY_ID: &str = "tigerstyle.embedded.bounded-queues";
    const FCIS_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
    const ROOT_THINNESS_POLICY_ID: &str = "tigerstyle.architecture.root-crate-thinness";
    const SOURCE_PATH: &str = "src/lib.rs";
    const DEVTOOLS_SOURCE: &str = "docs/standards/source/devtools.adoc#enforcing-not-advising";
    const EMBEDDED_SOURCE: &str = "docs/standards/source/embedded.adoc#bounded-everything";
    const OWNER: &str = "owner=platform";
    const RATIONALE: &str = "rationale=hardware boundary";
    const REVIEW: &str = "review=next hardware revision";
    const SAFER_DEFAULT: &str = "safer_default=bounded queue";
    const MISSING_SUPPRESSION_METADATA_DIAGNOSTICS: usize = 4;
    const MISSING_FIXTURE_COVERAGE_DIAGNOSTICS: usize = 4;
    const THICK_ROOT_LINE_COUNT: usize = 50;
    const BOUNDED_CHANNEL_CAPACITY: u16 = 16;

    #[test]
    fn valid_suppression_is_reported_as_active() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(SOURCE_PATH, valid_suppression_line(DEVTOOLS_POLICY_ID))],
            selection: FamilySelection::all(),
        });

        assert!(report.diagnostics.is_empty());
        assert_eq!(report.active_suppressions.len(), 1);
        assert_eq!(report.active_suppressions[0].policy_id, DEVTOOLS_POLICY_ID);
        assert_eq!(report.active_suppressions[0].line, 1);
    }

    #[test]
    fn missing_suppression_metadata_returns_blocking_diagnostics() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(
                SOURCE_PATH,
                "// tigerstyle-allow: policy=tigerstyle.devtools.lint-suppressions; owner=platform",
            )],
            selection: FamilySelection::all(),
        });

        assert!(report.has_blocking_diagnostics());
        assert_eq!(report.active_suppressions.len(), 0);
        assert_eq!(report.diagnostics.len(), MISSING_SUPPRESSION_METADATA_DIAGNOSTICS);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("required `source` metadata"))
        );
    }

    #[test]
    fn blanket_suppression_scope_is_rejected() {
        let line = format!("{}; scope=crate", valid_suppression_line(DEVTOOLS_POLICY_ID));
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(SOURCE_PATH, line)],
            selection: FamilySelection::all(),
        });

        assert!(report.has_blocking_diagnostics());
        assert_eq!(report.active_suppressions.len(), 0);
        assert_eq!(report.diagnostics[0].message, "blanket suppression scope `crate` is not allowed");
    }

    #[test]
    fn unknown_policy_is_rejected() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(
                SOURCE_PATH,
                "// tigerstyle-allow: policy=tigerstyle.unknown.rule; source=doc; owner=a; rationale=b; review=c; safer_default=d",
            )],
            selection: FamilySelection::all(),
        });

        assert!(report.has_blocking_diagnostics());
        assert_eq!(report.diagnostics[0].policy_id, DEVTOOLS_POLICY_ID);
        assert_eq!(report.diagnostics[0].message, "suppression references unknown policy `tigerstyle.unknown.rule`");
    }

    #[test]
    fn family_selection_skips_unselected_policy() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(SOURCE_PATH, valid_suppression_line(EMBEDDED_POLICY_ID))],
            selection: FamilySelection::from_families(["devtools"]),
        });

        assert!(report.diagnostics.is_empty());
        assert!(report.active_suppressions.is_empty());
        assert_eq!(report.evaluated_policy_ids, vec![DEVTOOLS_POLICY_ID.to_owned()]);
    }

    #[test]
    fn json_output_is_stable() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(SOURCE_PATH, valid_suppression_line(DEVTOOLS_POLICY_ID))],
            selection: FamilySelection::from_families(["devtools"]),
        });
        let json = render_json(&report).expect("report should serialize");

        assert!(json.contains("\"active_suppressions\""));
        assert!(json.contains(DEVTOOLS_POLICY_ID));
        assert!(!report.has_blocking_diagnostics());
    }

    #[test]
    fn markdown_output_is_stable() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(SOURCE_PATH, valid_suppression_line(DEVTOOLS_POLICY_ID))],
            selection: FamilySelection::from_families(["devtools"]),
        });
        let markdown = render_markdown(&report);

        assert!(markdown.starts_with("# Tiger Style Standards Report"));
        assert!(markdown.contains("## Active Suppressions"));
        assert!(markdown.contains(DEVTOOLS_POLICY_ID));
    }

    #[test]
    fn architecture_checks_accept_compliant_workspace_shape() {
        let sources = vec![
            SourceFile::new("crates/tigerstyle-core/src/lib.rs", "#![no_std]\n"),
            SourceFile::new("crates/tigerstyle-core/Cargo.toml", "[dependencies]\nsnafu = \"0.8\"\n"),
            SourceFile::new("crates/tigerstyle-std/src/lib.rs", "pub fn shell() {}\n"),
            SourceFile::new("src/lib.rs", "pub use tigerstyle_core as core;\n"),
        ];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn architecture_checks_reject_core_runtime_dependency() {
        let sources = vec![SourceFile::new(
            "crates/tigerstyle-core/Cargo.toml",
            "[dependencies]\ntokio = \"1\"\n[dev-dependencies]\nclap = \"4\"\n",
        )];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, FCIS_POLICY_ID);
        assert!(diagnostics[0].message.contains("runtime crate `tokio`"));
    }

    #[test]
    fn architecture_checks_reject_missing_core_no_std() {
        let sources = vec![SourceFile::new(
            "crates/tigerstyle-core/src/lib.rs",
            "pub fn core() {}\n",
        )];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, FCIS_POLICY_ID);
        assert!(diagnostics[0].message.contains("missing #![no_std]"));
    }

    #[test]
    fn architecture_checks_reject_thick_root_file() {
        let thick_root = (0..THICK_ROOT_LINE_COUNT).map(|_| "pub fn helper() {}\n").collect::<String>();
        let sources = vec![SourceFile::new("src/lib.rs", thick_root)];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, ROOT_THINNESS_POLICY_ID);
        assert!(diagnostics[0].message.contains("thin-shell limit"));
    }

    #[test]
    fn resource_checks_reject_unbounded_channel() {
        let sources = vec![SourceFile::new(
            "src/device.rs",
            "fn start() { let (_tx, _rx) = tokio::sync::mpsc::unbounded_channel(); }",
        )];
        let diagnostics = check_resources(&sample_registry(), &sources, &FamilySelection::from_families(["embedded"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, EMBEDDED_POLICY_ID);
        assert!(diagnostics[0].message.contains("unbounded_channel"));
    }

    #[test]
    fn resource_checks_accept_bounded_channel() {
        let sources = vec![SourceFile::new(
            "src/device.rs",
            format!("fn start() {{ let (_tx, _rx) = tokio::sync::mpsc::channel({BOUNDED_CHANNEL_CAPACITY}); }}"),
        )];
        let diagnostics = check_resources(&sample_registry(), &sources, &FamilySelection::from_families(["embedded"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn fixture_manifest_accepts_covered_hard_policies() {
        let diagnostics = validate_hard_policy_fixture_coverage(&sample_registry(), &covered_fixture_manifest());

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn advisory_only_report_is_non_blocking() {
        let report = CheckReport {
            diagnostics: vec![Diagnostic::new(DiagnosticInput {
                policy_id: "tigerstyle.fixture.advisory".to_owned(),
                source_section: "docs/standards/source/fitness.adoc#1-readability".to_owned(),
                path: SOURCE_PATH.to_owned(),
                line: Some(1),
                severity: Severity::Warning,
                message: "advisory finding".to_owned(),
                remediation: "inspect trend".to_owned(),
            })],
            active_suppressions: Vec::new(),
            evaluated_policy_ids: Vec::new(),
        };

        assert!(report.is_advisory_only());
        assert!(!report.has_blocking_diagnostics());
    }

    #[test]
    fn fixture_manifest_rejects_missing_noncompliant_fixture() {
        let manifest = FixtureManifest {
            policy_coverage: vec![PolicyFixtureCoverage {
                policy_id: DEVTOOLS_POLICY_ID.to_owned(),
                compliant: vec!["ok.rs".to_owned()],
                noncompliant: Vec::new(),
            }],
        };
        let diagnostics = validate_hard_policy_fixture_coverage(&sample_registry(), &manifest);

        assert_eq!(diagnostics.len(), MISSING_FIXTURE_COVERAGE_DIAGNOSTICS);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("no noncompliant fixture")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("missing fixture coverage")));
    }

    fn sample_registry() -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["tigerstyle-standards".to_owned()],
            policies: vec![
                sample_policy(DEVTOOLS_POLICY_ID, "devtools", DEVTOOLS_SOURCE),
                sample_policy(EMBEDDED_POLICY_ID, "embedded", EMBEDDED_SOURCE),
                sample_policy(
                    FCIS_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#boundaries-the-compiler-can-see",
                ),
                sample_policy(
                    ROOT_THINNESS_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#boundaries-the-compiler-can-see",
                ),
            ],
        }
    }

    fn sample_policy(policy_id: &str, pillar: &str, source_anchor: &str) -> PolicyEntry {
        PolicyEntry {
            policy_id: policy_id.to_owned(),
            pillar: pillar.to_owned(),
            source_path: "docs/standards/source/devtools.adoc".to_owned(),
            source_heading: "Enforcing, not advising".to_owned(),
            source_anchor: source_anchor.to_owned(),
            summary: "sample policy".to_owned(),
            classification: Classification::Hard,
            enforcement_level: EnforcementLevel::Hard,
            checker_owner: "test".to_owned(),
            checker_id: "tigerstyle-standards".to_owned(),
            default_severity: Severity::Error,
            suppression_requirements: hard_suppression_requirements(),
            fixture_coverage: FixtureCoverage {
                status: FixtureStatus::Planned,
                compliant: Vec::new(),
                noncompliant: Vec::new(),
            },
            verification_evidence_requirements: vec!["positive fixture".to_owned(), "negative fixture".to_owned()],
            classification_rationale: "deterministic test policy".to_owned(),
            false_positive_risk: "low".to_owned(),
            existing_enforcement: Vec::new(),
            custom_checker_justification: "test".to_owned(),
        }
    }

    fn hard_suppression_requirements() -> SuppressionRequirements {
        SuppressionRequirements {
            requires_policy_id: true,
            requires_source_section: true,
            requires_owner: true,
            requires_rationale: true,
            requires_review_trigger: true,
            requires_safer_default: true,
        }
    }

    fn valid_suppression_line(policy_id: &str) -> String {
        format!(
            "// tigerstyle-allow: policy={policy_id}; source={DEVTOOLS_SOURCE}; {OWNER}; {RATIONALE}; {REVIEW}; {SAFER_DEFAULT}"
        )
    }

    fn covered_fixture_manifest() -> FixtureManifest {
        FixtureManifest {
            policy_coverage: vec![
                PolicyFixtureCoverage {
                    policy_id: DEVTOOLS_POLICY_ID.to_owned(),
                    compliant: vec!["devtools/ok.rs".to_owned()],
                    noncompliant: vec!["devtools/bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: EMBEDDED_POLICY_ID.to_owned(),
                    compliant: vec!["embedded/ok.rs".to_owned()],
                    noncompliant: vec!["embedded/bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: FCIS_POLICY_ID.to_owned(),
                    compliant: vec!["architecture/ok.rs".to_owned()],
                    noncompliant: vec!["architecture/bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: ROOT_THINNESS_POLICY_ID.to_owned(),
                    compliant: vec!["architecture/root-ok.rs".to_owned()],
                    noncompliant: vec!["architecture/root-bad.rs".to_owned()],
                },
            ],
        }
    }
}
