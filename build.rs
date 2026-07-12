fn main() {
    #[cfg(feature = "napi")]
    napi_build::setup();

    // perry-ffi calls into symbols provided by perry-runtime at final
    // link time. Cargo still emits a cdylib for dual Node/Perry crates,
    // so allow unresolved symbols on that artifact. The staticlib (.a)
    // is what Perry actually links.
    #[cfg(feature = "perry")]
    {
        let target = std::env::var("TARGET").unwrap_or_default();
        if target.contains("apple") {
            println!("cargo:rustc-cdylib-link-arg=-Wl,-undefined,dynamic_lookup");
        } else if target.contains("windows") {
            // Windows is unsupported for NSM; no special handling.
        } else {
            println!("cargo:rustc-cdylib-link-arg=-Wl,--unresolved-symbols=ignore-all");
        }
    }
}
