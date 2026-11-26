// r[verify octet.spec_trace_parity.rail]
// r[verify octet.spec_trace_parity.freshness]
// r[verify octet.spec_trace_parity.replay]
// r[verify octet.spec_trace_parity.adapter]
// r[verify octet.spec_trace_parity.receipt]
// r[verify octet.spec_trace_parity.fixtures]
use std::collections::VecDeque;

use super::*;

const CHECKER_VERSION: &str = "1.0.0";

fn model() -> Vec<u8> {
    b"model: counter increments by one".to_vec()
}

fn checker() -> CheckerIdentity {
    CheckerIdentity {
        identity: prefixed_identity(b"checker-build"),
        version: CHECKER_VERSION.to_owned(),
    }
}

fn traces() -> TraceSet {
    TraceSet {
        format: TRACE_FORMAT.to_owned(),
        traces: vec![
            Trace {
                trace_id: "trace-a".to_owned(),
                steps: vec![
                    TraceStep {
                        action: "increment".to_owned(),
                        expected_state: "1".to_owned(),
                    },
                    TraceStep {
                        action: "increment".to_owned(),
                        expected_state: "2".to_owned(),
                    },
                ],
            },
            Trace {
                trace_id: "trace-b".to_owned(),
                steps: vec![TraceStep {
                    action: "reset".to_owned(),
                    expected_state: "0".to_owned(),
                }],
            },
        ],
    }
}

fn projection() -> StateProjection {
    StateProjection {
        model_states: "counter values".to_owned(),
        implementation_states: "u32 counter values".to_owned(),
        description: "exact counter projection".to_owned(),
    }
}

fn declaration() -> AdapterDeclaration {
    AdapterDeclaration {
        adapter_id: "fixture-adapter".to_owned(),
        trace_format: TRACE_FORMAT.to_owned(),
        projection: projection(),
        loss_bound: LossBound::Exact,
    }
}

fn lossy_declaration() -> AdapterDeclaration {
    AdapterDeclaration {
        adapter_id: "lossy-adapter".to_owned(),
        trace_format: TRACE_FORMAT.to_owned(),
        projection: StateProjection {
            model_states: "ok|error".to_owned(),
            implementation_states: "s1|s2|error".to_owned(),
            description: "maps implementation states s1 and s2 onto model state ok".to_owned(),
        },
        loss_bound: LossBound::Bounded {
            bound: "s1 and s2 collapse to ok".to_owned(),
        },
    }
}

fn lossy_traces() -> TraceSet {
    TraceSet {
        format: TRACE_FORMAT.to_owned(),
        traces: vec![Trace {
            trace_id: "lossy-trace".to_owned(),
            steps: vec![
                TraceStep {
                    action: "open".to_owned(),
                    expected_state: "ok".to_owned(),
                },
                TraceStep {
                    action: "close".to_owned(),
                    expected_state: "ok".to_owned(),
                },
            ],
        }],
    }
}

fn request_with(trace_set: TraceSet, adapter: Option<&AdapterDeclaration>) -> ParityRequest {
    let checker = checker();
    ParityRequest {
        pins: ParityPins {
            model_identity: prefixed_identity(&model()),
            checker_identity: checker.identity.clone(),
            checker_version: checker.version.clone(),
            seed: 42,
            trace_set_identity: trace_set_identity(&trace_set),
            adapter_identity: adapter.map(adapter_identity),
        },
        model_bytes: model(),
        checker,
        seed: 42,
        trace_set,
        bounds: RunBounds {
            max_steps: 16,
            max_millis: 1_000,
        },
    }
}

fn request() -> ParityRequest {
    request_with(traces(), Some(&declaration()))
}

struct ScriptedAdapter {
    declaration: AdapterDeclaration,
    states: VecDeque<String>,
    elapsed: VecDeque<u64>,
    calls: usize,
    observed_seeds: Vec<u64>,
}

impl ScriptedAdapter {
    fn new(declaration: &AdapterDeclaration, states: &[&str]) -> Self {
        Self {
            declaration: declaration.clone(),
            states: states.iter().map(|state| (*state).to_owned()).collect(),
            elapsed: VecDeque::new(),
            calls: 0,
            observed_seeds: Vec::new(),
        }
    }

    fn with_elapsed(mut self, elapsed: &[u64]) -> Self {
        self.elapsed = elapsed.iter().copied().collect();
        self
    }
}

