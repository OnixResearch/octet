#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
//! Functional core for the Tiger Style policy checker.

pub mod architecture;
pub mod artifacts;
pub mod diagnostic;
pub mod fixture;
pub mod model;
pub mod proof;
pub mod report;
pub mod resource;
pub mod runner;
pub mod source;
pub mod suppression;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::architecture::check_architecture;
    use crate::fixture::FixtureManifest;
    use crate::fixture::PolicyFixtureCoverage;
    use crate::fixture::validate_hard_policy_fixture_coverage;
    use crate::model::Classification;
    use crate::model::EnforcementLevel;
    use crate::model::EnforcementMode;
    use crate::model::EvidenceCommand;
    use crate::model::FamilySelection;
    use crate::model::FixtureCoverage;
    use crate::model::FixtureStatus;
    use crate::model::Mechanism;
    use crate::model::MechanismKind;
    use crate::model::PolicyEntry;
    use crate::model::PolicyRegistry;
    use crate::model::ProofRail;
    use crate::model::ReceiptProvenance;
    use crate::model::ReceiptVerifier;
    use crate::model::Severity;
    use crate::model::SuppressionRequirements;
    use crate::proof::check_verified_proof_rails;
    use crate::report::render_json;
    use crate::report::render_markdown;
    use crate::resource::check_resources;
    use crate::runner::CheckInput;
    use crate::runner::run_checks;
    use crate::source::SourceFile;

    const DEVTOOLS_POLICY_ID: &str = "tigerstyle.devtools.lint-suppressions";
    const EMBEDDED_POLICY_ID: &str = "tigerstyle.embedded.bounded-queues";
    const FCIS_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
    const EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID: &str = "tigerstyle.architecture.external-domain-leakage";
    const ROOT_THINNESS_POLICY_ID: &str = "tigerstyle.architecture.root-crate-thinness";
    const PUBLIC_API_BUDGET_POLICY_ID: &str = "tigerstyle.architecture.public-api-budget";
    const EFFECT_CAPABILITY_POLICY_ID: &str = "tigerstyle.architecture.effect-capability-boundaries";
    const TYPE_STATE_POLICY_ID: &str = "tigerstyle.idioms.nominal-type-boundaries";
    const VERIFIED_POLICY_ID: &str = "tigerstyle.verification.deterministic-effect-handlers";
    const SOURCE_PATH: &str = "src/lib.rs";
    const DEVTOOLS_SOURCE: &str = "docs/standards/source/devtools.adoc#enforcing-not-advising";
    const EMBEDDED_SOURCE: &str = "docs/standards/source/embedded.adoc#bounded-everything";
    const OWNER: &str = "owner=platform";
    const RATIONALE: &str = "rationale=hardware boundary";
    const REVIEW: &str = "review=next hardware revision";
    const SAFER_DEFAULT: &str = "safer_default=bounded queue";
    const MISSING_SUPPRESSION_METADATA_DIAGNOSTICS: usize = 4;
    const MISSING_FIXTURE_COVERAGE_DIAGNOSTICS: usize = 8;
    const THICK_ROOT_LINE_COUNT: usize = 50;
    const BOUNDED_CHANNEL_CAPACITY: u16 = 16;
    const PUBLIC_API_BUDGET_FIXTURE_PATH: &str =
        "standards/check-fixtures/architecture/advisory/public-api-budget/src/lib.rs";
    const PUBLIC_API_BUDGET_FIXTURE_SOURCE: &str =
        include_str!("../../../standards/check-fixtures/architecture/advisory/public-api-budget/src/lib.rs");

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
    fn architecture_checks_reject_external_domain_leakage() {
        let sources = vec![SourceFile::new(
            "crates/tigerstyle-core/src/lib.rs",
            "#![no_std]\n\npub fn accept(input: axum::Json<DomainCommand>) -> DomainCommand {\n    input.0\n}\n\npub struct DomainCommand;\n",
        )];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID);
        assert!(diagnostics[0].message.contains("axum::"));
        assert!(diagnostics[0].remediation.contains("translate at the boundary"));
    }

    #[test]
    fn architecture_checks_reject_multiple_core_fixture_shapes() {
        let sources = vec![
            SourceFile::new("missing/crates/tigerstyle-core/src/lib.rs", "pub fn core() {}\n"),
            SourceFile::new("leak/crates/tigerstyle-core/src/lib.rs", "#![no_std]\n\npub use tonic::Status;\n"),
        ];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.policy_id == FCIS_POLICY_ID));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.policy_id == EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID));
    }

    #[test]
    fn architecture_checks_reject_thick_root_file() {
        let thick_root = (0..THICK_ROOT_LINE_COUNT).map(|_| "fn helper() {}\n").collect::<String>();
        let sources = vec![SourceFile::new("src/lib.rs", thick_root)];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, ROOT_THINNESS_POLICY_ID);
        assert!(diagnostics[0].message.contains("thin-shell limit"));
    }

    #[test]
    fn architecture_checks_reject_artifact_backed_ambient_capability() {
        let sources = vec![SourceFile::new(
            "crates/tigerstyle-core/src/lib.rs",
            "#![no_std]\n\npub fn process_id() -> u32 { std::process::id() }\n",
        )];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, EFFECT_CAPABILITY_POLICY_ID);
        assert!(diagnostics[0].message.contains("std::process::"));
        assert!(diagnostics[0].remediation.contains("Clock"));
    }

    #[test]
    fn architecture_checks_accept_artifact_backed_shell_constructor_scope() {
        let sources = vec![
            SourceFile::new(
                "crates/tigerstyle-core/src/lib.rs",
                "#![no_std]\n\npub trait Clock { fn now_ms(&self) -> u64; }\n",
            ),
            SourceFile::new(
                "crates/tigerstyle-std/src/lib.rs",
                "pub fn now_ms() -> u64 { std::time::SystemTime::now().elapsed().map_or(0, |_| 1) }\n",
            ),
        ];
        let diagnostics =
            check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["architecture"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn idioms_selection_skips_architecture_family_diagnostics() {
        let sources = vec![SourceFile::new(
            "crates/tigerstyle-core/src/lib.rs",
            "pub fn core() {}\n",
        )];
        let diagnostics = check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["idioms"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn architecture_checks_reject_artifact_backed_raw_type_state() {
        let sources = vec![SourceFile::new(
            "standards/check-fixtures/architecture/noncompliant/typestate-raw/src/lib.rs",
            "type ConnectionState = bool;

pub fn send_raw(_state: ConnectionState, _bytes: &[u8]) {}
",
        )];
        let diagnostics = check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["idioms"]));

        assert_eq!(diagnostics.len(), 3);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.policy_id == TYPE_STATE_POLICY_ID));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("type ConnectionState = bool")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("send_raw(")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("missing required marker")));
    }

    #[test]
    fn architecture_checks_accept_artifact_backed_type_state_api() {
        let sources = vec![SourceFile::new(
            "standards/check-fixtures/architecture/compliant/typestate-protocol/src/lib.rs",
            "pub struct Disconnected;
pub struct Connected;
pub struct Authenticated;
impl Session<Disconnected> { pub fn connect(self) -> Session<Connected> { todo!() } }
impl Session<Connected> { pub fn authenticate(self) -> Session<Authenticated> { todo!() } }
impl Session<Authenticated> { pub fn send(self) {} }
",
        )];
        let diagnostics = check_architecture(&sample_registry(), &sources, &FamilySelection::from_families(["idioms"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn proof_rail_checks_accept_passing_receipt() {
        let sources = vec![
            SourceFile::new(
                "standards/check-fixtures/verification/compliant/src/lib.rs",
                "pub enum Effect { ReadEnv }
",
            ),
            SourceFile::new(
                "standards/check-fixtures/verification/compliant/proof-receipt.txt",
                "proof-result: pass deterministic-effect-handlers
",
            ),
        ];
        let diagnostics =
            check_verified_proof_rails(&sample_registry(), &sources, &FamilySelection::from_families(["verification"]));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn proof_rail_checks_reject_missing_pass_signal() {
        let sources = vec![
            SourceFile::new(
                "standards/check-fixtures/verification/noncompliant/src/lib.rs",
                "pub enum Effect { ReadEnv }
",
            ),
            SourceFile::new(
                "standards/check-fixtures/verification/noncompliant/proof-receipt.txt",
                "proof-result: fail deterministic-effect-handlers
",
            ),
        ];
        let diagnostics =
            check_verified_proof_rails(&sample_registry(), &sources, &FamilySelection::from_families(["verification"]));

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, VERIFIED_POLICY_ID);
        assert!(diagnostics[0].message.contains("missing pass signal"));
        assert!(diagnostics[0].remediation.contains("rerun the configured proof command"));
    }

    #[test]
    fn proof_rail_default_selection_rejects_relevant_noncompliant_receipt() {
        let sources = vec![
            SourceFile::new(
                "standards/check-fixtures/verification/noncompliant/src/lib.rs",
                "pub enum Effect { ReadEnv }\n",
            ),
            SourceFile::new(
                "standards/check-fixtures/verification/noncompliant/proof-receipt.txt",
                "proof-result: fail deterministic-effect-handlers\n",
            ),
        ];
        let diagnostics = check_verified_proof_rails(&sample_registry(), &sources, &FamilySelection::all());

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].policy_id, VERIFIED_POLICY_ID);
        assert!(diagnostics[0].message.contains("missing pass signal"));
    }

    #[test]
    fn proof_rail_family_isolation_skips_verification_diagnostics() {
        let sources = vec![SourceFile::new(
            "standards/check-fixtures/verification/noncompliant/proof-receipt.txt",
            "proof-result: fail deterministic-effect-handlers
",
        )];
        let diagnostics =
            check_verified_proof_rails(&sample_registry(), &sources, &FamilySelection::from_families(["embedded"]));

        assert!(diagnostics.is_empty());
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
    fn advisory_rule_reports_through_run_checks_without_blocking() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(
                PUBLIC_API_BUDGET_FIXTURE_PATH,
                PUBLIC_API_BUDGET_FIXTURE_SOURCE,
            )],
            selection: FamilySelection::from_families(["architecture"]),
        });

        assert_eq!(report.diagnostics.len(), 1);
        let diagnostic = &report.diagnostics[0];
        assert_eq!(diagnostic.policy_id, PUBLIC_API_BUDGET_POLICY_ID);
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert!(diagnostic.source_section.contains("when-boundaries-are-too-tight"));
        assert!(diagnostic.message.contains("advisory budget"));
        assert!(report.is_advisory_only());
        assert!(!report.has_blocking_diagnostics());
        assert!(report.evaluated_policy_ids.iter().any(|policy_id| policy_id == PUBLIC_API_BUDGET_POLICY_ID));
    }

    #[test]
    fn advisory_rule_accepts_small_public_surface() {
        let report = run_checks(&CheckInput {
            registry: sample_registry(),
            sources: vec![SourceFile::new(SOURCE_PATH, "pub fn one_public_item() {}\n")],
            selection: FamilySelection::from_families(["architecture"]),
        });

        assert!(report.diagnostics.is_empty());
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
                    EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#boundaries-as-architecture",
                ),
                sample_policy(
                    ROOT_THINNESS_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#boundaries-the-compiler-can-see",
                ),
                sample_advisory_policy(
                    PUBLIC_API_BUDGET_POLICY_ID,
                    "architecture",
                    "docs/standards/source/architecture.adoc#when-boundaries-are-too-tight",
                ),
                sample_effect_capability_policy(),
                sample_type_state_policy(),
                sample_verified_policy(),
            ],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn sample_advisory_policy(policy_id: &str, pillar: &str, source_anchor: &str) -> PolicyEntry {
        PolicyEntry {
            classification: Classification::Advisory,
            enforcement_level: EnforcementLevel::Advisory,
            default_severity: Severity::Warning,
            suppression_requirements: advisory_suppression_requirements(),
            fixture_coverage: FixtureCoverage {
                status: FixtureStatus::NotApplicable,
                compliant: Vec::new(),
                noncompliant: Vec::new(),
            },
            verification_evidence_requirements: vec![
                "deterministic sample input".to_owned(),
                "stable advisory report".to_owned(),
            ],
            classification_rationale: "sample advisory policy".to_owned(),
            false_positive_risk: "medium".to_owned(),
            ..sample_policy(policy_id, pillar, source_anchor)
        }
    }

    fn sample_effect_capability_policy() -> PolicyEntry {
        PolicyEntry {
            mechanisms: vec![Mechanism {
                kind: MechanismKind::CapabilityApi,
                description: "test capability mechanism".to_owned(),
                forbidden_ambient_apis: vec!["std::process::".to_owned(), "SystemTime::now(".to_owned()],
                core_source_scopes: vec!["crates/tigerstyle-core/src/".to_owned()],
                shell_constructor_scopes: vec!["crates/tigerstyle-std/src/".to_owned()],
                approved_capability_names: vec!["Clock".to_owned()],
                ..Mechanism::default()
            }],
            current_enforcement_mode: EnforcementMode::Gate,
            target_enforcement_mode: EnforcementMode::Structural,
            ..sample_policy(
                EFFECT_CAPABILITY_POLICY_ID,
                "architecture",
                "docs/standards/source/architecture.adoc#effect-capability-boundaries",
            )
        }
    }

    fn sample_type_state_policy() -> PolicyEntry {
        PolicyEntry {
            mechanisms: vec![Mechanism {
                kind: MechanismKind::TypeState,
                description: "test type-state mechanism".to_owned(),
                type_state_source_scopes: vec!["typestate-protocol/src/".to_owned(), "typestate-raw/src/".to_owned()],
                raw_state_alias_patterns: vec!["type ConnectionState = bool".to_owned()],
                raw_transition_patterns: vec!["send_raw(".to_owned()],
                required_state_markers: vec![
                    "Disconnected".to_owned(),
                    "Connected".to_owned(),
                    "Authenticated".to_owned(),
                ],
                required_transition_methods: vec![
                    "connect(".to_owned(),
                    "authenticate(".to_owned(),
                    "send(".to_owned(),
                ],
                ..Mechanism::default()
            }],
            current_enforcement_mode: EnforcementMode::Structural,
            target_enforcement_mode: EnforcementMode::Structural,
            ..sample_policy(
                TYPE_STATE_POLICY_ID,
                "idioms",
                "docs/standards/source/idioms.adoc#nominal-and-structural-type-boundaries",
            )
        }
    }

    fn sample_verified_policy() -> PolicyEntry {
        PolicyEntry {
            current_enforcement_mode: EnforcementMode::Verified,
            target_enforcement_mode: EnforcementMode::Verified,
            proof_rails: vec![ProofRail {
                backend: "local-receipt".to_owned(),
                command: "cargo test proof_rail".to_owned(),
                source_paths: vec!["standards/check-fixtures/verification/".to_owned()],
                proof_paths: vec!["proof-receipt.txt".to_owned()],
                pass_signal: "proof-result: pass deterministic-effect-handlers".to_owned(),
                trust_boundaries: vec!["fixture receipt only".to_owned()],
                receipt_binding: "none".to_owned(),
                receipt_provenance: ReceiptProvenance::default(),
                receipt_verifier: ReceiptVerifier::default(),
            }],
            ..sample_policy(
                VERIFIED_POLICY_ID,
                "verification",
                "docs/standards/source/verification.adoc#deterministic-effect-handler-tests",
            )
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
            current_enforcement_mode: EnforcementMode::Gate,
            target_enforcement_mode: EnforcementMode::Gate,
            escalation_criteria: "already gated".to_owned(),
            mechanisms: Vec::new(),
            deterministic_checker: EvidenceCommand::default(),
            proof_rails: Vec::new(),
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

    fn advisory_suppression_requirements() -> SuppressionRequirements {
        SuppressionRequirements {
            requires_policy_id: true,
            requires_source_section: true,
            requires_owner: false,
            requires_rationale: true,
            requires_review_trigger: false,
            requires_safer_default: false,
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
                    policy_id: EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID.to_owned(),
                    compliant: vec!["architecture/domain-ok.rs".to_owned()],
                    noncompliant: vec!["architecture/domain-bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: ROOT_THINNESS_POLICY_ID.to_owned(),
                    compliant: vec!["architecture/root-ok.rs".to_owned()],
                    noncompliant: vec!["architecture/root-bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: EFFECT_CAPABILITY_POLICY_ID.to_owned(),
                    compliant: vec!["architecture/capability-ok.rs".to_owned()],
                    noncompliant: vec!["architecture/capability-bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: TYPE_STATE_POLICY_ID.to_owned(),
                    compliant: vec!["architecture/typestate-ok.rs".to_owned()],
                    noncompliant: vec!["architecture/typestate-bad.rs".to_owned()],
                },
                PolicyFixtureCoverage {
                    policy_id: VERIFIED_POLICY_ID.to_owned(),
                    compliant: vec!["verification/proof-ok.txt".to_owned()],
                    noncompliant: vec!["verification/proof-bad.txt".to_owned()],
                },
            ],
        }
    }
}
