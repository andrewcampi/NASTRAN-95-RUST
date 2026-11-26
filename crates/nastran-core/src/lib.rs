//! Ported NASTRAN routines live here, one module per Fortran entry point.
//!
//! Parity profile: `cargo build -p nastran-core --profile parity`.
//! The parity build deliberately uses opt-level 0, no fast-math, and no FMA
//! contraction so it matches the pinned Fortran baseline's arithmetic.

/// Placeholder until the first leaf module ticket lands.
pub fn abi_version() -> u32 {
    1
}
