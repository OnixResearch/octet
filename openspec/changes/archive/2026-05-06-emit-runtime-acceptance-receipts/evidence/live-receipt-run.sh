#!/usr/bin/env bash
set -euo pipefail
tmp=$(mktemp -d)
nickel export --format json standards/policies/registry.ncl > "$tmp/registry.json"
nickel export --format json standards/profiles/local.ncl > "$tmp/profile.json"
cargo run -p tigerstyle-standards -- export-artifacts \
  --registry "$tmp/registry.json" \
  --out-dir "$tmp/artifacts" \
  --profile-json "$tmp/profile.json" \
  --receipt-out "$tmp/receipt.json" \
  --verification-command "nickel export --format json standards/profiles/local.ncl"
python3 - "$tmp/profile.json" "$tmp/receipt.json" <<'PY'
import sys, json, pathlib
profile_path = pathlib.Path(sys.argv[1])
receipt_path = pathlib.Path(sys.argv[2])
receipt = json.loads(receipt_path.read_text())
assert receipt['schema_version'] == 'tigerstyle-acceptance-receipt/v1'
assert receipt['inputs'][0]['path'] == str(profile_path)
assert receipt['config_digest'] == receipt['config_digest_detail']['value']
assert receipt['inputs'][0]['blake3'] == receipt['config_digest']
assert receipt['config_digest_detail']['algorithm'] == 'blake3'
assert receipt['status'] == 'passed'
assert receipt['raw_log_path'] == ''
assert 'stdout' not in receipt
assert 'stderr' not in receipt
artifact_paths = {pathlib.Path(a['path']).name for a in receipt['artifacts']}
assert 'tigerstyle-artifact-manifest.json' in artifact_paths
assert 'tigerstyle-config-receipt-families.json' in artifact_paths
print('receipt assertions passed')
PY
cat > "$tmp/validate-receipt.ncl" <<NCL
let contracts = import "$(pwd)/standards/receipts/contracts.ncl" in
(import "$tmp/receipt.json") | contracts.AcceptanceReceipt
NCL
nickel export --format json "$tmp/validate-receipt.ncl" >/dev/null
