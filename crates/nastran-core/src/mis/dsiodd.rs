//! Port of `mds/dsiodd.f` — the GINO/DSIOF state initializer.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE DSIODD
//!       INCLUDE 'GINOX.COM'
//!       INCLUDE 'DSIOF.COM'
//!       LGINOX = 5*NUMFCB + NUMSOF + 2
//!       LHALF  = 16
//!       LENDSP = 0
//!       LENWPB = 0
//!       MASKH1 = 'FFFF0000'X
//!       ...
//!       NWRDEL( 4 ) = 4
//!       RETURN
//!       END
//! ```
//!
//! `DSIODD` has no dummy arguments: it writes module-level state held in
//! `COMMON /GINOX/` (`LGINOX`) and `COMMON /DSIO/` (everything else), using
//! `NUMFCB` and `NUMSOF` from the `PARAMETER` statement in `GINOX.COM`.
//!
//! Per the ticket, the `COMMON` blocks are **not** ported as globals. The
//! state is modelled as an owned struct, [`DsioddState`], whose constructor
//! returns the fully initialized value. That keeps the module testable
//! without global mutable state while preserving the exact Fortran state
//! image: every field `DSIODD` stores is a field of the struct, stored in
//! source order, with the same 32-bit two's-complement values.
//!
//! The Fortran remains the oracle; `parity/golden/dsiodd.txt` is captured
//! from `parity/drivers/dsiodd.f` built against the pinned baseline.

/// `NUMFCB` from the `PARAMETER` statement in `GINOX.COM`.
pub const NUMFCB: i32 = 89;

/// `NUMSOF` from the `PARAMETER` statement in `GINOX.COM`.
pub const NUMSOF: i32 = 10;

/// Number of `INTEGER` words in the Fortran `NWRDEL(4)` array.
pub const NWRDEL_LEN: usize = 4;

/// The state image written by `DSIODD`.
///
/// Field names are the Fortran `COMMON` variable names, lowercased. All
/// values are `i32` because the Fortran variables are default `INTEGER`
/// (32-bit) and the hex masks are stored as signed two's-complement words —
/// e.g. `MASKH1 = 'FFFF0000'X` is `-65536`, not `4294901760`.
///
/// `MASKQ1..4` are aliased to `MASKE1..4` in the Fortran source (plain
/// assignments, not `EQUIVALENCE`), so they are separate fields here that
/// happen to receive the same values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DsioddState {
    // COMMON /GINOX/
    pub lginox: i32,
    // COMMON /DSIO/
    pub lhalf: i32,
    pub lendsp: i32,
    pub lenwpb: i32,
    pub maskh1: i32,
    pub maskh2: i32,
    pub maske1: i32,
    pub maske2: i32,
    pub maske3: i32,
    pub maske4: i32,
    pub mcbmas: i32,
    pub maxdsn: i32,
    pub maskq1: i32,
    pub maskq2: i32,
    pub maskq3: i32,
    pub maskq4: i32,
    pub mulq1: i32,
    pub mulq2: i32,
    pub mulq3: i32,
    pub idsx: i32,
    pub idsp: i32,
    pub idsc: i32,
    pub idsrh: i32,
    pub idsrt: i32,
    pub idssb: i32,
    pub idsse: i32,
    pub idsch: i32,
    pub idsct: i32,
    pub idssh: i32,
    pub idsst: i32,
    pub idssd: i32,
    pub idseb: i32,
    pub idsef: i32,
    pub nwrdel: [i32; NWRDEL_LEN],
}

