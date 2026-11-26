//! Port of `mis/scalex.f` — `SUBROUTINE SCALEX(ILVAL,CODE,L)`.
//!
//! Fortran source (oracle, `vendor/nastran95/mis/scalex.f`):
//!
//! ```fortran
//!       SUBROUTINE SCALEX(ILVAL,CODE,L)
//!       INTEGER L(1),CODE,EXPND(6)
//!       DO 101 I=1,6
//!   101 L(I)=0
//!       IF(CODE) 102,102,103
//!   102 L(1) = ILVAL
//!       GO TO 110
//!   103 ID=CODE
//!       DO 104 I=1,6
//!       INV=7-I
//!       EXPND(INV)=MOD(ID,10)
//!   104 ID=ID/10
//!       J=0
//!       DO 107 I=1,6
//!       IF(EXPND(I).EQ.0) GO TO 107
//!       IF(I.LT.2) GO TO 106
//!       II=I-1
//!       DO 105 K=1,II
//!       IF(EXPND(K).EQ.EXPND(I)) GO TO 107
//!  105 CONTINUE
//!  106 J=J+1
//!       L(J)=EXPND(I)
//!  107 CONTINUE
//!       I=0
//!  108 I=I+1
//!       L(I)=ILVAL+L(I)-1
//!       IF(I-J) 108,110,110
//!  110 RETURN
//!       END
//! ```
//!
//! Behaviour, reproduced exactly:
//!   * `L(1..6)` is zeroed first;
//!   * the arithmetic IF `IF(CODE) 102,102,103` is a **three-way** branch
//!     whose first two arms are the same label, so `CODE < 0` and `CODE = 0`
//!     both take the `L(1) = ILVAL` path and leave `L(2..6)` zero. This is
//!     the documented trap: a naive `if code > 0 { ... } else { ... }` is
//!     equivalent here, but the port keeps the three-way shape explicit so
//!     the negative/zero collapse is visible and pinned by the unit test;
//!   * for `CODE > 0` the six decimal digits are split into `EXPND(1..6)`
//!     most-significant first, the first occurrence of each nonzero digit is
//!     kept in order into `L(1..J)`, and then `L(I) = ILVAL + L(I) - 1` for
//!     `I = 1..J`.
//!
//! Two further details are preserved because they are observable:
//!   * the `108` loop is a `DO`-style bottom-tested loop (`I=0` then
//!     `I=I+1` before the first store), so it always executes at least once.
//!     When `J = 0` — every digit of `CODE` is zero, e.g. `CODE = 1000000` —
//!     the Fortran still stores `L(1) = ILVAL + 0 - 1 = ILVAL - 1`;
//!   * `MOD`/`/` on the positive `CODE` are truncating integer ops, which
//!     Rust's `%` and `/` match.
//!
//! The Fortran declares `L(1)` but writes `L(1..6)`; the caller must supply
//! six words, so the Rust signature takes `&mut [i32; 6]`.

