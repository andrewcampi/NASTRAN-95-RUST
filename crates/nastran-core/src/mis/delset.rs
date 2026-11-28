//! Port of `mis/delset.f` — set the `/GPTA1/` table entries for the nine
//! dummy element types.
//!
//! Literal transcription of the Fortran oracle:
//!
//! ```text
//!       SUBROUTINE DELSET
//!       INTEGER DUMTYP(9)
//!       COMMON/SYSTEM/ NSKIP(45), IDUM(9)
//!       COMMON/GPTA1 / NELEM, LAST, INCR, NE(1)
//!       DATA DUMTYP/53,54,55,56,57,58,59,60,61/
//!       DO 100 I = 1,9
//!       NGRIDS = IDUM(I) / 10000000
//!       NC = MOD( IDUM(I),10000000) / 10000
//!       NP = MOD( IDUM(I),10000) / 10
//!       ND = MOD(IDUM(I), 10)
//!       IZERO = (DUMTYP(I) - 1)*INCR
//!       NE(IZERO+6) = NC + NGRIDS + 2
//!       NE(IZERO+9) = NP +2
//!       IF(NP.EQ. 0) NE(IZERO+9) =  0
//!       NE(IZERO+10) = NGRIDS
//!       N = 5*NGRIDS + 3 + NP + NC
//!       NE(IZERO+12) = N
//!       NE(IZERO+15) = NGRIDS + 2
//!   100 CONTINUE
//!       RETURN
//!       END
//! ```
//!
//! The COMMON blocks are *not* ported as globals: `IDUM` is an explicit
//! `&[i32]` parameter, the `/GPTA1/` word array `NE` is an explicit
//! `&mut [i32]` parameter, and `INCR` is passed by value. `NELEM` and
//! `LAST` are not read or written by DELSET, so they are not parameters.
//!
//! All Fortran indices below are 1-based; [`at`] converts to a 0-based
//! slice index at the point of access so the transcription stays literal.

/// `DUMTYP` from the `DATA` statement: the nine dummy element type ids.
pub const DUMTYP: [i32; 9] = [53, 54, 55, 56, 57, 58, 59, 60, 61];

/// 0-based index of the 1-based Fortran element `i` of a table.
#[inline]
fn at(i: i32) -> usize {
    (i - 1) as usize
}

/// Populate the `/GPTA1/` entries for the nine dummy element types.
///
/// * `idum` is `/SYSTEM/`'s `IDUM(9)`, each word packing
///   `NGRIDS*10000000 + NC*10000 + NP*10 + ND`.
/// * `incr` is `/GPTA1/`'s `INCR`, the stride between element-type blocks.
/// * `ne` is `/GPTA1/`'s `NE` word array, 1-based in the oracle. The
///   routine writes `NE(IZERO+6)`, `NE(IZERO+9)`, `NE(IZERO+10)`,
///   `NE(IZERO+12)` and `NE(IZERO+15)` for
///   `IZERO = (DUMTYP(I)-1)*INCR`, `I = 1..9`; every other word is left
///   untouched.
///
/// `ND` is decoded but not used here (the oracle's comment notes it is
/// consumed by DS1/DS1A), so it is decoded into `_nd` to keep the
/// division/modulo chain literal without a dead-code warning.
pub fn delset(idum: &[i32], incr: i32, ne: &mut [i32]) {
    // DO 100 I = 1,9
    for i in 1..=9 {
        // NGRIDS = IDUM(I) / 10000000
        let ngrids = idum[at(i)] / 10_000_000;
        // NC = MOD( IDUM(I),10000000) / 10000
        let nc = (idum[at(i)]) % 10_000_000 / 10_000;
        // NP = MOD( IDUM(I),10000) / 10
        let np = (idum[at(i)]) % 10_000 / 10;
        // ND = MOD(IDUM(I), 10)
        let _nd = (idum[at(i)]) % 10;

        // IZERO = (DUMTYP(I) - 1)*INCR
        let izero = (DUMTYP[at(i)] - 1) * incr;

        // NE(IZERO+6) = NC + NGRIDS + 2
        ne[at(izero + 6)] = nc + ngrids + 2;
        // NE(IZERO+9) = NP +2
        ne[at(izero + 9)] = np + 2;
        // IF(NP.EQ. 0) NE(IZERO+9) =  0
        if np == 0 {
            ne[at(izero + 9)] = 0;
        }
        // NE(IZERO+10) = NGRIDS
        ne[at(izero + 10)] = ngrids;
        // N = 5*NGRIDS + 3 + NP + NC
        let n = 5 * ngrids + 3 + np + nc;
        // NE(IZERO+12) = N
        ne[at(izero + 12)] = n;
        // NE(IZERO+15) = NGRIDS + 2
        ne[at(izero + 15)] = ngrids + 2;
        // 100 CONTINUE
    }
}

#[cfg(test)]
mod tests {
    use super::delset;

    /// Emit exactly the text produced by `parity/drivers/delset.f`.
    ///
    /// The driver fixes `INCR = 1`, seeds `NE(1..96)` with the `-1`
    /// sentinel, sets `IDUM(1..9)` to a fixed mix of zero and non-zero
    /// `NGRIDS`/`NC`/`NP` (entries 2 and 3 are all zero, entries 4, 5 and
    /// 9 have `NP = 0`), calls `DELSET`, then prints `NE(1..96)` in fixed
    /// `I12` format.
    fn driver_output() -> String {
        // INCR  = 1
        let incr: i32 = 1;

        // Sentinel so that words DELSET does not write are identifiable.
        let mut ne = [-1_i32; 96];

        // IDUM(I) = NGRIDS*10000000 + NC*10000 + NP*10 + ND
        let idum: [i32; 9] = [
            10020034, // type 53: NGRIDS=1 NC=2 NP=3 ND=4
            0,        // type 54: NGRIDS=0 NC=0 NP=0 ND=0   (all zero)
            20000051, // type 55: NGRIDS=2 NC=0 NP=5 ND=1
            30007,    // type 56: NGRIDS=0 NC=3 NP=0 ND=7   (NP = 0)
            30010000, // type 57: NGRIDS=3 NC=1 NP=0 ND=0   (NP = 0)
            99,       // type 58: NGRIDS=0 NC=0 NP=9 ND=9
            10010011, // type 59: NGRIDS=1 NC=1 NP=1 ND=1
            90090099, // type 60: NGRIDS=9 NC=9 NP=9 ND=9
            50003,    // type 61: NGRIDS=0 NC=5 NP=0 ND=3   (NP = 0, last type)
        ];

        delset(&idum, incr, &mut ne);

        // DO 30 I = 1, 96 / WRITE(6,900) NE(I) / 900 FORMAT(I12)
        let mut out = String::new();
        for v in ne.iter() {
            out.push_str(&format!("{:>12}\n", v));
        }
        out
    }

    #[test]
    fn delset_parity() {
        let golden = include_str!("../../../../parity/golden/delset.txt");
        let got = driver_output();
        // Printed so the parity diff can be reproduced from the test run.
        print!("{}", got);
        assert_eq!(got, golden, "DELSET output diverges from Fortran golden");
    }
}
