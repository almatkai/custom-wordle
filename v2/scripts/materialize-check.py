#!/usr/bin/env python3
"""Cargo drops nested packages. Templates keep extracted validation self-contained."""
from pathlib import Path
root = Path(__file__).resolve().parents[1]
for relative in ['adapter-check/Cargo.toml', 'adapter-check/tauri-stub/Cargo.toml']:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes((root / relative.replace('adapter-check/', 'checks/', 1)).with_suffix('.toml.in').read_bytes())
fixture = root / 'checks/Cargo.lock.fixture'
if fixture.exists():
    (root / 'adapter-check/Cargo.lock').write_bytes(fixture.read_bytes())

# Registration overlay only. Keep pinned source fixtures byte-exact.
import json
out = root / 'adapter-check/generated'
out.mkdir(parents=True, exist_ok=True)
beta = root / 'fixtures/beta/src-tauri/src/native_core/extensions'
source = (beta / 'v2_runtime.rs').read_text()
changes = [
    ('"amphetamine" | "audio-device"', '"custom-wordle" | "amphetamine" | "audio-device"'),
    ('"amphetamine" => super::v2_amphetamine::commands(),', '"custom-wordle" => super::v2_custom_wordle::commands(),\n        "amphetamine" => super::v2_amphetamine::commands(),'),
    ('"amphetamine" => {', '"custom-wordle" => super::v2_custom_wordle::handle(ctx, command, operation, payload, state).await,\n        "amphetamine" => {'),
]
for before, after in changes:
    assert source.count(before) == 1, before
    source = source.replace(before, after, 1)
(out / 'v2_runtime.rs').write_text(source)
(out / 'v2_catalog.rs').write_bytes((beta / 'v2_catalog.rs').read_bytes())
catalog = json.loads((beta / 'v2_catalog.json').read_text())
catalog.append(json.loads((root / 'manifest.json').read_text()))
(out / 'v2_catalog.json').write_text(json.dumps(catalog, indent=2) + '\n')
