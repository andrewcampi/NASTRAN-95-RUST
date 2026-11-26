fn main() {
    // The Rust driver arrives after the first leaf module is ported.
    println!(
        "nastran-rs skeleton (core ABI {})",
        nastran_core::abi_version()
    );
}