impl TraceAdapter for ScriptedAdapter {
    fn declaration(&self) -> &AdapterDeclaration {
        &self.declaration
    }

    fn observe(&mut self, step: &ReplayStep) -> StepObservation {
        self.calls += 1;
        self.observed_seeds.push(step.seed);
        StepObservation {
            implementation_state: self.states.pop_front().unwrap_or_else(|| step.expected_state.clone()),
            elapsed_millis: self.elapsed.pop_front().unwrap_or(0),
        }
    }
}

struct PanicAdapter {
    declaration: AdapterDeclaration,
}

impl PanicAdapter {
    fn new(declaration: AdapterDeclaration) -> Self {
        Self { declaration }
    }
}

impl TraceAdapter for PanicAdapter {
    fn declaration(&self) -> &AdapterDeclaration {
        &self.declaration
    }

    fn observe(&mut self, _step: &ReplayStep) -> StepObservation {
        panic!("adapter must not run before admission and bounds checks");
    }
}

fn validate(
    receipt: &ParityReceipt,
    request: &ParityRequest,
    adapter: Option<&AdapterDeclaration>,
    run: &ParityRun,
) -> Vec<String> {
    validate_receipt(receipt, request, adapter, run.record.as_ref())
}

fn denied_layers(receipt: &ParityReceipt) -> Vec<DenialLayer> {
    match &receipt.outcome {
        ReceiptOutcome::Denied { denials } => denials.iter().map(|denial| denial.layer).collect(),
        ReceiptOutcome::Verdict(_) => panic!("expected a denial receipt"),
    }
}

fn reidentified(mut receipt: ParityReceipt) -> ParityReceipt {
    receipt.receipt_identity = receipt_identity(&receipt).expect("receipt identity");
    receipt
}

#[test]
fn fresh_parity_is_deterministic_and_identity_bound() {
    let request = request();
    let mut first_adapter = ScriptedAdapter::new(&declaration(), &["1", "2", "0"]);
    let first = run(&request, Some(&mut first_adapter)).expect("first run");
    let mut second_adapter = ScriptedAdapter::new(&declaration(), &["1", "2", "0"]);
    let second = run(&request, Some(&mut second_adapter)).expect("second run");

    assert_eq!(first.receipt, second.receipt);
    assert_eq!(first.record, second.record);
    assert_eq!(first.receipt.outcome, ReceiptOutcome::Verdict(VerdictClass::Parity));
    assert_eq!(first.receipt.replayed_steps, 3);
    assert!(first.receipt.divergence_samples.is_empty());
    assert!(is_b3(&first.receipt.receipt_identity));
    assert_eq!(first_adapter.observed_seeds, vec![42, 42, 42]);
    assert_eq!(validate(&first.receipt, &request, Some(&declaration()), &first), Vec::<String>::new());
}

#[test]
fn replay_plan_records_projection_and_loss_bound() {
    let request = request();
    let plan = plan(&request, Some(&declaration())).expect("admitted plan");
    let binding = plan.identities.adapter.as_ref().expect("adapter binding");
    assert_eq!(binding.projection, projection());
    assert_eq!(binding.loss_bound, LossBound::Exact);

    let mut adapter = ScriptedAdapter::new(&declaration(), &["1", "2", "0"]);
    let result = run(&request, Some(&mut adapter)).expect("run");
    assert_eq!(result.receipt.adapter.as_ref(), Some(binding));
    assert_eq!(result.record.as_ref().expect("record").adapter, *binding);
}

#[test]
fn lossy_adapter_surfaces_projection_instead_of_undisclosed_parity() {
    let request = request_with(lossy_traces(), Some(&lossy_declaration()));
    let mut adapter = ScriptedAdapter::new(&lossy_declaration(), &["ok", "ok"]);
    let result = run(&request, Some(&mut adapter)).expect("run");

    assert_eq!(result.receipt.outcome, ReceiptOutcome::Verdict(VerdictClass::Parity));
    let binding = result.receipt.adapter.as_ref().expect("adapter binding");
    assert_eq!(binding.loss_bound, LossBound::Bounded {
        bound: "s1 and s2 collapse to ok".to_owned()
    });
    assert!(binding.projection.description.contains("s1"));
    assert!(result.receipt.non_claims.contains(&PROJECTION_NON_CLAIM.to_owned()));
    assert_eq!(result.record.as_ref().expect("replay record").adapter, *binding);
    assert_eq!(validate(&result.receipt, &request, Some(&lossy_declaration()), &result), Vec::<String>::new());

    let mut undisclosed = result.receipt.clone();
    undisclosed.non_claims.retain(|claim| claim != PROJECTION_NON_CLAIM);
    let undisclosed = reidentified(undisclosed);
    assert!(!validate(&undisclosed, &request, Some(&lossy_declaration()), &result).is_empty());
}

