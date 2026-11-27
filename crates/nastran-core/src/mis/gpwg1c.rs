//! Port of `mis/gpwg1c.f` — double-precision 3x3 Jacobi eigensolver.
//!
//! Literal transcription of the Fortran oracle:
//!
//! ```text
//!       SUBROUTINE GPWG1C  (B,E,EIG,IFLAG)
//!       DOUBLE PRECISION  E(3,3),EP(3,3),B(3,3),BP(3,3),EIG(3)
//!       DOUBLE PRECISION  DETB,EPSIL,BMAX,R,S,C,T
//!       DETB = 0.0D0
//!       DO 5 I = 1,3
//!       DO 5 J = 1,3
//!       DETB = DETB+B(I,J)*B(I,J)
//!     5 CONTINUE
//!       EPSIL = DSQRT(DETB)*1.0D-5
//!       IFLAG =0
//!       II = 1
//!       ... (identity E, DETB .EQ. 0 early exit, 21-iteration cap) ...
//! ```
//!
//! Everything stays in `f64`; the pivot selection order (`B(1,2)`, then
//! `B(1,3)`, then `B(2,3)`), the three `R`-magnitude branches, the `BP`/`EP`
//! update formulas and the copy-back loop are preserved verbatim. No FMA, no
//! fast-math, no reordering, no algebraic rewrites, no state.

/// Largest off-diagonal magnitude of the working matrix.
#[inline]
fn bmax_of(b: &[[f64; 3]; 3]) -> f64 {
    // DMAX1(DABS(B(1,2)),DABS(B(1,3)),DABS(B(2,3)))
    let a = b[0][1].abs();
    let c = b[0][2].abs();
    let d = b[1][2].abs();
    let m = if a > c { a } else { c };
    if m > d {
        m
    } else {
        d
    }
}

