//! Port of `mis/dbmrlb.f` — release an in-memory block back to the free chain.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE DBMRLB ( INDEX )
//! C********************************************************************
//! C  DBMRLB  -   RELEASES AN IN-MEMORY BLOCK THAT IS CURRENTLY
//! C              ALLOCATED AS THE LAST BLOCK OF AN IN-MEMORY FILE.
//! C              THIS IS USED TO RELEASE THE NEXT ALLOCATED BLOCK FOR A
//! C              FILE OPENED FOR WRITE BUT WAS NEVER USED BECAUSE THE
//! C              FILE WAS CLOSED--I.E., THE LAST BLOCK ALLOCATED FOR A
//! C              FILE OPENED FOR WRITE IS NEVER USED BUT IT MUST HAVE
//! C              BEEN ALLOCATED JUST IN CASE THE FILE IS NOT TO BE CLOSED.
//! C********************************************************************
//!       INCLUDE   'DSIOF.COM'
//!       COMMON / ZZZZZZ / MEM(4)
//!       COMMON / SYSTEM / ISYSBF, IWR
//!       INDEXL = INDEX
//! C CHECK IF OTHER BLOCKS ARE CHAINED TO THE END OF THIS BLOCK
//!       IF ( MEM( INDEX+1 ) .NE. 0 ) GO TO 100
//! C SET "NEXT" OF PREVIOUS BLOCK TO ZERO, IF IT EXISTS
//! 5     LINDEX = MEM( INDEX )
//!       IF ( LINDEX .EQ. 0 ) GO TO 10
//!       MEM( LINDEX+1 ) = 0
//! 10    IF ( IDBFRE .NE. 0 ) GO TO 20
//! C FREE CHAIN IS EMPTY, THIS BLOCK BECOMES FREE CHAIN
//!       IDBFRE = INDEX
//! C SET "NEXT" AND "PREVIOUS" OF THIS CHAIN TO ZERO
//!       MEM( INDEX    ) = 0
//!       MEM( INDEXL+1 ) = 0
//!       GO TO 700
//! C SET BLOCKS TO BE FREED AT FIRST OF FREE CHAIN AND
//! C THEN CONNECT FREE CHAIN TO THIS BLOCK
//! 20    ISAVE  = IDBFRE
//!       IDBFRE = INDEX
//!       MEM( ISAVE    ) = INDEXL
//!       MEM( INDEX    ) = 0
//!       MEM( INDEXL+1 ) = ISAVE
//!       GO TO 700
//! C MORE THAN ONE BLOCK IN THIS CHAIN TO RELEASE BACK TO FREE CHAIN
//! 100   CONTINUE
//! 110   IF ( MEM( INDEXL+1 ) .EQ. 0 ) GO TO 5
//! CWKBR SPR94012 10/94      INDEXL = MEM( INDEX+1 )
//!       INDEXL = MEM( INDEXL+1 )
//!       GO TO 110
//! 700   CONTINUE
//!       RETURN
//!       END
//! ```
//!
//! `MEM` is the `/ZZZZZZ/` open-core array and `IDBFRE` is the head of the
//! free chain from `/DBM/` (declared in `DSIOF.COM`). Neither is ported as a
//! global: `mem` is a `&mut [i32]` and `idbfre` a `&mut i32`, so the caller
//! owns the state exactly as the Fortran caller owns the COMMON blocks.
//!
//! Two details of the oracle are preserved literally because they are
//! observable:
//!
//! * `MEM(INDEX)` — the "previous" word of the *original* `INDEX`, not of the
//!   walked-to tail `INDEXL` — is what gets read into `LINDEX` at label 5.
//!   After the 110 forward walk `INDEXL != INDEX`, and the two are not
//!   interchangeable.
//! * The 110 walk advances `INDEXL` through the "next" words until it lands on
//!   the block whose "next" word is zero, i.e. the tail of the chain.
//!
//! The routine performs no bounds checking and no validation; the Fortran
//! remains the oracle.

/// 0-based index of the 1-based Fortran word `MEM(i)`.
#[inline]
fn word(i: i32) -> usize {
    (i - 1) as usize
}

/// Port of `SUBROUTINE DBMRLB ( INDEX )`.
///
/// Releases the in-memory block `index` (a 1-based word offset into `mem`)
/// back to the free chain headed by `idbfre`.
///
/// * If `mem[index+1] == 0` the block is already the tail of its chain and the
///   release happens immediately.
/// * Otherwise the 110 loop walks forward through the "next" words until it
///   reaches the tail block, and that tail is what gets released.
///
/// In both cases the "next" word of the previous block is zeroed (when a
/// previous block exists), then the released block either becomes the whole
/// free chain (`idbfre == 0`) or is spliced in front of the existing one.
pub fn dbmrlb(mem: &mut [i32], idbfre: &mut i32, index: i32) {
    // INDEXL = INDEX
    let mut indexl = index;

    // CHECK IF OTHER BLOCKS ARE CHAINED TO THE END OF THIS BLOCK
    if mem[word(index + 1)] != 0 {
        // 100 CONTINUE
        // MORE THAN ONE BLOCK IN THIS CHAIN TO RELEASE BACK TO FREE CHAIN
        // 110 IF ( MEM( INDEXL+1 ) .EQ. 0 ) GO TO 5
        while mem[word(indexl + 1)] != 0 {
            // INDEXL = MEM( INDEXL+1 )
            indexl = mem[word(indexl + 1)];
        }
    }

    // 5  SET "NEXT" OF PREVIOUS BLOCK TO ZERO, IF IT EXISTS
    //    LINDEX = MEM( INDEX )
    let lindex = mem[word(index)];
    if lindex != 0 {
        // MEM( LINDEX+1 ) = 0
        mem[word(lindex + 1)] = 0;
    }

    // 10 IF ( IDBFRE .NE. 0 ) GO TO 20
    if *idbfre == 0 {
        // FREE CHAIN IS EMPTY, THIS BLOCK BECOMES FREE CHAIN
        *idbfre = index;
        // SET "NEXT" AND "PREVIOUS" OF THIS CHAIN TO ZERO
        mem[word(index)] = 0;
        mem[word(indexl + 1)] = 0;
        return;
    }

    // 20 SET BLOCKS TO BE FREED AT FIRST OF FREE CHAIN AND
    //    THEN CONNECT FREE CHAIN TO THIS BLOCK
    let isave = *idbfre;
    *idbfre = index;
    mem[word(isave)] = indexl;
    mem[word(index)] = 0;
    mem[word(indexl + 1)] = isave;
}