#[test]
fn adapter_missing_projection_or_loss_bound_is_rejected() {
    let mut missing_projection = declaration();
    missing_projection.projection.description.clear();
    assert!(validate_adapter(&missing_projection).iter().any(|item| item.contains("projection")));

    let mut missing_bound = declaration();
    missing_bound.loss_bound = LossBound::Bounded { bound: String::new() };
    assert!(validate_adapter(&missing_bound).iter().any(|item| item.contains("loss bound")));

    let mut unsupported_format = declaration();
    unsupported_format.trace_format = "foreign/itf/v1".to_owned();
    assert!(validate_adapter(&unsupported_format).iter().any(|item| item.contains("trace format")));

    let mut request = request();
    request.pins.adapter_identity = Some(adapter_identity(&missing_projection));
    let mut adapter = PanicAdapter::new(missing_projection);
    let result = run(&request, Some(&mut adapter)).expect("run");
    assert!(denied_layers(&result.receipt).contains(&DenialLayer::AdapterDeclaration));
    assert!(result.record.is_none());
}

#[test]
fn unsupported_verifier_without_adapter_emits_no_parity_claim() {
    let request = request_with(traces(), None);
    let result = run(&request, None).expect("run");
    assert_eq!(result.receipt.outcome, ReceiptOutcome::Verdict(VerdictClass::UnsupportedVerifier));
    assert_eq!(result.receipt.replayed_steps, 0);
    assert!(result.receipt.adapter.is_none());
    assert!(result.receipt.divergence_samples.is_empty());
    assert!(result.record.is_none());
    assert_eq!(validate_receipt(&result.receipt, &request, None, None), Vec::<String>::new());

    let mut forged = result.receipt.clone();
    forged.outcome = ReceiptOutcome::Verdict(VerdictClass::Parity);
    let forged = reidentified(forged);
    assert!(!validate_receipt(&forged, &request, None, None).is_empty());
}

#[test]
fn stale_model_identity_fails_closed_before_replay() {
    let mut request = request();
    let expected = request.pins.model_identity.clone();
    request.model_bytes = b"tampered model".to_vec();
    let mut adapter = PanicAdapter::new(declaration());
    let result = run(&request, Some(&mut adapter)).expect("run");

    assert_eq!(result.receipt.outcome, ReceiptOutcome::Denied {
        denials: vec![Denial {
            layer: DenialLayer::Model,
            expected,
            observed: prefixed_identity(b"tampered model"),
        }]
    });
    assert!(result.record.is_none());
    assert_eq!(validate(&result.receipt, &request, Some(&declaration()), &result), Vec::<String>::new());

    let mut tampered = result.receipt.clone();
    if let ReceiptOutcome::Denied { denials } = &mut tampered.outcome {
        denials[0].layer = DenialLayer::Seed;
    }
    let tampered = reidentified(tampered);
    assert!(!validate(&tampered, &request, Some(&declaration()), &result).is_empty());
}

#[test]
fn receipt_json_round_trips_with_a_stable_identity() {
    let request = request();
    let mut adapter = ScriptedAdapter::new(&declaration(), &["1", "2", "0"]);
    let result = run(&request, Some(&mut adapter)).expect("run");
    let json = serde_json::to_string_pretty(&result.receipt).expect("receipt JSON");
    let decoded: ParityReceipt = serde_json::from_str(&json).expect("decoded receipt");
    assert_eq!(decoded, result.receipt);
    assert_eq!(receipt_identity(&decoded).expect("decoded identity"), result.receipt.receipt_identity);
    assert_eq!(
        validate_receipt(&decoded, &request, Some(&declaration()), result.record.as_ref()),
        Vec::<String>::new()
    );
}

