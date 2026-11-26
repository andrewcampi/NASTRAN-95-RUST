//! Port of `mis/fornum.f` — integer-field scan over a `CHARACTER*1` form buffer.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE FORNUM ( FORM, ICHAR, IMULT )
//! C
//! C THIS SUBROUTINE CONVERTS ALL NUMBERS TO INTEGER FORMAT
//! C
//!       CHARACTER*1   FORM(200), BLANK, NUMBER(2)
//!       DATA          BLANK /' ' /
//!       DATA          NUMBER /'0','9'/
//!       IMULT = 0
//! 10    IF ( FORM( ICHAR ) .NE. BLANK      ) GO TO 20
//!       ICHAR = ICHAR + 1
//!       GO TO 10
//! 20    IF ( FORM( ICHAR ) .LT. NUMBER(1) .OR.
//!      &     FORM( ICHAR ) .GT. NUMBER(2) ) GO TO 700
//!       READ ( FORM( ICHAR ), 901 ) II
//! 901   FORMAT(I1)
//!       IMULT = IMULT*10 + II
//!       ICHAR = ICHAR + 1
//!       GO TO 20
//! 700   CONTINUE
//!       RETURN
//!       END
//! ```
//!
//! The scan order is preserved exactly: `IMULT` is zeroed first, blanks are
//! skipped one at a time from `ICHAR`, then digits are accumulated left to
//! right as `IMULT = IMULT*10 + digit` with `ICHAR` advanced after each digit.
//! The routine returns with `ICHAR` pointing at the first non-digit character
//! (or at the first non-blank when the field is empty). `ICHAR` is both an
//! input and an output, so it is taken by `&mut`.
//!
//! `ICHAR` is 1-based, exactly as in the Fortran, so the returned value is
//! directly comparable with the oracle. The internal `READ ( FORM( ICHAR ), 901 )`
//! with `FORMAT(I1)` is a plain character-to-integer conversion of a single
//! digit; there is no trap or error path in this module.

/// Number of `CHARACTER*1` elements in the Fortran `FORM` dummy argument.
pub const FORM_LEN: usize = 200;

/// Port of `SUBROUTINE FORNUM ( FORM, ICHAR, IMULT )`.
///
/// Starting at the 1-based position `*ichar`, skips `CHARACTER*1` blanks in
/// `form`, then accumulates consecutive digit characters into
/// `*imult = *imult*10 + digit`, advancing `*ichar` after each digit. Stops at
/// the first non-digit and returns with `*ichar` pointing at it. `*imult` is
/// set to 0 on entry, as in the Fortran.
///
/// # Panics
///
/// Panics if the scan would read outside `form`, which is the same
/// out-of-bounds access the Fortran would perform on `FORM(200)`.
#[inline]
pub fn fornum(form: &[u8; FORM_LEN], ichar: &mut i32, imult: &mut i32) {
    // IMULT = 0
    *imult = 0;

    // 10: skip blanks, one at a time.
    while form[fortran_index(*ichar)] == b' ' {
        *ichar += 1;
    }

    // 20: accumulate digits until the first non-digit (700).
    loop {
        let c = form[fortran_index(*ichar)];
        // IF ( FORM(ICHAR) .LT. '0' .OR. FORM(ICHAR) .GT. '9' ) GO TO 700
        if c < b'0' || c > b'9' {
            break;
        }
        // READ ( FORM( ICHAR ), 901 ) II  with  FORMAT(I1)
        let ii = i32::from(c - b'0');
        // IMULT = IMULT*10 + II
        *imult = *imult * 10 + ii;
        // ICHAR = ICHAR + 1
        *ichar += 1;
    }
}

/// Converts the 1-based Fortran subscript `ichar` to a 0-based Rust index.
#[inline]
fn fortran_index(ichar: i32) -> usize {
    (ichar - 1) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Blanks out the whole 200-character form buffer, like the driver's FILL.
    fn fill() -> [u8; FORM_LEN] {
        [b' '; FORM_LEN]
    }

    /// Emits the same fixed-format text as `parity/drivers/fornum.f`.
    fn driver_output() -> String {
        let mut out = String::new();

        // CASE 1: leading blanks, then a multi-digit field terminated by a
        // non-digit. ICHAR must come back pointing at the terminator.
        let mut form = fill();
        form[0] = b' ';
        form[1] = b' ';
        form[2] = b' ';
        form[3] = b'1';
        form[4] = b'2';
        form[5] = b'3';
        form[6] = b'X';
        let mut ichar = 1;
        let mut imult = 0;
        fornum(&form, &mut ichar, &mut imult);
        out.push_str(&format!("{:<5}{:>8}{:>8}\n", "CASE1", imult, ichar));

        // CASE 2: multi-digit field starting at the first character, with no
        // leading blanks at all.
        let mut form = fill();
        form[0] = b'9';
        form[1] = b'8';
        form[2] = b'7';
        form[3] = b'6';
        form[4] = b'5';
        form[5] = b' ';
        let mut ichar = 1;
        let mut imult = 0;
        fornum(&form, &mut ichar, &mut imult);
        out.push_str(&format!("{:<5}{:>8}{:>8}\n", "CASE2", imult, ichar));

        // CASE 3: zero-length field -- the very first character is already a
        // non-digit, so IMULT stays 0 and ICHAR is unchanged.
        let mut form = fill();
        form[0] = b'A';
        form[1] = b'4';
        form[2] = b'2';
        let mut ichar = 1;
        let mut imult = 0;
        fornum(&form, &mut ichar, &mut imult);
        out.push_str(&format!("{:<5}{:>8}{:>8}\n", "CASE3", imult, ichar));

        // CASE 4: field at the end of the buffer. The digits run up to the
        // last element FORM(200), which is the non-digit terminator, so the
        // scan never reads past the end of FORM.
        let mut form = fill();
        form[195] = b'1';
        form[196] = b'2';
        form[197] = b'3';
        form[198] = b'4';
        form[199] = b'Z';
        let mut ichar = 196;
        let mut imult = 0;
        fornum(&form, &mut ichar, &mut imult);
        out.push_str(&format!("{:<5}{:>8}{:>8}\n", "CASE4", imult, ichar));

        out
    }

    #[test]
    fn fornum_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("fornum_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/fornum.txt");
        assert_eq!(got, want, "FORNUM output differs from Fortran golden");
    }

    #[test]
    fn fornum_ichar_is_input_and_output() {
        // Starting mid-buffer: the scan begins at the given ICHAR, not at 1.
        let mut form = fill();
        form[9] = b'4';
        form[10] = b'2';
        form[11] = b' ';
        let mut ichar = 10;
        let mut imult = 0;
        fornum(&form, &mut ichar, &mut imult);
        assert_eq!(imult, 42);
        assert_eq!(ichar, 12);

        // IMULT is zeroed on entry, not accumulated onto the caller's value.
        let mut form = fill();
        form[0] = b'7';
        form[1] = b' ';
        let mut ichar = 1;
        let mut imult = 999;
        fornum(&form, &mut ichar, &mut imult);
        assert_eq!(imult, 7);
        assert_eq!(ichar, 2);
    }
}
