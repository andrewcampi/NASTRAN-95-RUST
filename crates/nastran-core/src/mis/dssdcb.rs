//! Port of `mds/dssdcb.f` — DSIOF dataset control-block update.
//!
//! The Fortran source is:
//!
//! ```text
//!         SUBROUTINE DSSDCB
//!       INCLUDE 'DSIOF.COM'
//!       INCLUDE 'XNSTRN.COM'
//!         ICLR  =  INDCLR - INDBAS + 1
//!         FCB( 3, IFILEX )  = ICLR
//!         FCB( 4, IFILEX )  = NBLOCK
//!         IBASE( INDBAS+1 ) = INDCBP - INDBAS + 1
//!         IBASE( INDBAS+2 ) = ICLR
//!         LASNAM = NAME
//! C        WRITE(6,40646)IFILEX,NBLOCK,ICLR,INDBAS,INDCLR
//! 40646   FORMAT(' DSSDCB,IFILEX,NBLOCK,ICLR,BAS,CLR=',I3,I5,6I7)
//!         RETURN
//!         END
//! ```
//!
//! The two `INCLUDE` files contribute the state this routine reads and
//! writes:
//!
//! * `DSIOF.COM` — `COMMON /DSIO/` (which holds `LASNAM`), `COMMON /DBM/`
//!   (which holds `INDBAS`, `INDCLR`, `INDCBP`, `NBLOCK`, `IFILEX`, `NAME`)
//!   and `COMMON /FCB/ FCB(17,MAXFCB)` with `MAXFCB = 89`.
//! * `XNSTRN.COM` — `COMMON /ZZZZZZ/ IBASE(700000)`.
//!
//! Those COMMON blocks are *not* ported as globals. The state this routine
//! touches is carried in [`DsiofState`] and passed in as an explicit
//! `&mut` parameter, so the caller owns the storage and the routine has no
//! hidden state.
//!
//! The commented-out `WRITE(6,40646)` and its `FORMAT` are dead code in the
//! Fortran and are deliberately not ported: this routine produces no
//! output.
//!
//! Indexing note: `FCB` and `IBASE` are 1-based in Fortran, and the source
//! uses the literal subscripts `3`, `4`, `INDBAS+1` and `INDBAS+2`. The
//! accessors below take 1-based subscripts and translate them to Rust
//! indices, so the arithmetic in [`dssdcb`] reads exactly like the Fortran
//! and the `+1` offsets are preserved verbatim.

/// `MAXFCB` from `DSIOF.COM`: the number of columns of `FCB`.
pub const MAXFCB: usize = 89;

/// First dimension of `FCB(17,MAXFCB)` from `DSIOF.COM`.
pub const FCB_ROWS: usize = 17;

/// Length of `IBASE(700000)` from `XNSTRN.COM`.
pub const IBASE_LEN: usize = 700_000;

/// The DSIOF/XNSTRN state that `DSSDCB` reads and writes.
///
/// This is the explicit replacement for the `COMMON /DSIO/`, `COMMON /DBM/`,
/// `COMMON /FCB/` and `COMMON /ZZZZZZ/` storage the Fortran reaches through
/// its `INCLUDE` lines. Only the members `DSSDCB` actually touches are
/// modelled; the rest of those COMMON blocks is irrelevant to this routine.
///
/// `fcb` is stored column-major, exactly as Fortran lays out
/// `FCB(17,MAXFCB)`: element `FCB(i,j)` lives at `(j-1)*FCB_ROWS + (i-1)`.
/// `ibase` is the flat `IBASE(700000)` array.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DsiofState {
    /// `COMMON /FCB/ FCB(17,MAXFCB)`, column-major.
    pub fcb: Vec<i32>,
    /// `COMMON /ZZZZZZ/ IBASE(700000)`.
    pub ibase: Vec<i32>,
    /// `COMMON /DBM/ INDCLR` — the current clear pointer.
    pub indclr: i32,
    /// `COMMON /DBM/ INDBAS` — the base of the current block.
    pub indbas: i32,
    /// `COMMON /DBM/ INDCBP` — the current buffer pointer.
    pub indcbp: i32,
    /// `COMMON /DBM/ NBLOCK` — the current block number.
    pub nblock: i32,
    /// `COMMON /DBM/ IFILEX` — the file index (the `FCB` column).
    pub ifilex: i32,
    /// `COMMON /DBM/ NAME` — the dataset name.
    pub name: i32,
    /// `COMMON /DSIO/ LASNAM` — the last assigned dataset name.
    pub lasnam: i32,
}

impl DsiofState {
    /// A state with every word zeroed, sized like the Fortran COMMON blocks.
    pub fn new() -> Self {
        Self {
            fcb: vec![0; FCB_ROWS * MAXFCB],
            ibase: vec![0; IBASE_LEN],
            indclr: 0,
            indbas: 0,
            indcbp: 0,
            nblock: 0,
            ifilex: 0,
            name: 0,
            lasnam: 0,
        }
    }