#[test]
fn stale_checker_identity_and_version_fail_closed() {
    let mut identity_drift = request();
    identity_drift.checker.identity = prefixed_identity(b"other-checker");
    let result = run(&identity_drift, Some(&mut PanicAdapter::new(declaration()))).expect("run");
    assert_eq!(denied_layers(&result.receipt), vec![DenialLayer::CheckerIdentity]);

    let mut version_drift = request();
    version_drift.checker.version = "2.0.0".to_owned();
    let result = run(&version_drift, Some(&mut PanicAdapter::new(declaration()))).expect("run");
    assert_eq!(denied_layers(&result.receipt), vec![DenialLayer::CheckerVersion]);
    assert_eq!(validate(&result.receipt, &version_drift, Some(&declaration()), &result), Vec::<String>::new());
}

#[test]
fn seed_drift_fails_closed() {
    let mut request = request();
    request.seed = 7;
    let result = run(&request, Some(&mut PanicAdapter::new(declaration()))).expect("run");
    assert_eq!(result.receipt.outcome, ReceiptOutcome::Denied {
        denials: vec![Denial {
            layer: DenialLayer::Seed,
            expected: "42".to_owned(),
            observed: "7".to_owned(),
        }]
    });
    assert_eq!(validate(&result.receipt, &request, Some(&declaration()), &result), Vec::<String>::new());
}

#[test]
fn tampered_trace_set_fails_closed() {
    let mut request = request();
    let expected = request.pins.trace_set_identity.clone();
    request.trace_set.traces[0].steps[0].expected_state = "tampered".to_owned();
    let observed = trace_set_identity(&request.trace_set);
    let result = run(&request, Some(&mut PanicAdapter::new(declaration()))).expect("run");
    assert_eq!(result.receipt.outcome, ReceiptOutcome::Denied {
        denials: vec![Denial {
            layer: DenialLayer::TraceSet,
            expected,
            observed,
        }]
    });
    assert_eq!(validate(&result.receipt, &request, Some(&declaration()), &result), Vec::<String>::new());
}

#[test]
fn unsupported_trace_format_fails_closed() {
    let mut request = request();
    request.trace_set.format = "foreign/itf/v1".to_owned();
    request.pins.trace_set_identity = trace_set_identity(&request.trace_set);
    let result = run(&request, Some(&mut PanicAdapter::new(declaration()))).expect("run");
    let layers = denied_layers(&result.receipt);
    assert!(layers.contains(&DenialLayer::TraceFormat));
    assert!(layers.contains(&DenialLayer::AdapterTraceFormat));
}

#[test]
fn divergence_is_classified_with_bounded_samples() {
    let request = request();
    let mut adapter = ScriptedAdapter::new(&declaration(), &["1", "999", "0"]);
    let result = run(&request, Some(&mut adapter)).expect("run");

    assert_eq!(result.receipt.outcome, ReceiptOutcome::Verdict(VerdictClass::Divergence));
    assert_eq!(result.receipt.replayed_steps, 3);
    assert_eq!(result.receipt.divergence_samples, vec![DivergenceSample {
        trace_id: "trace-a".to_owned(),
        step_index: 1,
        action: "increment".to_owned(),
        expected_state: "2".to_owned(),
        observed_state: "999".to_owned(),
    }]);
    assert_eq!(validate(&result.receipt, &request, Some(&declaration()), &result), Vec::<String>::new());

    let mut forged = result.receipt.clone();
    forged.outcome = ReceiptOutcome::Verdict(VerdictClass::Parity);
    let forged = reidentified(forged);
    assert!(!validate(&forged, &request, Some(&declaration()), &result).is_empty());
}

#[test]
fn step_budget_denial_terminates_before_replay() {
    let mut request = request();
    request.bounds.max_steps = 2;
    let mut adapter = PanicAdapter::new(declaration());
    let result = run(&request, Some(&mut adapter)).expect("run");

    assert_eq!(result.receipt.outcome, ReceiptOutcome::Denied {
        denials: vec![Denial {
            layer: DenialLayer::StepBudget,
            expected: "2".to_owned(),
            observed: "3".to_owned(),
        }]
    });
    let record = result.record.as_ref().expect("bounded record");
    assert!(record.steps.is_empty());
    assert_eq!(record.bound_denial, Some(BoundDenial::StepBudget { limit: 2, observed: 3 }));
    assert_eq!(validate(&result.receipt, &request, Some(&declaration()), &result), Vec::<String>::new());
}

