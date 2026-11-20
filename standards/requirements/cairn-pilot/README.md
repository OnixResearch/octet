# Cairn pilot generated source record

`generated-input.json` is tool output, not authored text. It was written by this
command, run from the Octet checkout at revision
`d5b34c2d64b1227417a46df8ed8427f2d10ed3c8` (`/home/brittonr/wt/octet-rev`):

```text
python3 - <<'EOF'
import json, subprocess
rev = "/home/brittonr/wt/octet-rev"
job = json.load(open(".../job.json"))            # the producer job record
locators = [f"{job['source']['root']}/{p}" for p in job["source"]["paths"]]
b3 = lambda p: subprocess.run(["b3sum", p], cwd=rev, check=True, capture_output=True, text=True).stdout.split()[0]
# specifications: plain b3sum of the reviewed specification file
# entries: one source per file job.json declares, repo-relative at REV
EOF
```

`generated.ncl` imports that JSON and applies Cairn's `GeneratedSources`
contract; its values are never typed into a Nickel or JSON file by hand.
`nickel export --format json generated.ncl | jq -S .` writes `generated.json`.

## BLAKE3 values and the producer proof card

The realized producer output `P` (the directory
`/datapool/mantle-store/convergence-pilot-20261119/store/<root basename>`)
declares the same files twice: as the copied source tree `source/…` and as the
proof card's bound artifacts. Each plain `b3sum` above equals the value Octet's
proof card in `P/proof-cards.json` lists for the copied file:

| repo-relative locator at REV | plain BLAKE3 | proof card entry |
| --- | --- | --- |
| `standards/proof-rails/aeneas-lean/fixtures/external-consumer/src/lib.rs` | `691e8de8b8f92665dd00b938b48b398b451c71a06c99dae10b38b38184d4dddd` | `proofs/octet.aeneas_consumer_contract.external-fixture-rail-0.aeneas/source/src/lib.rs` |
| `standards/proof-rails/aeneas-lean/fixtures/external-consumer/Theorem.lean` | `776e9f85261080fbcfd75cb481c4f563cff50ffefaf8bdaae3d5fad010d8c77f` | `proofs/octet.aeneas_consumer_contract.external-fixture-rail-0.aeneas/theorem.lean` |
| `standards/proof-rails/aeneas-lean/fixtures/external-consumer/Cargo.toml` | `28826659c5095a872ca9b17c747b96be1bc3afc147f2f383a7df5a527ab04370` | not listed among the card's artifact identities; equals `P/source/Cargo.toml` |

The specification row is separate: `specifications[0].blake3` is the **plain**
BLAKE3 `7b6bbfdae779ada85f40f563c5d4db8c9afec9c00581f07794b4d907ec6f15c9` of
`.cairn/specs/aeneas-consumer-contract/spec.md`. The registry row's
`specification_blake3` `7529c7f5…` is Cairn's domain-separated identity and is
carried only on `targets.ncl`'s `specification`, where the projection checks it
against the row.

## External receipt

`standards/requirements/fixtures/cairn-pilot/external-receipt.json` is a
byte-copy of the realized output's external-consumer receipt:

- Source: `/datapool/mantle-store/convergence-pilot-20261119/store/zjq6whhf2c641vkz89kgwwpbajjq5cbq-octet-mantle-aeneas-consumer-job-v1/external-receipt.json`
- Plain BLAKE3: `84d2e77717013a203a4daf6ce649de8e501c7386f142b7abc66f32d08c33420e`

That digest equals the value the realized output's proof card records for
`proofs/octet.aeneas_consumer_contract.external-fixture-rail-0.receipt` in
`cards[0].artifact_blake3_identities` (and the matching provenance record). It
is a digest of the receipt file bytes, not the receipt's own
`consumer.receipt_identity`: that field, `b3:56284e96388b9ae3ca50f8afa0bffe91192662f5a4204d31b27d94d91ff1c139`,
is Octet's domain-separated identity over the receipt projection with its
identity field cleared. Verify the copy with
`b3sum standards/requirements/fixtures/cairn-pilot/external-receipt.json`;
`cargo octet requirements check` does not read this file, because it judges the
contract and evidence bundle only.
