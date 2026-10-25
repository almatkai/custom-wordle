# Validation record

Executed on 2026-10-25 in the owned Custom Wordle checkout, branch `feat/native-v2-rust`, source HEAD `5c7c4792f534a45001e7b71b1b41ead825914f80`. No changes were staged, committed, pushed or published. The completed clone remains for parent review. Host and profile files were not modified.

Environment: aarch64 macOS, Rust stable **1.98.0** and **1.85.0**, Node **26.7.0**, Python **3.9.6**, locked TypeScript **5.9.3**. Both Rust toolchains have `x86_64-pc-windows-gnu` installed.

## Observed gates

- Engine: **4 tests pass** on stable and 1.85. Source lengths/key cycles, ECMAScript whitespace, Unicode/UTF-16, malformed argument types, resource limits and strict effect-ack matrices are covered.
- Production adapter + actual beta SDK/CoreContext/NativeSessions/catalog: **12 integration tests pass** on stable and 1.85. Tests cover copy-before-open, fixed targets, malformed/forged/stale/cross-session capabilities, discarded refresh/search clones, explicit retries, copy/open failure matrices, cancellation checkpoints, lifecycle/shutdown/uninstall, compiled catalog coverage, and actual SDK atomic storage/corruption/unknown-key behavior.
- Actual own TypeScript oracle: **74,623 exact parity cases pass**, executing the original own source through recording Raycast boundaries and the actual locked compiler. Coverage includes every single UTF-16 unit (including lone surrogates), trim-edge/mixed whitespace, ASCII words of varied lengths, special characters, malformed values, 2,000 supplementary scalars, 2,000 mixed UTF-16 sequences, 5,000 seeded words and a 1,048,576-unit valid word. **4 additional source effect-failure probes** record synchronous copy failure, unawaited copy rejection, toast failure and explicit open failure. **1 separate resource-boundary probe** documents source acceptance vs explicit native rejection at 1,048,577 units. No oracle algorithm is handwritten.
- Actual pinned renderer functions: **9 tests pass**. They execute Detail rendering callbacks, Surface/overlay/keyboard/IPC routing, actual List/Form parsing, draft reconciliation, complete submit maps and `{value}` callbacks. The actual adapter output also typechecks against pinned beta runtime types using the locked compiler and stock typings; an unsupported action-kind negative is rejected. React/DOM/IPC are labeled recording stubs; no desktop render or visual QA is inferred. The source itself has no Form/List UI to port.
- Full pinned JSON Schema Draft 2020-12 validation passes, plus **26 schema negatives** and source metadata/credit/platform invariants. This uses `jsonschema`, not a partial handwritten schema checker.
- `cargo fmt --check` and strict `cargo clippy --all-targets -- -D warnings` pass for engine and adapter harness on **both stable and 1.85**. Windows GNU **check and strict all-target Clippy pass on both toolchains**; Windows binaries are not linked or executed.
- Disposable own V1 fixture: **1 smoke test and `tsc --noEmit` pass** after locked `npm ci --ignore-scripts`. Original root source and lock remain unchanged.
- **60 provenance file hashes** pass, including actual license texts, locked compiler/typing libraries, byte-exact own/beta/original fixtures, and exact icon. Every provenance path is visible to Git; the index is empty. All 16 original tracked V1 files remain byte-exact except the permitted README discovery link.
- Cargo package verification and extracted **offline** engine/adapter tests, strict Clippy, provenance verification, 74,623-case source oracle and 9 renderer tests pass. Mandatory fixtures, templates and locked libraries are present; generated environments, node_modules, reports and targets are excluded. Nested Cargo manifests/lock are materialized from shipped templates because Cargo otherwise omits nested packages. Initial registry caches are a prerequisite, not bundled Rust dependencies.

## Reproduce in this checkout

Run from `v2/`:

```sh
python3 scripts/materialize-check.py
cargo +stable fetch --locked
cargo +stable fetch --locked --manifest-path adapter-check/Cargo.toml --target x86_64-pc-windows-gnu
npm ci --ignore-scripts --no-audit --no-fund --prefix fixtures/own
python3 -m venv reports/schema-env
reports/schema-env/bin/python -m pip install -r scripts/schema-requirements.txt
SCHEMA_PYTHON="$PWD/reports/schema-env/bin/python" python3 scripts/gates.py
python3 scripts/package-check.py
```

Install toolchains/components/Windows target if absent. Initial fetch/npm/pip commands need registry access; all Rust build/test/Clippy gates and extracted checks use `--locked --offline`. The selected TypeScript compiler/typings and own source are already included for the offline Node oracle/renderer checks; a separate Raycast runtime is not required. Schema validation needs the Python environment. Logs/reports are generated under ignored `reports/`.

`verify-provenance.py --git` is for this Git checkout; extracted verification omits `--git`. The package gate extracts under owned `reports/extracted/`, materializes independent check manifests, and executes the actual extracted files. It never reads sibling extensions or substitutes a host build from another tree.

## Coverage boundaries and remaining work

XERJ was used first with canonical root prefix `ax-277bc2cf62fe` and isolated workspace state. The initial index covered all 3 own code files; it skipped hidden/ignored/non-code items. A bounded final refresh aborted after 0.3 seconds: `v2/checks/Cargo.toml.in` was rejected because the frozen dataset cannot accept the new docs family/fields. V2 index coverage is therefore incomplete; no new prefix, shared schema change or global service restart was attempted. Exact `rg`, live file reads and byte/hash checks provide exhaustive checks for the selected files.

No OS effect was performed: no clipboard read/write, selected-text read, browser, window, focus, paste, real Tauri app or Wordle-site request. Only pinned GitHub/npm dependency retrieval and build-tool setup used the network. Real macOS/Windows launcher/UI/icon/clipboard/browser/focus QA, host registration/rebuild and managed host installation are **not-run**. Both manifest verification states remain `not-run`.

The harness compiles real pinned common/SDK/NativeSessions and catalog source. Runtime source has only the documented three Custom Wordle registration additions. Tauri, installation lookup, external effect router and unrelated adapters are explicit test seams. It does not claim full-app integration, real registry/profile compatibility or upstream unrelated adapter tests. The actual beta JSON transport rejects some malformed UTF-16 escapes before the adapter; pure-engine UTF-16 coverage does not change that host limitation. All unknown legacy storage stays untouched.

V2 deliberately waits for copy acknowledgement, replaces unsupported toast presentation with Detail/actions, adds an explicit copy/retry action, and imposes the documented resource bound. The actual beta `window:hide` null acknowledgement is not accepted; this adapter does not use that route. Native session registration/capacity/cancellation limits are inherited from the pinned host. Parent review/publication and real desktop QA remain separate.