#[test]
fn time_budget_denial_terminates_after_one_observed_step() {
    let mut request = request();
    request.bounds.max_millis = 5;
    let mut adapter = ScriptedAdapter::new(&declaration(), &["1", "2", "0"]).with_elapsed(&[10, 10, 10]);
    let result = run(&request, Some(&mut adapter)).expect("run");

    assert_eq!(result.receipt.outcome, ReceiptOutcome::Denied {
        denials: vec![Denial {
            layer: DenialLayer::TimeBudget,
            expected: "5".to_owned(),
            observed: "exceeded".to_owned(),
        }]
    });
    let record = result.record.as_ref().expect("bounded record");
    assert_eq!(record.steps.len(), 1);
    assert_eq!(record.bound_denial, Some(BoundDenial::TimeBudget { limit_millis: 5 }));
    assert_eq!(validate(&result.receipt, &request, Some(&declaration()), &result), Vec::<String>::new());
}

#[test]
fn replay_is_bound_to_the_admitted_seed() {
    struct SeedAdapter {
        declaration: AdapterDeclaration,
        expected_seed: u64,
    }

    impl TraceAdapter for SeedAdapter {
        fn declaration(&self) -> &AdapterDeclaration {
            &self.declaration
        }

        fn observe(&mut self, step: &ReplayStep) -> StepObservation {
            StepObservation {
                implementation_state: if step.seed == self.expected_seed {
                    step.expected_state.clone()
                } else {
                    "seed-mismatch".to_owned()
                },
                elapsed_millis: 0,
            }
        }
    }

    let request = request();
    let mut matching = SeedAdapter {
        declaration: declaration(),
        expected_seed: 42,
    };
    let matched = run(&request, Some(&mut matching)).expect("matching seed");
    assert_eq!(matched.receipt.outcome, ReceiptOutcome::Verdict(VerdictClass::Parity));

    let mut drifted = request.clone();
    drifted.seed = 9;
    drifted.pins.seed = 9;
    let mut adapter = SeedAdapter {
        declaration: declaration(),
        expected_seed: 42,
    };
    let result = run(&drifted, Some(&mut adapter)).expect("drifted seed");
    assert_eq!(result.receipt.outcome, ReceiptOutcome::Verdict(VerdictClass::Divergence));
}

#[test]
fn receipt_identity_binds_every_pin_and_bound() {
    let request = request();
    let mut adapter = ScriptedAdapter::new(&declaration(), &["1", "2", "0"]);
    let result = run(&request, Some(&mut adapter)).expect("run");
    let receipt = result.receipt.clone();

    let mut model_drift = receipt.clone();
    model_drift.model_identity = prefixed_identity(b"other-model");
    assert!(!validate(&reidentified(model_drift), &request, Some(&declaration()), &result).is_empty());

    let mut checker_drift = receipt.clone();
    checker_drift.checker_identity = prefixed_identity(b"other-checker");
    assert!(!validate(&reidentified(checker_drift), &request, Some(&declaration()), &result).is_empty());

    let mut version_drift = receipt.clone();
    version_drift.checker_version = "9.9.9".to_owned();
    assert!(!validate(&reidentified(version_drift), &request, Some(&declaration()), &result).is_empty());

    let mut seed_drift = receipt.clone();
    seed_drift.seed = 1;
    assert!(!validate(&reidentified(seed_drift), &request, Some(&declaration()), &result).is_empty());

    let mut trace_drift = receipt.clone();
    trace_drift.trace_set_identity = prefixed_identity(b"other-traces");
    assert!(!validate(&reidentified(trace_drift), &request, Some(&declaration()), &result).is_empty());

    let mut adapter_drift = receipt.clone();
    let binding = adapter_drift.adapter.as_mut().expect("adapter binding");
    binding.identity = prefixed_identity(b"other-adapter");
    assert!(!validate(&reidentified(adapter_drift), &request, Some(&declaration()), &result).is_empty());

    let mut bounds_drift = receipt;
    bounds_drift.bounds.max_steps = 1;
    assert!(!validate(&reidentified(bounds_drift), &request, Some(&declaration()), &result).is_empty());
}

#[test]
fn receipt_rejects_record_and_step_count_tampering() {
    let request = request();
    let mut adapter = ScriptedAdapter::new(&declaration(), &["1", "2", "0"]);
    let result = run(&request, Some(&mut adapter)).expect("run");

    let mut truncated = result.record.clone().expect("record");
    truncated.steps.pop();
    assert!(!validate_receipt(&result.receipt, &request, Some(&declaration()), Some(&truncated)).is_empty());

    let mut step_drift = result.receipt.clone();
    step_drift.replayed_steps = 1;
    let step_drift = reidentified(step_drift);
    assert!(!validate(&step_drift, &request, Some(&declaration()), &result).is_empty());
}

