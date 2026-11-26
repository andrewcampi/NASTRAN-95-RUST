//! Port of `mis/permut.f` — stable permutation index for a sort key.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE PERMUT(IA,ID,N,ISW)
//!       DIMENSION IA(1),IB(32),IC(32),ID(10)
//!       DO 10 I=1,N
//!       IC(I)=IA(I)
//!    10 IB(I)=I
//!       N1=N-1
//!       DO 20 I=1,N1
//!       I1=I+1
//!       DO 30 J=I1,N
//!       IF(IC(J)-IC(I))40,30,30
//!    40 IS1=IB(J)
//!       IB(J)=IB(I)
//!       IB(I)=IS1
//!       IS1 = IC(J)
//!       IC(J)=IC(I)
//!       IC(I) = IS1
//!    30 CONTINUE
//!    20 CONTINUE
//!       DO 50 I = 1,N
//!       IF(IC(I)-ISW)50,60,60
//!    50 CONTINUE
//!       K=1
//!       GO TO 71
//!    60 DO 70 J=I,N
//!       K=J-I+1
//!    70 ID(K)=IB(J)
//!       IF(K .EQ. N) GO TO 90
//!       K=K+1
//!    71 DO 80 J=K,N
//!       L=J-K+1
//!    80 ID(J)=IB(L)
//!    90 RETURN
//!       END
//! ```
//!
//! This is *not* a plain sort. Two details are load-bearing and are
//! transcribed literally rather than rewritten idiomatically:
//!
//! * The exchange sort compares `IC(J)-IC(I)` and swaps only on a strictly
//!   negative difference (`IF(IC(J)-IC(I)) 40,30,30`). Equal keys are never
//!   swapped *against each other*, but this is an exchange sort, not a stable
//!   sort: an equal key can still be displaced indirectly when a smaller key
//!   is swapped past it. The exact resulting permutation is therefore
//!   reproduced here rather than replaced by a stable sort — the Fortran is
//!   the oracle (see `permut_tie_break_matches_fortran`).
//! * The rotation is driven by `GOTO`s. The `DO 50` scan looks for the first
//!   `I` with `IC(I) >= ISW`; if every key is below `ISW` the loop falls
//!   through with `K=1` and jumps straight to label 71, which copies `IB`
//!   unchanged. Otherwise label 60 copies `IB(I..N)` to the front of `ID` and
//!   label 71 wraps `IB(1..I-1)` onto the tail.
//!
//! The Fortran declares `IB(32)`, `IC(32)` and `ID(10)`; the same limits are
//! enforced here by the slice lengths the caller supplies (`ia`/`id` must hold
//! at least `n` elements, and `n` must not exceed 32 for the internal
//! work arrays).

/// Work-array bound from the Fortran `DIMENSION IB(32),IC(32)`.
pub const PERMUT_MAX_N: usize = 32;

