//! Port of `mds/dbmlbk.f` — last block number allocated to a unit.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE DBMLBK ( LASBLK )
//! C
//! C THIS SUBROUTINE WILL RETURN THE LAST BLOCK NUMBER ALLOCATED TO THE
//! C UNIT "IFILEX"
//! C
//!       INCLUDE   'DSIOF.COM'
//!       INCLUDE   'ZZZZZZ.COM'
//!       LASBLK = FCB( 6, IFILEX )
//!       IF ( LASBLK .NE. 0 ) GO TO 7000
//!       INDEX  = FCB( 10, IFILEX )
//!       IF ( INDEX .EQ. 0 ) GO TO 200
//!       LASBLK = MEM( INDEX+3 )
//!       GO TO 7000
//! 200   LASBLK = 0
//! 7000  CONTINUE
//!       RETURN
//!       END
//! ```
//!
//! The two `INCLUDE` files contribute the state this routine reads:
//!
//! * `DSIOF.COM` — `COMMON /FCB/ FCB(17,MAXFCB)` with `MAXFCB = 89`.
//! * `ZZZZZZ.COM` — `COMMON /ZZZZZZ/ MEM(10)`.
//!
//! Those COMMON blocks are *not* ported as globals. `FCB`, `MEM` and `IFILEX`
//! are explicit parameters of [`dbmlbk`], so the caller owns the storage and
//! the routine has no hidden state.
//!
//! Indexing note: `FCB` and `MEM` are 1-based in Fortran. The accessors below
//! take 1-based subscripts and translate them to Rust indices, so the
//! arithmetic in [`dbmlbk`] reads exactly like the Fortran and the literal
//! subscripts `6`, `10` and `INDEX+3` are preserved verbatim.
//!
//! The routine performs no bounds checking and no validation; the Fortran
//! remains the oracle.

/// `MAXFCB` from `DSIOF.COM`: the number of columns of `FCB`.
pub const MAXFCB: usize = 89;

/// First dimension of `FCB(17,MAXFCB)` from `DSIOF.COM`.
pub const FCB_ROWS: usize = 17;

/// 0-based index of the 1-based Fortran word `MEM(i)`.
#[inline]
fn word(i: i32) -> usize {
    (i - 1) as usize
}

/// 0-based index of the 1-based Fortran element `FCB(i,j)`.
///
/// `FCB` is stored column-major, exactly as Fortran lays out
/// `FCB(17,MAXFCB)`: element `FCB(i,j)` lives at `(j-1)*FCB_ROWS + (i-1)`.
#[inline]
fn fcb_word(i: i32, j: i32) -> usize {
    ((j - 1) as usize) * FCB_ROWS + ((i - 1) as usize)
}

/// Port of `SUBROUTINE DBMLBK ( LASBLK )`.
///
/// Returns the last block number allocated to the unit `ifilex`:
///
/// 1. `LASBLK = FCB(6,IFILEX)`; if it is non-zero, that is the answer and
///    `FCB(10,IFILEX)` / `MEM` are never consulted.
/// 2. Otherwise `INDEX = FCB(10,IFILEX)`; if `INDEX` is zero the answer is 0.
/// 3. Otherwise the answer is `MEM(INDEX+3)` — the 1-based `+3` offset into
///    the open-core array is preserved literally.
///
/// `fcb` is the flat column-major `FCB(17,MAXFCB)` array and `mem` the flat
/// `MEM` open-core array, both 1-based in the Fortran sense.
pub fn dbmlbk(fcb: &[i32], mem: &[i32], ifilex: i32) -> i32 {
    // LASBLK = FCB( 6, IFILEX )
    let mut lasblk = fcb[fcb_word(6, ifilex)];

    // IF ( LASBLK .NE. 0 ) GO TO 7000
    if lasblk != 0 {
        return lasblk;
    }

    // INDEX = FCB( 10, IFILEX )
    let index = fcb[fcb_word(10, ifilex)];

    // IF ( INDEX .EQ. 0 ) GO TO 200
    if index == 0 {
        // 200 LASBLK = 0
        lasblk = 0;
    } else {
        // LASBLK = MEM( INDEX+3 )
        lasblk = mem[word(index + 3)];
    }

    // 7000 CONTINUE / RETURN
    lasblk
}

#[cfg(test)]
mod tests {
    use super::{dbmlbk, FCB_ROWS, MAXFCB};

    /// The fixed seed state used by `parity/drivers/dbmlbk.f`: every `FCB`
    /// word is `-777` and every `MEM` word is `-888`, so the printed `LASBLK`
    /// proves which word was actually read.
    fn seed_fcb() -> Vec<i32> {
        vec![-777i32; FCB_ROWS * MAXFCB]
    }

