## Context

Octet has adopted native Cairn lifecycle content but has not consumed Cairn's COB-only collaboration provider. The provider accepts an explicit Git repository and an explicit mutation keystore, signs standard COB operations, refuses hidden runtime synchronization, and leaves Git ref movement to the operator. The pilot must exercise those surfaces without turning ephemeral signing authority into a checked-in secret or adding Radicle product dependencies.

## Success Contract

The pilot is complete only when deterministic evidence shows all of the following:

1. A valid Octet native change is previewed and signed into a separate bare Git store.
2. A later active-change revision is previewed and appended by the embedded creator.
3. Exact branch and namespaced refs are moved through an explicit local Git command into a second store with matching object IDs and no configured store remotes.
4. A second Octet checkout reports the change as absent, previews materialization, executes checkout, and matches the primary active-change bytes.
5. Conflict inspection reports the bounded conflict-free state.
6. Checked-in validation rejects tampered cross-receipt identities and preserves the provider's non-claims.

False completion includes mock-only receipts, reuse of Cairn implementation tests as Octet evidence, a checked-in private key, an umbrella `radicle` or `radicle-node` dependency, hidden remote configuration, live node/profile use, or a claim of publication, synchronization, convergence, durability, authority beyond embedded keys, or release eligibility.

## Portfolio Search

| Family | Mechanism | State | Discriminating evidence |
| --- | --- | --- | --- |
| Separate explicit COB store | Seed a bare store from Octet's base commit, run Cairn mutations there, and move explicit refs to another bare store | selected | signed import/update receipts, exact ref equality, no configured remotes, second checkout |
| Octet worktree Git database | Write namespaced COB refs directly into Octet's primary `.git` | rejected | exercises the same provider but unnecessarily changes durable operator refs and weakens pilot cleanup |
| Mocked collaboration adapter | Validate synthetic JSON without signed Git writes | falsified | cannot establish downstream signing, ref encoding, transport, or checkout behavior |
| Live Radicle profile/node | Publish and fetch through Radicle runtime services | excluded | contradicts the COB-only boundary and would overclaim network behavior |

The selected mechanism changes the consuming repository and operator workflow while retaining Cairn as an external CLI. Its remaining correlation is explicit: both sides use Cairn's provider, so this is downstream integration evidence rather than a second format implementation.

## Decisions

### Decision: Keep COB storage separate from Octet's primary Git database

**Choice:** Clone the Octet base revision into a temporary bare store under ignored `target/`, configure repository-local display identity there, and pass that path explicitly to every Cairn command.

**Rationale:** The active lifecycle root remains a real Octet checkout while signed COB refs stay isolated, inspectable, disposable, and free of hidden repository discovery.

### Decision: Keep signer authority ephemeral

**Choice:** Generate an unencrypted Ed25519 OpenSSH key under ignored `target/`, record only its public actor ID, and delete no user-owned keys.

**Rationale:** The pilot needs real signatures but no production authority. Committing the private key would turn a local integration probe into reusable public authority.

### Decision: Demonstrate update before transport

**Choice:** Import the accepted pilot plan, implement the guide and evidence verifier, then append the changed lifecycle artifacts with `cob update` before transport.

**Rationale:** This makes the update non-noop and proves preview/execute behavior against an actual Octet change revision.

### Decision: Treat Git as the only transport shell

**Choice:** Initialize a second bare store and fetch only `refs/heads/main` plus `refs/namespaces/*` from the first local store by path. Verify matching targets and no configured remotes before Cairn status/checkout.

**Rationale:** This demonstrates the provider boundary without smuggling node discovery, announcement, seeding, or synchronization into Cairn.

### Decision: Separate pure evidence validation from file loading

**Choice:** The Rust regression test parses checked-in manifest and receipt bytes in a thin shell, then passes owned values to a deterministic validator that returns explicit diagnostics. Positive and tampered negative cases exercise the pure core without mocks.

**Rationale:** Cross-receipt identity checks remain reviewable and testable without standing up Git, Cairn, keys, or network services.

### Decision: Author pilot metadata in Nickel

**Choice:** `fixtures/cairn-cob-pilot/manifest.ncl` owns the typed human-reviewed manifest and exports byte-stable `manifest.json` for the Rust test.

**Rationale:** Nickel catches missing or mistyped evidence bindings before the runtime JSON consumer runs.

## Evidence Outputs

- `fixtures/cairn-cob-pilot/manifest.ncl` and generated `manifest.json`
- `fixtures/cairn-cob-pilot/evidence/import-preview.json`
- `fixtures/cairn-cob-pilot/evidence/import-execute.json`
- `fixtures/cairn-cob-pilot/evidence/update-preview.json`
- `fixtures/cairn-cob-pilot/evidence/update-execute.json`
- `fixtures/cairn-cob-pilot/evidence/status-secondary.json`
- `fixtures/cairn-cob-pilot/evidence/checkout-preview.json`
- `fixtures/cairn-cob-pilot/evidence/checkout-execute.json`
- `fixtures/cairn-cob-pilot/evidence/conflicts-secondary.json`
- `fixtures/cairn-cob-pilot/evidence/transport.json`
- `fixtures/cairn-cob-pilot/evidence/validation.md`

Each JSON receipt must retain its exact expected command, backend, object/ref identity, mutation posture, blockers, and non-claims. `transport.json` additionally binds the source/target ref map, base revision, public actor, source Cairn commit, and BLAKE3 identities of retained receipts.

## Risks / Trade-offs

- The pilot uses one Cairn build and one local filesystem, so it does not establish cross-platform behavior or implementation independence.
- The second store is populated by local Git fetch, so it proves explicit ref transport mechanics but not remote availability, authentication, authorization, or durability.
- The source repository already has unrelated local commits ahead of `origin/main`; the pilot binds its exact base revision and must not rewrite or publish those commits implicitly.
- Existing Octet traceability has unrelated baseline gaps. Validation must report that baseline honestly and prove only that the pilot's new IDs are covered.
