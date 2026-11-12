## Context

The accepted Octet COB adoption evidence covers one creator's linear history. Cairn already requires immutable root-embedded creator/delegate authority, refuses ordinary updates over conflicting heads, exposes metadata-only conflict inspection by default, reveals one exact value only through an explicit selector, and requires resolution to supersede every current head. The downstream drill must exercise those behaviors with real signatures and explicit Git refs without transferring Cairn's implementation claims to Octet.

## Success Contract

Completion requires deterministic evidence for all of the following:

1. One creator imports this native change with one delegate embedded at the signed root.
2. Creator and delegate independently sign different `tasks.md` values from the same imported basis.
3. Explicit local Git transport retains both actor namespace refs and Cairn reports a tasks conflict with both operation IDs.
4. Default inspection reveals metadata only; two explicit `--show-head` receipts reveal exactly the selected bounded values.
5. Ordinary update fails closed without mutation, while explicit resolution previews and executes the selected creator head over the complete conflict basis.
6. A third local store reloads the resolved graph, reports no current conflict, materializes the selected bytes, and still contains both divergent revisions as ancestors of the resolution operation.

False completion includes synthetic-only receipts, one actor overwriting its own ref, a resolution that omits a current head, checking only the selected value, deleting or making the losing revision unreachable, implicit filesystem winner selection, a checked-in private key, hidden Git remotes, node/profile use, or distributed and release claims.

## Portfolio Search

| Family | Mechanism | State | Discriminating evidence |
| --- | --- | --- | --- |
| Two embedded authorities | Creator and delegate sign different updates from the same imported root into separate stores, then explicit refs are combined | selected | distinct actor refs and operation IDs, common basis, conflict metadata, complete-basis resolution ancestry |
| One actor with divergent local stores | One signer advances the same namespace ref differently in two stores | falsified | a single actor ref cannot retain both tips simultaneously after ordinary ref transport |
| Synthetic ref/object injection | Hand-build conflicting Git commits or alter retained receipts | rejected | does not exercise Cairn authorization, signatures, update refusal, or resolution execution |
| Live Radicle runtime | Publish branches through profiles, peers, or nodes | excluded | contradicts the COB-only boundary and introduces unsupported network claims |

The selected mechanism is independently audited by the static fixture validator and Git ancestry gate, but its producer and consumer observations still use Cairn's provider. This is downstream integration evidence, not an independent format implementation.

## Decisions

### Decision: Use separate stores for each same-basis update

**Choice:** Import once, copy only the initial branch and signed root ref into a delegate store, then perform the creator and delegate updates without exchanging their new refs until conflict construction.

**Rationale:** Both operations are forced to cite the same root basis. Updating one shared store sequentially would create ancestry instead of divergence.

### Decision: Keep conflict construction in the Git transport shell

**Choice:** Populate a convergence store with the creator ref from one store and the delegate ref from the other through explicit local `git fetch --no-tags` refspecs. Verify exact targets and no configured remotes.

**Rationale:** Cairn observes a valid multi-head COB graph while remaining uninvolved in transport, discovery, or synchronization.

### Decision: Inspect metadata and values separately

**Choice:** Retain one default `cob conflicts` receipt and one `--show-head tasks=REVISION` receipt for each divergent operation.

**Rationale:** The default receipt proves bounded metadata disclosure. Explicit receipts prove exact-value access without turning conflict inspection into an implicit winner.

### Decision: Resolve by explicit complete-basis selection

**Choice:** Preview and execute `cob resolve --choose-head tasks=CREATOR_REVISION` with the creator key, then require the resolution operation to descend from both divergent operations.

**Rationale:** Selecting the creator value is a reviewed fixture choice, not a semantic merge claim. Two-parent ancestry proves both current heads were superseded and keeps the losing delegate revision reachable.

### Decision: Keep evidence validation pure and typed

**Choice:** Author manifest metadata in Nickel. The Rust test's shell loads bytes and JSON, while deterministic functions validate cross-receipt identities, conflict bases, selections, hashes, and non-claims. A hermetic Nix check performs bundle import, ref, and ancestry validation.

**Rationale:** Logic can be tested without standing up Git, keys, or Cairn, while Git-specific graph facts remain checked by Git itself.

## Evidence Outputs

- `fixtures/cairn-cob-conflict-pilot/manifest.ncl` and generated `manifest.json`
- `fixtures/cairn-cob-conflict-pilot/octet-cob-conflict.bundle`
- signed import, creator-update, delegate-update, conflict-inspection, update-refusal, resolution, reload, checkout, and transport receipts under `fixtures/cairn-cob-conflict-pilot/evidence/`
- `tests/cairn_cob_conflict_pilot.rs`
- `checks.x86_64-linux.cairn-cob-conflict-pilot`

Every receipt is bound by BLAKE3 in the manifest. The final bundle contains the base branch and both actor namespace refs. The final creator ref targets the resolution operation; the delegate ref retains its divergent update; both divergent updates remain ancestors of the resolution operation.

## Risks / Trade-offs

- Random ephemeral fixture keys make the production run non-reproducible byte-for-byte, while the checked-in static bundle and manifest remain deterministically verifiable.
- Both producer and reload observations use the same Cairn version, so the drill does not prove cross-version compatibility or implementation independence.
- Local Git ref transport does not establish remote authentication, authorization, publication, availability, durability, synchronization, or convergence.
- Choosing one head proves explicit operator selection and graph supersession only, not semantic merge correctness or the invalidity of the losing head.
