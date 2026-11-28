//! Port of `mds/dsprcl.f` — unpack a DSIOF block header into `BLOCK(15)`.
//!
//! Literal transcription of the Fortran oracle:
//!
//! ```text
//!         SUBROUTINE DSPRCL ( BLOCK )
//!       INCLUDE 'DSIOF.COM'
//!       INCLUDE 'XNSTRN.COM'
//!         INTEGER BLOCK( 15 )
//!         INTEGER IDIV( 4 )
//!         DATA    IDIV / 1, 2, 1, 2 /
//!         BLOCK(  2 ) = IAND( IBASE( INDCBP ), MASKQ4 )
//!         BLOCK(  3 ) = IAND( IBASE( INDCBP ), MASKQ3 )
//!         BLOCK(  3 ) = BLOCK( 3 ) / MULQ3
//!         BLOCK( 11 ) = NWRDEL( BLOCK( 2 ) )
//!         BLOCK( 12 ) = IBASE( INDCBP+1 )
//!         BLOCK( 14 ) = IDIV( BLOCK( 2 ) )
//!         RETURN
//!         END
//! ```
//!
//! The `COMMON` blocks (`/DSIO/` from `DSIOF.COM`, `/ZZZZZZ/` from
//! `XNSTRN.COM`) are *not* ported as globals. Per the ticket the state
//! `DSPRCL` reads is passed explicitly: `IBASE`, `INDCBP`, `MASKQ3`,
//! `MASKQ4`, `MULQ3` and `NWRDEL`. The filled `BLOCK` array is returned.
//!
//! Two details are load-bearing and are reproduced exactly:
//!
//! * `IBASE` is 1-based in the oracle, so `IBASE(INDCBP)` is
//!   `ibase[indcbp - 1]` and `IBASE(INDCBP+1)` is `ibase[indcbp]`.
//! * `BLOCK(2)` is used as a 1-based index into `NWRDEL` and `IDIV`, so
//!   `NWRDEL(BLOCK(2))` is `nwrdel[block[1] - 1]`. There is no clamping
//!   and no bounds check beyond what Rust's slice indexing does; the
//!   Fortran has none either.
//!
//! `BLOCK(3) = BLOCK(3) / MULQ3` is Fortran `INTEGER` division, which
//! truncates toward zero — the same operation as Rust's `i32` `/`. The
//! mask-then-divide order is preserved: the divide is applied to the
//! already-masked word, never to the raw `IBASE` word.
//!
//! The Fortran remains the oracle; `parity/golden/dsprcl.txt` is captured
//! from `parity/drivers/dsprcl.f` built against the pinned baseline.

/// Number of `INTEGER` words in the Fortran `BLOCK(15)` array.
pub const BLOCK_LEN: usize = 15;

/// `IDIV` from the `DATA` statement: `DATA IDIV / 1, 2, 1, 2 /`.
pub const IDIV: [i32; 4] = [1, 2, 1, 2];

/// Number of `INTEGER` words in the Fortran `NWRDEL(4)` array.
pub const NWRDEL_LEN: usize = 4;

/// 0-based index of the 1-based Fortran element `i`.
///
/// Kept as a named helper so every subscript in the transcription below
/// reads like the oracle. No clamping: the Fortran does not clamp either.
#[inline]
fn at(i: i32) -> usize {
    (i - 1) as usize
}

