# Third-party notices and provenance

Octet's `AGPL-3.0-or-later` grant does not replace third-party licenses,
copyrights, or attribution. Preserve upstream notices when redistributing
source or binaries. This inventory covers identified checked-in material;
it is not an exhaustive license inventory of Cargo or Nix dependency closures.

## Dylint support sources

- Paths: `tests/support/dylint/driver/`, `tests/support/dylint/internal/`,
  including the internal template and `template.tar`.
- Upstream: <https://github.com/trailofbits/dylint>.
- Reviewed baseline: `0e0a71eefe6f01563d11acc6e1d7af1d505934a9`.
- Copyright (c) 2021-2022 Trail of Bits. Upstream package metadata also
  credits Samuel E. Moelius III; those author fields are retained.
- Upstream offers `MIT OR Apache-2.0`. The bundled copy is redistributed
  using the MIT option. Full texts: [MIT](LICENSES/dylint-MIT.txt) and
  [Apache-2.0](LICENSES/dylint-APACHE.txt).

Comparison with that immutable baseline found the internal subtree identical.
The driver differs in `Cargo.toml` and `build/main.rs`; these are Octet's local
support patches, not an unmodified upstream release. The production flake
separately fetches the pinned upstream source and adds a binary entry point.

### Copied Clippy helper

`tests/support/dylint/internal/src/match_def_path.rs` retains its source link
to Rust Clippy revision `f62f26965817f2573c2649288faa489a03ed1665`:

<https://github.com/rust-lang/rust-clippy/tree/f62f26965817f2573c2649288faa489a03ed1665>

Copyright (c) 2014-2025 The Rust Project Developers. The copied helper is
redistributed under the [MIT license](LICENSES/clippy-MIT.txt).

## Imported standards prose: publication blocked pending permission

The removed `docs/standards/source/` material was credited to Adam Murray in
`front-matter.adoc` and `tigerstyle-standards.adoc`. The archived
`openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/` records
migration from a standalone repository, but does not identify an immutable
upstream source, original license, or redistribution permission.

The attribution is preserved; the prose is not part of this sanitized source
history. Old development clones and the `becae962` public export contain copies,
including embedded Git bundles, and require separate history cleanup. Octet's
project notice does not establish a right to relicense this material. Before
reintroducing it, maintainers must record the original source and license or
written permission.

## Cairn policy provenance

The following import-closure projections match Cairn revision
`2747aa6eeb6e9eb9862f2bc4e11336e768e2e10b` byte for byte:

- `cairn-policy/upstream-default.ncl` (upstream `cairn-policy/default.ncl`)
- `cairn-policy/contracts.ncl`
- `cairn-policy/lifecycle-store-contracts.ncl`
- `cairn-policy/nominal-evidence-identities.ncl`

Upstream identity: <https://github.com/OnixResearch/cairn/tree/2747aa6eeb6e9eb9862f2bc4e11336e768e2e10b>.
The exact source selection, four projection mappings, and archive SHA256
`05dde82a652ffa8791bef1fc1cdcbaed9cf28dd8306b2a89b043a5065e14fb55`
are recorded in [`vendor/dependencies.ncl`](vendor/dependencies.ncl).
`tools/import-public-dependencies.py --sync` refreshes these projections from
the admitted source; its integrity checks bind the copies to that source and
`vendor/SHA256SUMS`. This replaces the prior `695124d459574ba7aeba6097310d237f393c243c`
snapshot without representing the local Octet overlay as upstream material.

The Cairn copyright holders' explicit grant in the selected upstream
[`cairn-policy/LICENSING.md`](vendor/sources/cairn/cairn-policy/LICENSING.md)
offers the four policy inputs and generated policy contents under
`AGPL-3.0-or-later OR MIT OR Apache-2.0`. These imported policy copies use
the upstream `AGPL-3.0-or-later` option; its full text is preserved at
[`vendor/sources/cairn/LICENSES/AGPL-3.0-or-later.txt`](vendor/sources/cairn/LICENSES/AGPL-3.0-or-later.txt).
This is an upstream grant, not permission inferred from Octet's project
license. It does not relicense Cairn's CLI, exporter/toolchain, other policy
files, documentation, or independently licensed third-party material.
The revision records observed local source identity, not anonymous remote
availability, lifecycle approval, or release qualification.

## Fetched dependencies and references

Fetched dependencies retain their own licenses. For each binary release,
inventory the exact locked Rust and native/Nix dependencies and include their
required notices and corresponding source. A reference link or proof citation
alone is not a claim that its implementation is bundled or relicensed.

### Admitted source distributions

[`vendor/dependencies.ncl`](vendor/dependencies.ncl) owns the exact upstream
repositories, revisions, selected paths, licenses, archive identities, and
admission status. `tools/import-public-dependencies.py` creates and checks its
JSON projection, unmodified source selections, Cargo directory sources, and
checksums. Octet owns distribution intake; upstream projects retain source,
licensing, behavior, and claim authority. Admission is not release approval.

The selected sources include distinct Valence reviewed/historical snapshots,
Bounded Exec snapshots for Octet and Cairn, Nickel Export, Artifact, Kiln,
Mantle Build Contract, Cairn, and Subwayrat. Preserve the notices shipped with
each selection. The copied Cairn policy family above remains bound to `695124`;
it is not relabelled as the separate current provider candidate.

Subwayrat `rat-canvas` and `rat-nodegraph` at
`9d68c69f13ad5c4385e8c96ff727bd6a7a1f8dd2` inherit the original workspace's
`MIT` declaration. That exact tree has no standalone license text. Its source
selection remains unmodified. The complete upstream MIT copyright/permission
notice is supplied **separately**, from public commit
[`0244e5d36feb65a472d3b76765c546c8c318c250`](https://github.com/brittonr/subwayrat/blob/0244e5d36feb65a472d3b76765c546c8c318c250/LICENSE),
under `vendor/sources/subwayrat-license-notice/LICENSE`. The notice credits
Subwayrat contributors. This is not a claim that the later file existed in the
`9d68` archive, and it does not change either snapshot's license or source bytes.
The integrated Cairn output preserves the separately attributed notice under
`share/licenses/cairn/`. Maintainer distribution approval and release
qualification remain separate requirements.

The frozen `crates/valence-core/` compatibility fixture retains its existing
AGPL declaration. It is not the standalone Valence provider and does not
establish that provider's license. See its README and the release checklist.
