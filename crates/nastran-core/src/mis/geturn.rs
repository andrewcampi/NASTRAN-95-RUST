//! Port of `mds/geturn.f` — resolve a DMAP file name to a GINO unit and
//! refresh the DSIOF open-state.
//!
//! Literal transcription of the Fortran oracle:
//!
//! ```text
//!         SUBROUTINE GETURN ( NAMFIL )
//!       INCLUDE 'DSIOF.COM'
//!         COMMON / XFIST  / IFSTMX, IFSTCA, IFIST( 100 )
//!         COMMON / XFIAT  / IFATUF, IFATMX, IFATCA, IFIAT( 640 )
//!         COMMON / XXFIAT / IXFIAT( 19 )
//!         INTEGER*2         IUNIT
//!         COMMON / DSUNIT / IUNIT( 220 )
//!         DATA  MASK / '00007FFF'X /
//!         IF ( NAMFIL .EQ. LASNAM .AND. IFILEX .NE. 0 ) GO TO 20
//!         IFILEX = 0
//!         LIM = 2 * IFSTCA - 1
//!         DO 15 IFST = 1, LIM, 2
//!         IF ( NAMFIL .NE. IFIST( IFST ) ) GO TO 15
//!         IF ( NAMFIL .GE. 101 .AND. NAMFIL .LE. 320 ) GO TO 10
//!         IF ( IFIST( IFST + 1 ) .GT. 0 ) GO TO 5
//!         IFILEX = IXFIAT( IABS( IFIST( IFST+1 ) ) + 1 )
//!         IF (IFILEX .LE. MAXPRI) GO TO 20
//!    2    IFILEX = 0
//!         GO TO 200
//!    5    IFILEX = IAND( IFIAT( IFIST( IFST+1 ) - 2 ), MASK )
//!         GO TO 20
//!   10   IFILEX = IAND( IFIAT( IFIST( IFST+1 ) - 2 ), MASK )
//!         IF (IFILEX .GT. MAXPRI) GO TO 2
//!         IUNIT( NAMFIL-100 ) = IFILEX
//!         GO TO 20
//!   15   CONTINUE
//!         GO TO 200
//!   20   IPRVOP = FCB( 1, IFILEX )
//!         IF ( IPRVOP .EQ. 2 ) IPRVOP = 0
//!         NLR    = FCB( 3, IFILEX )
//!         NBLOCK = FCB( 4, IFILEX )
//!         LASNAM = NAMFIL
//!   200   CONTINUE
//!         RETURN
//!         END
//! ```
//!
//! The COMMON blocks are *not* ported as globals: the tables (`/XFIST/`,
//! `/XFIAT/`, `/XXFIAT/`, `/DSUNIT/`, `/FCB/`) and the mutable DSIOF fields
//! (`IFILEX`, `LASNAM`, `IPRVOP`, `NLR`, `NBLOCK`) are explicit parameters.
//! All Fortran indices below are 1-based; the helpers convert to 0-based
//! slices at the point of access so the transcription stays literal.

/// `MAXPRI` from `DSIOF.COM`: the largest legal GINO unit number.
pub const MAXPRI: i32 = 80;

/// `MASK` from the `DATA` statement: `'00007FFF'X`.
pub const MASK: i32 = 0x0000_7FFF;

/// Number of rows in the `/FCB/` table (`FCB(17,MAXFCB)`).
pub const FCB_ROWS: usize = 17;

/// 0-based index of the 1-based Fortran element `i` of a table.
#[inline]
fn at(i: i32) -> usize {
    (i - 1) as usize
}

/// `FCB(row, col)` of the column-major `/FCB/` table, 1-based in both
/// subscripts exactly as in the oracle.
#[inline]
fn fcb(fcb: &[i32], row: i32, col: i32) -> i32 {
    fcb[at(col) * FCB_ROWS + at(row)]
}

