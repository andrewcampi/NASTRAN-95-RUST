//! Port of `mis/dnorm.f` — `SUBROUTINE DNORM(X,MAG)`.
//!
//! Fortran source (oracle, `vendor/nastran95/mis/dnorm.f`):
//!
//! ```fortran
//!       SUBROUTINE DNORM(X,MAG)
//!       DOUBLE PRECISION        X(3)     ,MAG      ,A
//!       MAG= 0.D0
//!       A= X(1)*X(1) + X(2)*X(2) +X(3)*X(3)
//!       IF (A .GT. 0.D0) MAG= DSQRT(A)
//!       IF(MAG .EQ. 0.0D0) RETURN
//!       X(1) = X(1) / MAG
//!       X(2) = X(2) / MAG
//!       X(3) = X(3) / MAG
//!       RETURN
//!       END
//! ```
//!
//! Two behaviours are preserved exactly:
//!   * `X` is normalized in place and the magnitude is returned;
//!   * for the zero vector (and for any input whose squared sum is not
//!     strictly positive, e.g. NaN) `MAG` is `0.0` and `X` is left
//!     completely untouched — no division, no NaN.
//!
//! Operand order and association are kept identical to the Fortran:
//! `(x1*x1 + x2*x2) + x3*x3`, and `f64::sqrt` matches `DSQRT`.

/// Normalize the 3-vector `x` in place and return its magnitude.
///
/// Mirrors `SUBROUTINE DNORM(X,MAG)`: the returned value is `MAG`.
pub fn dnorm(x: &mut [f64; 3]) -> f64 {
    let mut mag: f64 = 0.0;
    // A = X(1)*X(1) + X(2)*X(2) + X(3)*X(3)  (left-to-right association)
    let a: f64 = x[0] * x[0] + x[1] * x[1] + x[2] * x[2];
    if a > 0.0 {
        mag = a.sqrt();
    }
    // IF (MAG .EQ. 0.0D0) RETURN  -- zero vector: X untouched.
    if mag == 0.0 {
        return mag;
    }
    x[0] = x[0] / mag;
    x[1] = x[1] / mag;
    x[2] = x[2] / mag;
    mag
}

#[cfg(test)]
mod tests {
    use super::dnorm;

    /// Format a value exactly like Fortran `ES24.16E3`:
    /// sign, one leading digit, '.', 16 digits, 'E', exponent sign,
    /// 3 exponent digits — total field width 24.
    fn es24_16e3(v: f64) -> String {
        let s = format!("{:.16E}", v);
        let (mantissa, exp) = s.split_once('E').expect("exponent in float format");
        let exp: i32 = exp.parse().expect("integer exponent");
        let sign = if exp < 0 { '-' } else { '+' };
        // Fortran right-justifies the value in the 24-character field.
        format!("{:>24}", format!("{}{}{}{:03}", mantissa, 'E', sign, exp.abs()))
    }

    /// The fixed-input cases exercised by `parity/drivers/dnorm.f`.
    const CASES: [[f64; 3]; 4] = [
        [3.0, 4.0, 0.0],
        [-1.5, 2.25, -7.0],
        [0.6, 0.8, 0.0],
        [0.0, 0.0, 0.0],
    ];

    /// Emit the same fixed-format text as the Fortran driver.
    fn render() -> String {
        let mut out = String::new();
        for (i, case) in CASES.iter().enumerate() {
            let mut x = *case;
            let mag = dnorm(&mut x);
            out.push_str(&format!(
                "CASE {} MAG={} X={} {} {}\n",
                i + 1,
                es24_16e3(mag),
                es24_16e3(x[0]),
                es24_16e3(x[1]),
                es24_16e3(x[2]),
            ));
        }
        out
    }

    #[test]
    fn dnorm_matches_fortran_golden() {
        let got = render();
        // Visible with `cargo test -- --nocapture` for the diff gate.
        print!("{}", got);
        let want = include_str!("../../../../parity/golden/dnorm.txt");
        assert_eq!(got, want, "DNORM output diverges from Fortran baseline");
    }

    #[test]
    fn zero_vector_is_untouched() {
        let mut x = [0.0f64, 0.0, 0.0];
        let mag = dnorm(&mut x);
        assert_eq!(mag, 0.0);
        assert_eq!(x, [0.0f64, 0.0, 0.0]);
        assert!(x.iter().all(|v| !v.is_nan()));
    }

    #[test]
    fn general_vector_is_normalized_in_place() {
        let mut x = [3.0f64, 4.0, 0.0];
        let mag = dnorm(&mut x);
        assert_eq!(mag, 5.0);
        assert_eq!(x, [0.6f64, 0.8, 0.0]);
    }
}
