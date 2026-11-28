//! Port of `mis/xsave.f` — the SAVE DMAP instruction.
//!
//! The Fortran oracle is:
//!
//! ```text
//!       SUBROUTINE XSAVE
//! C     THE PURPOSE OF THIS ROUTINE IS TO PERFORM THE FUNCTIONS ASSIGNED
//! C     TO THE SAVE DMAP INSTRUCTION.
//! C
//!       COMMON/XVPS/ IVPS(1)
//!       COMMON/BLANK/ IPAR(1)
//!       COMMON /OSCENT/ IOSCR(7)
//! C     GET NUMBER OF PARAMETERS FROM OSCAR
//!       N = IOSCR(7)*2 + 6
//!       DO 20 I1 = 8,N,2
//! C     GET VPS POINTER AND POINTER TO VALUE IN BLANK COMMON.
//!       J = IOSCR(I1)
//!       K = IOSCR(I1+1)
//! C     GET LENGTH OF VALUE FROM VPS
//!       L = IVPS(J-1)
//! C     TRANSFER VALUE FROM BLANK COMMON TO VPS
//!       DO 10 I2 = 1,L
//!       IVPS(J) = IPAR(K)
//!       J = J + 1
//!    10 K = K + 1
//!    20 CONTINUE
//!       RETURN
//!       END
//! ```
//!
//! The three COMMON blocks are *not* ported as globals. The caller supplies
//! them as slices, in the same order the Fortran names them:
//!
//! * `ivps`  — `COMMON /XVPS/ IVPS`, the VPS array (read *and* written),
//! * `ipar`  — `COMMON /BLANK/ IPAR`, the BLANK COMMON payload (read only),
//! * `ioscr` — `COMMON /OSCENT/ IOSCR`, the OSCAR scratch area (read only).
//!
//! Every index in the oracle is 1-based, so each access below subtracts one.
//! The two details that are easy to get wrong and are reproduced verbatim:
//!
//! * the value length is read from `IVPS(J-1)`, i.e. the word *immediately
//!   before* the destination — the VPS stores a length word in front of every
//!   value, and
//! * the outer loop is `DO I1 = 8,N,2` with `N = IOSCR(7)*2 + 6`, so the
//!   pointer pairs live at `IOSCR(8),IOSCR(9)`, `IOSCR(10),IOSCR(11)`, ... and
//!   the loop bound is recomputed from the parameter count, not from the
//!   length of the `IOSCR` slice.
//!
//! `J` and `K` are incremented together inside the copy loop, so the source
//! and destination advance in lockstep. A zero-length value (`L = 0`) makes
//! the inner loop a no-op and leaves both pointers untouched for the next
//! parameter pair.
//!
//! Note that the oracle declares `IOSCR(7)` yet indexes it from 8 upwards; the
//! declaration is simply too small for the data it addresses. The port reads
//! `ioscr` at the indices the oracle actually uses and does not clamp them to
//! seven words.

/// `SUBROUTINE XSAVE` — copy saved DMAP parameter values from BLANK COMMON
/// into the VPS.
///
/// * `ivps`  — `COMMON /XVPS/ IVPS` (1-based in Fortran; written here),
/// * `ipar`  — `COMMON /BLANK/ IPAR` (1-based in Fortran; read only),
/// * `ioscr` — `COMMON /OSCENT/ IOSCR` (1-based in Fortran; read only).
///
/// The parameter count comes from `IOSCR(7)`; each parameter contributes a
/// `(VPS pointer, BLANK pointer)` pair at `IOSCR(8),IOSCR(9)`,
/// `IOSCR(10),IOSCR(11)`, ... The value length is `IVPS(J-1)` and that many
/// words are copied from `IPAR(K...)` to `IVPS(J...)`.
pub fn xsave(ivps: &mut [i32], ipar: &[i32], ioscr: &[i32]) {
    // GET NUMBER OF PARAMETERS FROM OSCAR
    // N = IOSCR(7)*2 + 6
    let n = ioscr[6] * 2 + 6;

    // DO 20 I1 = 8,N,2
    let mut i1 = 8;
    while i1 <= n {
        // GET VPS POINTER AND POINTER TO VALUE IN BLANK COMMON.
        // J = IOSCR(I1) ; K = IOSCR(I1+1)
        let mut j = ioscr[i1 as usize - 1];
        let mut k = ioscr[i1 as usize];

        // GET LENGTH OF VALUE FROM VPS
        // L = IVPS(J-1)
        let l = ivps[(j - 1) as usize - 1];

        // TRANSFER VALUE FROM BLANK COMMON TO VPS
        // DO 10 I2 = 1,L ... J = J + 1 ... 10 K = K + 1
        let mut i2 = 1;
        while i2 <= l {
            ivps[j as usize - 1] = ipar[k as usize - 1];
            j += 1;
            k += 1;
            i2 += 1;
        }

        i1 += 2;
    }
}

#[cfg(test)]
mod tests {
    use super::xsave;