/// Double-precision 3x3 Jacobi eigensolver, `GPWG1C(B,E,EIG,IFLAG)`.
///
/// `b` is the symmetric 3x3 matrix and is overwritten with the diagonalised
/// matrix; `e` receives the accumulated rotation matrix; `eig` receives the
/// eigenvalues; `iflag` is 0 on success and 1 when no solution was found
/// within 21 iterations.
pub fn gpwg1c(
    b: &mut [[f64; 3]; 3],
    e: &mut [[f64; 3]; 3],
    eig: &mut [f64; 3],
    iflag: &mut i32,
) {
    // DETB = sum of squares of all nine B entries, in Fortran column-major
    // loop order (I outer, J inner).
    let mut detb = 0.0_f64;
    for i in 0..3 {
        for j in 0..3 {
            detb = detb + b[i][j] * b[i][j];
        }
    }
    let epsil = detb.sqrt() * 1.0e-5;
    *iflag = 0;
    let mut ii: i32 = 1;

    for i in 0..3 {
        for j in 0..3 {
            e[i][j] = 0.0;
            if i == j {
                e[i][j] = 1.0;
            }
        }
    }

    if detb == 0.0 {
        // GO TO 100
        for i in 0..3 {
            eig[i] = b[i][i];
        }
        return;
    }

    let mut bp = [[0.0_f64; 3]; 3];
    let mut ep = [[0.0_f64; 3]; 3];

    loop {
        // 15
        let bmax = bmax_of(b);
        if bmax.abs() < epsil {
            // GO TO 100
            for i in 0..3 {
                eig[i] = b[i][i];
            }
            return;
        }

        // Pivot selection: B(1,2), then B(1,3), then B(2,3).
        let (i, j, k);
        if bmax != b[0][1].abs() {
            // GO TO 20
            if bmax != b[0][2].abs() {
                // GO TO 30
                i = 1;
                j = 2;
                k = 0;
            } else {
                i = 0;
                j = 2;
                k = 1;
            }
        } else {
            i = 0;
            j = 1;
            k = 2;
        }

        // 40
        let r = (b[j][j] - b[i][i]) / b[i][j];
        let (s, c);
        if r.abs() < 1.0e-6 {
            // 50
            s = 0.5_f64.sqrt();
            c = s;
        } else if r.abs() > 1.0e+6 {
            // 60
            s = 0.0;
            c = 1.0;
        } else {
            let t = ((r * r) / 4.0 + 1.0).sqrt() - 0.5 * r;
            let cc = (1.0 + t * t).sqrt();
            s = t / cc;
            c = 1.0 / cc;
        }

        // 70
        bp[i][i] = b[i][i] * c * c + b[j][j] * s * s - 2.0 * b[i][j] * s * c;
        bp[j][j] = b[i][i] * s * s + b[j][j] * c * c + 2.0 * b[i][j] * s * c;
        bp[k][k] = b[k][k];
        bp[j][i] = 0.0;
        bp[i][j] = 0.0;
        bp[k][i] = b[i][k] * c - b[j][k] * s;
        bp[i][k] = bp[k][i];
        bp[k][j] = b[j][k] * c + b[i][k] * s;
        bp[j][k] = bp[k][j];

        ep[i][0] = e[i][0] * c - e[j][0] * s;
        ep[j][0] = e[i][0] * s + e[j][0] * c;
        ep[k][0] = e[k][0];
        ep[i][1] = e[i][1] * c - e[j][1] * s;
        ep[j][1] = e[i][1] * s + e[j][1] * c;
        ep[k][1] = e[k][1];
        ep[i][2] = e[i][2] * c - e[j][2] * s;
        ep[j][2] = e[i][2] * s + e[j][2] * c;
        ep[k][2] = e[k][2];

        for i in 0..3 {
            for j in 0..3 {
                b[i][j] = bp[i][j];
                e[i][j] = ep[i][j];
            }
        }

        if ii >= 21 {
            // GO TO 90
            *iflag = 1;
            return;
        }
        ii = ii + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::gpwg1c;

    /// Reproduce Fortran `ES24.16E3`: 16 fractional digits, 3-digit signed
    /// exponent, right-justified in a 24-character field.
    fn es24_16e3(v: f64) -> String {
        let s = format!("{:.16E}", v);
        let (mantissa, exp) = s.split_once('E').expect("exponent marker");
        let exp: i32 = exp.parse().expect("exponent value");
        format!("{:>24}", format!("{}E{:+04}", mantissa, exp))
    }

    /// Emit exactly the text produced by `parity/drivers/gpwg1c.f`.
    fn driver_output() -> String {
        let mut out = String::new();

        let cases: [(&str, [[f64; 3]; 3]); 4] = [
            (
                "CASE1",
                [
                    [4.0, 1.0, 2.0],
                    [1.0, 3.0, 0.5],
                    [2.0, 0.5, 1.0],
                ],
            ),
            (
                "CASE2",
                [
                    [2.0, 0.0, 0.0],
                    [0.0, -3.0, 0.0],
                    [0.0, 0.0, 5.0],
                ],
            ),
            ("CASE3", [[0.0; 3]; 3]),
            (
                "CASE4",
                [
                    [2.0, 1.0, 0.0],
                    [1.0, 2.0, 0.0],
                    [0.0, 0.0, 3.0],
                ],
            ),
        ];

        for (label, mut b) in cases {
            let mut e = [[0.0_f64; 3]; 3];
            let mut eig = [0.0_f64; 3];
            let mut iflag: i32 = 0;
            gpwg1c(&mut b, &mut e, &mut eig, &mut iflag);
            out.push_str(&format!("{} IFLAG={:>2}\n", label, iflag));
            for (i, v) in eig.iter().enumerate() {
                out.push_str(&format!("{} EIG({}) = {}\n", label, i + 1, es24_16e3(*v)));
            }
            for i in 0..3 {
                for j in 0..3 {
                    out.push_str(&format!(
                        "{} E({},{}) = {}\n",
                        label,
                        i + 1,
                        j + 1,
                        es24_16e3(e[i][j])
                    ));
                }
            }
        }

        out
    }

    #[test]
    fn gpwg1c_parity() {
        let golden = include_str!("../../../../parity/golden/gpwg1c.txt");
        let got = driver_output();
        // Printed so the parity diff can be reproduced from the test run.
        print!("{}", got);
        assert_eq!(got, golden, "GPWG1C output diverges from Fortran golden");
    }
}
