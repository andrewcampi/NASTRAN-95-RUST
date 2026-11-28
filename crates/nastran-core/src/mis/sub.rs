//! Port of `mis/sub.f` — `SUBROUTINE SUB(X,Y,A,B)`.
//!
//! Fortran source (oracle, `vendor/nastran95/mis/sub.f`):
//!
//! ```fortran
//!       SUBROUTINE SUB(X,Y,A,B)
//! C*******
//! C     SUB WILL FORM Y = A*X - B*Y  WHERE A AND B ARE SCALAR MULTIPLIERS
//! C     FOR THE VECTORS X AND Y
//! C*******
//!       DOUBLE PRECISION   X(1)      ,Y(1)     ,A        ,B
//!       COMMON   /INVPWX/  XX        ,NCOL
//!       DO 10 I = 1,NCOL
//!    10 Y(I) = X(I)*A - Y(I)*B
//!       RETURN
//!       END
//! ```
//!
//! Behaviour, reproduced exactly:
//!   * `Y(I) = X(I)*A - Y(I)*B` for `I = 1..NCOL`, in place in `Y`;
//!   * the loop is a Fortran `DO` with a positive stride, so `NCOL <= 0`
//!     executes the body zero times and `SUB` is a no-op;
//!   * all arithmetic is `DOUBLE PRECISION` (`f64`).
//!
//! Operand order and association are preserved exactly: each element is
//! `(X(I) * A) - (Y(I) * B)` — `X(I)*A` first, then `Y(I)*B`, then the
//! subtraction. There is no FMA contraction, no reassociation into
//! `A*X - B*Y` with a different grouping, and no algebraic rewrite (e.g.
//! `X*A + Y*(-B)`), all of which change the last bits.
//!
//! Integration shape: the Fortran reads `NCOL` from `COMMON /INVPWX/`
//! (`XX, NCOL`). The COMMON block is *not* ported as a global; `NCOL` is an
//! explicit parameter. `XX` is not read by `SUB`, so it has no counterpart
//! here. `X` is `&[f64]` and `Y` is `&mut [f64]`; the caller must supply at
//! least `NCOL` elements in each.
//!
//! Layout note for the Fortran driver: `SUB` has no `IMPLICIT NONE`, so its
//! `XX` is implicitly `REAL*4` and `NCOL` therefore sits at byte offset 4 of
//! `/INVPWX/`. `parity/drivers/sub.f` declares `REAL XX` to match; declaring
//! `DOUBLE PRECISION XX` there would put `NCOL` at offset 8 and `SUB` would
//! read the wrong word (silently making every call a no-op).

