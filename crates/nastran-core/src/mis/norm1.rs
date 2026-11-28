//! Port of `mis/norm1.f` — `SUBROUTINE NORM1(X,DIV)`.
//!
//! Fortran source (oracle, `vendor/nastran95/mis/norm1.f`):
//!
//! ```fortran
//!       SUBROUTINE NORM1 (X,DIV)
//!       DOUBLE PRECISION   X(1)      ,MAX      ,DIV
//!       COMMON   /INVPWX/  FILEK(7)
//!       EQUIVALENCE        (NCOL,FILEK(2))
//!       DATA IND1 /1/
//!       MAX = 0.D0
//!       DO 10 I=1,NCOL
//!       DIV = DABS( X(I) )
//!       IF( DIV .LE. MAX ) GO TO 10
//!       MAX = DIV
//!       IND = I
//!    10 CONTINUE
//!       IF( X(IND) .LT. 0.D0 ) IND = -IND
//!       I = IABS(IND1)
//!       XX = X(I)
//!       DIV = SIGN(1.,XX)*FLOAT(ISIGN(1,IND1))*MAX
//!       XX = DIV
//!       IND1 = IND*IFIX(SIGN(1.,XX))
//!       MAX = 1.D0/DIV
//!       DO 20 I=1,NCOL
//!       XI = X(I)*MAX
//!       IF (ABS(XI) .LT. 1.E-36) XI=0.
//!    20 X(I)= XI
//!       RETURN
//!       END
//! ```
//!
//! Integration shape: the Fortran reads `NCOL` out of `COMMON /INVPWX/` via
//! `EQUIVALENCE (NCOL,FILEK(2))`. That COMMON block is *not* ported as a
//! global here; `ncol` is an explicit argument. `IND1` is `DATA`-initialized
//! to 1 and persists across calls, changing the sign of the next call's
//! `DIV`, so it is threaded through as an explicit `&mut i32` owned by the
//! caller (no global static, no shared state crate).
//!
//! HAZARD — deliberately not "fixed": if every `X(I)` is 0.0, the Fortran
//! loop never assigns `IND` and then reads an undefined value in
//! `IF (X(IND) .LT. 0.D0)`. No defined behaviour is invented for that case
//! here; the port keeps the same shape (see the comment at the `ind`
//! declaration) and the parity driver only feeds non-zero vectors.

/// Normalize `x[0..ncol]` so its maximum-magnitude element becomes 1.0 and
/// return the divisor (`DIV` in the Fortran).
///
/// * `x` — the vector, normalized in place.
/// * `ncol` — `NCOL` from `COMMON /INVPWX/` (`FILEK(2)`), passed explicitly.
/// * `ind1` — the persistent `DATA IND1 /1/` state. It is read to pick the
///   element whose sign drives `DIV`, and written back so the *next* call
///   sees the carried-over sign.
///
/// Operand order, association and the `1.E-36` flush-to-zero are reproduced
/// exactly; no fast-math, FMA or algebraic rewriting is used.
///
/// PRECISION DETAIL (required for byte parity): in the Fortran only `X`,
/// `MAX` and `DIV` are `DOUBLE PRECISION`. `XX` and `XI` are *undeclared* and
/// therefore implicitly `REAL` (single precision). So `XX = X(I)` and
/// `XI = X(I)*MAX` round the double result to f32, the `ABS(XI) .LT. 1.E-36`
/// test is a single-precision comparison, and `X(I) = XI` widens the f32
/// value back to f64. The port keeps those roundings; dropping them changes
/// the low bits of every normalized element.
pub fn norm1(x: &mut [f64], ncol: usize, ind1: &mut i32) -> f64 {
    // MAX = 0.D0
    let mut max: f64 = 0.0;
    // HAZARD: in the Fortran, IND is left unassigned when every X(I) is 0.0
    // and the following `X(IND)` reads an undefined value. That undefined
    // behaviour is intentionally not given a defined meaning here; the
    // initializer only keeps the port well-formed for the non-zero inputs
    // the parity driver uses.
    let mut ind: i32 = 0;

    // DO 10 I=1,NCOL ... IF (DIV .LE. MAX) GO TO 10
    // Strictly-greater update, so ties keep the first index, as in Fortran.
    for i in 0..ncol {
        let div = x[i].abs();
        if div <= max {
            continue;
        }
        max = div;
        ind = (i + 1) as i32;
    }

    // IF (X(IND) .LT. 0.D0) IND = -IND
    if x[(ind - 1) as usize] < 0.0 {
        ind = -ind;
    }

    // I = IABS(IND1) ; XX = X(I)   -- XX is implicitly REAL(4).
    let i = ind1.unsigned_abs() as usize;
    let xx: f32 = x[i - 1] as f32;

    // DIV = SIGN(1.,XX)*FLOAT(ISIGN(1,IND1))*MAX
    // SIGN/ISIGN are single-precision/integer intrinsics: the two unit
    // factors are formed in f32 and multiplied there, then widened to f64
    // before scaling MAX, exactly as the Fortran expression is evaluated.
    let sign_xx: f32 = if xx < 0.0 { -1.0 } else { 1.0 };
    let sign_ind1: f32 = if *ind1 < 0 { -1.0 } else { 1.0 };
    let div: f64 = (sign_xx * sign_ind1) as f64 * max;

    // XX = DIV ; IND1 = IND*IFIX(SIGN(1.,XX))   -- XX is implicitly REAL(4).
    let xx: f32 = div as f32;
    let sign_div: f32 = if xx < 0.0 { -1.0 } else { 1.0 };
    *ind1 = ind * (sign_div as i32);

    // MAX = 1.D0/DIV
    let max = 1.0 / div;

    // DO 20 I=1,NCOL ... IF (ABS(XI) .LT. 1.E-36) XI=0.
    // XI is implicitly REAL(4): the double product is rounded to f32, the
    // flush test is done in f32 against the f32 literal 1.E-36, and the
    // surviving f32 value is widened back into the double array.
    let tiny: f32 = 1.0e-36f32;
    for i in 0..ncol {
        let mut xi: f32 = (x[i] * max) as f32;
        if xi.abs() < tiny {
            xi = 0.0;
        }
        x[i] = xi as f64;
    }

    div
}