#[test]
fn malformed_requests_fail_closed_before_replay() {
    let mut empty_traces = request();
    empty_traces.trace_set.traces.clear();
    empty_traces.pins.trace_set_identity = trace_set_identity(&empty_traces.trace_set);
    let result = run(&empty_traces, Some(&mut PanicAdapter::new(declaration()))).expect("run");
    assert_eq!(denied_layers(&result.receipt), vec![DenialLayer::Request]);

    let mut unbounded = request();
    unbounded.bounds.max_steps = 0;
    let result = run(&unbounded, Some(&mut PanicAdapter::new(declaration()))).expect("run");
    assert_eq!(denied_layers(&result.receipt), vec![DenialLayer::Request]);
}

#[test]
fn dive_tracker_pin_records_schema4_main_commit_and_non_claims() {
    let pin = super::external::dive_pin();
    assert_eq!(pin.repository, "https://github.com/Chelis-Lang/dive");
    assert_eq!(pin.tracker, "OnixResearch/trace-lab#6");
    assert_eq!(pin.commit_sha, "0fc00914b7cd93d88e97e5d7d66aac805c06228b");
    assert_eq!(pin.report_schema_version, 4);
    assert_eq!(pin.doc_path, "docs/user/traces.md");
    assert!(pin.non_claims.iter().any(|claim| claim.contains("default-off")));
    assert!(pin.non_claims.iter().any(|claim| claim.contains("Aspen #62")));
}

#[test]
fn dive_report_admission_binds_complete_schema4_stops_unsupported_and_denies_drift() {
    let mut base = request();
    base.pins.adapter_identity = None;
    base.checker.identity = "b3:2a39fc7ca7c05679c7521c1f390183dc8182c5f9e2e4b7eca2ce50bfe8e7f09c".to_owned();
    base.checker.version = "dive-main-0fc00914".to_owned();
    base.pins.checker_identity = base.checker.identity.clone();
    base.pins.checker_version = base.checker.version.clone();

    let complete = super::external::DiveReport {
        schema_version: 4,
        dive_commit_sha: super::external::DIVE_MAIN_COMMIT.to_owned(),
        checker_identity: base.checker.identity.clone(),
        checker_version: base.checker.version.clone(),
        model_bytes: Some(base.model_bytes.clone()),
        seed: Some(base.seed),
        mapped_set: Some(base.trace_set.clone()),
    };
    let admitted = super::external::admit_dive_report(&base.pins, &complete, base.bounds);
    match admitted {
        super::external::DiveAdmission::Admitted(identities) => {
            assert_eq!(identities.checker_identity, base.pins.checker_identity);
            assert_eq!(identities.model_identity, base.pins.model_identity);
            assert_eq!(identities.seed, base.pins.seed);
            assert_eq!(identities.trace_set_identity, base.pins.trace_set_identity);
        }
        other => panic!("expected complete schema-4 Dive report to admit, got {other:?}"),
    }

    // Incomplete schema-4 report (e.g. raw handoff without seed/mapped trace set) stops as Unsupported.
    let mut incomplete = complete.clone();
    incomplete.seed = None;
    incomplete.mapped_set = None;
    match super::external::admit_dive_report(&base.pins, &incomplete, base.bounds) {
        super::external::DiveAdmission::Unsupported { reason, non_claims } => {
            assert!(reason.contains("seed") && reason.contains("mapped_set"), "{reason}");
            assert!(!non_claims.is_empty());
        }
        other => panic!("expected incomplete schema-4 report to stop as Unsupported, got {other:?}"),
    }

    // Dive 0.1.0 commit must be denied for trace-lab#6.
    let mut old_pin = complete.clone();
    old_pin.dive_commit_sha = "94cbbed9ae02ad3d30bbe2191bd2619a839b8795".to_owned();
    match super::external::admit_dive_report(&base.pins, &old_pin, base.bounds) {
        super::external::DiveAdmission::Denied { denials } => {
            assert_eq!(denials.len(), 1);
            assert_eq!(denials[0].layer, DenialLayer::CheckerVersion);
        }
        other => panic!("expected Dive 0.1.0 pin to be denied, got {other:?}"),
    }

    // Non-schema-4 report must be denied.
    let mut wrong_schema = complete.clone();
    wrong_schema.schema_version = 3;
    match super::external::admit_dive_report(&base.pins, &wrong_schema, base.bounds) {
        super::external::DiveAdmission::Denied { denials } => {
            assert_eq!(denials.len(), 1);
            assert_eq!(denials[0].layer, DenialLayer::TraceFormat);
        }
        other => panic!("expected non-schema-4 report to be denied, got {other:?}"),
    }

    // Stale model bytes fail closed through spec_trace_parity::admit.
    let mut stale_model = complete;
    stale_model.model_bytes = Some(b"drifted-model".to_vec());
    match super::external::admit_dive_report(&base.pins, &stale_model, base.bounds) {
        super::external::DiveAdmission::Denied { denials } => {
            assert_eq!(denials.len(), 1);
            assert_eq!(denials[0].layer, DenialLayer::Model);
        }
        other => panic!("expected stale model bytes to be denied, got {other:?}"),
    }
}

