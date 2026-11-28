//! Port of `mis/dbmmov.f` — copy `NO` words within the in-memory database.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE DBMMOV ( INDEX1, INDEX2, NO )
//!       INCLUDE 'ZZZZZZ.COM'
//!       DO 10 I = 1, NO
//!       MEM( INDEX2+I-1 ) = MEM( INDEX1+I-1 )
//! 10    CONTINUE
//!       RETURN
//!       END
//! ```
//!
//! `ZZZZZZ.COM` declares `COMMON / ZZZZZZ / MEM(10)`; the block is *not*
//! ported as a global. `MEM` is an explicit `&mut [i32]` parameter, exactly as
//! the ticket requires, and the 1-based Fortran indexing is reproduced by the
//! `index2 + i - 1` / `index1 + i - 1` offsets below.
//!
//! The copy is a **forward** loop, so overlapping ranges behave like the
//! Fortran: when `INDEX2 > INDEX1` and the ranges overlap, the words already
//! written are re-read by later iterations. That is deliberately *not* a
//! `copy_within`/`memmove`; substituting one would change the result of the
//! overlapping case pinned by `parity/golden/dbmmov.txt`.
//!
//! `DBMMOV` is pure apart from `MEM`: no I/O, no saved state, no other COMMON.

/// Port of `SUBROUTINE DBMMOV ( INDEX1, INDEX2, NO )`.
///
/// Copies `no` words from the 1-based Fortran range starting at `index1` to
/// the 1-based Fortran range starting at `index2`:
///
/// ```text
/// DO 10 I = 1, NO
/// MEM( INDEX2+I-1 ) = MEM( INDEX1+I-1 )
/// 10 CONTINUE
/// ```
///
/// The loop runs forward, one word at a time, so overlapping ranges are copied
/// with Fortran forward-copy semantics rather than `memmove` semantics.
///
/// `no <= 0` performs no work, matching `DO 10 I = 1, NO` with a non-positive
/// trip count.
///
/// # Panics
///
/// Panics if either 1-based range falls outside `mem`, which in Fortran would
/// be an out-of-bounds reference into the `ZZZZZZ` common block.
pub fn dbmmov(mem: &mut [i32], index1: i32, index2: i32, no: i32) {
    // DO 10 I = 1, NO
    for i in 1..=no {
        // MEM( INDEX2+I-1 ) = MEM( INDEX1+I-1 )
        let src = (index1 + i - 1) as usize - 1;
        let dst = (index2 + i - 1) as usize - 1;
        mem[dst] = mem[src];
    }
}

#[cfg(test)]
mod tests {
    use super::dbmmov;

    /// Emit exactly the text produced by `parity/drivers/dbmmov.f`.
    ///
    /// `MEM(1..20)` is seeded once with `1000 + I` and printed after each of
    /// the three calls, so the output pins both the words `DBMMOV` writes and
    /// the forward-copy order of the overlapping case.
    fn driver_output() -> String {
        // (INDEX1, INDEX2, NO) per case.
        let cases: [(i32, i32, i32); 3] = [(1, 11, 5), (1, 3, 5), (1, 11, 0)];

        // DO 5 I = 1, 20 / MEM(I) = 1000 + I
        let mut mem = [0_i32; 20];
        for (i, slot) in mem.iter_mut().enumerate() {
            *slot = 1000 + (i as i32 + 1);
        }

        let mut out = String::new();
        for (k, &(index1, index2, no)) in cases.iter().enumerate() {
            let k = k + 1;

            dbmmov(&mut mem, index1, index2, no);

            out.push_str(&format!(
                "CASE {} INDEX1={:3} INDEX2={:3} NO={:3}\n",
                k, index1, index2, no
            ));
            for (i, v) in mem.iter().enumerate() {
                out.push_str(&format!("MEM {:3}{:12}\n", i + 1, v));
            }
        }
        out
    }

    #[test]
    fn dbmmov_parity() {
        let golden = include_str!("../../../../parity/golden/dbmmov.txt");
        let got = driver_output();
        // Printed so the parity diff can be reproduced from the test run.
        print!("{}", got);
        assert_eq!(got, golden, "DBMMOV output diverges from Fortran golden");
    }

    #[test]
    fn dbmmov_overlap_is_forward_copy_not_memmove() {
        // INDEX1=1, INDEX2=3, NO=5: the forward loop re-reads words it has
        // already overwritten, so MEM(3..7) becomes 1,2,1,2,1 -- not 1..5.
        let mut mem = [0_i32; 20];
        for (i, slot) in mem.iter_mut().enumerate() {
            *slot = 1000 + (i as i32 + 1);
        }
        dbmmov(&mut mem, 1, 3, 5);
        assert_eq!(&mem[2..7], &[1001, 1002, 1001, 1002, 1001]);
        // A memmove-style copy would have produced 1001..1005 here.
        assert_ne!(&mem[2..7], &[1001, 1002, 1003, 1004, 1005]);
    }

    #[test]
    fn dbmmov_no_is_noop() {
        let mut mem = [0_i32; 20];
        for (i, slot) in mem.iter_mut().enumerate() {
            *slot = 1000 + (i as i32 + 1);
        }
        let before = mem;
        dbmmov(&mut mem, 1, 11, 0);
        assert_eq!(mem, before);
        dbmmov(&mut mem, 1, 11, -3);
        assert_eq!(mem, before);
    }
}
