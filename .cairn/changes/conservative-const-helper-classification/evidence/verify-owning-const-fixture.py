import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

root = Path('/home/brittonr/git/OnixResearch/octet')
driver = Path(sys.argv[1])
library = Path('/tmp/octet-const-owning-proof-20261016/debug/liboctet.so')
fixture = root / 'ui_tests/missing_const_fn.rs'
expected = (root / 'ui_tests/missing_const_fn.stderr').read_text()
assert driver.is_file(), driver
assert library.is_file(), library
host = next(line.removeprefix('host: ') for line in subprocess.check_output(['rustc', '-vV'], text=True).splitlines() if line.startswith('host: '))
with tempfile.TemporaryDirectory(prefix='octet-owning-const-fixture-') as directory:
    work = Path(directory)
    aux = work / 'aux'
    aux.mkdir()
    alias = work / f"liboctet@{os.environ['RUSTUP_TOOLCHAIN']}.so"
    alias.symlink_to(library)
    env = dict(os.environ, CLIPPY_DISABLE_DOCS_LINKS='true', DYLINT_LIBS=json.dumps([str(alias)]))
    command = [str(driver), str(fixture), '-L', str(work), f'--target={host}', '--error-format', 'human', '-C', 'prefer-dynamic', '-o', str(work / 'missing_const_fn'), '-A', 'unused', '--edition=2024', '--emit=metadata', '-Zui-testing', '-L', str(aux)]
    result = subprocess.run(command, cwd=root, env=env, text=True, capture_output=True)
    actual = result.stderr.replace('\r\n', '\n').replace(str(fixture), '$DIR/missing_const_fn.rs').replace(str(root), '$ROOT')
    print('[ui fixture] missing_const_fn')
    print('actual owning library:', library)
    print('owning driver:', driver)
    print('command:', json.dumps(command))
    print('exit:', result.returncode)
    print('[actual normalized snapshot]')
    print(actual)
    assert result.returncode == 0, result.stderr
    assert result.stdout == '', result.stdout
    assert actual.rstrip('\n') == expected.rstrip('\n'), 'existing snapshot mismatch'
    print('PASS: complete existing fixture matches unchanged checked-in snapshot')
