#!/usr/bin/env python3
"""Exercise the frozen ordinary Octet owner job; preserve original owner evidence."""
import json
from pathlib import Path
import shutil
import subprocess
import time

BASE = Path('/home/brittonr/mantle-cairn-finish')
SOURCE = BASE / 'xrepo/octet-mantle-producer-20261020'
RUN = BASE / 'runs/octet-mantle-producer-acceptance-20261020-v10'
REVISION = '9237f931e8e65dd4254011839d5fd207f2ad85a5'
RUN.mkdir(exist_ok=False)
if subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=SOURCE, text=True).strip() != REVISION:
    raise SystemExit('producer source differs before immutable acceptance')
flake = 'git+file://' + str(SOURCE) + '?rev=' + REVISION
nix = Path(shutil.which('nix')).resolve(strict=True)
b3sum = Path(shutil.which('b3sum')).resolve(strict=True)
nickel = Path('/nix/store/ck3a4k9az87z5rjwgy8nmpcs34xp9cc5-nickel-lang-rev-fixup-1.17.0/bin/nickel').resolve(strict=True)
cairn = Path('/nix/store/ynjawpza9ci9i8ha7f3ywaqwbd60m83k-cairn-0.1.0/bin/cairn')
records = []

def capture(label, args, timeout=3600, expected=0):
    command = [str(arg) for arg in args]
    started = time.monotonic()
    out = RUN / (label + '.stdout')
    with out.open('xb') as stdout, (RUN / (label + '.stderr')).open('xb') as stderr:
        result = subprocess.run(command, cwd=SOURCE, stdout=stdout, stderr=stderr, timeout=timeout)
    row = dict(label=label, command=command, cwd=str(SOURCE), exit=result.returncode, expected=expected, elapsed_s=time.monotonic() - started)
    records.append(row)
    (RUN / 'results.json').write_text(json.dumps(records, indent=2) + '\n')
    print(json.dumps(row), flush=True)
    if result.returncode != expected:
        raise SystemExit("unexpected exit for " + label + ": " + str(result.returncode))
    return out

capture('job-contract', [nickel, 'export', '--format', 'json', 'standards/proof-rails/aeneas-lean/mantle-job.ncl'])
capture('registry-contract', [nickel, 'export', '--format', 'json', 'standards/policies/registry.ncl'])
capture('nix-export-check', [nix, 'build', '--no-link', '--no-write-lock-file', flake + '#checks.x86_64-linux.mantle-producer-export'])
capture('owner-policy-check', [nix, 'build', '--no-link', '--no-write-lock-file', flake + '#checks.x86_64-linux.policy-registry'])
capture('owner-consumer-contract-check', [nix, 'build', '--no-link', '--no-write-lock-file', flake + '#checks.x86_64-linux.aeneas-consumer-contract'])
capture('actual-source-export', [Path(shutil.which('python3')).resolve(), '-B', SOURCE / 'tools/mantle-producer/export.py', '--nix', nix, '--nickel', nickel, '--revision', REVISION, '--flake', flake, '--graph-format', 'nix-derivation-show-v4', '--out', RUN / 'export'])
for gate in ['proposal', 'design', 'tasks']:
    result = json.loads(capture('native-gate-' + gate, [cairn, 'gate', gate, 'publish-mantle-producer-handoff', '--root', '.']).read_text())
    if result.get('verdict') != 'PASS' or result.get('valid') is not True:
        raise SystemExit('native gate did not semantically pass: ' + gate)
capture('owner-job-build-plan', [nix, 'build', '--dry-run', '--no-link', '--no-write-lock-file', flake + '#packages.x86_64-linux.octet-mantle-aeneas-consumer-job'])
output = capture('actual-owner-job', [nix, 'build', '--no-link', '--print-out-paths', '--no-write-lock-file', '--keep-failed', '--max-jobs', '1', '--cores', '32', '-L', flake + '#packages.x86_64-linux.octet-mantle-aeneas-consumer-job'], timeout=42000).read_text().strip()
root = Path(output)
if not root.is_dir() or (root / 'source-revision').read_text().strip() != REVISION:
    raise SystemExit('actual output revision differs')