#[cfg(test)]
mod tests {
    use super::dbmrlb;

    /// The fixed seed state used by `parity/drivers/dbmrlb.f`: a 3-block
    /// allocated chain `1 -> 5 -> 9` (each block is prev, next, data, data)
    /// plus a 2-block free chain `13 -> 17`, in a 24-word `MEM`.
    fn seed() -> [i32; 24] {
        let mut mem = [0i32; 24];
        // ALLOCATED CHAIN 1 -> 5 -> 9
        mem[0] = 0;
        mem[1] = 5;
        mem[2] = 101;
        mem[3] = 102;
        mem[4] = 1;
        mem[5] = 9;
        mem[6] = 103;
        mem[7] = 104;
        mem[8] = 5;
        mem[9] = 0;
        mem[10] = 105;
        mem[11] = 106;
        // FREE CHAIN 13 -> 17
        mem[12] = 0;
        mem[13] = 17;
        mem[14] = 201;
        mem[15] = 202;
        mem[16] = 13;
        mem[17] = 0;
        mem[18] = 203;
        mem[19] = 204;
        mem
    }

    /// Emit exactly the text produced by `parity/drivers/dbmrlb.f`.
    ///
    /// Three cases, `MEM` reset to the seed state before each one:
    ///
    /// 1. release tail block 9 with an empty free chain (`IDBFRE = 0`);
    /// 2. release tail block 9 with a non-empty free chain (`IDBFRE = 13`);
    /// 3. release head block 1, forcing the 110 forward walk to the tail.
    fn driver_output() -> String {
        // (IDBFRE, INDEX) per case.
        let cases: [(i32, i32); 3] = [(0, 9), (13, 9), (13, 1)];

        let mut out = String::new();
        for (k, &(idbfre_seed, index)) in cases.iter().enumerate() {
            let k = k + 1;

            // RESET MEM TO THE FIXED SEED STATE
            let mut mem = seed();
            let mut idbfre = idbfre_seed;

            dbmrlb(&mut mem, &mut idbfre, index);

            out.push_str(&format!("CASE {} IDBFRE={:12}\n", k, idbfre));
            for (i, v) in mem.iter().enumerate() {
                out.push_str(&format!("  MEM{:3}={:12}\n", i + 1, v));
            }
        }
        out
    }

    #[test]
    fn dbmrlb_parity() {
        let golden = include_str!("../../../../parity/golden/dbmrlb.txt");
        let got = driver_output();
        // Printed so the parity diff can be reproduced from the test run.
        print!("{}", got);
        assert_eq!(got, golden, "DBMRLB output diverges from Fortran golden");
    }

    #[test]
    fn dbmrlb_forward_walk_releases_tail_not_head() {
        // Releasing the head of a 3-block chain must walk to the tail (9) and
        // splice *that* block in front of the free chain, leaving the head's
        // own "next" word untouched.
        let mut mem = seed();
        let mut idbfre = 13;
        dbmrlb(&mut mem, &mut idbfre, 1);
        assert_eq!(idbfre, 1);
        // MEM(ISAVE) = INDEXL: the old free head now points back at the tail.
        assert_eq!(mem[12], 9);
        // MEM(INDEX) = 0: the released head's "previous" word is cleared.
        assert_eq!(mem[0], 0);
        // MEM(INDEXL+1) = ISAVE: the tail's "next" word links the free chain.
        assert_eq!(mem[9], 13);
        // The head's "next" word is not touched by this path.
        assert_eq!(mem[1], 5);
    }

    #[test]
    fn dbmrlb_empty_free_chain_makes_block_the_chain() {
        // With IDBFRE == 0 the released tail becomes the whole free chain and
        // both of its link words are zeroed.
        let mut mem = seed();
        let mut idbfre = 0;
        dbmrlb(&mut mem, &mut idbfre, 9);
        assert_eq!(idbfre, 9);
        assert_eq!(mem[8], 0); // MEM(INDEX) = 0
        assert_eq!(mem[9], 0); // MEM(INDEXL+1) = 0
        assert_eq!(mem[5], 0); // MEM(LINDEX+1) = 0, previous block's "next"
    }
}
