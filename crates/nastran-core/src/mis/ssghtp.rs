//! Port of `mis/ssghtp.f` — special in-place partitioning used only by SSGHT.
//!
//! The Fortran source is:
//!
//! ```text
//!       SUBROUTINE SSGHTP(ORDER,Z,LZ)
//! C*****
//! C  SPECIAL IN-PLACE PARTITIONING ROUTINE USED ONLY BY SSGHT MODULE.
//! C*****
//!       INTEGER ORDER(LZ), Z(LZ), SAVE1, SAVE2, PTR
//!       I = 1
//!       ISAVE = 1
//! C
//! C     CHECK TO SEE THAT POINTER TO NEXT SLOT HAS NOT BEEN USED YET.
//! C
//!    10 PTR = ORDER(I)
//!       IF( PTR .GT. 1000000 ) GO TO 40
//!       ORDER(I) = PTR + 1000000
//! C
//! C     IF THE MOVE-TO LOCATION IS THE SAME, THEN DO NOTHING.
//! C
//!       IF( PTR .EQ. I ) GO TO 40
//! C
//! C     SAVE VALUE CURRENTLY IN SLOT WE ARE MOVING TO.
//! C
//!       SAVE1 = Z(PTR)
//! C
//! C     MOVE ITEM INTO SLOT
//! C
//!       Z(PTR) = Z(I)
//! C
//! C     SET POINTER TO WHERE -SAVE1- IS TO BE MOVED.
//! C
//!    20 JPTR = ORDER(PTR)
//!       IF( JPTR .GT. 1000000 ) GO TO 30
//!       ORDER(PTR) = JPTR + 1000000
//!       SAVE2 = Z(JPTR)
//!       Z(JPTR) = SAVE1
//!       SAVE1 = SAVE2
//!       PTR = JPTR
//!       GO TO 20
//! C
//! C     END OF CHAIN.  GO BACK AND LOOK FOR ANOTHER.
//! C
//!    30 I = ISAVE + 1
//!       GO TO 50
//!    40 I = I + 1
//!    50 ISAVE = I
//!       IF( I .LE. LZ ) GO TO 10
//! C
//! C     CLEAR OUT FLAGS AND RETURN.
//! C
//!       DO 60 I = 1,LZ
//!       ORDER(I) = ORDER(I) - 1000000
//!    60 CONTINUE
//!       RETURN
//!       END
//! ```
//!
//! `ORDER` is a permutation of `1..=LZ`; `Z` is permuted in place so that the
//! value that was in slot `I` ends up in slot `ORDER(I)`. The routine is
//! *not* a straightforward cycle walk, and three details are load-bearing:
//!
//! * Visited slots are marked by adding the literal sentinel `1000000` to the
//!   `ORDER` entry, and "already visited" is tested with a strict
//!   `PTR .GT. 1000000` comparison. The sentinel value and the strictness of
//!   the comparison are reproduced verbatim.
//! * The outer scan restarts at `I = ISAVE + 1` when a chain ends (label 30)
//!   and at `I = I + 1` when a slot was already flagged or is a fixed point
//!   (label 40). `ISAVE` is refreshed at label 50 on every iteration, so both
//!   paths advance by one, but the two labels are kept distinct here so the
//!   control flow stays recognisable against the oracle.
//! * The flags are cleared by a separate final `DO 60` loop that subtracts the
//!   sentinel from *every* slot, including slots that were never marked. The
//!   clear loop runs after the scan terminates, never inside it.
//!
//! The only deviation from the Fortran is a guard for `LZ = 0`: the oracle
//! would read `ORDER(1)` out of bounds there, which is undefined behaviour, so
//! this port returns immediately instead.

/// Sentinel added to `ORDER` entries to flag a visited slot (`1000000` in the
/// Fortran). The exact value is part of the observable behaviour.
pub const SSGHTP_FLAG: i32 = 1_000_000;

