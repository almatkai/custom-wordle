#!/usr/bin/env python3
from pathlib import Path
import hashlib
import json
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
record = json.loads((root / 'provenance.json').read_text())
paths = []
for entry in record['files']:
    path = root / entry['path']
    data = path.read_bytes()
    assert len(data) == entry['bytes'], entry['path']
    assert hashlib.sha256(data).hexdigest() == entry['sha256'], entry['path']
    if 'copyOf' in entry:
        assert data == (root / entry['copyOf']).read_bytes()
    paths.append(str(path))
assert len(paths) == len(set(paths)), 'duplicate provenance path'
icon = (root / 'assets/custom-wordle-icon.png').read_bytes()
assert len(icon) == 19722
assert hashlib.sha256(icon).hexdigest() == 'd4033620dc53983df5721e784708f14c93d5fdc84ba64f882a14c71566278705'
assert icon == (root / 'fixtures/own/assets/custom-wordle-icon.png').read_bytes()
if '--git' in sys.argv:
    checkout = root.parent
    assert Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],cwd=checkout,text=True).strip()).resolve() == checkout.resolve()
    assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=checkout), 'unexpected staging'
    for relative in subprocess.check_output(['git','ls-tree','-r','--name-only',record['sourceSha']],cwd=checkout,text=True).splitlines():
        original = subprocess.check_output(['git','show',record['sourceSha']+':'+relative],cwd=checkout)
        current = (checkout / relative).read_bytes()
        if relative == 'README.md':
            assert current == original + b'\n\n[Native V2 Rust port](v2/README.md) (V1 remains available unchanged).\n'
        else:
            assert original == current, f'V1 modified: {relative}'
        assert original == (root / 'fixtures/own' / relative).read_bytes()
    for path in paths + [str(root / 'provenance.json')]:
        result = subprocess.run(['git','check-ignore','--no-index',path],cwd=checkout,capture_output=True,text=True)
        assert result.returncode == 1, f'Ignored provenance path: {path}: {result.stdout}'
    print('Git visibility audit: every provenance path visible; V1 byte-exact except README discovery link; index empty')
print(f'Provenance/actual licenses/icon: {len(paths)} file hashes pass')