#[cfg(test)]
mod tests {
    use super::norm1;

    /// Format a value exactly like Fortran `1P,1D24.16`: sign, one leading
    /// digit, '.', 16 digits, 'D', exponent sign, at least two exponent
    /// digits, right-justified in a 24-character field.
    fn d24_16(v: f64) -> String {
        let s = format!("{:.16E}", v);
        let (mantissa, exp) = s.split_once('E').expect("exponent in float format");
        let exp: i32 = exp.parse().expect("integer exponent");
        let sign = if exp < 0 { '-' } else { '+' };
        let body = format!("{}D{}{:02}", mantissa, sign, exp.abs());
        format!("{:>24}", body)
    }

    /// `FORMAT(A9,1P,1D24.16)` — label left-justified in 9 columns.
    fn line1(label: &str, v: f64) -> String {
        format!("{:<9}{}\n", label, d24_16(v))
    }

    /// `FORMAT(A9,1P,4D24.16)`.
    fn line4(label: &str, x: &[f64; 4]) -> String {
        format!(
            "{:<9}{}{}{}{}\n",
            label,
            d24_16(x[0]),
            d24_16(x[1]),
            d24_16(x[2]),
            d24_16(x[3]),
        )
    }

    /// The three fixed inputs of `parity/drivers/norm1.f`, in call order.
    /// `IND1` starts at 1 (the Fortran `DATA IND1 /1/`) and is carried over
    /// between calls, which flips the sign of `DIV` on calls 2 and 3.
    const CALLS: [[f64; 4]; 3] = [
        [2.0, -8.0, 4.0, 1.0],
        [-3.0, 1.5, 6.0, -0.5],
        [1.0, 1.0e-40, -2.0, 3.0],
    ];

    /// Emit the same fixed-format text as the Fortran driver.
    fn render() -> String {
        let mut out = String::new();
        let mut ind1: i32 = 1;
        for (n, case) in CALLS.iter().enumerate() {
            let mut x = *case;
            let div = norm1(&mut x, 4, &mut ind1);
            out.push_str(&line1(&format!("CALL{} DIV", n + 1), div));
            out.push_str(&line4(&format!("CALL{} X", n + 1), &x));
        }
        out
    }

    #[test]
    fn norm1_matches_fortran_golden() {
        let got = render();
        // Visible with `cargo test -- --nocapture` for the diff gate.
        print!("{}", got);
        let want = include_str!("../../../../parity/golden/norm1.txt");
        assert_eq!(got, want, "NORM1 output diverges from Fortran baseline");
    }

    #[test]
    fn ind1_carry_over_flips_div_sign() {
        // Call 1: IND1 = 1 -> XX = X(1) = 2.0 > 0, X(2) < 0 so IND = -2,
        // DIV = +8.0 and IND1 becomes -2.
        let mut x = [2.0f64, -8.0, 4.0, 1.0];
        let mut ind1: i32 = 1;
        let div = norm1(&mut x, 4, &mut ind1);
        assert_eq!(div, 8.0);
        assert_eq!(ind1, -2);
        assert_eq!(x, [0.25f64, -1.0, 0.5, 0.125]);

        // Call 2: IND1 = -2 -> XX = X(2) = 1.5 > 0, X(3) > 0 so IND = 3,
        // DIV = -6.0 and IND1 becomes -3.
        let mut x = [-3.0f64, 1.5, 6.0, -0.5];
        let div = norm1(&mut x, 4, &mut ind1);
        assert_eq!(div, -6.0);
        assert_eq!(ind1, -3);
    }

    #[test]
    fn subnormal_result_is_flushed_to_zero() {
        // 1.0D-40 / 3.0 is below 1.E-36 and must be flushed to exactly 0.0.
        let mut x = [1.0f64, 1.0e-40, -2.0, 3.0];
        let mut ind1: i32 = -3;
        let div = norm1(&mut x, 4, &mut ind1);
        assert_eq!(div, 3.0);
        assert_eq!(x[1], 0.0);
        assert_eq!(x[3], 1.0);
    }
}
