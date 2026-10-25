//! Validation harness: actual beta common/SDK/NativeSessions, recording OS router,
//! labeled installation/registry/dispatch seams. This is not the application binary.
// Compile fixtures in a normal library, so their unrelated internal #[cfg(test)]
// modules are not substituted/rewritten. Integration tests exercise actual methods.
#[cfg(not(test))]
#[allow(dead_code)]
#[path = "native_core.rs"]
pub mod native_core;
