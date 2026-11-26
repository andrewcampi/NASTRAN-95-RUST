//! Port of `mis/daxb.f` — double precision cross product `C = A x B`.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE DAXB(A,B,C)
//!       DOUBLE PRECISION A(3), B(3), C(3), D(3)
//!       D(1) = A(2)*B(3) - A(3)*B(2)
//!       D(2) = A(3)*B(1) - A(1)*B(3)
//!       D(3) = A(1)*B(2) - A(2)*B(1)
//!       C(1) = D(1)
//!       C(2) = D(2)
//!       C(3) = D(3)
//!       RETURN
//!       END
//! ```
//!
//! Operand order and association are preserved exactly: each component is a
//! single subtraction of two products, with no FMA contraction, no reordering
//! and no algebraic rewriting.
//!
//! The Fortran computes all three components into the local `D` before storing
//! any of them to `C`, specifically so that `C` may overlap `A` or `B` in core.
//! That contract is preserved here: [`daxb`] computes the whole result into a
//! temporary before writing a single element of `C`, and the overlap helpers
//! ([`daxb_overlap_a`], [`daxb_overlap_b`]) model the aliasing call sites.

/// Computes `C = A x B`, writing all three components of `C`.
///
/// All three components are computed into a temporary before any element of
/// `C` is written, so a caller whose `C` aliases `A` or `B` cannot corrupt the
/// result. (Rust's borrow rules forbid expressing that aliasing through
/// `&`/`&mut` directly; use [`daxb_overlap_a`] / [`daxb_overlap_b`] for the
/// aliased call sites.)
#[inline]
pub fn daxb(a: &[f64; 3], b: &[f64; 3], c: &mut [f64; 3]) {
    // D(1..3) first, exactly as the Fortran local D.
    let d = daxb_cross(a, b);
    // Only now store into C.
    c[0] = d[0];
    c[1] = d[1];
    c[2] = d[2];
}

/// Pure form of `DAXB`: returns `A x B` without touching any caller storage.
#[inline]
pub fn daxb_cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    let d1 = a[1] * b[2] - a[2] * b[1];
    let d2 = a[2] * b[0] - a[0] * b[2];
    let d3 = a[0] * b[1] - a[1] * b[0];
    [d1, d2, d3]
}

/// `CALL DAXB(X, B, X)` — the same array passed as both `A` and `C`.
#[inline]
pub fn daxb_overlap_a(x: &mut [f64; 3], b: &[f64; 3]) {
    let d = daxb_cross(x, b);
    *x = d;
}

/// `CALL DAXB(A, Y, Y)` — the same array passed as both `B` and `C`.
#[inline]
pub fn daxb_overlap_b(a: &[f64; 3], y: &mut [f64; 3]) {
    let d = daxb_cross(a, y);
    *y = d;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Formats one value exactly like Fortran `ES24.16E3`:
    /// sign, one digit, '.', 16 digits, 'E', exponent sign, 3 exponent digits.
    fn es24_16e3(v: f64) -> String {
        let s = format!("{:.16E}", v.abs());
        let (mantissa, exp) = s.split_once('E').expect("exponent in {:E} output");
        let exp: i32 = exp.parse().expect("integer exponent");
        let sign = if v.is_sign_negative() { '-' } else { ' ' };
        let exp_sign = if exp < 0 { '-' } else { '+' };
        format!("{}{}E{}{:03}", sign, mantissa, exp_sign, exp.abs())
    }

    fn line(tag: &str, c: &[f64; 3]) -> String {
        format!(
            "{:<5}{}{}{}",
            tag,
            es24_16e3(c[0]),
            es24_16e3(c[1]),
            es24_16e3(c[2])
        )
    }

    /// Emits the same fixed-format text as `parity/drivers/daxb.f`.
    fn driver_output() -> String {
        let mut out = String::new();

        // CASE 1: plain integer-valued components.
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        let mut c = [0.0; 3];
        daxb(&a, &b, &mut c);
        out.push_str(&line("CASE1", &c));
        out.push('\n');

        // CASE 2: negative and zero components.
        let a = [0.0, -2.5, 1.25];
        let b = [-3.0, 0.0, 4.5];
        let mut c = [0.0; 3];
        daxb(&a, &b, &mut c);
        out.push_str(&line("CASE2", &c));
        out.push('\n');

        // CASE 3: documented overlap path -- same array as A and C.
        let mut x = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        daxb_overlap_a(&mut x, &b);
        out.push_str(&line("CASE3", &x));
        out.push('\n');

        // CASE 4: overlap path -- same array as B and C.
        let a = [1.0, 2.0, 3.0];
        let mut y = [4.0, 5.0, 6.0];
        daxb_overlap_b(&a, &mut y);
        out.push_str(&line("CASE4", &y));
        out.push('\n');

        // CASE 5: inexact decimal components (rounding-sensitive).
        let a = [0.1, 0.2, 0.3];
        let b = [0.7, 1.1, -0.3];
        let mut c = [0.0; 3];
        daxb(&a, &b, &mut c);
        out.push_str(&line("CASE5", &c));
        out.push('\n');

        out
    }

    #[test]
    fn daxb_matches_fortran_golden() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("daxb_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/daxb.txt");
        assert_eq!(got, want, "DAXB output differs from Fortran golden");
    }

    #[test]
    fn daxb_overlap_does_not_corrupt_result() {
        // C aliasing A: the result must equal the non-aliased computation.
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        let mut c = [0.0; 3];
        daxb(&a, &b, &mut c);
        let mut x = a;
        daxb_overlap_a(&mut x, &b);
        assert_eq!(x, c);

        // C aliasing B.
        let mut y = b;
        daxb_overlap_b(&a, &mut y);
        assert_eq!(y, c);
    }
}
