# Compiled-in integration against the pinned beta

This package does not modify the host, its dirty `fix/deep-search-state` work, any profile, or the publication ledger. Apply the following only during a separately reviewed host change. Keep V1 installed for fallback; no legacy-storage migration is asserted.

1. Vendor this `v2/` package at the stable host path `src-tauri/native-extensions/custom-wordle/` (exclude generated `target/`, `reports/`, `adapter-check/` and disposable `node_modules/`). Retain source, manifest, icon and all notices/provenance. Add to `src-tauri/Cargo.toml`:

   ```toml
   tezbar-custom-wordle-v2 = { path = "native-extensions/custom-wordle" }
   ```

   The host already supplies `serde_json`, `uuid` with `v4`, and the shared SDK. Resolve/review the host Cargo.lock separately; this package has standalone engine and contract locks with MSRV 1.85 gates.

2. In `src-tauri/src/native_core/extensions/mod.rs`, register the adapter as a sibling of `v2_sdk`:

   ```rust
   #[path = "../../../native-extensions/custom-wordle/adapter.rs"]
   mod v2_custom_wordle;
   ```

3. Add `"custom-wordle"` to the accepted slugs in `v2_runtime::slug`. Add the commands arm:

   ```rust
   "custom-wordle" => super::v2_custom_wordle::commands(),
   ```

   Add the handle arm:

   ```rust
   "custom-wordle" => {
       super::v2_custom_wordle::handle(ctx, command, operation, payload, state).await
   }
   ```

   Both `com.tezbar.native.custom-wordle` and `raycast.custom-wordle` must resolve to that compiled handler; unknown commands must remain unsupported. The source operation names are the actual beta IPC channels: `extension:run-command`, `extension:invoke-action`, `extension:search-text-changed`, `extension:refresh-session`, `extension:load-more`, `extension:dispose-session`. Launch reads `payload.argumentValues.word`; the host injects its preference snapshot at `payload.preferences` (the source declares none).

4. Append `manifest.json` as one object in the host `v2_catalog.json`. Keep the schema root free of unsupported `author`/custom credit fields; credit is in description/notices/provenance. Preserve the actual command title, description, arguments and source `mode: no-view`; the native result intentionally returns a Detail view to replace the toast primary action. Assert one total command, compiled `rust` coverage on declared macOS/Windows, correct aliases, and `not-run` verification on both. Do not report static metadata as executed coverage.

5. Copy the exact icon to `src/renderer/public/extensions/icons/custom-wordle.png` and verify its pinned hash. Rebuild the Rust/Tauri application and renderer, then install the canonical ID through the host's managed NativeV2 install flow. Its manifest/marker cannot supply code to an unrecompiled binary. The original Raycast package and source assets stay in place.

The contract harness runs the unchanged beta catalog code against an appended **test-only catalog data overlay**, and the actual runtime source with the three registration additions above. `scripts/materialize-check.py` asserts the pinned replacement sites before generating this overlay. The unrelated eleven adapters, Tauri AppHandle/Emitter, and registry/install lookup boundary are explicitly labeled test stubs. Real SDK, CoreContext, catalog file-writing/coverage logic and NativeSessions execute. No test stub belongs in production. The harness creates only temporary profiles.

Do not add window hiding to make the result resemble a toast. The pinned `window:hide` performs the hide and returns JSON null; strict effect acknowledgement must reject that response. A future adapter using it requires a reviewed host change returning an explicit object acknowledgement. Do not reinterpret arbitrary null as success. This adapter does not call hide/paste. For other adapters, `core:platform:paste-text [text]` already writes, hides, restores focus, waits and pastes; `core:platform:paste []` uses a prepared clipboard. These are not additional Custom Wordle requirements.

After a host rebuild, manually check launcher arguments, original icon beside V1, validation notices, initial clipboard copy, explicit browser open and deliberate retry on both OSes. No actual site play or site-content testing is needed. Record host rebuild and real QA evidence before changing either verification state.
