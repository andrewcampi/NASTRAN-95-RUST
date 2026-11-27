//! Port of `mis/shstts.f` — shell stress-tensor transformation matrices.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE SHSTTS (TAB,UAB,VAB)
//! C
//! C     TO CREATE STRESS TENSOR TRANSFORMATION MATRICES FROM AN ORTHOGONAL
//! C     TRANSFORMATION FOR SHELL ELEMENTS.
//! C
//! C     INPUT :
//! C           TAB    - ORTHOGONAL INPLANE ROTATION TRANSFORMATION
//! C     OUTPUT:
//! C           UAB    - TENSOR TRANSFORMATION FOR NORMAL AND INPLANE SHEAR
//! C                    COMPONENTS
//! C           VAB    - TENSOR TRANSFORMATION FOR OUT-OF-PLANE SHEAR
//! C
//! C     USAGE:
//! C           THE INPUT IS ASSUMED TO BE ROW-LOADED.
//! C           OUTPUTS ARE CREATED ROW-LOADED.
//! ...
//!       REAL     TAB(9),UAB(9),VAB(4)
//! C
//!       UAB(1) = TAB(1)*TAB(1)
//!       UAB(2) = TAB(4)*TAB(4)
//!       UAB(3) = TAB(1)*TAB(4)
//!       UAB(4) = TAB(2)*TAB(2)
//!       UAB(5) = TAB(5)*TAB(5)
//!       UAB(6) = TAB(2)*TAB(5)
//!       UAB(7) = TAB(1)*TAB(2)*2.0
//!       UAB(8) = TAB(4)*TAB(5)*2.0
//!       UAB(9) = TAB(1)*TAB(5) + TAB(2)*TAB(4)
//! C
//!       VAB(1) = TAB(5)*TAB(9)
//!       VAB(2) = TAB(2)*TAB(9)
//!       VAB(3) = TAB(4)*TAB(9)
//!       VAB(4) = TAB(1)*TAB(9)
//! C
//!       RETURN
//!       END
//! ```
//!
//! `TAB` is a row-loaded 3x3 orthogonal inplane rotation: `TAB(1..3)` is row 1,
//! `TAB(4..6)` is row 2 and `TAB(7..9)` is row 3. The outputs are row-loaded
//! too. The routine is pure and argument-only — no `COMMON`, no saved state.
//!
//! The index pattern is transcribed literally, including the two places where
//! it is *not* the textbook tensor transform:
//!
//! * `UAB(2)` uses `TAB(4)*TAB(4)` and `UAB(4)` uses `TAB(2)*TAB(2)`, so the
//!   pattern is not symmetric in the way a naive re-derivation would suggest.
//! * `UAB(7)` and `UAB(8)` carry the literal `2.0` factor, and `UAB(9)` is a
//!   two-term sum `TAB(1)*TAB(5) + TAB(2)*TAB(4)`.
//! * All four `VAB` products carry a `TAB(9)` factor.
//!
//! Everything is single precision (`REAL`), so the port uses `f32` throughout
//! and keeps the Fortran operand order and association verbatim. No FMA, no
//! fast-math, no algebraic rewrites.

/// `SUBROUTINE SHSTTS(TAB,UAB,VAB)`.
///
/// `tab` is the row-loaded 3x3 orthogonal inplane rotation; `uab` receives the
/// 3x3 tensor transformation for the normal and inplane shear components and
/// `vab` the 2x2 tensor transformation for the out-of-plane shear, both
/// row-loaded.
#[inline]
pub fn shstts(tab: &[f32; 9], uab: &mut [f32; 9], vab: &mut [f32; 4]) {
    // UAB(1) = TAB(1)*TAB(1)
    uab[0] = tab[0] * tab[0];
    // UAB(2) = TAB(4)*TAB(4)
    uab[1] = tab[3] * tab[3];
    // UAB(3) = TAB(1)*TAB(4)
    uab[2] = tab[0] * tab[3];
    // UAB(4) = TAB(2)*TAB(2)
    uab[3] = tab[1] * tab[1];
    // UAB(5) = TAB(5)*TAB(5)
    uab[4] = tab[4] * tab[4];
    // UAB(6) = TAB(2)*TAB(5)
    uab[5] = tab[1] * tab[4];
    // UAB(7) = TAB(1)*TAB(2)*2.0
    uab[6] = tab[0] * tab[1] * 2.0;
    // UAB(8) = TAB(4)*TAB(5)*2.0
    uab[7] = tab[3] * tab[4] * 2.0;
    // UAB(9) = TAB(1)*TAB(5) + TAB(2)*TAB(4)
    uab[8] = tab[0] * tab[4] + tab[1] * tab[3];

    // VAB(1) = TAB(5)*TAB(9)
    vab[0] = tab[4] * tab[8];
    // VAB(2) = TAB(2)*TAB(9)
    vab[1] = tab[1] * tab[8];
    // VAB(3) = TAB(4)*TAB(9)
    vab[2] = tab[3] * tab[8];
    // VAB(4) = TAB(1)*TAB(9)
    vab[3] = tab[0] * tab[8];
}

