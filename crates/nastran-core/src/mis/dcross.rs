//! Port of `mis/dcross.f` — double-precision cross product.
//!
//! Literal transcription of the Fortran oracle:
//!
//! ```text
//!       SUBROUTINE DCROSS(X,Y,Z)
//!       DOUBLE PRECISION        X(3)     ,Y(3)     ,Z(3)
//!       Z(1) = X(2)*Y(3) - X(3)*Y(2)
//!       Z(2) = Y(1)*X(3) - Y(3)*X(1)
//!       Z(3) = X(1)*Y(2) - X(2)*Y(1)
//!       RETURN
//!       END
//! ```
//!
//! Operand order and association are preserved verbatim, including the
//! `Y(1)*X(3) - Y(3)*X(1)` form of `Z(2)`, so the port stays a literal
//! transcription rather than an algebraic rewrite. No FMA, no fast-math,
//! no reordering.

/// Double-precision cross product, `Z = X x Y`.
///
/// Mirrors `SUBROUTINE DCROSS(X,Y,Z)`: `x` and `y` are read, `z` is written.
#[inline]
pub fn dcross(x: &[f64; 3], y: &[f64; 3], z: &mut [f64; 3]) {
    z[0] = x[1] * y[2] - x[2] * y[1];
    z[1] = y[0] * x[2] - y[2] * x[0];
    z[2] = x[0] * y[1] - x[1] * y[0];
}

#[cfg(test)]
mod tests {
    use super::dcross;

    /// Reproduce Fortran `ES24.16E3`: 16 fractional digits, 3-digit signed
    /// exponent, right-justified in a 24-character field.
    fn es24_16e3(v: f64) -> String {
        let s = format!("{:.16E}", v);
        let (mantissa, exp) = s.split_once('E').expect("exponent marker");
        let exp: i32 = exp.parse().expect("exponent value");
        format!("{:>24}", format!("{}E{:+04}", mantissa, exp))
    }

    /// Emit exactly the text produced by `parity/drivers/dcross.f`.
    fn driver_output() -> String {
        let mut out = String::new();

        // Case 1: all positive, non-degenerate.
        let x = [1.0_f64, 2.0, 3.0];
        let y = [4.0_f64, 5.0, 6.0];
        let mut z = [0.0_f64; 3];
        dcross(&x, &y, &mut z);
        for (i, zi) in z.iter().enumerate() {
            out.push_str(&format!("Z({}) = {}\n", i + 1, es24_16e3(*zi)));
        }

        // Case 2: negative and zero components.
        let x = [-1.5_f64, 0.0, 2.25];
        let y = [3.0_f64, -4.0, 0.0];
        let mut z = [0.0_f64; 3];
        dcross(&x, &y, &mut z);
        for (i, zi) in z.iter().enumerate() {
            out.push_str(&format!("Z({}) = {}\n", i + 1, es24_16e3(*zi)));
        }

        out
    }

    #[test]
    fn dcross_matches_fortran_golden() {
        let golden = include_str!("../../../../parity/golden/dcross.txt");
        let got = driver_output();
        // Printed so the parity diff can be reproduced from the test run.
        print!("{}", got);
        assert_eq!(got, golden, "DCROSS output diverges from Fortran golden");
    }

    #[test]
    fn dcross_operand_order_is_literal() {
        // Z(2) must be Y(1)*X(3) - Y(3)*X(1), not the rewritten form.
        let x = [2.0_f64, 3.0, 5.0];
        let y = [7.0_f64, 11.0, 13.0];
        let mut z = [0.0_f64; 3];
        dcross(&x, &y, &mut z);
        assert_eq!(z[0], x[1] * y[2] - x[2] * y[1]);
        assert_eq!(z[1], y[0] * x[2] - y[2] * x[0]);
        assert_eq!(z[2], x[0] * y[1] - x[1] * y[0]);
    }
}
