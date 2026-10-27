#!/usr/bin/env python3
"""Package, extract inside this owned checkout, then execute locked offline checks."""
from pathlib import Path
import subprocess
import sys
import tarfile
import json

root = Path(__file__).resolve().parents[1]
def run(*args, cwd=root):
    print('+', ' '.join(map(str,args)), flush=True)
    subprocess.run(list(map(str,args)),cwd=cwd,check=True)

listing = subprocess.check_output(['cargo','package','--allow-dirty','--locked','--offline','--list'],cwd=root,text=True).splitlines()
for name in listing:
    assert not any(part in name.split('/') for part in ['node_modules','reports','target']), name
required = ['adapter.rs','checks/Cargo.toml.in','checks/Cargo.lock.fixture','checks/tauri-stub/Cargo.toml.in','scripts/materialize-check.py']
required += [f['path'] for f in json.loads((root/'provenance.json').read_text())['files']]
assert set(required) <= set(listing), sorted(set(required)-set(listing))
run('cargo','package','--allow-dirty','--locked','--offline')
destination = root / 'reports/extracted'
destination.mkdir(parents=True,exist_ok=True)
with tarfile.open(root / 'target/package/tezbar-custom-wordle-v2-0.1.0.crate') as archive:
    for member in archive.getmembers():
        target=(destination/member.name).resolve()
        assert target.is_relative_to(destination.resolve()) and not member.issym() and not member.islnk()
    archive.extractall(destination)
extracted=destination/'tezbar-custom-wordle-v2-0.1.0'
run(sys.executable,extracted/'scripts/verify-provenance.py',cwd=extracted)
run(sys.executable,extracted/'scripts/materialize-check.py',cwd=extracted)
for manifest in ['Cargo.toml','adapter-check/Cargo.toml']:
    run('cargo','test','--manifest-path',manifest,'--offline','--locked',cwd=extracted)
    run('cargo','clippy','--manifest-path',manifest,'--offline','--locked','--all-targets','--','-D','warnings',cwd=extracted)
run('node',extracted/'scripts/oracle.cjs',cwd=extracted)
run('node','--test',extracted/'scripts/renderer.cjs',cwd=extracted)
print(f'Package/extracted offline checks passed: {len(listing)} files; every provenance fixture included')