verification = json.loads((root / 'producer-artifact-verification.json').read_text())
receipt = json.loads((root / 'external-receipt.json').read_text())
if verification.get('schema_version') != 1 or verification.get('status') != 'valid' or verification.get('diagnostics') != []:
    raise SystemExit('owner artifact verification did not admit the output')
if receipt.get('schema_version') != 'octet-formal-proof-envelope/v1' or receipt.get('status') != 'passed' or receipt.get('consumer', {}).get('outcome') != 'supported' or receipt['consumer'].get('verification_role') != 'property':
    raise SystemExit('owner proof receipt was not externally supported')
if (RUN / 'export/job.json').read_bytes() != (root / 'job.json').read_bytes():
    raise SystemExit('exported and executed job bytes differ')
if (root / 'consumer-profile.json').read_bytes() != (root / 'external-profile.json').read_bytes():
    raise SystemExit('external profile differs from owner profile')
if (root / 'external-receipt.json').read_bytes() != (root / 'artifacts/proofs/octet.aeneas_consumer_contract.external-fixture-rail-0.receipt').read_bytes():
    raise SystemExit('external receipt differs from original emitted envelope')
runner_root = Path(capture('owner-verifier-binary', [nix, 'build', '--no-link', '--print-out-paths', '--no-write-lock-file', flake + '#cargo-octet']).read_text().strip())
runner = runner_root / 'bin/cargo-octet'
positive = json.loads(capture('verify-real-proof-payload', [runner, 'artifact', 'verify', '--artifact-dir', root / 'artifacts', '--format', 'json']).read_text())
if positive.get('status') != 'valid' or positive.get('diagnostics') != []:
    raise SystemExit('original emitted artifact payload failed independent owner verification')
negative = RUN / 'tampered-artifacts'
shutil.copytree(root / 'artifacts', negative)
lean = negative / 'proofs/aeneas/Llbc.lean'
lean.chmod(lean.stat().st_mode | 0o200)
with lean.open('ab') as changed:
    changed.write(b'\n-- tampered after real producer execution\n')
rejected = json.loads(capture('reject-tampered-real-proof', [runner, 'artifact', 'verify', '--artifact-dir', negative, '--format', 'json'], expected=1).read_text())
if rejected.get('status') != 'invalid' or not rejected.get('diagnostics'):
    raise SystemExit('tampered real proof payload was not rejected')
incompatible = RUN / 'incompatible-wire'
capture('reject-incompatible-original-wire', [Path(shutil.which('python3')).resolve(), '-B', SOURCE / 'tools/mantle-producer/export.py', '--nix', nix, '--nickel', nickel, '--revision', REVISION, '--flake', flake, '--graph-format', 'nix-derivation-show-legacy', '--out', incompatible], expected=1)
if incompatible.exists() or 'exported graph format differs from explicit consumer contract' not in (RUN / 'reject-incompatible-original-wire.stderr').read_text():
    raise SystemExit('incompatible original wire did not fail closed before destination creation')
paths = ['flake.nix', 'standards/policies/registry.ncl', 'docs/aeneas-consumer-contract.md', 'nix/mantle-aeneas-job.nix', 'standards/proof-rails/aeneas-lean/mantle-job.ncl', 'tools/mantle-producer/export.py', 'tools/mantle-producer/test_export.py']
identities = [dict(path=p, blake3=subprocess.check_output([b3sum, '--no-names', SOURCE / p], text=True).strip()) for p in paths]
artifacts = [dict(path=name, blake3=subprocess.check_output([b3sum, '--no-names', root / name], text=True).strip()) for name in ['producer-prove.json', 'producer-artifact-verification.json', 'external-receipt.json', 'external-profile.json', 'job.json', 'valence-provenance.json']]
(RUN / 'complete.json').write_text(json.dumps(dict(revision=REVISION, source=str(SOURCE), source_blake3=identities, records=records, export=str(RUN / 'export'), actual_owner_output=str(root), preserved_artifacts=artifacts, owner_job_executed=True, external_proof_accepted=True, mantle_acceptance=False, release_authority=False), indent=2) + '\n')