    /// `FCB(i,j)` with 1-based Fortran subscripts.
    ///
    /// # Panics
    ///
    /// Panics if `i` or `j` is outside `1..=FCB_ROWS` / `1..=MAXFCB`, which
    /// is the same out-of-bounds access the Fortran would make.
    #[inline]
    pub fn fcb(&self, i: i32, j: i32) -> i32 {
        self.fcb[fcb_index(i, j)]
    }

    /// `FCB(i,j) = value` with 1-based Fortran subscripts.
    ///
    /// # Panics
    ///
    /// Panics if `i` or `j` is outside `1..=FCB_ROWS` / `1..=MAXFCB`.
    #[inline]
    pub fn set_fcb(&mut self, i: i32, j: i32, value: i32) {
        let k = fcb_index(i, j);
        self.fcb[k] = value;
    }

    /// `IBASE(i)` with a 1-based Fortran subscript.
    ///
    /// # Panics
    ///
    /// Panics if `i` is outside `1..=IBASE_LEN`.
    #[inline]
    pub fn ibase(&self, i: i32) -> i32 {
        self.ibase[ibase_index(i)]
    }

    /// `IBASE(i) = value` with a 1-based Fortran subscript.
    ///
    /// # Panics
    ///
    /// Panics if `i` is outside `1..=IBASE_LEN`.
    #[inline]
    pub fn set_ibase(&mut self, i: i32, value: i32) {
        let k = ibase_index(i);
        self.ibase[k] = value;
    }
}

impl Default for DsiofState {
    fn default() -> Self {
        Self::new()
    }
}

/// Translate the 1-based Fortran subscripts `(i,j)` of `FCB(17,MAXFCB)` into
/// a column-major Rust index.
#[inline]
fn fcb_index(i: i32, j: i32) -> usize {
    assert!(
        (1..=FCB_ROWS as i32).contains(&i),
        "FCB first subscript {i} out of range 1..={FCB_ROWS}"
    );
    assert!(
        (1..=MAXFCB as i32).contains(&j),
        "FCB second subscript {j} out of range 1..={MAXFCB}"
    );
    (j as usize - 1) * FCB_ROWS + (i as usize - 1)
}

/// Translate the 1-based Fortran subscript `i` of `IBASE(700000)` into a
/// Rust index.
#[inline]
fn ibase_index(i: i32) -> usize {
    assert!(
        (1..=IBASE_LEN as i32).contains(&i),
        "IBASE subscript {i} out of range 1..={IBASE_LEN}"
    );
    i as usize - 1
}

