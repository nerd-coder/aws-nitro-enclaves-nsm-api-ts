#![deny(clippy::all)]

// N-API exports are registered via `#[napi]` inventory, not direct Rust
// call sites — clippy's dead_code lint can't see that registration.
#[cfg(feature = "napi")]
#[allow(dead_code)]
mod napi_api;

#[cfg(feature = "perry")]
mod perry_api;
