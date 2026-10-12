pub fn run_version() {
    println!(
        "uhegit {} ({})",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION")
    );
    println!("Minimum supported Rust version: 1.75");
    println!("Built with Rust {}", rustc_version_runtime());
}

fn rustc_version_runtime() -> &'static str {
    // This is populated at build time via the RUSTC_VERSION env var if set,
    // otherwise falls back to the compile-time constant.
    option_env!("RUSTC_VERSION").unwrap_or("unknown")
}
