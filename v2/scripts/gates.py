#!/usr/bin/env python3
"""No OS effects. Initial dependency download is separate from locked offline gates."""
import os
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
def run(*args):
    print('+', ' '.join(map(str,args)), flush=True)
    subprocess.run(list(map(str,args)), cwd=root, check=True)

run(sys.executable, root / 'scripts/materialize-check.py')
run(sys.executable, root / 'scripts/verify-provenance.py', '--git')
for toolchain in ['stable','1.85.0']:
    for manifest in ['Cargo.toml','adapter-check/Cargo.toml']:
        run('cargo','+'+toolchain,'fmt','--manifest-path',manifest,'--check')
        run('cargo','+'+toolchain,'test','--manifest-path',manifest,'--locked','--offline')
        run('cargo','+'+toolchain,'clippy','--manifest-path',manifest,'--locked','--offline','--all-targets','--','-D','warnings')
        run('cargo','+'+toolchain,'check','--manifest-path',manifest,'--locked','--offline','--all-targets','--target','x86_64-pc-windows-gnu')
        run('cargo','+'+toolchain,'clippy','--manifest-path',manifest,'--locked','--offline','--all-targets','--target','x86_64-pc-windows-gnu','--','-D','warnings')
run('node',root / 'scripts/oracle.cjs')
run('node','--test',root / 'scripts/renderer.cjs')
run(os.environ.get('SCHEMA_PYTHON',sys.executable),root / 'scripts/schema.py')
run('npm','test','--prefix',root / 'fixtures/own')
run('npm','run','typecheck','--prefix',root / 'fixtures/own')
run(sys.executable,root / 'scripts/verify-provenance.py','--git')
