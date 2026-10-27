#!/usr/bin/env python3
"""Full pinned Draft 2020-12 schema, plus source/dispatch metadata invariants."""
from pathlib import Path
from copy import deepcopy
import json
from jsonschema import Draft202012Validator

root = Path(__file__).resolve().parents[1]
schema = json.loads((root / 'fixtures/beta/schemas/extension-v2.schema.json').read_text())
Draft202012Validator.check_schema(schema)
validate = Draft202012Validator(schema)
manifest = json.loads((root / 'manifest.json').read_text())
validate.validate(manifest)
own = json.loads((root / 'fixtures/own/package.json').read_text())
original = json.loads((root / 'fixtures/original/package.json').read_text())
assert manifest['id'] == 'com.tezbar.native.' + manifest['slug']
assert manifest['replaces'] == ['raycast.custom-wordle']
assert manifest['preferences'] == [] and 'preferences' not in own
assert original['author'] == 'pallav_agarwal' and 'contributors' not in original
assert 'author' not in manifest and original['author'] in manifest['description']
assert len(manifest['commands']) == len(own['commands']) == 1
for key in ['name', 'title', 'description', 'mode', 'arguments']:
    assert manifest['commands'][0][key] == own['commands'][0][key]
assert manifest['commands'][0]['runtime'] == 'rust'
assert manifest['verification'] == [
    {'platform': platform, 'state': 'not-run', 'note': 'Host registration/rebuild and real desktop UI, clipboard and browser QA not run.'}
    for platform in ['macOS', 'Windows']
]
negatives = []
for key in schema['required']:
    v = deepcopy(manifest); del v[key]; negatives.append(v)
for key, value in [('author','invented'),('schemaVersion',1),('runtime','javascript'),('id','raycast.custom-wordle'),('slug','BAD/SLUG'),('icon','assets/custom-wordle-icon.png'),('replaces',[]),('replaces',['custom-wordle']),('commands',[]),('verification',[]),('platforms',['macOS','macOS'])]:
    v = deepcopy(manifest); v[key] = value; negatives.append(v)
for key, value in [('runtime','native'),('platforms',[]),('name','')]:
    v = deepcopy(manifest); v['commands'][0][key] = value; negatives.append(v)
for key, value in [('platform','macos'),('state','tested')]:
    v = deepcopy(manifest); v['verification'][0][key] = value; negatives.append(v)
for i, value in enumerate(negatives):
    assert not validate.is_valid(value), f'negative {i} unexpectedly valid'
print(f'Full pinned Draft 2020-12 schema: pass; {len(negatives)} negative cases; source metadata invariants pass')