/// Port of `SUBROUTINE DSSDCB`.
///
/// Updates the DSIOF dataset control block from the current `/DBM/` state,
/// in the Fortran order:
///
/// ```text
/// ICLR              = INDCLR - INDBAS + 1
/// FCB( 3, IFILEX )  = ICLR
/// FCB( 4, IFILEX )  = NBLOCK
/// IBASE( INDBAS+1 ) = INDCBP - INDBAS + 1
/// IBASE( INDBAS+2 ) = ICLR
/// LASNAM            = NAME
/// ```
///
/// The subscripts are the literal 1-based Fortran ones, including the `+1`
/// offsets on `IBASE`. `ICLR` is computed once, before any store, and the
/// same value is written to both `FCB(3,IFILEX)` and `IBASE(INDBAS+2)`.
///
/// The routine has no output: the `WRITE(6,40646)` in the Fortran source is
/// commented out and is not ported.
///
/// The updated state is returned for convenience; it is the same object that
/// was passed in.
#[inline]
pub fn dssdcb(state: &mut DsiofState) -> &mut DsiofState {
    // ICLR = INDCLR - INDBAS + 1
    let iclr = state.indclr - state.indbas + 1;
    // FCB( 3, IFILEX ) = ICLR
    state.set_fcb(3, state.ifilex, iclr);
    // FCB( 4, IFILEX ) = NBLOCK
    state.set_fcb(4, state.ifilex, state.nblock);
    // IBASE( INDBAS+1 ) = INDCBP - INDBAS + 1
    state.set_ibase(state.indbas + 1, state.indcbp - state.indbas + 1);
    // IBASE( INDBAS+2 ) = ICLR
    state.set_ibase(state.indbas + 2, iclr);
    // LASNAM = NAME
    state.lasnam = state.name;
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sentinel used by the Fortran driver to seed `FCB`, `IBASE` and
    /// `LASNAM`, so untouched words are visible in the output.
    const SENTINEL: i32 = -777;

    /// Emits the same fixed-format text as `parity/drivers/dssdcb.f`:
    /// a left-justified `A5` case label, then `I12` values.
    fn driver_output() -> String {
        let mut out = String::new();

        // CASE 1: the fixed inputs from the ticket.
        let mut st = DsiofState::new();
        for word in st.fcb.iter_mut() {
            *word = SENTINEL;
        }
        for word in st.ibase.iter_mut().take(40) {
            *word = SENTINEL;
        }
        st.lasnam = SENTINEL;
        st.ifilex = 2;
        st.nblock = 17;
        st.indbas = 5;
        st.indclr = 9;
        st.indcbp = 13;
        st.name = 1_234_567;
        dssdcb(&mut st);
        out.push_str(&format!("{:<5}\n", "CASE1"));
        for v in [
            st.fcb(3, 2),
            st.fcb(4, 2),
            st.ibase(6),
            st.ibase(7),
            st.lasnam,
            // untouched neighbours: FCB(1,2), FCB(3,1), FCB(3,3),
            // IBASE(5), IBASE(8)
            st.fcb(1, 2),
            st.fcb(3, 1),
            st.fcb(3, 3),
            st.ibase(5),
            st.ibase(8),
        ] {
            out.push_str(&format!("{v:>12}\n"));
        }

        // CASE 2: a different column/offset set.
        let mut st = DsiofState::new();
        for word in st.fcb.iter_mut() {
            *word = SENTINEL;
        }
        for word in st.ibase.iter_mut().take(40) {
            *word = SENTINEL;
        }
        st.lasnam = SENTINEL;
        st.ifilex = 4;
        st.nblock = 33;
        st.indbas = 2;
        st.indclr = 11;
        st.indcbp = 20;
        st.name = 7_654_321;
        dssdcb(&mut st);
        out.push_str(&format!("{:<5}\n", "CASE2"));
        for v in [
            st.fcb(3, 4),
            st.fcb(4, 4),
            st.ibase(3),
            st.ibase(4),
            st.lasnam,
            // untouched neighbours: FCB(1,4), FCB(3,3), FCB(3,5),
            // IBASE(2), IBASE(5)
            st.fcb(1, 4),
            st.fcb(3, 3),
            st.fcb(3, 5),
            st.ibase(2),
            st.ibase(5),
        ] {
            out.push_str(&format!("{v:>12}\n"));
        }

        out
    }

    #[test]
    fn dssdcb_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("dssdcb_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/dssdcb.txt");
        assert_eq!(got, want, "DSSDCB output differs from Fortran golden");
    }

    #[test]
    fn dssdcb_writes_exactly_the_five_words() {
        // Every other word of FCB and IBASE keeps its sentinel, and the
        // written words land on the 1-based Fortran subscripts.
        let mut st = DsiofState::new();
        for word in st.fcb.iter_mut() {
            *word = SENTINEL;
        }
        for word in st.ibase.iter_mut() {
            *word = SENTINEL;
        }
        st.lasnam = SENTINEL;
        st.ifilex = 2;
        st.nblock = 17;
        st.indbas = 5;
        st.indclr = 9;
        st.indcbp = 13;
        st.name = 1_234_567;
        dssdcb(&mut st);

        // ICLR = 9 - 5 + 1 = 5
        assert_eq!(st.fcb(3, 2), 5);
        assert_eq!(st.fcb(4, 2), 17);
        // IBASE(INDBAS+1) = IBASE(6) = 13 - 5 + 1 = 9
        assert_eq!(st.ibase(6), 9);
        // IBASE(INDBAS+2) = IBASE(7) = ICLR = 5
        assert_eq!(st.ibase(7), 5);
        assert_eq!(st.lasnam, 1_234_567);

        // Exactly five words changed: two in FCB, two in IBASE, one LASNAM.
        let fcb_changed = st.fcb.iter().filter(|&&w| w != SENTINEL).count();
        let ibase_changed = st.ibase.iter().filter(|&&w| w != SENTINEL).count();
        assert_eq!(fcb_changed, 2, "DSSDCB must write exactly two FCB words");
        assert_eq!(
            ibase_changed, 2,
            "DSSDCB must write exactly two IBASE words"
        );

        // The column is IFILEX, not a fixed column: column 1 is untouched.
        assert_eq!(st.fcb(3, 1), SENTINEL);
        assert_eq!(st.fcb(4, 1), SENTINEL);
        // The rows are 3 and 4, not 1 and 2.
        assert_eq!(st.fcb(1, 2), SENTINEL);
        assert_eq!(st.fcb(2, 2), SENTINEL);
        assert_eq!(st.fcb(5, 2), SENTINEL);
        // The IBASE offsets are INDBAS+1 and INDBAS+2, not INDBAS itself.
        assert_eq!(st.ibase(5), SENTINEL);
        assert_eq!(st.ibase(8), SENTINEL);
    }

    #[test]
    fn dssdcb_returns_the_same_state() {
        let mut st = DsiofState::new();
        st.ifilex = 1;
        st.indbas = 1;
        st.indclr = 1;
        st.indcbp = 1;
        st.nblock = 1;
        st.name = 42;
        let returned = dssdcb(&mut st) as *const DsiofState;
        assert_eq!(returned, &st as *const DsiofState);
        assert_eq!(st.lasnam, 42);
    }
}