    fn seed_mem() -> Vec<i32> {
        vec![-888i32; 10]
    }

    /// 1-based `FCB(i,j)` write, column-major like Fortran.
    fn set_fcb(fcb: &mut [i32], i: i32, j: i32, v: i32) {
        fcb[((j - 1) as usize) * FCB_ROWS + ((i - 1) as usize)] = v;
    }

    /// Emit exactly the text produced by `parity/drivers/dbmlbk.f`.
    ///
    /// `IFILEX` is 1 in every case. `MEM(3)` is seeded to 5555 (the word a
    /// buggy `INDEX = 0` path would read) and `MEM(8)` to 1234 (i.e.
    /// `MEM(INDEX+3)` for `INDEX = 5`). The neighbouring column
    /// `FCB(6,2)`/`FCB(10,2)` is seeded with different values so a wrong
    /// column would also show up.
    ///
    /// 1. `FCB(6,1) = 42` (non-zero) -> 42; `FCB(10,1)` stays at `-777`.
    /// 2. `FCB(6,1) = 0`, `FCB(10,1) = 0` -> 0.
    /// 3. `FCB(6,1) = 0`, `FCB(10,1) = 5` -> `MEM(8) = 1234`.
    fn driver_output() -> String {
        // (FCB(6,1), FCB(10,1)) per case.
        let cases: [(i32, i32); 3] = [(42, -777), (0, 0), (0, 5)];

        let mut out = String::new();
        for (k, &(fcb6, fcb10)) in cases.iter().enumerate() {
            let k = k + 1;

            // RESET FCB AND MEM TO THE FIXED SEED STATE
            let mut fcb = seed_fcb();
            let mut mem = seed_mem();

            // MEM(3) IS THE WORD A BUGGY INDEX=0 PATH WOULD READ;
            // MEM(8) IS MEM(INDEX+3) FOR INDEX = 5.
            mem[2] = 5555;
            mem[7] = 1234;

            // PER-CASE ARGUMENTS (IFILEX = 1 THROUGHOUT)
            let ifilex = 1;
            set_fcb(&mut fcb, 6, 1, fcb6);
            set_fcb(&mut fcb, 10, 1, fcb10);

            // NEIGHBOURING COLUMN: A WRONG COLUMN WOULD RETURN -777
            set_fcb(&mut fcb, 6, 2, 99);
            set_fcb(&mut fcb, 10, 2, 7);

            let lasblk = dbmlbk(&fcb, &mem, ifilex);

            out.push_str(&format!("CASE {} LASBLK={:12}\n", k, lasblk));
        }
        out
    }

    #[test]
    fn dbmlbk_parity() {
        let golden = include_str!("../../../../parity/golden/dbmlbk.txt");
        let got = driver_output();
        // Printed so the parity diff can be reproduced from the test run.
        print!("{}", got);
        assert_eq!(got, golden, "DBMLBK output diverges from Fortran golden");
    }

    #[test]
    fn dbmlbk_nonzero_fcb6_short_circuits() {
        // Branch 1: a non-zero FCB(6,IFILEX) is returned as-is and neither
        // FCB(10,IFILEX) nor MEM is consulted.
        let mut fcb = seed_fcb();
        let mem = seed_mem();
        set_fcb(&mut fcb, 6, 1, 42);
        set_fcb(&mut fcb, 10, 1, 5);
        assert_eq!(dbmlbk(&fcb, &mem, 1), 42);
    }

    #[test]
    fn dbmlbk_zero_index_returns_zero() {
        // Branch 2: FCB(6,IFILEX) = 0 and FCB(10,IFILEX) = 0 -> 0, even though
        // MEM(3) holds a non-zero sentinel.
        let mut fcb = seed_fcb();
        let mut mem = seed_mem();
        mem[2] = 5555;
        set_fcb(&mut fcb, 6, 1, 0);
        set_fcb(&mut fcb, 10, 1, 0);
        assert_eq!(dbmlbk(&fcb, &mem, 1), 0);
    }

    #[test]
    fn dbmlbk_index_resolves_through_mem_plus_three() {
        // Branch 3: FCB(6,IFILEX) = 0 and INDEX = FCB(10,IFILEX) = 5 resolves
        // through the 1-based MEM(INDEX+3) = MEM(8).
        let mut fcb = seed_fcb();
        let mut mem = seed_mem();
        mem[7] = 1234;
        set_fcb(&mut fcb, 6, 1, 0);
        set_fcb(&mut fcb, 10, 1, 5);
        assert_eq!(dbmlbk(&fcb, &mem, 1), 1234);
    }
}