fn dive_base_and_complete_report() -> (ParityRequest, super::external::DiveReport) {
    let mut base = request();
    base.pins.adapter_identity = None;
    base.checker.identity = "b3:2a39fc7ca7c05679c7521c1f390183dc8182c5f9e2e4b7eca2ce50bfe8e7f09c".to_owned();
    base.checker.version = "dive-main-0fc00914".to_owned();
    base.pins.checker_identity = base.checker.identity.clone();
    base.pins.checker_version = base.checker.version.clone();

    let report = super::external::DiveReport {
        schema_version: 4,
        dive_commit_sha: super::external::DIVE_MAIN_COMMIT.to_owned(),
        checker_identity: base.checker.identity.clone(),
        checker_version: base.checker.version.clone(),
        model_bytes: Some(base.model_bytes.clone()),
        seed: Some(base.seed),
        mapped_set: Some(base.trace_set.clone()),
    };
    (base, report)
}

#[test]
fn dive_pin_serializes_and_dive_report_json_deserializes_with_deny_unknown_fields() {
    let pin_json = serde_json::to_value(super::external::dive_pin()).expect("serialize DivePin");
    assert_eq!(pin_json["repository"], "https://github.com/Chelis-Lang/dive");
    assert_eq!(pin_json["tracker"], "OnixResearch/trace-lab#6");
    assert_eq!(pin_json["commit_sha"], super::external::DIVE_MAIN_COMMIT);
    assert_eq!(pin_json["report_schema_version"], 4);
    assert_eq!(pin_json["doc_path"], "docs/user/traces.md");
    assert_eq!(pin_json["non_claims"].as_array().map(Vec::len), Some(super::external::DIVE_NON_CLAIMS.len()));

    let (base, complete) = dive_base_and_complete_report();
    let complete_json = serde_json::to_string(&complete).expect("serialize DiveReport");
    let decoded: super::external::DiveReport = serde_json::from_str(&complete_json).expect("deserialize DiveReport");
    assert_eq!(decoded, complete);
    assert!(matches!(
        super::external::admit_dive_report(&base.pins, &decoded, base.bounds),
        super::external::DiveAdmission::Admitted(_)
    ));

    let raw_without_bindings = r#"{
        "schema_version": 4,
        "dive_commit_sha": "0fc00914b7cd93d88e97e5d7d66aac805c06228b",
        "checker_identity": "b3:2a39fc7ca7c05679c7521c1f390183dc8182c5f9e2e4b7eca2ce50bfe8e7f09c",
        "checker_version": "dive-main-0fc00914",
        "model_bytes": null,
        "seed": null,
        "mapped_set": null
    }"#;
    let partial: super::external::DiveReport =
        serde_json::from_str(raw_without_bindings).expect("deserialize DiveReport with null bindings");
    assert_eq!(partial.model_bytes, None);
    assert_eq!(partial.seed, None);
    assert_eq!(partial.mapped_set, None);

    let unknown_field_json = r#"{
        "schema_version": 4,
        "dive_commit_sha": "0fc00914b7cd93d88e97e5d7d66aac805c06228b",
        "checker_identity": "b3:2a39fc7ca7c05679c7521c1f390183dc8182c5f9e2e4b7eca2ce50bfe8e7f09c",
        "checker_version": "dive-main-0fc00914",
        "model_bytes": null,
        "seed": null,
        "mapped_set": null,
        "extra_field": true
    }"#;
    let err = serde_json::from_str::<super::external::DiveReport>(unknown_field_json)
        .expect_err("unknown field must be rejected");
    assert!(err.to_string().contains("unknown field"), "{err}");
}

