# Pilot assurance targets: Octet's Aeneas external-consumer fixture over the
# realized Mantle producer output P.
#
# The target binds the content-bound registry row for
# octet.aeneas_consumer_contract.compatibility by requirement id, derived from
# the checked-in registry export
# `fixtures/cairn-pilot/requirement-registry.json` (Octet revision
# d5b34c2d64b1227417a46df8ed8427f2d10ed3c8). The row fields and the contract
# specification are not copied: Cairn's `requirement_from_registry` and
# `specification_from_registry` select exactly one row from the export and fail
# closed otherwise.
# The claim restates the theorem the producer job actually proved; the informal
# remainder records what the requirement prose asks beyond it.
#
# Export deterministic JSON with:
#   nickel export --format json standards/requirements/cairn-pilot/targets.ncl \
#     | jq -S . > standards/requirements/cairn-pilot/targets.json
let Cairn = import "../../../cairn-policy/assurance-targets.ncl" in
let registry = import "../fixtures/cairn-pilot/requirement-registry.json" in

{
  schema_version = "cairn.assurance-targets.v1",
  contracts = [
    {
      contract_id = "octet.cairn-pilot.aeneas-consumer",
      specification = Cairn.specification_from_registry registry "octet.aeneas_consumer_contract.compatibility",
      targets = [
        {
          requirement = Cairn.requirement_from_registry registry "octet.aeneas_consumer_contract.compatibility",
          assurance = "checked_proof",
          claim = "The external-consumer fixture's bound implementation is increment: its Aeneas-generated Lean model satisfies the named theorem increment_matches_model, `external_consumer.increment 1#u32 = .ok 2#u32`, which Lean checks by reduction, and the theorem is judged non-vacuous (decision non_vacuous, no forbidden declarations) over the exact bound Rust source, generated Lean model, and proof artifacts.",
          informal_remainder = "The requirement prose asks for more than this theorem. It also requires that the shipped Aeneas fixture rail, the existing Verus rails, and the ordinary starter checks stay compatible, with their accepted outputs and expected failures retaining their declared outcomes. This target's claim covers only the named external-consumer theorem over its bound source and artifacts; it neither proves nor tests the preservation of the other rails.",
          evidence_job = {
            job_id = "octet.aeneas.external-consumer.mantle.v1",
            definition = "standards/proof-rails/aeneas-lean/mantle-job.ncl",
          },
          checked_scope = [
            "formal-proof:external-consumer:octet.aeneas_consumer_contract.external-fixture#rail-0",
          ],
          evidence_ids = [
            "aeneas.consumer.increment-matches-model",
          ],
          evidence_classes = ["proof_card"],
          provider = "aeneas-lean",
          negative_controls = [],
        },
      ],
      assumptions = [],
      exclusions = [],
      non_claims = [
        "evaluates supplied reviewed contracts and evidence only; does not authenticate reviewers or producers, prove verifier soundness, or establish whole-program correctness",
        "phase-1 pilot non-claim: the evidence is a fixture theorem over a fixture rail; it is not a production theorem, not whole-program correctness, and not release eligibility",
        "does not prove Aeneas or Charon translation soundness",
        "does not prove Lean or verifier soundness",
        "does not prove rustc compiler correctness",
        "does not prove semantic equivalence",
        "does not establish release eligibility",
      ],
    },
  ],
} | Cairn.TargetsFile
