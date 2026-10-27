//! TEST ONLY: unchanged common.rs needs AppHandle/Emitter. No Tauri/OS event delivery.
#[derive(Clone)]
pub struct AppHandle;
pub trait Emitter {
    fn emit<T>(&self, channel: &str, payload: T) -> Result<(), String>;
}
impl Emitter for AppHandle {
    fn emit<T>(&self, _: &str, _: T) -> Result<(), String> {
        Err("Tauri emit unavailable in recording harness".into())
    }
}
