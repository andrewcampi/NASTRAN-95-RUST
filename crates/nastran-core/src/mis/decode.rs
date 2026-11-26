//! Port of `mis/decode.f` — expand a bit word into a list of set bit positions.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE DECODE (CODE,LIST,N)
//! C
//! C     DECODE DECODES THE BITS IN A WORD AND RETURNS A LIST OF INTEGERS
//! C     CORRESPONDING TO THE BIT POSITIONS WHICH ARE ON. NUMBERING
//! C     CONVENTION IS RIGHT (LOW ORDER) TO LEFT (HIGH ORDER) 00 THRU 31.
//! C
//! C     ARGUMENTS
//! C
//! C     CODE - INPUT  - THE WORD TO BE DECODED
//! C     LIST - OUTPUT - AN ARRAY OF DIMENSION .GE. 32 WHERE THE INTEGERS
//! C                     CORRESPONDING TO BIT POSITIONS ARE STORED
//! C     N    - OUTPUT - THE NUMBER OF ENTRIES IN THE LIST  I.E. THE NO.
//! C                     OF 1-BITS IN THE WORD
//! C
//! C
//!       EXTERNAL     ANDF
//!       INTEGER      CODE,ANDF,TWO,LIST(1)
//!       COMMON /TWO/ TWO(32)
//! C
//!       N = 0
//!       DO 8 I = 1,32
//!       IF (ANDF(TWO(33-I),CODE) .EQ. 0) GO TO 8
//!       N = N + 1
//!       LIST(N) = I - 1
//!     8 CONTINUE
//! C
//!       RETURN
//!       END
//! ```
//!
//! The `/TWO/` COMMON block holds the powers of two, `TWO(I) = 2**(I-1)`;
//! it is modelled here as the module-level constant [`TWO`]. `ANDF` is the
//! `IAND` entry of `mds/mapfns.f`, i.e. a plain bitwise AND on the 32-bit
//! two's-complement word.
//!
//! The scan direction is part of the contract and is preserved exactly: the
//! loop runs `I = 1..32` testing `TWO(33-I)`, so it walks from the high-order
//! bit (`I = 1`, `TWO(32)`, bit 31) down to the low-order bit (`I = 32`,
//! `TWO(1)`, bit 0). On a hit `N` is incremented and `I-1` is stored, so the
//! stored value is `31 - bit` and `LIST` comes out in descending scan order.
//! Nothing is reordered or rewritten.
//!
//! There is no I/O, a single entry point, no alternate returns and no
//! Hollerith in this module.

/// The `/TWO/` COMMON block: `TWO(I) = 2**(I-1)` for `I = 1..32`.
///
/// The top entry is the sign bit, `2**31`, which is `i32::MIN` in the
/// two's-complement word the Fortran operates on.
pub const TWO: [i32; 32] = [
    1,
    2,
    4,
    8,
    16,
    32,
    64,
    128,
    256,
    512,
    1024,
    2048,
    4096,
    8192,
    16384,
    32768,
    65536,
    131_072,
    262_144,
    524_288,
    1_048_576,
    2_097_152,
    4_194_304,
    8_388_608,
    16_777_216,
    33_554_432,
    67_108_864,
    134_217_728,
    268_435_456,
    536_870_912,
    1_073_741_824,
    i32::MIN,
];

/// Number of entries in the Fortran `LIST` dummy argument's declared
/// dimension (`.GE. 32`).
pub const LIST_LEN: usize = 32;

/// Port of `SUBROUTINE DECODE (CODE,LIST,N)`.
///
/// Decodes the bits of `code` and stores, for every set bit, `I-1` into
/// `list` in scan order, returning the number of set bits (`N`).
///
/// The scan is `I = 1..32` testing `TWO[33-I]` against `code`; the first hit
/// is the highest-order set bit and lands in `list[0]`. Only the first `N`
/// entries of `list` are written, exactly as in the Fortran.
#[inline]
pub fn decode(code: i32, list: &mut [i32; LIST_LEN]) -> i32 {
    // N = 0
    let mut n: i32 = 0;

    // DO 8 I = 1,32
    for i in 1..=32i32 {
        // IF (ANDF(TWO(33-I),CODE) .EQ. 0) GO TO 8
        // TWO is 1-based in the Fortran, so TWO(33-I) is TWO[(33-I)-1].
        let two = TWO[(33 - i - 1) as usize];
        if two & code == 0 {
            continue;
        }
        // N = N + 1
        n += 1;
        // LIST(N) = I - 1
        list[(n - 1) as usize] = i - 1;
    }

    n
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Emits the same fixed-format text as `parity/drivers/decode.f`.
    fn driver_output() -> String {
        let mut out = String::new();

        // The driver's cases, in order: no bits, a single low bit, a single
        // high bit (bit 31), the two alternating patterns, every bit set, and
        // a mixed pattern with both end bits and a gap in the middle.
        let cases: [(&str, i32); 7] = [
            ("CASE1", 0),
            ("CASE2", 1),
            ("CASE3", i32::MIN),
            ("CASE4", 1_431_655_765),
            ("CASE5", -1_431_655_766),
            ("CASE6", -1),
            ("CASE7", i32::MIN | 1056),
        ];

        for (tag, code) in cases {
            let mut list = [0i32; LIST_LEN];
            let n = decode(code, &mut list);
            // WRITE(6,900) TAG, CODE, N   with   FORMAT(A5,' CODE=',I12,' N=',I4)
            out.push_str(&format!("{:<5} CODE={:>12} N={:>4}\n", tag, code, n));
            // WRITE(6,901) J, LIST(J)   with   FORMAT(4X,'LIST(',I2,')=',I4)
            for j in 1..=n {
                out.push_str(&format!(
                    "    LIST({:>2})={:>4}\n",
                    j,
                    list[(j - 1) as usize]
                ));
            }
        }

        out
    }

    #[test]
    fn decode_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("decode_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/decode.txt");
        assert_eq!(got, want, "DECODE output differs from Fortran golden");
    }

    #[test]
    fn decode_two_table_is_powers_of_two() {
        for (i, &t) in TWO.iter().enumerate() {
            assert_eq!(t, 1i32.wrapping_shl(i as u32), "TWO({})", i + 1);
        }
    }

    #[test]
    fn decode_scan_order_is_high_to_low() {
        // A single low bit is found last, at I = 32, so LIST(1) = 31.
        let mut list = [0i32; LIST_LEN];
        assert_eq!(decode(1, &mut list), 1);
        assert_eq!(list[0], 31);

        // A single high bit is found first, at I = 1, so LIST(1) = 0.
        let mut list = [0i32; LIST_LEN];
        assert_eq!(decode(i32::MIN, &mut list), 1);
        assert_eq!(list[0], 0);

        // No bits set: N is 0 and LIST is left untouched.
        let mut list = [-7i32; LIST_LEN];
        assert_eq!(decode(0, &mut list), 0);
        assert!(list.iter().all(|&v| v == -7));

        // Every bit set: N is 32 and LIST(J) = J-1.
        let mut list = [0i32; LIST_LEN];
        assert_eq!(decode(-1, &mut list), 32);
        for (j, &v) in list.iter().enumerate() {
            assert_eq!(v, j as i32);
        }
    }
}
