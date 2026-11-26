//! Port of `mis/curvps.f` — principal stresses/strains and angle of maximum.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE CURVPS (SIGS, PRIN)
//! C*****
//! C  COMPUTES PRINCIPAL STRESSES OR STRAINS AND ANGLE OF MAXIMUM.
//! C*****
//!       REAL SIGS(3), PRIN(4)
//! C
//!       TEMP = SIGS(1) - SIGS(2)
//!       PRIN(4) = SQRT( (TEMP/2.0)**2 + SIGS(3)**2 )
//!       DELTA = ( SIGS(1) + SIGS(2) ) / 2.0
//!       PRIN(2) = DELTA + PRIN(4)
//!       PRIN(3) = DELTA - PRIN(4)
//!       DELTA = 2.0 * SIGS(3)
//!       IF( ABS(DELTA).LT.1.0E-15 .AND. ABS(TEMP).LT.1.0E-15 ) GO TO 100
//!       PRIN(1) = ATAN2( DELTA, TEMP )*28.6478898E0
//!       RETURN
//! C
//!   100 PRIN(1) = 0.0
//!       RETURN
//!       END
//! ```
//!
//! Everything is single precision (`REAL`), so the port uses `f32` throughout
//! and keeps the Fortran operand order and association verbatim: `TEMP/2.0`
//! before squaring, `(SIGS(1)+SIGS(2))/2.0` as a sum then a divide, and the
//! literal constant `28.6478898E0` (not `180/pi` recomputed). No FMA, no
//! fast-math, no `hypot`, no algebraic rewrites.
//!
//! Note that the Fortran reuses the local `DELTA` for `2.0*SIGS(3)` before the
//! degeneracy test, so the test compares `|2*SIGS(3)|` and `|TEMP|` against
//! `1.0E-15`; that reuse is preserved here.

/// `SUBROUTINE CURVPS(SIGS, PRIN)`.
///
/// `sigs` is the three-component stress/strain vector; `prin` receives
/// `PRIN(1)` = angle of maximum (degrees), `PRIN(2)`/`PRIN(3)` = maximum and
/// minimum principal values, `PRIN(4)` = radius of Mohr's circle.
#[inline]
pub fn curvps(sigs: &[f32; 3], prin: &mut [f32; 4]) {
    // TEMP = SIGS(1) - SIGS(2)
    let temp = sigs[0] - sigs[1];
    // PRIN(4) = SQRT( (TEMP/2.0)**2 + SIGS(3)**2 )
    let half_temp = temp / 2.0;
    prin[3] = (half_temp * half_temp + sigs[2] * sigs[2]).sqrt();
    // DELTA = ( SIGS(1) + SIGS(2) ) / 2.0
    let delta = (sigs[0] + sigs[1]) / 2.0;
    // PRIN(2) = DELTA + PRIN(4) ; PRIN(3) = DELTA - PRIN(4)
    prin[1] = delta + prin[3];
    prin[2] = delta - prin[3];
    // DELTA = 2.0 * SIGS(3)   (the Fortran reuses DELTA here)
    let delta = 2.0 * sigs[2];
    // IF( ABS(DELTA).LT.1.0E-15 .AND. ABS(TEMP).LT.1.0E-15 ) GO TO 100
    if delta.abs() < 1.0e-15_f32 && temp.abs() < 1.0e-15_f32 {
        // 100 PRIN(1) = 0.0
        prin[0] = 0.0;
        return;
    }
    // PRIN(1) = ATAN2( DELTA, TEMP )*28.6478898E0
    prin[0] = delta.atan2(temp) * 28.6478898_f32;
}

#[cfg(test)]
mod tests {
    use super::curvps;

    /// Reproduce Fortran `ES24.16E3` for a single-precision value: the value is
    /// widened to double (exactly, as gfortran does for a `REAL` list item),
    /// printed with 16 fractional digits and a 3-digit signed exponent, and
    /// right-justified in a 24-character field.
    fn es24_16e3(v: f32) -> String {
        let s = format!("{:.16E}", v as f64);
        let (mantissa, exp) = s.split_once('E').expect("exponent marker");
        let exp: i32 = exp.parse().expect("exponent value");
        format!("{:>24}", format!("{}E{:+04}", mantissa, exp))
    }

    /// Emit exactly the text produced by `parity/drivers/curvps.f`.
    fn driver_output() -> String {
        let cases: [(i32, [f32; 3]); 3] = [
            // Case 1: general biaxial state with shear.
            (1, [100.0, 50.0, 25.0]),
            // Case 2: negative components and negative shear.
            (2, [-30.0, 20.0, -12.5]),
            // Case 3: degenerate, takes the PRIN(1) = 0.0 branch.
            (3, [1.0, 1.0, 0.0]),
        ];

        let mut out = String::new();
        for (k, sigs) in cases {
            let mut prin = [0.0_f32; 4];
            curvps(&sigs, &mut prin);
            out.push_str(&format!("CASE {}\n", k));
            for (i, p) in prin.iter().enumerate() {
                out.push_str(&format!("PRIN({}) = {}\n", i + 1, es24_16e3(*p)));
            }
        }
        out
    }

    #[test]
    fn curvps_parity() {
        let golden = include_str!("../../../../parity/golden/curvps.txt");
        let got = driver_output();
        // Printed so the parity diff can be reproduced from the test run.
        print!("{}", got);
        assert_eq!(got, golden, "CURVPS output diverges from Fortran golden");
    }

    #[test]
    fn curvps_degenerate_branch_is_exactly_zero() {
        // SIGS(1) == SIGS(2) and SIGS(3) == 0 must take the PRIN(1) = 0.0 path.
        let mut prin = [1.0_f32; 4];
        curvps(&[1.0, 1.0, 0.0], &mut prin);
        assert_eq!(prin[0], 0.0);
        assert_eq!(prin[1], 1.0);
        assert_eq!(prin[2], 1.0);
        assert_eq!(prin[3], 0.0);
    }

    #[test]
    fn curvps_operand_order_is_literal() {
        // PRIN(4) must be SQRT((TEMP/2)**2 + SIGS(3)**2) with TEMP = SIGS(1)-SIGS(2),
        // and PRIN(1) must use the literal 28.6478898E0 constant.
        let sigs = [100.0_f32, 50.0, 25.0];
        let mut prin = [0.0_f32; 4];
        curvps(&sigs, &mut prin);
        let temp = sigs[0] - sigs[1];
        let half = temp / 2.0;
        assert_eq!(prin[3], (half * half + sigs[2] * sigs[2]).sqrt());
        let delta = (sigs[0] + sigs[1]) / 2.0;
        assert_eq!(prin[1], delta + prin[3]);
        assert_eq!(prin[2], delta - prin[3]);
        assert_eq!(prin[0], (2.0 * sigs[2]).atan2(temp) * 28.6478898_f32);
    }
}