/// Resolve `namfil` to a GINO unit and refresh the DSIOF open-state.
///
/// * `ifstca` is `IFSTCA` from `/XFIST/`; the scan runs `IFST = 1, 2*IFSTCA-1, 2`.
/// * `ifist` is `/XFIST/`'s `IFIST(100)`, `ifiat` is `/XFIAT/`'s `IFIAT(640)`,
///   `ixfiat` is `/XXFIAT/`'s `IXFIAT(19)`, `iunit` is `/DSUNIT/`'s
///   `IUNIT(220)` (`INTEGER*2`), and `fcb_tbl` is the column-major
///   `/FCB/` table `FCB(17,MAXFCB)` flattened.
/// * `ifilex`, `lasnam`, `iprvop`, `nlr`, `nblock` are the mutable DSIOF
///   fields of the same name.
///
/// On a successful resolution `iprvop`/`nlr`/`nblock` are reloaded from
/// `FCB(1..4, IFILEX)` and `lasnam` is set to `namfil`; on the no-match and
/// `MAXPRI`-rejection paths the routine returns with `ifilex = 0` and the
/// other DSIOF fields untouched.
#[allow(clippy::too_many_arguments)]
pub fn geturn(
    namfil: i32,
    ifstca: i32,
    ifist: &[i32],
    ifiat: &[i32],
    ixfiat: &[i32],
    iunit: &mut [i16],
    fcb_tbl: &[i32],
    ifilex: &mut i32,
    lasnam: &mut i32,
    iprvop: &mut i32,
    nlr: &mut i32,
    nblock: &mut i32,
) {
    // IF ( NAMFIL .EQ. LASNAM .AND. IFILEX .NE. 0 ) GO TO 20
    let fast_path = namfil == *lasnam && *ifilex != 0;

    if !fast_path {
        // IFILEX = 0
        *ifilex = 0;

        // LIM = 2 * IFSTCA - 1
        let lim = 2 * ifstca - 1;

        // DO 15 IFST = 1, LIM, 2
        let mut found = false;
        let mut ifst = 1;
        while ifst <= lim {
            // IF ( NAMFIL .NE. IFIST( IFST ) ) GO TO 15
            if namfil == ifist[at(ifst)] {
                // IF ( NAMFIL .GE. 101 .AND. NAMFIL .LE. 320 ) GO TO 10
                if namfil >= 101 && namfil <= 320 {
                    // 10   IFILEX = IAND( IFIAT( IFIST( IFST+1 ) - 2 ), MASK )
                    *ifilex = ifiat[at(ifist[at(ifst + 1)] - 2)] & MASK;
                    // IF (IFILEX .GT. MAXPRI) GO TO 2
                    if *ifilex > MAXPRI {
                        // 2    IFILEX = 0
                        *ifilex = 0;
                        // GO TO 200
                        return;
                    }
                    // IUNIT( NAMFIL-100 ) = IFILEX
                    iunit[at(namfil - 100)] = *ifilex as i16;
                    // GO TO 20
                    found = true;
                    break;
                }

                // IF ( IFIST( IFST + 1 ) .GT. 0 ) GO TO 5
                if ifist[at(ifst + 1)] > 0 {
                    // 5    IFILEX = IAND( IFIAT( IFIST( IFST+1 ) - 2 ), MASK )
                    *ifilex = ifiat[at(ifist[at(ifst + 1)] - 2)] & MASK;
                    // GO TO 20
                    found = true;
                    break;
                }

                // IFILEX = IXFIAT( IABS( IFIST( IFST+1 ) ) + 1 )
                *ifilex = ixfiat[at(ifist[at(ifst + 1)].abs() + 1)];
                // IF (IFILEX .LE. MAXPRI) GO TO 20
                if *ifilex <= MAXPRI {
                    found = true;
                    break;
                }
                // 2    IFILEX = 0
                *ifilex = 0;
                // GO TO 200
                return;
            }
            // 15   CONTINUE
            ifst += 2;
        }

        // GO TO 200 (fell out of the scan without a match)
        if !found {
            return;
        }
    }

    // 20   IPRVOP = FCB( 1, IFILEX )
    *iprvop = fcb(fcb_tbl, 1, *ifilex);
    // IF ( IPRVOP .EQ. 2 ) IPRVOP = 0
    if *iprvop == 2 {
        *iprvop = 0;
    }
    // NLR    = FCB( 3, IFILEX )
    *nlr = fcb(fcb_tbl, 3, *ifilex);
    // NBLOCK = FCB( 4, IFILEX )
    *nblock = fcb(fcb_tbl, 4, *ifilex);
    // LASNAM = NAMFIL
    *lasnam = namfil;
}

#[cfg(test)]
mod tests {
    use super::{geturn, FCB_ROWS};

    /// `MAXFCB` from `DSIOF.COM`.
    const MAXFCB: usize = 89;

