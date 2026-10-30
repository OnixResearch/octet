#!/usr/bin/env python3
"""Replay genuine provider artifacts; writes receipts outside the checkout."""
import json
import pathlib
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parents[4]
producer, wasm, destination = map(pathlib.Path, sys.argv[1:])
destination.mkdir(parents=True, exist_ok=True)
config = root / 'standards/wasm-artifact/generated/profiles.json'

def run(args, expected=0):
    args = list(map(str, args))
    result = subprocess.run(args, text=True, capture_output=True)
    print('$ ' + ' '.join(args), flush=True)
    print(result.stdout + result.stderr, end='', flush=True)
    print(f'exit={result.returncode}', flush=True)
    assert (result.returncode == 0) == (expected == 0), args
    return result

profiles = [('molten-abi', 'spacewasm-molten-abi-pilot'), ('host-segments', 'spacewasm-mvp-host-pilot')]
for fixture, profile in profiles:
    module = destination / (fixture + '.wasm')
    run([wasm, 'parse', root / f'standards/wasm-artifact/spacewasm/fixtures/positive/{fixture}.wat', '-o', module])
    receipt = destination / profile
    run([producer, 'evidence', 'collect', '--rail', 'wasm-artifact', '--input', module, '--config', config, '--profile', profile, '--artifact-dir', receipt])
    run([producer, 'artifact', 'verify', '--artifact-dir', receipt])
    data = json.loads((receipt / 'evidence-rail-receipt.json').read_text())
    details = data['wasm_artifact']
    assert data['schema_version'] == 'octet-evidence-rail-receipt/v1'
    assert details['target_runtime']['source_revision'] == 'e24cf09355a90497148eb5029fdb8e3400bd63e3'
    summary = {key: details[key] for key in ('profile_id', 'profile_identity', 'registry_identity', 'cohort_id', 'cohort_identity', 'exact_artifact_identity', 'interface_identity', 'target_runtime')}
    print(json.dumps({'receipt': str(receipt), 'schema': data['schema_version'], 'wasm_artifact': summary}, sort_keys=True), flush=True)

source = (root / 'standards/wasm-artifact/spacewasm/fixtures/positive/molten-abi.wat').read_text()
mutations = {
    'wrong-host-signature': source.replace('(param i32 i32)', '(param i32 i64)').replace('local.get $len call $assert', 'local.get $len i64.extend_i32_s call $assert'),
    'wrong-allocator-signature': source.replace('(param $len i32) (result i32)', '(param $len i32) (result i64)').replace('local.get $ptr)', 'local.get $ptr i64.extend_i32_u)'),
}
for name, text in mutations.items():
    wat = destination / (name + '.wat')
    module = destination / (name + '.wasm')
    wat.write_text(text)
    run([wasm, 'parse', wat, '-o', module])
    run([wasm, 'validate', module])
    run([producer, 'evidence', 'collect', '--rail', 'wasm-artifact', '--input', module, '--config', config, '--profile', 'spacewasm-molten-abi-pilot', '--artifact-dir', destination / name], 1)

source = (root / 'standards/wasm-artifact/spacewasm/fixtures/positive/host-segments.wat').read_text()
mutations = {
    'wrong-observe-signature': source.replace('(param i32)', '(param i64)').replace('call $observe', 'i64.extend_i32_s call $observe'),
    'immutable-total': source.replace('(mut i32)', 'i32').replace('global.set $total', 'drop'),
}
for name, text in mutations.items():
    wat = destination / (name + '.wat')
    module = destination / (name + '.wasm')
    wat.write_text(text)
    run([wasm, 'parse', wat, '-o', module])
    run([wasm, 'validate', module])
    run([producer, 'evidence', 'collect', '--rail', 'wasm-artifact', '--input', module, '--config', config, '--profile', 'spacewasm-mvp-host-pilot', '--artifact-dir', destination / name], 1)

module = destination / 'molten-abi.wasm'
registry = json.loads(config.read_text())
selected = next(profile for profile in registry['profiles'] if profile['profile_id'] == 'spacewasm-molten-abi-pilot')
oversized = destination / 'oversized.wasm'
with oversized.open('wb') as artifact:
    artifact.truncate(selected['resource_limits']['max_artifact_bytes'] + 1)
result = run([producer, 'evidence', 'collect', '--rail', 'wasm-artifact', '--input', oversized, '--config', config, '--profile', 'spacewasm-molten-abi-pilot', '--artifact-dir', destination / 'oversized'], 1)

for profile in registry['profiles']:
    if profile['profile_id'] == 'spacewasm-molten-abi-pilot':
        profile['target_runtime']['support_projection']['source_revision'] = 'f' * 40
stale = destination / 'stale-registry.json'
stale.write_text(json.dumps(registry))
run([producer, 'evidence', 'collect', '--rail', 'wasm-artifact', '--input', module, '--config', stale, '--profile', 'spacewasm-molten-abi-pilot', '--artifact-dir', destination / 'stale-cohort'], 1)
run([producer, 'evidence', 'collect', '--rail', 'wasm-artifact', '--input', module, '--config', config, '--profile', 'spacewasm-molten-abi-pilot', '--external-report', root / 'standards/wasm-artifact/spacewasm/fixtures/external/overclaim.json', '--artifact-dir', destination / 'forged-external'], 1)
print('two genuine pilot receipts verified; stale cohort and forged external evidence denied', flush=True)