/// Port of `SUBROUTINE DSPRCL ( BLOCK )`.
///
/// Unpacks the DSIOF block header at `IBASE(INDCBP)` into `block` and
/// returns the filled array. Only `BLOCK(2)`, `BLOCK(3)`, `BLOCK(11)`,
/// `BLOCK(12)` and `BLOCK(14)` are written; every other element is left
/// exactly as the caller supplied it, as in the oracle.
///
/// * `block` — the `INTEGER BLOCK(15)` dummy argument, 0-based here.
/// * `ibase` — `COMMON /ZZZZZZ/ IBASE(700000)` from `XNSTRN.COM`, 0-based
///   here; the 1-based subscripts of the oracle are converted with [`at`].
/// * `indcbp` — `INDCBP` from `COMMON /DBM/` in `DSIOF.COM`.
/// * `maskq3`, `maskq4`, `mulq3` — the same-named `COMMON /DSIO/` fields.
/// * `nwrdel` — `NWRDEL(4)` from `COMMON /DSIO/`.
///
/// The statements are in the exact source order of `mds/dsprcl.f`; the
/// mask/divide order and the 1-based `IBASE`/`NWRDEL`/`IDIV` subscripts
/// are preserved verbatim.
pub fn dsprcl(
    block: &mut [i32; BLOCK_LEN],
    ibase: &[i32],
    indcbp: i32,
    maskq3: i32,
    maskq4: i32,
    mulq3: i32,
    nwrdel: &[i32; NWRDEL_LEN],
) -> [i32; BLOCK_LEN] {
    // BLOCK(  2 ) = IAND( IBASE( INDCBP ), MASKQ4 )
    block[at(2)] = ibase[at(indcbp)] & maskq4;
    // BLOCK(  3 ) = IAND( IBASE( INDCBP ), MASKQ3 )
    block[at(3)] = ibase[at(indcbp)] & maskq3;
    // BLOCK(  3 ) = BLOCK( 3 ) / MULQ3
    block[at(3)] = block[at(3)] / mulq3;
    // BLOCK( 11 ) = NWRDEL( BLOCK( 2 ) )
    block[at(11)] = nwrdel[at(block[at(2)])];
    // BLOCK( 12 ) = IBASE( INDCBP+1 )
    block[at(12)] = ibase[at(indcbp + 1)];
    // BLOCK( 14 ) = IDIV( BLOCK( 2 ) )
    block[at(14)] = IDIV[at(block[at(2)])];

    *block
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mis::dsiodd::DsioddState;

    /// `INDCBP` used by the driver: `DSPRCL` reads `IBASE(5)` and `IBASE(6)`.
    const INDCBP: i32 = 5;

    /// `IBASE(INDCBP)` for the four calls. The low byte selects `BLOCK(2)`
    /// (1, 2, 3, 4); the second byte selects `BLOCK(3)` after the `MASKQ3`
    /// mask and the `MULQ3` divide. High bits are set on purpose: the masks
    /// must clear them.
    const SEED: [i32; 4] = [
        0x1234_A501u32 as i32,
        0xFFFF_0002u32 as i32,
        0x0000_0003u32 as i32,
        0x7FFF_7F04u32 as i32,
    ];

    /// `IBASE(INDCBP+1)` for the four calls; becomes `BLOCK(12)` verbatim.
    const SEED2: [i32; 4] = [100, -200, 300000, 0];

    /// One `WRITE(6,900) NAME, VALUE` line: `FORMAT(A6,1X,Z8.8)`.
    fn hex_line(out: &mut String, name: &str, value: i32) {
        out.push_str(&format!("{:<6} {:08X}\n", name, value as u32));
    }

    /// One `WRITE(6,901) 'NWRDEL', I, NWRDEL(I)` line: `FORMAT(A6,I1,1X,Z8.8)`.
    fn nwrdel_line(out: &mut String, index: usize, value: i32) {
        out.push_str(&format!("{:<6}{} {:08X}\n", "NWRDEL", index, value as u32));
    }

    /// Emits exactly the text `parity/drivers/dsprcl.f` writes: the state
    /// `DSPRCL` reads, then the five fields it stores for each of the four
    /// calls, plus the `BLOCK(1)`/`BLOCK(15)` sentinels it must not touch.
    fn driver_output() -> String {
        // The DSIODD defaults for the state DSPRCL reads.
        let s = DsioddState::with_ginox_defaults();
        let mut out = String::new();

        // The state DSPRCL reads, printed once so the golden pins it.
        hex_line(&mut out, "MASKQ3", s.maskq3);
        hex_line(&mut out, "MASKQ4", s.maskq4);
        hex_line(&mut out, "MULQ3 ", s.mulq3);
        for (i, value) in s.nwrdel.iter().enumerate() {
            nwrdel_line(&mut out, i + 1, *value);
        }
        // WRITE(6,902) 'INDCBP', INDCBP  --  FORMAT(A6,1X,I12)
        out.push_str(&format!("{:<6} {:>12}\n", "INDCBP", INDCBP));

        // IBASE(4) and IBASE(7) are seeded with distinct sentinels so an
        // off-by-one in the 1-based indexing is visible.
        let mut ibase = vec![0i32; 16];
        ibase[at(4)] = 0x1111_1111u32 as i32;
        ibase[at(7)] = 0x2222_2222u32 as i32;

        for i in 0..4 {
            ibase[at(INDCBP)] = SEED[i];
            ibase[at(INDCBP + 1)] = SEED2[i];

            let mut block = [-999i32; BLOCK_LEN];
            let filled = dsprcl(
                &mut block,
                &ibase,
                INDCBP,
                s.maskq3,
                s.maskq4,
                s.mulq3,
                &s.nwrdel,
            );
            // The returned array is the filled dummy argument.
            assert_eq!(filled, block);

            // WRITE(6,903) I, IBASE(INDCBP), IBASE(INDCBP+1)
            //   FORMAT('CALL',I2,1X,Z8.8,1X,Z8.8)
            out.push_str(&format!(
                "CALL{:>2} {:08X} {:08X}\n",
                i + 1,
                ibase[at(INDCBP)] as u32,
                ibase[at(INDCBP + 1)] as u32
            ));
            // WRITE(6,904) BLOCK(2), BLOCK(3), BLOCK(11), BLOCK(12), BLOCK(14)
            //   FORMAT(5I12)
            out.push_str(&format!(
                "{:>12}{:>12}{:>12}{:>12}{:>12}\n",
                block[at(2)],
                block[at(3)],
                block[at(11)],
                block[at(12)],
                block[at(14)]
            ));
            // WRITE(6,905) BLOCK(1), BLOCK(15)  --  FORMAT(2I12)
            out.push_str(&format!(
                "{:>12}{:>12}\n",
                block[at(1)],
                block[at(15)]
            ));
        }

        out
    }

    #[test]
    fn dsprcl_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("dsprcl_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/dsprcl.txt");
        assert_eq!(got, want, "DSPRCL output differs from Fortran golden");
    }
}
