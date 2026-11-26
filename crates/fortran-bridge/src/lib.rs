//! C ABI shims that let the Fortran driver call Rust replacements.
//!
//! Fortran symbols are lowercase with a trailing underscore. A ported routine
//! exports, for example:
//!
//! ```ignore
//! #[no_mangle]
//! pub unsafe extern "C" fn alg01_(...) { ... }
//! ```
//!
//! The mixed link step must exclude the original Fortran object for the ported
//! symbol so the Rust definition wins.

/// Version of the bridge ABI.
#[no_mangle]
pub extern "C" fn gaggle_bridge_abi() -> u32 {
    1
}
