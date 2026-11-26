//! Port of `mis/smccrd.f` — vector-scaled accumulate.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE SMCCRD ( DTEMP, ZIL, ILIM, ZOL )
//!       DOUBLE PRECISION DTEMP( ILIM ), ZIL( ILIM ), ZOL
//!       DO 10 I = 1, ILIM
//!       DTEMP( I ) = DTEMP( I ) + ZIL( I ) * ZOL
//! 10    CONTINUE
//!       RETURN
//!       END
//! ```
//!
//! For `I = 1..ILIM` the routine performs `DTEMP(I) = DTEMP(I) + ZIL(I) * ZOL`
//! in double precision. Operand order and association are preserved exactly:
//! one multiply followed by one add per element, with no FMA contraction, no
//! reordering, no fast-math and no algebraic rewriting. The routine has no
//! `COMMON`, no `SAVE`, no I/O and no error traps; it touches only the first
//! `ILIM` elements of `DTEMP` and `ZIL`.

/// `CALL SMCCRD(DTEMP, ZIL, ILIM, ZOL)`.
///
/// Accumulates `DTEMP(I) = DTEMP(I) + ZIL(I) * ZOL` for `I = 1..ilim`.
///
/// `ilim` is the Fortran loop bound: it is clamped to the shorter of the two
/// slices so the Rust port can never index out of bounds, and a non-positive
/// `ilim` leaves both arrays untouched, exactly like the Fortran `DO` loop
/// with an empty trip count.
///
/// # Panics
///
/// Panics if `ilim` is positive and larger than the length of either slice;
/// the Fortran caller contract requires `DTEMP` and `ZIL` to be declared with
/// at least `ILIM` elements.
#[inline]
pub fn smccrd(dtemp: &mut [f64], zil: &[f64], ilim: usize, zol: f64) {
    assert!(
        ilim <= dtemp.len() && ilim <= zil.len(),
        "SMCCRD: ILIM={ilim} exceeds array length (DTEMP={}, ZIL={})",
        dtemp.len(),
        zil.len()
    );
    for i in 0..ilim {
        // Fortran: DTEMP(I) = DTEMP(I) + ZIL(I) * ZOL
        // Multiply first, then add; no FMA, no reassociation.
        dtemp[i] = dtemp[i] + zil[i] * zol;
    }
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

    fn line(tag: &str, d: &[f64; 4]) -> String {
        format!(
            "{:<5}{}{}{}{}",
            tag,
            es24_16e3(d[0]),
            es24_16e3(d[1]),
            es24_16e3(d[2]),
            es24_16e3(d[3])
        )
    }

    /// Emits the same fixed-format text as `parity/drivers/smccrd.f`.
    fn driver_output() -> String {
        let mut out = String::new();

        // CASE 1: plain integer-valued data, ZOL = 2.5.
        let mut dtemp = [1.0, 2.0, 3.0, 4.0];
        let zil = [1.0, 2.0, 3.0, 4.0];
        smccrd(&mut dtemp, &zil, 4, 2.5);
        out.push_str(&line("CASE1", &dtemp));
        out.push('\n');

        // CASE 2: negative and zero entries.
        let mut dtemp = [0.0, -2.5, 1.25, -0.0];
        let zil = [-3.0, 0.0, 4.5, -1.0];
        smccrd(&mut dtemp, &zil, 4, 2.5);
        out.push_str(&line("CASE2", &dtemp));
        out.push('\n');

        // CASE 3: inexact decimal data (rounding-sensitive).
        let mut dtemp = [0.1, 0.2, 0.3, 1.0];
        let zil = [0.7, 1.1, -0.3, 0.3333333333333333];
        smccrd(&mut dtemp, &zil, 4, 2.5);
        out.push_str(&line("CASE3", &dtemp));
        out.push('\n');

        // CASE 4: partial update, ILIM = 2 -- elements 3 and 4 untouched.
        let mut dtemp = [10.0, 20.0, 30.0, 40.0];
        let zil = [1.0, -1.0, 100.0, 100.0];
        smccrd(&mut dtemp, &zil, 2, 2.5);
        out.push_str(&line("CASE4", &dtemp));
        out.push('\n');

        // CASE 5: ILIM = 0 -- the loop body must not execute at all.
        let mut dtemp = [1.0, 2.0, 3.0, 4.0];
        let zil = [5.0, 6.0, 7.0, 8.0];
        smccrd(&mut dtemp, &zil, 0, 2.5);
        out.push_str(&line("CASE5", &dtemp));
        out.push('\n');

        out
    }

    #[test]
    fn smccrd_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("smccrd_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/smccrd.txt");
        assert_eq!(got, want, "SMCCRD output differs from Fortran golden");
    }

    #[test]
    fn smccrd_accumulates_in_place() {
        // Repeated calls accumulate, matching the Fortran in-place update.
        let mut dtemp = [1.0, 2.0, 3.0, 4.0];
        let zil = [1.0, 1.0, 1.0, 1.0];
        smccrd(&mut dtemp, &zil, 4, 0.5);
        smccrd(&mut dtemp, &zil, 4, 0.5);
        assert_eq!(dtemp, [2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn smccrd_respects_ilim() {
        // Only the first ILIM elements may change.
        let mut dtemp = [1.0, 2.0, 3.0, 4.0];
        let zil = [10.0, 10.0, 10.0, 10.0];
        smccrd(&mut dtemp, &zil, 1, 2.0);
        assert_eq!(dtemp, [21.0, 2.0, 3.0, 4.0]);

        // ILIM = 0 is a no-op.
        let mut dtemp = [1.0, 2.0, 3.0, 4.0];
        smccrd(&mut dtemp, &zil, 0, 2.0);
        assert_eq!(dtemp, [1.0, 2.0, 3.0, 4.0]);
    }
}
