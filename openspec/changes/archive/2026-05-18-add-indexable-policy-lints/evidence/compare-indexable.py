#!/usr/bin/env python3
import os
import pathlib
import re
import subprocess

slotcar = pathlib.Path('/home/brittonr/git/slotcar')
fork = pathlib.Path('/home/brittonr/git/pi-repos/indexable-inc--clippy')
upstream = pathlib.Path('/home/brittonr/git/pi-repos/rust-lang--rust-clippy')
expected = [
    'drop_must_use',
    'excessive_file_length',
    'fallible_int_fallback',
    'magic_number',
    'module_file_count',
    'non_trait_imports',
    'path_segment_repetition',
    'renamed_imports',
    'underscore_in_module_filename',
]


def git(repo, *args):
    return subprocess.check_output(['git', '-C', str(repo), *args], text=True).strip()


def clippy_lints(root):
    names = set()
    for path in (root / 'clippy_lints/src').rglob('*.rs'):
        text = path.read_text(errors='ignore')
        for macro in re.finditer(r'declare_clippy_lint!\s*\{(.*?)\n\}', text, re.S):
            match = re.search(r'pub\s+([A-Z0-9_]+)\s*,', macro.group(1))
            if match:
                names.add(match.group(1).lower())
    return names


def octet_inventory_at(ref):
    text = subprocess.check_output(
        ['git', '-C', str(slotcar), 'show', f'{ref}:src/lib.rs'],
        text=True,
    )
    return set(re.findall(r'^\s*"([^"]+)"\s*,\s*[A-Za-z_][A-Za-z0-9_:]*::[A-Z0-9_]+\s*=>', text, re.M))


def octet_inventory_current():
    text = (slotcar / 'src/lib.rs').read_text()
    return set(re.findall(r'^\s*"([^"]+)"\s*,\s*[A-Za-z_][A-Za-z0-9_:]*::[A-Z0-9_]+\s*=>', text, re.M))


baseline_ref = os.environ.get('OCTET_BASELINE_REF', 'b42259a7')
coverage_ref = os.environ.get('OCTET_COVERAGE_REF', '76915357')

fork_lints = clippy_lints(fork)
upstream_lints = clippy_lints(upstream)
fork_specific = sorted(fork_lints - upstream_lints)
octet_b422 = octet_inventory_at(baseline_ref)
octet_current = octet_inventory_at(coverage_ref)
covered = sorted(set(expected) & octet_current)
missing = sorted(set(expected) - octet_current)
preexisting = sorted(set(expected) & octet_b422)

print('# Indexable fork comparison evidence')
print()
print('Inputs:')
print(f'- indexable-inc/clippy: {git(fork, "rev-parse", "HEAD")}')
print(f'- rust-lang/rust-clippy comparison baseline: {git(upstream, "rev-parse", "HEAD")}')
print(f'- Octet pre-change baseline: {git(slotcar, "rev-parse", baseline_ref)}')
print(f'- Octet coverage target: {git(slotcar, "rev-parse", coverage_ref)}')
print()
print('Counts:')
print(f'- fork declared Clippy lints: {len(fork_lints)}')
print(f'- upstream declared Clippy lints: {len(upstream_lints)}')
print(f'- fork-specific lints: {len(fork_specific)}')
print()
print('Fork-specific lints:')
for name in fork_specific:
    print(f'- {name}')
print()
print('Octet b422 pre-existing matches:')
if preexisting:
    for name in preexisting:
        print(f'- {name}')
else:
    print('- none')
print()
print('Octet current coverage:')
for name in covered:
    print(f'- {name}')
print()
print('Missing from Octet current:')
if missing:
    for name in missing:
        print(f'- {name}')
else:
    print('- none')
print()
assert fork_specific == expected, f'fork-specific mismatch: {fork_specific!r}'
assert not preexisting, f'baseline already had expected lints: {preexisting!r}'
assert not missing, f'current Octet missing lints: {missing!r}'
print('ASSERTION: PASS - the change covers the nine fork-specific Indexable lints not already present as Octet lints.')
