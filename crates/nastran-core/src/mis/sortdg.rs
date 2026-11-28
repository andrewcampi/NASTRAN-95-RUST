//! Port of `mis/sortdg.f` — `SUBROUTINE SORTDG (STK1,STK2,X1,X2,NDEG)`.
//!
//! Fortran source (oracle, `vendor/nastran95/mis/sortdg.f`):
//!
//! ```fortran
//!       SUBROUTINE SORTDG (STK1,STK2,X1,X2,NDEG)
//! C
//! C     SORTDG SORTS STK2 BY DEGREE OF THE NODE AND ADDS IT TO THE END
//! C     OF STK1 IN ORDER OF LOWEST TO HIGHEST DEGREE.  X1 AND X2 ARE THE
//! C     NUMBER OF NODES IN STK1 AND STK2 RESPECTIVELY.
//! C
//!       INTEGER          X1,       X2,       STK1,     STK2,     TEMP
//!       DIMENSION        NDEG(1),  STK1(1),  STK2(1)
//!       COMMON /BANDG /  N,        IDPTH
//! C
//!       IND=X2
//!    10 ITEST=0
//!       IND=IND-1
//!       IF (IND.LT.1) GO TO 40
//!       DO 30 I=1,IND
//!       J=I+1
//!       ISTK2=STK2(I)
//!       JSTK2=STK2(J)
//!       IF (NDEG(ISTK2).LE.NDEG(JSTK2)) GO TO 30
//!       ITEST=1
//!       TEMP=STK2(I)
//!       STK2(I)=STK2(J)
//!       STK2(J)=TEMP
//!    30 CONTINUE
//!       IF (ITEST.EQ.1) GO TO 10
//!    40 DO 50 I=1,X2
//!       X1=X1+1
//!       STK1(X1)=STK2(I)
//!    50 CONTINUE
//!       RETURN
//!       END
//! ```
//!
//! Behaviour, reproduced exactly:
//!   * `STK2(1..X2)` is sorted in place by `NDEG` ascending, using the exact
//!     bubble-pass structure above: `IND` starts at `X2` and is decremented
//!     once per outer pass, the inner loop runs `I = 1..IND` and swaps
//!     `STK2(I)`/`STK2(I+1)` only when `NDEG(STK2(I)) > NDEG(STK2(I+1))`
//!     (strictly greater), and the passes repeat while `ITEST = 1` and
//!     `IND >= 1`;
//!   * the comparison is strict, so equal degrees are never swapped and the
//!     pass loop stops as soon as a pass makes no swap;
//!   * `STK2(1..X2)` is then appended to `STK1` starting at `X1+1`, with
//!     `X1` incremented once per appended node, so `X1` is an in/out
//!     argument and is returned updated;
//!   * `X2 <= 0` makes the sort a no-op (`IND` goes below 1 immediately) and
//!     the append loop `DO 50 I=1,X2` executes zero times.
//!
//! This is deliberately *not* a library sort: the pass structure and the
//! strict-greater comparison are part of the observable behaviour (which
//! elements of equal degree end up where, and how many passes run), so a
//! `sort_by_key`/`sort_unstable_by_key` substitution would not be a port.
//!
//! Integration shape: the Fortran declares `COMMON /BANDG/ N, IDPTH` but
//! never reads either variable, so the COMMON block is *not* ported as a
//! global and no counterpart is needed. The explicit parameters are
//! `NDEG`, `STK1`, `STK2`, `X1`, `X2`.
//!
//! Indexing: Fortran node ids in `STK1`/`STK2` are 1-based and are used to
//! index `NDEG`, so `NDEG(ISTK2)` is `ndeg[istk2 - 1]` here. `X1`/`X2` are
//! `INTEGER` (`i32`) and are kept as `i32` so the in/out `X1` matches the
//! Fortran exactly.