/// `SUBROUTINE SSGHTP(ORDER,Z,LZ)`.
///
/// Applies the permutation `order[0..lz]` (1-based values) to `z[0..lz]` in
/// place, using the `1000000` sentinel to mark visited slots and clearing every
/// flag before returning. `order` and `z` must each hold at least `lz`
/// elements.
pub fn ssghtp(order: &mut [i32], z: &mut [i32], lz: usize) {
    assert!(order.len() >= lz, "SSGHTP: ORDER shorter than LZ");
    assert!(z.len() >= lz, "SSGHTP: Z shorter than LZ");

    // The Fortran reads ORDER(1) unconditionally; with LZ=0 that is out of
    // bounds, so bail out instead of reproducing undefined behaviour.
    if lz == 0 {
        return;
    }

    // I = 1 ; ISAVE = 1
    let mut i: usize = 1;
    let mut isave: usize = 1;

    loop {
        // 10 PTR = ORDER(I)
        let ptr_val = order[i - 1];
        if ptr_val > SSGHTP_FLAG {
            // IF( PTR .GT. 1000000 ) GO TO 40
            i += 1;
        } else {
            // ORDER(I) = PTR + 1000000
            order[i - 1] = ptr_val + SSGHTP_FLAG;

            if ptr_val == i as i32 {
                // IF( PTR .EQ. I ) GO TO 40 — fixed point, do nothing.
                i += 1;
            } else {
                // SAVE1 = Z(PTR) ; Z(PTR) = Z(I)
                let mut ptr = ptr_val as usize;
                let mut save1 = z[ptr - 1];
                z[ptr - 1] = z[i - 1];

                // 20 JPTR = ORDER(PTR) — follow the chain to its end.
                loop {
                    let jptr_val = order[ptr - 1];
                    if jptr_val > SSGHTP_FLAG {
                        // IF( JPTR .GT. 1000000 ) GO TO 30
                        // 30 I = ISAVE + 1
                        i = isave + 1;
                        break;
                    }
                    // ORDER(PTR) = JPTR + 1000000
                    order[ptr - 1] = jptr_val + SSGHTP_FLAG;
                    let jptr = jptr_val as usize;
                    // SAVE2 = Z(JPTR) ; Z(JPTR) = SAVE1 ; SAVE1 = SAVE2
                    let save2 = z[jptr - 1];
                    z[jptr - 1] = save1;
                    save1 = save2;
                    // PTR = JPTR ; GO TO 20
                    ptr = jptr;
                }
            }
        }

        // 50 ISAVE = I ; IF( I .LE. LZ ) GO TO 10
        isave = i;
        if i > lz {
            break;
        }
    }

    // DO 60 I = 1,LZ ; ORDER(I) = ORDER(I) - 1000000
    for slot in order.iter_mut().take(lz) {
        *slot -= SSGHTP_FLAG;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Emits the same fixed-format text as `parity/drivers/ssghtp.f`
    /// (`FORMAT(A5,I3,8I4)`).
    fn driver_output() -> String {
        let mut out = String::new();

        fn case(out: &mut String, tag: &str, ztag: &str, order: &mut [i32], z: &mut [i32]) {
            let lz = order.len();
            ssghtp(order, z, lz);
            out.push_str(&format!("{:<5}{:>3}", tag, lz));
            for v in order.iter() {
                out.push_str(&format!("{:>4}", v));
            }
            out.push('\n');
            out.push_str(&format!("{:<5}{:>3}", ztag, lz));
            for v in z.iter() {
                out.push_str(&format!("{:>4}", v));
            }
            out.push('\n');
        }

        // CASE 1: one 4-hop chain (1->3->2->5->1), one 3-hop chain
        // (4->7->8->4) and one fixed point (6->6).
        let mut order = [3, 5, 2, 7, 1, 6, 8, 4];
        let mut z = [10, 20, 30, 40, 50, 60, 70, 80];
        case(&mut out, "CASE1", "Z1", &mut order, &mut z);

        // CASE 2: first slot is a fixed point; chains are entered from the
        // middle (2->8->3->2 and 4->5->7->6->4).
        let mut order = [1, 8, 2, 5, 7, 4, 6, 3];
        let mut z = [-1, -2, -3, -4, -5, -6, -7, -8];
        case(&mut out, "CASE2", "Z2", &mut order, &mut z);

        out
    }

    #[test]
    fn ssghtp_parity() {
        let got = driver_output();
        // Leave the generated text where the parity diff can pick it up.
        let _ = std::fs::write(std::env::temp_dir().join("ssghtp_rust.out"), &got);
        let want = include_str!("../../../../parity/golden/ssghtp.txt");
        assert_eq!(got, want, "SSGHTP output differs from Fortran golden");
    }

    #[test]
    fn ssghtp_permutes_z_along_order() {
        // ORDER(I) is the destination of the value originally in slot I, so
        // Z(ORDER(I)) must receive the original Z(I) for every I.
        let order = [3, 5, 2, 7, 1, 6, 8, 4];
        let z0 = [10, 20, 30, 40, 50, 60, 70, 80];
        let mut order_mut = order;
        let mut z = z0;
        ssghtp(&mut order_mut, &mut z, 8);
        for i in 0..8 {
            assert_eq!(z[order[i] as usize - 1], z0[i], "slot {i} misrouted");
        }
        // ORDER itself is restored to the input permutation (flags cleared).
        assert_eq!(order_mut, order);
    }

    #[test]
    fn ssghtp_fixed_point_is_untouched() {
        // A permutation of fixed points must leave Z alone and clear all flags.
        let mut order = [1, 2, 3, 4];
        let mut z = [7, 8, 9, 10];
        ssghtp(&mut order, &mut z, 4);
        assert_eq!(order, [1, 2, 3, 4]);
        assert_eq!(z, [7, 8, 9, 10]);
    }

    #[test]
    fn ssghtp_single_element() {
        let mut order = [1];
        let mut z = [42];
        ssghtp(&mut order, &mut z, 1);
        assert_eq!(order, [1]);
        assert_eq!(z, [42]);
    }

    #[test]
    fn ssghtp_empty_is_noop() {
        let mut order: [i32; 0] = [];
        let mut z: [i32; 0] = [];
        ssghtp(&mut order, &mut z, 0);
    }
}