impl DsioddState {
    /// Port of `SUBROUTINE DSIODD`.
    ///
    /// `numfcb` and `numsof` are the Fortran `PARAMETER`s `NUMFCB` and
    /// `NUMSOF` from `GINOX.COM`; they are passed in so the initializer is
    /// testable with any values. Use [`DsioddState::with_ginox_defaults`] for
    /// the pinned baseline values.
    ///
    /// The assignments below are in the exact source order of
    /// `mds/dsiodd.f`. Nothing is reordered, folded, or "simplified": the
    /// hex literals are written as `0x...u32 as i32` so the signed
    /// two's-complement value matches what Fortran stores in an `INTEGER`.
    pub fn new(numfcb: i32, numsof: i32) -> Self {
        // LGINOX = 5*NUMFCB + NUMSOF + 2
        let lginox = 5 * numfcb + numsof + 2;
        // LHALF  = 16
        let lhalf = 16;
        // LENDSP = 0
        let lendsp = 0;
        // LENWPB = 0
        let lenwpb = 0;
        // MASKH1 = 'FFFF0000'X
        let maskh1 = 0xFFFF_0000u32 as i32;
        // MASKH2 = '0000FFFF'X
        let maskh2 = 0x0000_FFFFu32 as i32;
        // MASKE1 = 'FF000000'X
        let maske1 = 0xFF00_0000u32 as i32;
        // MASKE2 = '00FF0000'X
        let maske2 = 0x00FF_0000u32 as i32;
        // MASKE3 = '0000FF00'X
        let maske3 = 0x0000_FF00u32 as i32;
        // MASKE4 = '000000FF'X
        let maske4 = 0x0000_00FFu32 as i32;
        // MCBMAS = '40000000'X
        let mcbmas = 0x4000_0000u32 as i32;
        // MAXDSN = NUMFCB
        let maxdsn = numfcb;
        // MASKQ1 = MASKE1
        let maskq1 = maske1;
        // MASKQ2 = MASKE2
        let maskq2 = maske2;
        // MASKQ3 = MASKE3
        let maskq3 = maske3;
        // MASKQ4 = MASKE4
        let maskq4 = maske4;
        // MULQ1  = 2**24
        let mulq1 = 2i32.pow(24);
        // MULQ2  = 2**16
        let mulq2 = 2i32.pow(16);
        // MULQ3  = 2**8
        let mulq3 = 2i32.pow(8);
        // IDSX   = '00EE0000'X
        let idsx = 0x00EE_0000u32 as i32;
        // IDSP   = '000E0000'X
        let idsp = 0x000E_0000u32 as i32;
        // IDSC   = '000C0000'X
        let idsc = 0x000C_0000u32 as i32;
        // IDSRH  = '11000000'X
        let idsrh = 0x1100_0000u32 as i32;
        // IDSRT  = '77000000'X
        let idsrt = 0x7700_0000u32 as i32;
        // IDSSB  = '22000000'X
        let idssb = 0x2200_0000u32 as i32;
        // IDSSE  = '7F000000'X
        let idsse = 0x7F00_0000u32 as i32;
        // IDSCH  = '3B000000'X
        let idsch = 0x3B00_0000u32 as i32;
        // IDSCT  = '3F000000'X
        let idsct = 0x3F00_0000u32 as i32;
        // IDSSH  = '4B000000'X
        let idssh = 0x4B00_0000u32 as i32;
        // IDSST  = '4F000000'X
        let idsst = 0x4F00_0000u32 as i32;
        // IDSSD  = 'DD000000'X
        let idssd = 0xDD00_0000u32 as i32;
        // IDSEB  = 'EB000000'X
        let idseb = 0xEB00_0000u32 as i32;
        // IDSEF  = 'EF000000'X
        let idsef = 0xEF00_0000u32 as i32;
        // NWRDEL( 1 ) = 1
        // NWRDEL( 2 ) = 2
        // NWRDEL( 3 ) = 2
        // NWRDEL( 4 ) = 4
        let nwrdel = [1, 2, 2, 4];

        DsioddState {
            lginox,
            lhalf,
            lendsp,
            lenwpb,
            maskh1,
            maskh2,
            maske1,
            maske2,
            maske3,
            maske4,
            mcbmas,
            maxdsn,
            maskq1,
            maskq2,
            maskq3,
            maskq4,
            mulq1,
            mulq2,
            mulq3,
            idsx,
            idsp,
            idsc,
            idsrh,
            idsrt,
            idssb,
            idsse,
            idsch,
            idsct,
            idssh,
            idsst,
            idssd,
            idseb,
            idsef,
            nwrdel,
        }
    }

