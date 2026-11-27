//! Port of `mis/bpack.f` — pack an internal grid number into the `IG` table.
//!
//! The Fortran source (IBM/VAX branch only) is:
//!
//! ```text
//!       SUBROUTINE BPACK (IG,I,J,L)
//! C
//!       IMPLICIT INTEGER (A-Z)
//! C
//! CDC   NEXT 2 LINES FOR CDC AND UNIVAC ONLY
//! C     EXTERNAL         ORF,      LSHIFT
//! C     INTEGER          IG(1)
//! C
//! C     NEXT LINE FOR IBM, VAX, AND MACHINES THAT HAVE INTEGER*2
//!       INTEGER*2        IG(1)
//! C
//!       COMMON /BANDB /  NBIT,     DUM3B(3), IPASS,    NW,       DUM1B,
//!      1                 NBPW
//!       COMMON /BANDS /  DUM4S(4), II1,      DUM5S(5), MASK
//! C
//!       IPASS=IPASS+1
//!       LOC  =J-1
//! C
//! C     ... commented-out CDC/UNIVAC branch, NOT ported ...
//! C
//!       N1=II1*LOC+I
//!       IG(N1)=L
//!       RETURN
//!       END
//! ```
//!
//! Only the IBM/VAX path exists in this port. `IG` is `INTEGER*2` in the
//! Fortran, so it is modelled as `i16` here and the store of `L` keeps the
//! Fortran truncation behaviour: an `INTEGER*4` value assigned to an
//! `INTEGER*2` slot keeps its low 16 bits, which is exactly what `as i16`
//! does in Rust (a wrapping, not saturating, conversion).
//!
//! The two labelled COMMON blocks are modelled as explicit state structs with
//! the Fortran word layout preserved and documented, because the routine reads
//! `IPASS` from `/BANDB/` and `II1` from `/BANDS/` and writes `IPASS` back.
//! The remaining words are carried so the block layout stays faithful and the
//! caller can keep one instance per block.
//!
//! The routine does not check nor zero the packing slot before packing, and
//! `L` is assumed to be a positive integer of `NBIT` bits or less. Nothing is
//! validated here either: the Fortran remains the oracle.

/// `/BANDB/` — 8 `INTEGER` words, in Fortran order:
///
/// ```text
/// NBIT, DUM3B(1..3), IPASS, NW, DUM1B, NBPW
/// ```
///
/// `BPACK` reads and increments `IPASS`; the other words are unused by this
/// routine but are kept so the block layout matches the Fortran.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bandb {
    /// `NBIT` — number of bits per packed slot (unused by `BPACK`).
    pub nbit: i32,
    /// `DUM3B(1..3)` — filler words.
    pub dum3b: [i32; 3],
    /// `IPASS` — counter of calls to pack/unpack; incremented on entry.
    pub ipass: i32,
    /// `NW` — words per packed word (unused by `BPACK`).
    pub nw: i32,
    /// `DUM1B` — filler word.
    pub dum1b: i32,
    /// `NBPW` — bits per word (unused by `BPACK`).
    pub nbpw: i32,
}

/// `/BANDS/` — 11 `INTEGER` words, in Fortran order:
///
/// ```text
/// DUM4S(1..4), II1, DUM5S(1..5), MASK
/// ```
///
/// `BPACK` reads only `II1` (the column stride of the `IG` table); the other
/// words are unused by this routine but are kept so the block layout matches
/// the Fortran.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bands {
    /// `DUM4S(1..4)` — filler words.
    pub dum4s: [i32; 4],
    /// `II1` — column stride of the `IG` table.
    pub ii1: i32,
    /// `DUM5S(1..5)` — filler words.
    pub dum5s: [i32; 5],
    /// `MASK` — bit mask (unused by `BPACK`).
    pub mask: i32,
}