#[cfg(test)]
mod tests {
    use super::shstts;

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

    /// Emit exactly the text produced by `parity/drivers/shstts.f`
    /// (`FORMAT(A5,9ES24.16E3)` and `FORMAT(A5,4ES24.16E3)`).
    fn driver_output() -> String {
        let mut out = String::new();

        fn case(out: &mut String, tag: &str, vtag: &str, tab: &[f32; 9]) {
            let mut uab = [0.0_f32; 9];
            let mut vab = [0.0_f32; 4];
            shstts(tab, &mut uab, &mut vab);
            out.push_str(&format!("{:>5}", tag));
            for v in uab.iter() {
                out.push_str(&es24_16e3(*v));
            }
            out.push('\n');
            out.push_str(&format!("{:>5}", vtag));
            for v in vab.iter() {
                out.push_str(&es24_16e3(*v));
            }
            out.push('\n');
        }

        // CASE 1: 30-degree inplane rotation, row-loaded, with TAB(9) = 0.5 so
        // the VAB products are exercised with a non-unit scale.
        let tab1 = [
            0.86602540_f32,
            0.5,
            0.0,
            -0.5,
            0.86602540,
            0.0,
            0.0,
            0.0,
            0.5,
        ];
        case(&mut out, "CASE1", "VAB1", &tab1);

        // CASE 2: identity rotation (TAB(9) = 1.0, zero inplane rotation).
        let tab2 = [
            1.0_f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0,
        ];
        case(&mut out, "CASE2", "VAB2", &tab2);

        out
    }

    #[test]
    fn shstts_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("shstts_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/shstts.txt");
        assert_eq!(got, want, "SHSTTS output differs from Fortran golden");
    }

    #[test]
    fn shstts_identity_is_identity() {
        // The identity rotation must map both matrices to the identity.
        let tab = [1.0_f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let mut uab = [0.0_f32; 9];
        let mut vab = [0.0_f32; 4];
        shstts(&tab, &mut uab, &mut vab);
        assert_eq!(uab, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
        assert_eq!(vab, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn shstts_uses_literal_tab_indices() {
        // Pin the non-symmetric index pattern: UAB(2) is TAB(4)^2 and UAB(4) is
        // TAB(2)^2, and UAB(7)/UAB(8) carry the literal 2.0 factor.
        let tab = [
            2.0_f32, 3.0, 5.0, 7.0, 11.0, 13.0, 17.0, 19.0, 23.0,
        ];
        let mut uab = [0.0_f32; 9];
        let mut vab = [0.0_f32; 4];
        shstts(&tab, &mut uab, &mut vab);
        assert_eq!(uab[0], 2.0 * 2.0);
        assert_eq!(uab[1], 7.0 * 7.0);
        assert_eq!(uab[2], 2.0 * 7.0);
        assert_eq!(uab[3], 3.0 * 3.0);
        assert_eq!(uab[4], 11.0 * 11.0);
        assert_eq!(uab[5], 3.0 * 11.0);
        assert_eq!(uab[6], 2.0 * 3.0 * 2.0);
        assert_eq!(uab[7], 7.0 * 11.0 * 2.0);
        assert_eq!(uab[8], 2.0 * 11.0 + 3.0 * 7.0);
        assert_eq!(vab[0], 11.0 * 23.0);
        assert_eq!(vab[1], 3.0 * 23.0);
        assert_eq!(vab[2], 7.0 * 23.0);
        assert_eq!(vab[3], 2.0 * 23.0);
    }
}