    /// `DSIODD` called with the pinned `GINOX.COM` `PARAMETER` values.
    pub fn with_ginox_defaults() -> Self {
        Self::new(NUMFCB, NUMSOF)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One `WRITE(6,900) NAME, VALUE, VALUE` line:
    /// `FORMAT(A8,1X,Z8.8,1X,I12)`.
    ///
    /// `A8` right-justifies the name in 8 columns, `Z8.8` prints the 32-bit
    /// two's-complement word as 8 uppercase hex digits, and `I12`
    /// right-justifies the signed decimal value in 12 columns.
    fn line(out: &mut String, name: &str, value: i32) {
        out.push_str(&format!(
            "{:>8} {:08X} {:>12}\n",
            name, value as u32, value
        ));
    }

    /// One `WRITE(6,901) 'NWRDEL', I, NWRDEL(I), NWRDEL(I)` line:
    /// `FORMAT(A8,I1,1X,Z8.8,1X,I12)`.
    fn nwrdel_line(out: &mut String, index: usize, value: i32) {
        out.push_str(&format!(
            "{:>8}{} {:08X} {:>12}\n",
            "NWRDEL", index, value as u32, value
        ));
    }

    /// Emits exactly the text `parity/drivers/dsiodd.f` writes: the fixed
    /// `NUMFCB`/`NUMSOF` inputs, then the whole state image in source order
    /// of `mds/dsiodd.f`.
    fn driver_output() -> String {
        let s = DsioddState::with_ginox_defaults();
        let mut out = String::new();

        // Fixed inputs, so the golden pins the NUMFCB/NUMSOF the state was
        // built from.
        line(&mut out, "NUMFCB", NUMFCB);
        line(&mut out, "NUMSOF", NUMSOF);

        // The state image, in source order of mds/dsiodd.f.
        line(&mut out, "LGINOX", s.lginox);
        line(&mut out, "LHALF", s.lhalf);
        line(&mut out, "LENDSP", s.lendsp);
        line(&mut out, "LENWPB", s.lenwpb);
        line(&mut out, "MASKH1", s.maskh1);
        line(&mut out, "MASKH2", s.maskh2);
        line(&mut out, "MASKE1", s.maske1);
        line(&mut out, "MASKE2", s.maske2);
        line(&mut out, "MASKE3", s.maske3);
        line(&mut out, "MASKE4", s.maske4);
        line(&mut out, "MCBMAS", s.mcbmas);
        line(&mut out, "MAXDSN", s.maxdsn);
        line(&mut out, "MASKQ1", s.maskq1);
        line(&mut out, "MASKQ2", s.maskq2);
        line(&mut out, "MASKQ3", s.maskq3);
        line(&mut out, "MASKQ4", s.maskq4);
        line(&mut out, "MULQ1", s.mulq1);
        line(&mut out, "MULQ2", s.mulq2);
        line(&mut out, "MULQ3", s.mulq3);
        line(&mut out, "IDSX", s.idsx);
        line(&mut out, "IDSP", s.idsp);
        line(&mut out, "IDSC", s.idsc);
        line(&mut out, "IDSRH", s.idsrh);
        line(&mut out, "IDSRT", s.idsrt);
        line(&mut out, "IDSSB", s.idssb);
        line(&mut out, "IDSSE", s.idsse);
        line(&mut out, "IDSCH", s.idsch);
        line(&mut out, "IDSCT", s.idsct);
        line(&mut out, "IDSSH", s.idssh);
        line(&mut out, "IDSST", s.idsst);
        line(&mut out, "IDSSD", s.idssd);
        line(&mut out, "IDSEB", s.idseb);
        line(&mut out, "IDSEF", s.idsef);
        for (i, value) in s.nwrdel.iter().enumerate() {
            nwrdel_line(&mut out, i + 1, *value);
        }

        out
    }

    #[test]
    fn dsiodd_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("dsiodd_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/dsiodd.txt");
        assert_eq!(got, want, "DSIODD output differs from Fortran golden");
    }

    #[test]
    fn dsiodd_state_image_is_fully_initialized() {
        // Every field DSIODD stores is stored: nothing is left at a sentinel.
        let s = DsioddState::with_ginox_defaults();
        assert_eq!(s.lginox, 5 * NUMFCB + NUMSOF + 2);
        assert_eq!(s.lginox, 457);
        assert_eq!(s.lhalf, 16);
        assert_eq!(s.lendsp, 0);
        assert_eq!(s.lenwpb, 0);
        assert_eq!(s.maxdsn, NUMFCB);

        // The hex masks are signed 32-bit words, not unsigned.
        assert_eq!(s.maskh1, -65536);
        assert_eq!(s.maskh2, 65535);
        assert_eq!(s.maske1, -16777216);
        assert_eq!(s.maske2, 16711680);
        assert_eq!(s.maske3, 65280);
        assert_eq!(s.maske4, 255);
        assert_eq!(s.mcbmas, 1073741824);

        // MASKQ1..4 are assigned from MASKE1..4.
        assert_eq!(
            [s.maskq1, s.maskq2, s.maskq3, s.maskq4],
            [s.maske1, s.maske2, s.maske3, s.maske4]
        );

        // MULQ1..3 = 2**24 / 2**16 / 2**8.
        assert_eq!([s.mulq1, s.mulq2, s.mulq3], [16777216, 65536, 256]);

        // NWRDEL(1..4) = 1, 2, 2, 4.
        assert_eq!(s.nwrdel, [1, 2, 2, 4]);
    }

    #[test]
    fn dsiodd_generalizes_over_numfcb_and_numsof() {
        // LGINOX = 5*NUMFCB + NUMSOF + 2 and MAXDSN = NUMFCB for any inputs.
        for &(numfcb, numsof) in &[(0, 0), (1, 1), (89, 10), (200, 37)] {
            let s = DsioddState::new(numfcb, numsof);
            assert_eq!(s.lginox, 5 * numfcb + numsof + 2);
            assert_eq!(s.maxdsn, numfcb);
            // The constant part of the image does not depend on the inputs.
            assert_eq!(s.lhalf, 16);
            assert_eq!(s.maske1, -16777216);
            assert_eq!(s.nwrdel, [1, 2, 2, 4]);
        }
    }
}