/// Port of `SUBROUTINE BPACK ( IG, I, J, L )`, IBM/VAX branch.
///
/// ```text
/// IPASS = IPASS + 1
/// LOC   = J - 1
/// N1    = II1*LOC + I
/// IG(N1) = L
/// ```
///
/// `ig` is the `INTEGER*2` table, addressed 1-based exactly as in the Fortran
/// (`ig[(n1 - 1) as usize]`). `L` is stored through an `i16` conversion, so a
/// value outside the `i16` range is truncated to its low 16 bits, matching the
/// Fortran `INTEGER*4` → `INTEGER*2` assignment. The slot is written
/// unconditionally; it is neither checked nor zeroed first.
///
/// # Panics
///
/// Panics if the computed `N1` falls outside `ig`, which in Fortran would be
/// an out-of-bounds store into the caller's table.
#[inline]
pub fn bpack(ig: &mut [i16], i: i32, j: i32, l: i32, bandb: &mut Bandb, bands: &Bands) {
    // IPASS = IPASS + 1
    bandb.ipass = bandb.ipass.wrapping_add(1);
    // LOC = J - 1
    let loc = j.wrapping_sub(1);
    // N1 = II1*LOC + I
    let n1 = bands.ii1.wrapping_mul(loc).wrapping_add(i);
    // IG(N1) = L   (INTEGER*4 -> INTEGER*2 truncation)
    let idx = (n1 - 1) as usize;
    ig[idx] = l as i16;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Emits the same fixed-format text as `parity/drivers/bpack.f`:
    /// `FORMAT(A5)` for the labels and `FORMAT(I8)` for every value.
    fn driver_output() -> String {
        // Fixed state, exactly as the driver sets it: II1 = 4 (column
        // stride), IPASS = 0 (call counter), everything else fixed filler.
        let mut bandb = Bandb {
            nbit: 16,
            dum3b: [0; 3],
            ipass: 0,
            nw: 1,
            dum1b: 0,
            nbpw: 16,
        };
        let bands = Bands {
            dum4s: [0; 4],
            ii1: 4,
            dum5s: [0; 5],
            mask: 0,
        };

        // Sentinel-fill IG so the slots BPACK actually writes are visible.
        let mut ig = [-1i16; 16];

        // CALL 1: J = 1, so LOC = 0 and N1 = I = 1  ->  IG(1).
        bpack(&mut ig, 1, 1, 5, &mut bandb, &bands);
        // CALL 2: LOC = 2, N1 = 4*2 + 2 = 10  ->  IG(10).
        bpack(&mut ig, 2, 3, 7, &mut bandb, &bands);
        // CALL 3: LOC = 1, N1 = 4*1 + 3 = 7  ->  IG(7). L is the largest
        // value an INTEGER*2 slot can hold, i.e. the i16 boundary.
        bpack(&mut ig, 3, 2, 32767, &mut bandb, &bands);
        // CALL 4: LOC = 3, N1 = 4*3 + 4 = 16  ->  IG(16), the last slot.
        bpack(&mut ig, 4, 4, 12345, &mut bandb, &bands);

        let mut out = String::new();
        out.push_str(&format!("{:>5}\n", "IPASS"));
        out.push_str(&format!("{:>8}\n", bandb.ipass));
        out.push_str(&format!("{:>5}\n", "IG"));
        for slot in ig.iter() {
            out.push_str(&format!("{:>8}\n", slot));
        }
        out
    }

    #[test]
    fn bpack_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("bpack_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/bpack.txt");
        assert_eq!(got, want, "BPACK output differs from Fortran golden");
    }

    #[test]
    fn bpack_column_wise_index_and_ipass() {
        let mut bandb = Bandb::default();
        let bands = Bands {
            ii1: 4,
            ..Bands::default()
        };
        let mut ig = [0i16; 16];

        // J = 1 gives LOC = 0, so N1 = I: the first column is IG(1..4).
        for i in 1..=4 {
            bpack(&mut ig, i, 1, 100 + i, &mut bandb, &bands);
        }
        assert_eq!(&ig[0..4], &[101, 102, 103, 104]);
        assert_eq!(bandb.ipass, 4);

        // J = 2 gives LOC = 1, so N1 = 4 + I: the second column is IG(5..8).
        bpack(&mut ig, 1, 2, 201, &mut bandb, &bands);
        assert_eq!(ig[4], 201);
        assert_eq!(bandb.ipass, 5);
    }

    #[test]
    fn bpack_truncates_l_into_the_i16_slot() {
        let mut bandb = Bandb::default();
        let bands = Bands {
            ii1: 1,
            ..Bands::default()
        };
        let mut ig = [0i16; 4];

        // The Fortran stores an INTEGER*4 into an INTEGER*2 slot, keeping the
        // low 16 bits; the port must not saturate or reject the value.
        bpack(&mut ig, 1, 1, 32767, &mut bandb, &bands);
        assert_eq!(ig[0], 32767);
        bpack(&mut ig, 2, 1, 32768, &mut bandb, &bands);
        assert_eq!(ig[1], -32768);
        bpack(&mut ig, 3, 1, 65535, &mut bandb, &bands);
        assert_eq!(ig[2], -1);
        bpack(&mut ig, 4, 1, 65536, &mut bandb, &bands);
        assert_eq!(ig[3], 0);
    }
}