    /// Emit exactly the text produced by `parity/drivers/geturn.f`.
    ///
    /// The driver seeds `/XFIST/`, `/XFIAT/`, `/XXFIAT/`, `/DSUNIT/` and the
    /// `/FCB/` table with fixed values, then calls `GETURN` for six fixed
    /// `NAMFIL` values covering the fast path, branch (a), branch (b),
    /// branch (c), no-match and the `MAXPRI` rejection. After each call it
    /// prints `IFILEX`, `LASNAM`, `IPRVOP`, `NLR`, `NBLOCK` and `IUNIT(1..4)`
    /// in fixed `I12` format.
    fn driver_output() -> String {
        // ---- seed the FIST/FIAT tables with fixed values -----------------
        // IFSTCA = 6 name entries, so LIM = 2*IFSTCA-1 = 11 and the scan
        // visits IFST = 1,3,5,7,9,11.
        let ifstca: i32 = 6;
        let mut ifist = [0_i32; 100];
        // pair 1: name 101, value 5   -> branch (a), IFIAT(5-2)=IFIAT(3)
        ifist[0] = 101;
        ifist[1] = 5;
        // pair 2: name 205, value 9   -> branch (b), IFIAT(9-2)=IFIAT(7)
        ifist[2] = 205;
        ifist[3] = 9;
        // pair 3: name 400, value -3  -> branch (c), IXFIAT(IABS(-3)+1)=IXFIAT(4)
        ifist[4] = 400;
        ifist[5] = -3;
        // pair 4: name 104, value 13  -> branch (a), IFIAT(13-2)=IFIAT(11),
        //                                masked value exceeds MAXPRI
        ifist[6] = 104;
        ifist[7] = 13;
        // pair 5: name 500, value 1   -> branch (b), never matched below
        ifist[8] = 500;
        ifist[9] = 1;
        // pair 6: name 600, value -1  -> branch (c), never matched below
        ifist[10] = 600;
        ifist[11] = -1;

        let mut ifiat = [0_i32; 640];
        // high bits set on purpose: IAND with '00007FFF'X must clear them
        ifiat[2] = 65541;
        ifiat[6] = 131082;
        ifiat[10] = 255;

        let mut ixfiat = [0_i32; 19];
        ixfiat[3] = 12;

        // ---- seed the FCB so IPRVOP/NLR/NBLOCK are identifiable ----------
        let mut fcb_tbl = [0_i32; FCB_ROWS * MAXFCB];
        for j in 1..=MAXFCB as i32 {
            let col = (j - 1) as usize * FCB_ROWS;
            fcb_tbl[col] = 100 + j; // FCB(1,J)
            fcb_tbl[col + 1] = 0; // FCB(2,J)
            fcb_tbl[col + 2] = 200 + j; // FCB(3,J)
            fcb_tbl[col + 3] = 300 + j; // FCB(4,J)
        }
        // unit 10 has FCB(1,10) = 2, which GETURN must report as IPRVOP = 0
        fcb_tbl[9 * FCB_ROWS] = 2;

        let mut iunit = [0_i16; 220];

        // (TAG, NAMFIL, LASNAM seed, IFILEX seed) per case.
        let cases: [(&str, i32, i32, i32); 6] = [
            ("CASE1", 101, 101, 7), // fast path
            ("CASE2", 101, 0, 0),   // branch (a)
            ("CASE3", 205, 0, 0),   // branch (b)
            ("CASE4", 400, 0, 0),   // branch (c)
            ("CASE5", 999, 0, 0),   // no match
            ("CASE6", 104, 0, 0),   // MAXPRI rejection on branch (a)
        ];

        let mut out = String::new();
        for &(tag, namfil, lasnam_seed, ifilex_seed) in cases.iter() {
            // ---- RESET: restore the mutable DSIOF fields and IUNIT(1..4) --
            let mut ifilex = ifilex_seed;
            let mut lasnam = lasnam_seed;
            let mut iprvop = -1;
            let mut nlr = -1;
            let mut nblock = -1;
            for v in iunit.iter_mut().take(4) {
                *v = -1;
            }

            geturn(
                namfil,
                ifstca,
                &ifist,
                &ifiat,
                &ixfiat,
                &mut iunit,
                &fcb_tbl,
                &mut ifilex,
                &mut lasnam,
                &mut iprvop,
                &mut nlr,
                &mut nblock,
            );

            // ---- REPORT: TAG (A5) then the six fields plus IUNIT(1..4) ----
            out.push_str(&format!("{}\n", tag));
            for v in [ifilex, lasnam, iprvop, nlr, nblock] {
                out.push_str(&format!("{:>12}\n", v));
            }
            for j in 0..4 {
                out.push_str(&format!("{:>12}\n", iunit[j]));
            }
        }
        out
    }

    #[test]
    fn geturn_parity() {
        let golden = include_str!("../../../../parity/golden/geturn.txt");
        let got = driver_output();
        // Printed so the parity diff can be reproduced from the test run.
        print!("{}", got);
        assert_eq!(got, golden, "GETURN output diverges from Fortran golden");
    }
}
