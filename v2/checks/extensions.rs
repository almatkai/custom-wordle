use crate::native_core::common::{CoreContext, CoreResult};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

#[rustfmt::skip]
#[path = "../fixtures/beta/src-tauri/src/native_core/extensions/native.rs"]
mod native;
#[path = "../adapter.rs"]
mod v2_custom_wordle;
#[rustfmt::skip]
#[path = "../fixtures/beta/src-tauri/src/native_core/extensions/v2_sdk.rs"]
pub mod v2_sdk;

/// TEST ONLY install/registry seam. Reads disposable package.json; no app profile.
mod registry {
    use super::*;
    pub struct Installed;
    pub fn installed_command(ctx: &CoreContext, id: &str, command: &str) -> Option<Installed> {
        let path = ctx.paths.data_dir.join("package.json");
        if !path.exists() {
            return None;
        }
        let manifest: Value = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
        (manifest["id"] == id
            && manifest["commands"]
                .as_array()
                .is_some_and(|v| v.iter().any(|c| c["name"] == command)))
        .then_some(Installed)
    }
    pub fn preferences(_: &Installed, _: Option<&str>) -> CoreResult {
        Ok(json!({}))
    }
}

// Actual beta runtime with the three documented Custom Wordle registration additions.
// All unrelated host adapters are explicit unimplemented test stubs.
#[rustfmt::skip]
#[path = "../adapter-check/generated/v2_catalog.rs"]
mod v2_catalog;
#[rustfmt::skip]
#[path = "../adapter-check/generated/v2_runtime.rs"]
mod v2_runtime;
macro_rules! unrelated {
    ($($name:ident),* $(,)?) => {$(
        mod $name {
            use super::*;
            pub fn commands() -> &'static [&'static str] { &[] }
            pub async fn handle(_: &CoreContext, _: &str, _: &str, _: &Value, _: &mut Value) -> CoreResult {
                Err("Unrelated adapter unavailable in test harness".into())
            }
        }
    )*};
}
unrelated!(
    v2_amphetamine,
    v2_audio_device,
    v2_cleanshotx,
    v2_color_picker,
    v2_gif_search,
    v2_google_calendar,
    v2_google_search,
    v2_port_manager,
    v2_speedtest,
    v2_timers,
    v2_perplexity
);

pub fn coverage() -> Value {
    v2_catalog::manifest("custom-wordle").unwrap()
}
pub fn catalog_install(ctx: &CoreContext) -> CoreResult {
    v2_catalog::cli_install(ctx, "com.tezbar.native.custom-wordle")
}
pub fn catalog_installed(ctx: &CoreContext) -> bool {
    v2_catalog::installed(ctx, "com.tezbar.native.custom-wordle")
}
pub fn supports(id: &str, command: &str) -> bool {
    v2_runtime::supports(id, command)
}

#[derive(Clone, Default)]
pub struct Sessions {
    native: native::NativeSessions,
    files: Arc<Mutex<()>>,
}
impl Sessions {
    pub async fn call(&self, ctx: &CoreContext, channel: &str, body: Value) -> Value {
        self.native
            .handle(ctx, &self.files, channel, &[body])
            .await
            .expect("native handler")
            .expect("native result")
    }
    pub async fn clear(&self) {
        self.native.clear().await;
    }
    pub async fn clear_slug(&self) {
        self.native.clear_slug("custom-wordle").await;
    }
}

/// Direct production adapter entry for checkpoint, clone-discard and cancellation tests.
pub async fn adapter(
    ctx: &CoreContext,
    op: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    v2_custom_wordle::handle(ctx, "create-custom-wordle", op, payload, state).await
}