/// `SUBROUTINE PERMUT(IA,ID,N,ISW)`.
///
/// Fills `id[0..n]` with the stable permutation of `1..=n` that orders
/// `ia[0..n]` ascending, rotated so that it starts at the first sorted key
/// that is `>= isw` (wrapping to the front of the permutation).
///
/// `ia` and `id` must each hold at least `n` elements, and `n <= 32`.
pub fn permut(ia: &[i32], id: &mut [i32], n: usize, isw: i32) {
    assert!(n <= PERMUT_MAX_N, "PERMUT: N={n} exceeds IB/IC dimension 32");
    assert!(ia.len() >= n, "PERMUT: IA shorter than N");
    assert!(id.len() >= n, "PERMUT: ID shorter than N");

    // IB(32), IC(32) — the Fortran work arrays, 1-based in the source.
    let mut ib = [0i32; PERMUT_MAX_N];
    let mut ic = [0i32; PERMUT_MAX_N];

    // DO 10 I=1,N / IC(I)=IA(I) / IB(I)=I
    for i in 1..=n {
        ic[i - 1] = ia[i - 1];
        ib[i - 1] = i as i32;
    }

    // N1=N-1 ; DO 20 I=1,N1 ; I1=I+1 ; DO 30 J=I1,N
    let n1 = n.saturating_sub(1);
    for i in 1..=n1 {
        let i1 = i + 1;
        for j in i1..=n {
            // IF(IC(J)-IC(I)) 40,30,30 — swap only when strictly less.
            if ic[j - 1] - ic[i - 1] < 0 {
                // 40: swap IB(J)<->IB(I), then IC(J)<->IC(I).
                ib.swap(j - 1, i - 1);
                ic.swap(j - 1, i - 1);
            }
        }
    }

    // DO 50 I=1,N ; IF(IC(I)-ISW) 50,60,60
    // First I with IC(I) >= ISW, or fall through to `K=1; GO TO 71`.
    let mut start: Option<usize> = None;
    for i in 1..=n {
        if ic[i - 1] - isw >= 0 {
            start = Some(i);
            break;
        }
    }

    // K is assigned by the DO 70 loop at label 60; the fall-through path sets
    // it to 1 before jumping to label 71.
    let mut k: usize = 1;
    if let Some(i) = start {
        // 60: DO 70 J=I,N ; K=J-I+1 ; ID(K)=IB(J)
        for j in i..=n {
            k = j - i + 1;
            id[k - 1] = ib[j - 1];
        }
        // IF(K .EQ. N) GO TO 90
        if k == n {
            return;
        }
        // K=K+1
        k += 1;
    }

    // 71: DO 80 J=K,N ; L=J-K+1 ; ID(J)=IB(L)
    for j in k..=n {
        let l = j - k + 1;
        id[j - 1] = ib[l - 1];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Emits the same fixed-format text as `parity/drivers/permut.f`
    /// (`FORMAT(A5,I3,10I4)`).
    fn driver_output() -> String {
        let mut out = String::new();

        fn case(out: &mut String, tag: &str, ia: &[i32], isw: i32) {
            let n = ia.len();
            let mut id = [0i32; 10];
            permut(ia, &mut id, n, isw);
            out.push_str(&format!("{:<5}{:>3}", tag, n));
            for v in &id[..n] {
                out.push_str(&format!("{:>4}", v));
            }
            out.push('\n');
        }

        // CASE 1: distinct values, ISW equal to the minimum.
        case(&mut out, "CASE1", &[5, 3, 9, 1, 7, 2], 1);
        // CASE 2: duplicate values -- stable tie-break.
        case(&mut out, "CASE2", &[3, 1, 3, 2, 1, 3, 2], 1);
        // CASE 3: ISW below the minimum.
        case(&mut out, "CASE3", &[4, 2, 8, 6, 0], -5);
        // CASE 4: ISW above the maximum (DO 50 falls through, GOTO 71).
        case(&mut out, "CASE4", &[4, 2, 8, 6, 0], 100);
        // CASE 5: ISW equal to an interior value (rotation wraps).
        case(&mut out, "CASE5", &[4, 2, 8, 6], 6);

        out
    }

    #[test]
    fn permut_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("permut_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/permut.txt");
        assert_eq!(got, want, "PERMUT output differs from Fortran golden");
    }

    #[test]
    fn permut_tie_break_matches_fortran() {
        // Duplicate keys: the exchange sort's exact tie-break must be
        // reproduced, not an idiomatic stable sort. The pinned Fortran
        // baseline returns 2 5 4 7 3 6 1 for this input (verified by running
        // the driver against vendor/nastran95/mis/permut.f), i.e. the three
        // keys equal to 3 (original positions 1, 3, 6) come out as 3, 6, 1.
        let ia = [3, 1, 3, 2, 1, 3, 2];
        let mut id = [0i32; 10];
        permut(&ia, &mut id, ia.len(), 1);
        assert_eq!(&id[..7], &[2, 5, 4, 7, 3, 6, 1]);
    }

    #[test]
    fn permut_rotation_wraps_at_interior_key() {
        // Sorted keys 2,4,6,8 come from positions 2,1,4,3; starting at key 6
        // (position 3 of the sorted order) rotates to 4,3 then wraps 2,1.
        let ia = [4, 2, 8, 6];
        let mut id = [0i32; 10];
        permut(&ia, &mut id, ia.len(), 6);
        assert_eq!(&id[..4], &[4, 3, 2, 1]);
    }

    #[test]
    fn permut_single_element() {
        // N=1: no sort, and both rotation paths must yield ID(1)=1.
        let mut id = [0i32; 10];
        permut(&[7], &mut id, 1, 0);
        assert_eq!(id[0], 1);
        permut(&[7], &mut id, 1, 99);
        assert_eq!(id[0], 1);
    }
}