/// Sorts `stk2[..x2]` by node degree ascending and appends it to `stk1`
/// starting at `*x1 + 1`, updating `*x1`.
///
/// Mirrors `SUBROUTINE SORTDG (STK1,STK2,X1,X2,NDEG)` with the (unread)
/// `COMMON /BANDG/ N, IDPTH` dropped. `ndeg` is indexed by 1-based node id,
/// i.e. `ndeg[node - 1]`; `stk1` and `stk2` are 1-based in the Fortran and
/// 0-based here.
///
/// # Panics
///
/// Panics if `stk2` has fewer than `x2` elements, if `stk1` has fewer than
/// `*x1 + x2` elements, or if a node id in `stk2` is outside `1..=ndeg.len()`
/// (the Fortran would read/write out of bounds in those cases).
pub fn sortdg(stk1: &mut [i32], stk2: &mut [i32], x1: &mut i32, x2: i32, ndeg: &[i32]) {
    // IND=X2
    let mut ind = x2;

    // 10 ITEST=0 / IND=IND-1 / IF (IND.LT.1) GO TO 40
    loop {
        let mut itest = 0;
        ind -= 1;
        if ind < 1 {
            break;
        }

        // DO 30 I=1,IND
        for i in 1..=ind {
            let j = i + 1;
            // ISTK2=STK2(I) / JSTK2=STK2(J)
            let istk2 = stk2[(i - 1) as usize];
            let jstk2 = stk2[(j - 1) as usize];
            // IF (NDEG(ISTK2).LE.NDEG(JSTK2)) GO TO 30
            // Strictly-greater comparison: equal degrees are never swapped.
            if ndeg[(istk2 - 1) as usize] <= ndeg[(jstk2 - 1) as usize] {
                continue;
            }
            // ITEST=1 / TEMP=STK2(I) / STK2(I)=STK2(J) / STK2(J)=TEMP
            itest = 1;
            stk2[(i - 1) as usize] = jstk2;
            stk2[(j - 1) as usize] = istk2;
        }

        // IF (ITEST.EQ.1) GO TO 10
        if itest != 1 {
            break;
        }
    }

    // 40 DO 50 I=1,X2 / X1=X1+1 / STK1(X1)=STK2(I)
    // `1..=x2` is empty for `x2 <= 0`, exactly like the Fortran DO loop.
    for i in 1..=x2 {
        *x1 += 1;
        stk1[(*x1 - 1) as usize] = stk2[(i - 1) as usize];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Formats one driver record exactly like the Fortran `FORMAT(A6,21I12)`:
    /// the tag right-justified in 6 columns (the `A` edit descriptor
    /// right-justifies character data), then each value right-justified in a
    /// 12-column `I12` field.
    fn line(tag: &str, values: &[i32]) -> String {
        let mut s = format!("{:>6}", tag);
        for v in values {
            s.push_str(&format!("{:>12}", v));
        }
        s
    }

    /// The fixed `NDEG(1..12)` table used by `parity/drivers/sortdg.f`:
    /// node 1..12 -> degree 5 1 3 1 7 2 9 4 1 6 2 8.
    fn ndeg() -> [i32; 12] {
        [5, 1, 3, 1, 7, 2, 9, 4, 1, 6, 2, 8]
    }

    /// Emits the same fixed-format text as `parity/drivers/sortdg.f`.
    fn driver_output() -> String {
        let ndeg = ndeg();
        let mut out = String::new();

        // CASE 1: X2 = 0 -- nothing to sort, nothing appended.
        let mut stk1 = [11, 12, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut stk2 = [0i32; 16];
        let mut x1 = 3;
        sortdg(&mut stk1, &mut stk2, &mut x1, 0, &ndeg);
        out.push_str(&line("CASE1", &[&[x1][..], &stk1[..x1 as usize]].concat()));
        out.push('\n');

        // CASE 2: X2 = 1 -- single element, no inner loop iterations.
        let mut stk1 = [1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut stk2 = [5i32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut x1 = 2;
        sortdg(&mut stk1, &mut stk2, &mut x1, 1, &ndeg);
        out.push_str(&line("CASE2", &[&[x1][..], &stk1[..x1 as usize]].concat()));
        out.push('\n');

        // CASE 3: already sorted (ascending degree), X1 = 0.
        let mut stk1 = [0i32; 16];
        let mut stk2 = [2, 4, 9, 6, 11, 3, 8, 1, 10, 5, 12, 7, 0, 0, 0, 0];
        let mut x1 = 0;
        sortdg(&mut stk1, &mut stk2, &mut x1, 12, &ndeg);
        out.push_str(&line("CASE3", &[&[x1][..], &stk1[..x1 as usize]].concat()));
        out.push('\n');

        // CASE 4: reverse sorted (descending degree), X1 = 2.
        let mut stk1 = [3, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut stk2 = [7, 12, 5, 10, 1, 8, 3, 11, 6, 9, 4, 2, 0, 0, 0, 0];
        let mut x1 = 2;
        sortdg(&mut stk1, &mut stk2, &mut x1, 12, &ndeg);
        out.push_str(&line("CASE4", &[&[x1][..], &stk1[..x1 as usize]].concat()));
        out.push('\n');

        // CASE 5: duplicate degrees, scrambled, X1 = 1.
        let mut stk1 = [6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut stk2 = [6, 4, 11, 9, 2, 3, 8, 1, 10, 5, 12, 7, 0, 0, 0, 0];
        let mut x1 = 1;
        sortdg(&mut stk1, &mut stk2, &mut x1, 12, &ndeg);
        out.push_str(&line("CASE5", &[&[x1][..], &stk1[..x1 as usize]].concat()));
        out.push('\n');

        out
    }

    #[test]
    fn sortdg_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("sortdg_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/sortdg.txt");
        assert_eq!(got, want, "SORTDG output differs from Fortran golden");
    }

    #[test]
    fn sortdg_x2_zero_is_a_noop() {
        let ndeg = ndeg();
        let mut stk1 = [11, 12, 10, 0];
        let mut stk2 = [0i32; 4];
        let mut x1 = 3;
        sortdg(&mut stk1, &mut stk2, &mut x1, 0, &ndeg);
        assert_eq!(x1, 3, "X1 must be unchanged when X2 = 0");
        assert_eq!(stk1, [11, 12, 10, 0]);
    }

    #[test]
    fn sortdg_x1_is_in_out_and_appends_after_existing_entries() {
        let ndeg = ndeg();
        let mut stk1 = [3, 7, 0, 0, 0, 0];
        let mut stk2 = [7, 2, 0, 0, 0, 0];
        let mut x1 = 2;
        sortdg(&mut stk1, &mut stk2, &mut x1, 2, &ndeg);
        // NDEG(2) = 1 < NDEG(7) = 9, so STK2 becomes (2, 7).
        assert_eq!(x1, 4);
        assert_eq!(stk1, [3, 7, 2, 7, 0, 0]);
    }

    #[test]
    fn sortdg_never_swaps_equal_degrees() {
        // Degrees: 6 -> 2, 11 -> 2, 3 -> 3.  The strict-greater comparison
        // means the two degree-2 nodes keep their relative order.
        let ndeg = ndeg();
        let mut stk1 = [0i32; 4];
        let mut stk2 = [11, 6, 3, 0];
        let mut x1 = 0;
        sortdg(&mut stk1, &mut stk2, &mut x1, 3, &ndeg);
        assert_eq!(x1, 3);
        assert_eq!(stk1, [11, 6, 3, 0]);
    }

    #[test]
    fn sortdg_sorts_ascending_by_degree() {
        let ndeg = ndeg();
        let mut stk1 = [0i32; 12];
        let mut stk2 = [7, 12, 5, 10, 1, 8, 3, 11, 6, 9, 4, 2];
        let mut x1 = 0;
        sortdg(&mut stk1, &mut stk2, &mut x1, 12, &ndeg);
        assert_eq!(x1, 12);
        let degrees: Vec<i32> = stk1.iter().map(|n| ndeg[(n - 1) as usize]).collect();
        let mut sorted = degrees.clone();
        sorted.sort_unstable();
        assert_eq!(degrees, sorted, "STK1 must be in ascending degree order");
    }
}