    /// Emit exactly the text produced by `parity/drivers/xsave.f`.
    ///
    /// Three cases: a single-parameter save, a three-parameter save, and a
    /// zero-length value followed by a real one. `IVPS` is reset to the
    /// sentinel pattern `IVPS(I) = -I` before every case and `IVPS(1..24)` is
    /// printed afterwards in `12I12` format, so the output pins both the words
    /// `XSAVE` writes and the words it must leave alone.
    fn driver_output() -> String {
        // FIXED BLANK-COMMON PAYLOAD: IPAR(I) = 100 + I
        let mut ipar = [0_i32; 64];
        for (i, v) in ipar.iter_mut().enumerate() {
            *v = 100 + (i as i32) + 1;
        }

        let mut out = String::new();

        for case in 1..=3 {
            // RESET THE VPS WINDOW TO THE SENTINEL PATTERN IVPS(I) = -I
            let mut ivps = [0_i32; 64];
            for (i, v) in ivps.iter_mut().enumerate() {
                *v = -((i as i32) + 1);
            }

            // CLEAR THE OSCAR SCRATCH AREA
            let mut ioscr = [0_i32; 64];

            match case {
                // CASE 1: ONE PARAMETER. N = 1*2 + 6 = 8, so the only pair is
                // I1 = 8. J = 2, K = 1, L = IVPS(1) = 3.
                1 => {
                    ivps[0] = 3;
                    ioscr[6] = 1;
                    ioscr[7] = 2;
                    ioscr[8] = 1;
                }
                // CASE 2: THREE PARAMETERS. N = 3*2 + 6 = 12, so the pairs are
                // I1 = 8, 10, 12.
                2 => {
                    ivps[0] = 2;
                    ivps[3] = 3;
                    ivps[7] = 1;
                    ioscr[6] = 3;
                    ioscr[7] = 2;
                    ioscr[8] = 1;
                    ioscr[9] = 5;
                    ioscr[10] = 3;
                    ioscr[11] = 9;
                    ioscr[12] = 6;
                }
                // CASE 3: TWO PARAMETERS, the first with a zero-length value.
                // N = 2*2 + 6 = 10, so the pairs are I1 = 8, 10.
                _ => {
                    ivps[0] = 0;
                    ivps[3] = 2;
                    ioscr[6] = 2;
                    ioscr[7] = 2;
                    ioscr[8] = 1;
                    ioscr[9] = 5;
                    ioscr[10] = 3;
                }
            }

            xsave(&mut ivps, &ipar, &ioscr);

            // WRITE(6,900) K, IOSCR(7)  ->  'CASE ',I1,' NPARAM=',I2
            out.push_str(&format!("CASE {} NPARAM={:2}\n", case, ioscr[6]));

            // WRITE(6,910) (IVPS(I),I=1,24)  ->  12I12
            for row in 0..2 {
                for col in 0..12 {
                    out.push_str(&format!("{:12}", ivps[row * 12 + col]));
                }
                out.push('\n');
            }
        }

        out
    }

    #[test]
    fn xsave_parity() {
        let golden = include_str!("../../../../parity/golden/xsave.txt");
        let got = driver_output();
        // Printed so the parity diff can be reproduced from the test run.
        print!("{}", got);
        assert_eq!(got, golden, "XSAVE output diverges from Fortran golden");
    }

    #[test]
    fn xsave_zero_length_value_is_a_no_op() {
        // L = IVPS(J-1) = 0 makes the inner copy loop a no-op: the destination
        // words keep their sentinel values and the next pair still runs.
        let mut ivps = [0_i32; 16];
        for (i, v) in ivps.iter_mut().enumerate() {
            *v = -((i as i32) + 1);
        }
        let ipar = [7_i32; 16];
        let mut ioscr = [0_i32; 16];
        ivps[0] = 0; // length word for the value at IVPS(2)
        ivps[3] = 2; // length word for the value at IVPS(5)
        ioscr[6] = 2; // two parameters -> N = 10
        ioscr[7] = 2; // J
        ioscr[8] = 1; // K
        ioscr[9] = 5; // J
        ioscr[10] = 3; // K

        xsave(&mut ivps, &ipar, &ioscr);

        assert_eq!(ivps[1], -2, "IVPS(2) must be untouched when L = 0");
        assert_eq!(ivps[2], -3, "IVPS(3) must be untouched when L = 0");
        assert_eq!(ivps[4], 7, "IVPS(5) = IPAR(3)");
        assert_eq!(ivps[5], 7, "IVPS(6) = IPAR(4)");
        assert_eq!(ivps[6], -7, "IVPS(7) must be untouched");
    }

    #[test]
    fn xsave_loop_bound_comes_from_parameter_count() {
        // IOSCR(7) = 0 gives N = 6, so the stride-2 loop never runs even
        // though IOSCR holds a well-formed pair at 8/9.
        let mut ivps = [0_i32; 16];
        for (i, v) in ivps.iter_mut().enumerate() {
            *v = -((i as i32) + 1);
        }
        let ipar = [7_i32; 16];
        let mut ioscr = [0_i32; 16];
        ivps[0] = 3;
        ioscr[6] = 0;
        ioscr[7] = 2;
        ioscr[8] = 1;

        xsave(&mut ivps, &ipar, &ioscr);

        // IVPS(1) is the length word the driver seeded; the loop never ran, so
        // it still holds that seed and every other word keeps its sentinel.
        assert_eq!(ivps[0], 3, "IVPS(1) keeps the seeded length word");
        for (i, v) in ivps.iter().enumerate().skip(1) {
            assert_eq!(*v, -((i as i32) + 1), "IVPS({}) must be untouched", i + 1);
        }
    }
}