/// Forms `Y = A*X - B*Y` for `I = 1..ncol`, in place in `y`.
///
/// Mirrors `SUBROUTINE SUB(X,Y,A,B)` with `NCOL` passed explicitly instead of
/// read from `COMMON /INVPWX/`. `ncol <= 0` is a no-op, exactly like the
/// Fortran `DO 10 I = 1,NCOL` loop.
///
/// # Panics
///
/// Panics if `x` or `y` has fewer than `ncol` elements (the Fortran would
/// read/write out of bounds in that case).
#[inline]
pub fn sub(x: &[f64], y: &mut [f64], a: f64, b: f64, ncol: usize) {
    // DO 10 I = 1,NCOL
    for i in 0..ncol {
        // 10 Y(I) = X(I)*A - Y(I)*B
        // Operation order is fixed: X(I)*A, then Y(I)*B, then subtract.
        // `y[i]` is read before it is written, so the in-place update is
        // exactly the Fortran one.
        let xa = x[i] * a;
        let yb = y[i] * b;
        y[i] = xa - yb;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Formats one value exactly like Fortran `1P,D24.16`: sign, one digit,
    /// '.', 16 digits, 'D', exponent sign, 2 exponent digits, right-justified
    /// in a 24-column field.
    fn d24_16(v: f64) -> String {
        let s = format!("{:.16E}", v.abs());
        let (mantissa, exp) = s.split_once('E').expect("exponent in {:E} output");
        let exp: i32 = exp.parse().expect("integer exponent");
        let sign = if v.is_sign_negative() { '-' } else { ' ' };
        let exp_sign = if exp < 0 { '-' } else { '+' };
        format!(
            "{:>24}",
            format!("{}{}D{}{:02}", sign, mantissa, exp_sign, exp.abs())
        )
    }

    /// Formats one driver record: the tag right-justified in 6 columns (the
    /// Fortran `A` edit descriptor right-justifies character data), then
    /// `values` in `D24.16` fields (the driver's `FORMAT(A6,1P,4D24.16)`).
    fn line(tag: &str, values: &[f64]) -> String {
        let mut s = format!("{:>6}", tag);
        for v in values {
            s.push_str(&d24_16(*v));
        }
        s
    }

    /// Emits the same fixed-format text as `parity/drivers/sub.f`.
    fn driver_output() -> String {
        let a = 2.5;
        let b = -1.25;
        let mut out = String::new();

        // CASE 1: NCOL = 0 -- the loop body never runs, so SUB is a no-op.
        let x = [1.0, 2.0, 3.0, 4.0];
        let mut y = [10.0, 20.0, 30.0, 40.0];
        sub(&x, &mut y, a, b, 0);
        out.push_str(&line("CASE1", &y[..0]));
        out.push('\n');
        // Y(1..4) afterwards: untouched, proving the no-op.
        out.push_str(&line("CASE1U", &y[..4]));
        out.push('\n');

        // CASE 2: NCOL = 1 -- single element.
        let x = [0.1];
        let mut y = [3.7];
        sub(&x, &mut y, a, b, 1);
        out.push_str(&line("CASE2", &y[..1]));
        out.push('\n');

        // CASE 3: NCOL = 4 -- multi-element, negative B and a negative X.
        let x = [0.1, -2.3, 7.0, 1.0e-3];
        let mut y = [3.7, -0.25, 0.3333333333333333, 0.1428571428571429];
        sub(&x, &mut y, a, b, 4);
        out.push_str(&line("CASE3", &y[..4]));
        out.push('\n');

        out
    }

    #[test]
    fn sub_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("sub_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/sub.txt");
        assert_eq!(got, want, "SUB output differs from Fortran golden");
    }

    #[test]
    fn sub_ncol_zero_is_a_noop() {
        let x = [1.0, 2.0, 3.0];
        let mut y = [10.0, 20.0, 30.0];
        sub(&x, &mut y, 2.5, -1.25, 0);
        assert_eq!(y, [10.0, 20.0, 30.0]);
    }

    #[test]
    fn sub_operation_order_is_xa_minus_yb() {
        // Y = A*X - B*Y with A = 2.5, B = -1.25:
        //   Y(1) = 0.1*2.5 - 3.7*(-1.25)
        // The expected value is computed with the same grouping, so any
        // reassociation in the port would show up as a last-bit difference.
        let x = [0.1];
        let mut y = [3.7];
        sub(&x, &mut y, 2.5, -1.25, 1);
        let want = (0.1f64 * 2.5) - (3.7f64 * -1.25);
        assert_eq!(y[0].to_bits(), want.to_bits());
    }

    #[test]
    fn sub_does_not_contract_into_an_fma() {
        // Inputs chosen so that the fused form `X*A - Y*B` computed as
        // `fma(X, A, -(Y*B))` differs from the Fortran grouping in the last
        // bit. The port must match the non-fused grouping exactly.
        let x = [847.765824102157];
        let mut y = [-898.648476347353];
        sub(&x, &mut y, 2.5, -1.25, 1);
        let plain = (x[0] * 2.5) - (y_orig() * -1.25);
        let fused = x[0].mul_add(2.5, -(y_orig() * -1.25));
        assert_eq!(y[0].to_bits(), plain.to_bits(), "port must not use an FMA");
        assert_ne!(
            plain.to_bits(),
            fused.to_bits(),
            "inputs must be FMA-sensitive"
        );
    }

    /// The `Y` value used by [`tests::sub_does_not_contract_into_an_fma`].
    fn y_orig() -> f64 {
        -898.648476347353
    }

    #[test]
    fn sub_updates_in_place_for_every_element() {
        let x = [1.0, -2.0, 3.0];
        let mut y = [4.0, 5.0, -6.0];
        sub(&x, &mut y, 2.0, 3.0, 3);
        assert_eq!(
            y,
            [
                1.0 * 2.0 - 4.0 * 3.0,
                -2.0 * 2.0 - 5.0 * 3.0,
                3.0 * 2.0 - -6.0 * 3.0
            ]
        );
    }
}
