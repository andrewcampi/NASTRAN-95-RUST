//! Port of `mis/sanorm.f` — `SUBROUTINE SANORM (*,A)`.
//!
//! Fortran source (oracle, `vendor/nastran95/mis/sanorm.f`):
//!
//! ```fortran
//!       SUBROUTINE SANORM (*,A)
//!       DIMENSION  A(3)
//! C
//! C     VECTOR NORMALIZATION AND VECTOR LENGTH
//! C
//!       XL=A(1)*A(1) + A(2)*A(2) + A(3)*A(3)
//!       IF (XL .LE. 0.0) RETURN 1
//!       XL   = SQRT(XL)
//!       A(1) = A(1)/XL
//!       A(2) = A(2)/XL
//!       A(3) = A(3)/XL
//!       RETURN
//!       END
//! ```
//!
//! TRAP (documented): the Fortran entry point takes an *alternate return*
//! (`SUBROUTINE SANORM (*,A)`), so the caller can distinguish "vector was
//! normalized" from "vector had zero length and was left alone" only through
//! the return selector. The Rust API therefore returns an explicit
//! [`SanormOutcome`] instead of a bare `()`; the two variants map 1:1 onto
//! `RETURN` (selector 0) and `RETURN 1` (selector 1).
//!
//! Arithmetic is kept in single precision (`REAL`, i.e. `f32`) with the exact
//! Fortran operand order and association `(a1*a1 + a2*a2) + a3*a3`, the exact
//! `XL .LE. 0.0` comparison (so NaN takes the normal path, as in Fortran), and
//! `f32::sqrt` for `SQRT`. No fast-math, no FMA, no `hypot`, no rewrites.

/// Which exit of `SANORM` was taken.
///
/// * [`SanormOutcome::Normalized`] — Fortran `RETURN` (selector 0): `A` was
///   divided by its length.
/// * [`SanormOutcome::ZeroLength`] — Fortran `RETURN 1` (selector 1): the
///   squared length was `<= 0.0` and `A` was left completely untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SanormOutcome {
    /// Normal return: `A(1..3)` were divided by `SQRT(XL)`.
    Normalized,
    /// Alternate return 1: `XL .LE. 0.0`, `A` untouched.
    ZeroLength,
}

impl SanormOutcome {
    /// The Fortran alternate-return selector value: `0` for the normal
    /// return, `1` for `RETURN 1`.
    pub fn selector(self) -> i32 {
        match self {
            SanormOutcome::Normalized => 0,
            SanormOutcome::ZeroLength => 1,
        }
    }
}

/// Normalize the 3-vector `a` in place, mirroring `SUBROUTINE SANORM (*,A)`.
///
/// Returns [`SanormOutcome::ZeroLength`] (Fortran `RETURN 1`) when
/// `XL = A(1)**2 + A(2)**2 + A(3)**2` is `<= 0.0`, leaving `a` untouched;
/// otherwise divides `a` by `SQRT(XL)` and returns
/// [`SanormOutcome::Normalized`] (Fortran `RETURN`).
pub fn sanorm(a: &mut [f32; 3]) -> SanormOutcome {
    // XL = A(1)*A(1) + A(2)*A(2) + A(3)*A(3)   (left-to-right association)
    let mut xl: f32 = a[0] * a[0] + a[1] * a[1] + a[2] * a[2];
    // IF (XL .LE. 0.0) RETURN 1   -- alternate return, A untouched.
    if xl <= 0.0 {
        return SanormOutcome::ZeroLength;
    }
    xl = xl.sqrt();
    a[0] = a[0] / xl;
    a[1] = a[1] / xl;
    a[2] = a[2] / xl;
    SanormOutcome::Normalized
}

#[cfg(test)]
mod tests {
    use super::{sanorm, SanormOutcome};

    /// Format a value exactly like Fortran `ES15.7` for a `REAL(4)`:
    /// sign, one leading digit, '.', 7 digits, 'E', exponent sign, 2 exponent
    /// digits — right-justified in a 15-character field.
    fn es15_7(v: f32) -> String {
        // gfortran widens the REAL(4) to double for the edit descriptor, so the
        // exact binary value is rounded to 8 significant digits.
        let s = format!("{:.7E}", v as f64);
        let (mantissa, exp) = s.split_once('E').expect("exponent in float format");
        let exp: i32 = exp.parse().expect("integer exponent");
        let sign = if exp < 0 { '-' } else { '+' };
        format!("{:>15}", format!("{}E{}{:02}", mantissa, sign, exp.abs()))
    }

    /// The fixed-input cases exercised by `parity/drivers/sanorm.f`:
    /// the normal return and the alternate return 1.
    const CASES: [[f32; 3]; 2] = [[3.0, 4.0, 0.0], [0.0, 0.0, 0.0]];

    /// Emit the same fixed-format text as the Fortran driver.
    fn render() -> String {
        let mut out = String::new();
        for (i, case) in CASES.iter().enumerate() {
            let mut a = *case;
            let outcome = sanorm(&mut a);
            out.push_str(&format!(
                "CASE {} ISEL={} A={} {} {}\n",
                i + 1,
                outcome.selector(),
                es15_7(a[0]),
                es15_7(a[1]),
                es15_7(a[2]),
            ));
        }
        out
    }

    #[test]
    fn sanorm_parity() {
        let got = render();
        // Visible with `cargo test -- --nocapture` for the diff gate.
        print!("{}", got);
        let want = include_str!("../../../../parity/golden/sanorm.txt");
        assert_eq!(got, want, "SANORM output diverges from Fortran baseline");
    }

    #[test]
    fn normal_return_normalizes_in_place() {
        let mut a = [3.0f32, 4.0, 0.0];
        assert_eq!(sanorm(&mut a), SanormOutcome::Normalized);
        assert_eq!(sanorm(&mut a).selector(), 0);
        // 3/5 and 4/5 in single precision, exactly as the Fortran oracle.
        assert_eq!(a, [0.6f32, 0.8, 0.0]);
    }

    #[test]
    fn alternate_return_leaves_vector_untouched() {
        let mut a = [0.0f32, 0.0, 0.0];
        assert_eq!(sanorm(&mut a), SanormOutcome::ZeroLength);
        assert_eq!(SanormOutcome::ZeroLength.selector(), 1);
        assert_eq!(a, [0.0f32, 0.0, 0.0]);
        assert!(a.iter().all(|v| !v.is_nan()));
    }

    #[test]
    fn negative_squared_length_takes_alternate_return() {
        // XL .LE. 0.0 is the exact Fortran test; a negative XL (only reachable
        // through overflow/NaN-free inputs in principle) also takes RETURN 1.
        let mut a = [0.0f32, -0.0, 0.0];
        assert_eq!(sanorm(&mut a), SanormOutcome::ZeroLength);
        assert_eq!(a, [0.0f32, -0.0, 0.0]);
    }
}