/// Decode the expansion code `code` into `l[0..6]`, offset by `ilval`.
///
/// Mirrors `SUBROUTINE SCALEX(ILVAL,CODE,L)`; `l` is the Fortran `L(1..6)`.
pub fn scalex(ilval: i32, code: i32, l: &mut [i32; 6]) {
    // DO 101 I=1,6 / 101 L(I)=0
    for v in l.iter_mut() {
        *v = 0;
    }

    // IF(CODE) 102,102,103 -- arithmetic IF. Negative and zero share the
    // first arm (label 102); only strictly positive CODE reaches 103.
    match code {
        c if c < 0 => {
            // 102 L(1) = ILVAL
            l[0] = ilval;
            return;
        }
        0 => {
            // 102 L(1) = ILVAL  (same label as the negative arm)
            l[0] = ilval;
            return;
        }
        _ => {
            // 103 ID=CODE
        }
    }

    // DO 104 I=1,6 / INV=7-I / EXPND(INV)=MOD(ID,10) / ID=ID/10
    // Most significant digit first: INV runs 6,5,4,3,2,1.
    let mut id = code;
    let mut expnd = [0i32; 6];
    for i in 0..6 {
        // INV = 7-I in 1-based terms, i.e. 0-based index 5-i: the least
        // significant digit lands in EXPND(6) and the most significant in
        // EXPND(1).
        let inv = 5 - i;
        expnd[inv] = id % 10;
        id /= 10;
    }

    // DO 107 I=1,6 -- keep the first occurrence of each nonzero digit.
    let mut j: usize = 0;
    for i in 0..6 {
        if expnd[i] == 0 {
            continue; // IF(EXPND(I).EQ.0) GO TO 107
        }
        if i >= 1 {
            // II=I-1 / DO 105 K=1,II -- already seen?
            let mut seen = false;
            for k in 0..i {
                if expnd[k] == expnd[i] {
                    seen = true;
                    break;
                }
            }
            if seen {
                continue; // GO TO 107
            }
        }
        // 106 J=J+1 / L(J)=EXPND(I)
        l[j] = expnd[i];
        j += 1;
    }

    // I=0 / 108 I=I+1 / L(I)=ILVAL+L(I)-1 / IF(I-J) 108,110,110
    // Bottom-tested: at least one store, then continue while I < J.
    let mut i: usize = 0;
    loop {
        l[i] = ilval + l[i] - 1;
        i += 1;
        if i >= j {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Formats one case exactly like the driver's `FORMAT(A5,6I6)`:
    /// the tag left-justified in 5 columns, then six integers in width 6.
    fn line(tag: &str, l: &[i32; 6]) -> String {
        let mut s = format!("{:<5}", tag);
        for v in l {
            s.push_str(&format!("{:>6}", v));
        }
        s
    }

    /// Emits the same fixed-format text as `parity/drivers/scalex.f`.
    fn driver_output() -> String {
        let mut out = String::new();
        let cases: [(&str, i32, i32); 5] = [
            // CODE < 0: arithmetic IF sends it to the L(1)=ILVAL arm.
            ("CASE1", 10, -3),
            // CODE = 0: the middle arm, same branch as CASE1.
            ("CASE2", 10, 0),
            // Six distinct nonzero digits: J = 6.
            ("CASE3", 10, 123456),
            // Repeated digits: only first occurrences kept, J = 3.
            ("CASE4", 10, 112233),
            // Zeros interleaved with nonzero digits: J = 3.
            ("CASE5", 10, 102030),
        ];
        for (tag, ilval, code) in cases {
            let mut l = [0i32; 6];
            scalex(ilval, code, &mut l);
            out.push_str(&line(tag, &l));
            out.push('\n');
        }
        out
    }

    #[test]
    fn scalex_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("scalex_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/scalex.txt");
        assert_eq!(got, want, "SCALEX output differs from Fortran golden");
    }

    #[test]
    fn scalex_negative_and_zero_take_the_same_branch() {
        // The documented trap: CODE < 0 and CODE = 0 both give L(1) = ILVAL.
        let mut neg = [7i32; 6];
        let mut zero = [7i32; 6];
        scalex(10, -3, &mut neg);
        scalex(10, 0, &mut zero);
        assert_eq!(neg, [10, 0, 0, 0, 0, 0]);
        assert_eq!(neg, zero);
    }

    #[test]
    fn scalex_all_zero_digits_still_stores_l1() {
        // J = 0 path: the bottom-tested 108 loop runs once, so
        // L(1) = ILVAL + 0 - 1 = ILVAL - 1.
        let mut l = [0i32; 6];
        scalex(10, 1_000_000, &mut l);
        assert_eq!(l, [9, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn scalex_keeps_first_occurrence_only() {
        let mut l = [0i32; 6];
        scalex(10, 112233, &mut l);
        assert_eq!(l, [10, 11, 12, 0, 0, 0]);
        scalex(10, 102030, &mut l);
        assert_eq!(l, [10, 11, 12, 0, 0, 0]);
        scalex(10, 123456, &mut l);
        assert_eq!(l, [10, 11, 12, 13, 14, 15]);
    }
}
