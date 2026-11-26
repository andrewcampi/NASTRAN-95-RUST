//! Port of `mis/makmcb.f` — build a 7-word MCB block from five integers.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE MAKMCB (MCB,IFILE,IROW,IF,IT)
//! C
//!       INTEGER MCB(7)
//! C
//!       MCB(1) = IFILE
//!       MCB(2) = 0
//!       MCB(3) = IROW
//!       MCB(4) = IF
//!       MCB(5) = IT
//!       MCB(6) = 0
//!       MCB(7) = 0
//! C
//!       RETURN
//! C
//!       END
//! ```
//!
//! The routine is pure and argument-only: no COMMON, no I/O, no state. All
//! seven words of `MCB` are stored unconditionally, in the Fortran order, so
//! the three zero words overwrite whatever the caller had there. The
//! assignments are kept in source order; there is nothing to reorder or
//! "improve" here, and the Fortran remains the oracle.

/// Number of `INTEGER` words in the Fortran `MCB` dummy argument.
pub const MCB_LEN: usize = 7;

/// Port of `SUBROUTINE MAKMCB ( MCB, IFILE, IROW, IF, IT )`.
///
/// Stores, in Fortran order:
///
/// ```text
/// MCB(1) = IFILE
/// MCB(2) = 0
/// MCB(3) = IROW
/// MCB(4) = IF
/// MCB(5) = IT
/// MCB(6) = 0
/// MCB(7) = 0
/// ```
///
/// `if_` is the Fortran dummy argument `IF`; `if` is a Rust keyword, so the
/// name is escaped rather than changed in meaning. Every word is written, so
/// the caller's previous contents of `mcb` never survive the call.
#[inline]
pub fn makmcb(mcb: &mut [i32; MCB_LEN], ifile: i32, irow: i32, if_: i32, it: i32) {
    // MCB(1) = IFILE
    mcb[0] = ifile;
    // MCB(2) = 0
    mcb[1] = 0;
    // MCB(3) = IROW
    mcb[2] = irow;
    // MCB(4) = IF
    mcb[3] = if_;
    // MCB(5) = IT
    mcb[4] = it;
    // MCB(6) = 0
    mcb[5] = 0;
    // MCB(7) = 0
    mcb[6] = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Emits the same fixed-format text as `parity/drivers/makmcb.f`.
    fn driver_output() -> String {
        let mut out = String::new();

        // CASE 1: the fixed inputs from the ticket.
        let mut mcb = [0i32; MCB_LEN];
        makmcb(&mut mcb, 101, 7, 3, 9);
        out.push_str(&format!("{:<5}\n", "CASE1"));
        for word in mcb.iter() {
            out.push_str(&format!("{:>8}\n", word));
        }

        // CASE 2: MCB pre-filled with sentinels, so every one of the seven
        // words must be overwritten -- including the three zeros.
        let mut mcb = [-999i32; MCB_LEN];
        makmcb(&mut mcb, -5, 0, 12, -3);
        out.push_str(&format!("{:<5}\n", "CASE2"));
        for word in mcb.iter() {
            out.push_str(&format!("{:>8}\n", word));
        }

        out
    }

    #[test]
    fn makmcb_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("makmcb_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/makmcb.txt");
        assert_eq!(got, want, "MAKMCB output differs from Fortran golden");
    }

    #[test]
    fn makmcb_writes_all_seven_words_in_order() {
        // Every word is stored, so sentinels never survive the call.
        let mut mcb = [-1i32; MCB_LEN];
        makmcb(&mut mcb, 1, 2, 3, 4);
        assert_eq!(mcb, [1, 0, 2, 3, 4, 0, 0]);

        // The zero words are stored, not skipped: a non-zero MCB(2), MCB(6)
        // and MCB(7) on entry come back as zero.
        let mut mcb = [9i32; MCB_LEN];
        makmcb(&mut mcb, 0, 0, 0, 0);
        assert_eq!(mcb, [0; MCB_LEN]);
    }
}