#[test]
fn dive_report_missing_only_model_bytes_stops_as_unsupported() {
    let (base, mut report) = dive_base_and_complete_report();
    report.model_bytes = None;

    match super::external::admit_dive_report(&base.pins, &report, base.bounds) {
        super::external::DiveAdmission::Unsupported { reason, non_claims } => {
            assert!(reason.contains("model_bytes"), "{reason}");
            assert!(!reason.contains("seed"), "{reason}");
            assert!(!reason.contains("mapped_set"), "{reason}");
            assert_eq!(non_claims.len(), super::external::DIVE_NON_CLAIMS.len());
        }
        other => panic!("expected missing model_bytes alone to stop as Unsupported, got {other:?}"),
    }
}

#[test]
fn dive_report_with_simultaneous_schema_and_commit_drift_denies_both_layers() {
    let (base, mut report) = dive_base_and_complete_report();
    report.schema_version = 3;
    report.dive_commit_sha = "94cbbed9ae02ad3d30bbe2191bd2619a839b8795".to_owned();
    report.model_bytes = None;
    report.seed = None;
    report.mapped_set = None;

    match super::external::admit_dive_report(&base.pins, &report, base.bounds) {
        super::external::DiveAdmission::Denied { denials } => {
            let layers: Vec<DenialLayer> = denials.into_iter().map(|denial| denial.layer).collect();
            assert_eq!(layers, vec![DenialLayer::TraceFormat, DenialLayer::CheckerVersion]);
        }
        other => panic!("expected simultaneous schema and commit drift to deny both layers, got {other:?}"),
    }
}

#[test]
fn dive_report_admission_denies_checker_seed_trace_set_and_adapter_pin_drift() {
    let (base, complete) = dive_base_and_complete_report();

    let mut checker_id_drift = complete.clone();
    checker_id_drift.checker_identity = prefixed_identity(b"other-dive-checker");
    match super::external::admit_dive_report(&base.pins, &checker_id_drift, base.bounds) {
        super::external::DiveAdmission::Denied { denials } => {
            assert_eq!(denials.len(), 1);
            assert_eq!(denials[0].layer, DenialLayer::CheckerIdentity);
        }
        other => panic!("expected checker_identity drift to be denied, got {other:?}"),
    }

    let mut checker_version_drift = complete.clone();
    checker_version_drift.checker_version = "dive-0.1.0".to_owned();
    match super::external::admit_dive_report(&base.pins, &checker_version_drift, base.bounds) {
        super::external::DiveAdmission::Denied { denials } => {
            assert_eq!(denials.len(), 1);
            assert_eq!(denials[0].layer, DenialLayer::CheckerVersion);
        }
        other => panic!("expected checker_version drift to be denied, got {other:?}"),
    }

    let mut seed_drift = complete.clone();
    seed_drift.seed = Some(base.seed + 1);
    match super::external::admit_dive_report(&base.pins, &seed_drift, base.bounds) {
        super::external::DiveAdmission::Denied { denials } => {
            assert_eq!(denials.len(), 1);
            assert_eq!(denials[0].layer, DenialLayer::Seed);
        }
        other => panic!("expected seed drift to be denied, got {other:?}"),
    }

    let mut mapped_set_drift = complete.clone();
    mapped_set_drift.mapped_set.as_mut().expect("mapped_set").traces[0].trace_id = "tampered-trace".to_owned();
    match super::external::admit_dive_report(&base.pins, &mapped_set_drift, base.bounds) {
        super::external::DiveAdmission::Denied { denials } => {
            assert_eq!(denials.len(), 1);
            assert_eq!(denials[0].layer, DenialLayer::TraceSet);
        }
        other => panic!("expected mapped_set drift to be denied, got {other:?}"),
    }

    let mut adapter_pin = base.pins.clone();
    adapter_pin.adapter_identity = Some(prefixed_identity(b"unexpected-adapter"));
    match super::external::admit_dive_report(&adapter_pin, &complete, base.bounds) {
        super::external::DiveAdmission::Denied { denials } => {
            assert_eq!(denials.len(), 1);
            assert_eq!(denials[0].layer, DenialLayer::AdapterIdentity);
        }
        other => panic!("expected adapter_identity pin to be denied, got {other:?}"),
    }
}
