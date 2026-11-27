//! Port of `mis/dipole.f` — magnetic intensity of an MDIPOLE card.
//!
//! Literal transcription of the Fortran oracle:
//!
//! ```text
//!       SUBROUTINE DIPOLE(BUF,IBUF,XX,YY,ZZ,HC1,HC2,HC3)
//!       REAL MX,MY,MZ,MIN,MAX,MXA,MYB,MZC
//!       DIMENSION  BUF(50),IBUF(50)
//!       DATA FPI/12.566371/
//!       HC1=0.
//!       HC2=0.
//!       HC3=0.
//!       ICID=IBUF(1)
//!       CX=BUF(2)
//!       CY=BUF(3)
//!       CZ=BUF(4)
//!       MX=BUF(5)
//!       MY=BUF(6)
//!       MZ=BUF(7)
//!       MIN=BUF(8)
//!       MAX=BUF(9)
//!       RMR1=SQRT((XX-CX)**2+(YY-CY)**2+(ZZ-CZ)**2)
//!       IF(MIN.LE.1.E-6)GO TO 5
//!       IF(RMR1.LT.MIN)GO TO 20
//!     5 IF(MAX.LE.1.E-6)GO TO 10
//!       IF(RMR1.GT.MAX)GO TO 20
//!    10 MXA=3.*MX*(XX-CX)
//!       MYB=3.*MY*(YY-CY)
//!       MZC=3.*MZ*(ZZ-CZ)
//!       R3=RMR1**3
//!       R5=R3*RMR1**2
//!       XNUM=(MXA+MYB+MZC)/R5
//!       HC1=-MX/R3+XNUM*(XX-CX)
//!       HC1=HC1/FPI
//!       HC2=-MY/R3+XNUM*(YY-CY)
//!       HC2=HC2/FPI
//!       HC3=-MZ/R3+XNUM*(ZZ-CZ)
//!       HC3=HC3/FPI
//!    20 RETURN
//!       END
//! ```
//!
//! Everything stays in single precision (`f32`) to match the Fortran oracle
//! bit-for-bit. Operand order and association are preserved verbatim:
//! `3.*MX*(XX-CX)` is `(3*MX)*(XX-CX)`, `RMR1**3` is `(RMR1*RMR1)*RMR1`,
//! `R5=R3*RMR1**2` is `R3*(RMR1*RMR1)`, and each component is divided by
//! `FPI` only after the full expression is formed. No FMA, no fast-math, no
//! algebraic rewrites, no `hypot`.
//!
//! `DIPOLE` is pure: it reads `BUF`/`IBUF` and the evaluation point and
//! returns the three field components. There is no COMMON or saved state.

/// `DATA FPI/12.566371/` — the literal constant from the Fortran source.
const FPI: f32 = 12.566371;

/// Magnetic intensity at `(xx, yy, zz)` due to the MDIPOLE card in `buf`.
///
/// `buf` is the 50-word card image: `buf[1..4]` are the dipole coordinates
/// `CX,CY,CZ`, `buf[4..7]` the moment `MX,MY,MZ` and `buf[7..9]` the
/// `MIN`/`MAX` band. `ibuf` is accepted for signature parity with the
/// Fortran (`ICID=IBUF(1)` is read but unused). Returns `(HC1, HC2, HC3)`,
/// which are all zero when the evaluation point falls outside the band.
pub fn dipole(buf: &[f32; 50], _ibuf: &[i32; 50], xx: f32, yy: f32, zz: f32) -> (f32, f32, f32) {
    // COORDS OF POINT DIPOLE
    let cx = buf[1];
    let cy = buf[2];
    let cz = buf[3];
    let mx = buf[4];
    let my = buf[5];
    let mz = buf[6];
    let min = buf[7];
    let max = buf[8];

    // H WILL BE COMPUTED ONLY IF DISTANCE FROM (CX,CY,CZ) TO (XX,YY,ZZ) IS
    // BETWEEN MIN AND MAX - IF MAX IS 0, COMPUTE FOR ALL POINTS GREATER
    // THAN MIN
    let dx = xx - cx;
    let dy = yy - cy;
    let dz = zz - cz;
    let rmr1 = (dx * dx + dy * dy + dz * dz).sqrt();

    if min > 1.0e-6 && rmr1 < min {
        return (0.0, 0.0, 0.0);
    }
    if max > 1.0e-6 && rmr1 > max {
        return (0.0, 0.0, 0.0);
    }

    let mxa = 3.0 * mx * dx;
    let myb = 3.0 * my * dy;
    let mzc = 3.0 * mz * dz;

    let r3 = rmr1 * rmr1 * rmr1;
    let r5 = r3 * (rmr1 * rmr1);
    let xnum = (mxa + myb + mzc) / r5;

    let hc1 = (-mx / r3 + xnum * dx) / FPI;
    let hc2 = (-my / r3 + xnum * dy) / FPI;
    let hc3 = (-mz / r3 + xnum * dz) / FPI;

    (hc1, hc2, hc3)
}

#[cfg(test)]
mod tests {
    use super::dipole;

    /// Reproduce Fortran `ES24.16E3`: 16 fractional digits, 3-digit signed
    /// exponent, right-justified in a 24-character field.
    fn es24_16e3(v: f32) -> String {
        let s = format!("{:.16E}", v as f64);
        let (mantissa, exp) = s.split_once('E').expect("exponent marker");
        let exp: i32 = exp.parse().expect("exponent value");
        format!("{:>24}", format!("{}E{:+04}", mantissa, exp))
    }

    /// Emit exactly the text produced by `parity/drivers/dipole.f`.
    ///
    /// Columns: CX CY CZ MX MY MZ MIN MAX XX YY ZZ.
    fn driver_output() -> String {
        let cases: [[f32; 11]; 8] = [
            [1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 0.0, 0.0, 4.0, 6.0, 3.0],
            [0.0, 0.0, 0.0, 1.5, -2.25, 0.75, 0.0, 0.0, 0.5, -1.25, 7.5],
            [1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 2.0, 5.0, 4.0, 6.0, 3.0],
            [1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 2.0, 5.0, 2.0, 2.0, 3.0],
            [1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 2.0, 5.0, 11.0, 2.0, 3.0],
            [1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 2.0, 5.0, 3.0, 2.0, 3.0],
            [1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 2.0, 0.0, 11.0, 2.0, 3.0],
            [1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 0.0, 5.0, 2.0, 2.0, 3.0],
        ];

        let mut out = String::new();
        for (j, c) in cases.iter().enumerate() {
            let mut buf = [0.0_f32; 50];
            let ibuf = [0_i32; 50];
            buf[1] = c[0];
            buf[2] = c[1];
            buf[3] = c[2];
            buf[4] = c[3];
            buf[5] = c[4];
            buf[6] = c[5];
            buf[7] = c[6];
            buf[8] = c[7];
            let (hc1, hc2, hc3) = dipole(&buf, &ibuf, c[8], c[9], c[10]);
            out.push_str(&format!(
                "CASE {} HC={} {} {}\n",
                j + 1,
                es24_16e3(hc1),
                es24_16e3(hc2),
                es24_16e3(hc3)
            ));
        }
        out
    }

    #[test]
    fn dipole_parity() {
        let golden = include_str!("../../../../parity/golden/dipole.txt");
        let got = driver_output();
        // Printed so the parity diff can be reproduced from the test run.
        print!("{}", got);
        assert_eq!(got, golden, "DIPOLE output diverges from Fortran golden");
    }

    #[test]
    fn dipole_gating_returns_zero_outside_band() {
        let mut buf = [0.0_f32; 50];
        let ibuf = [0_i32; 50];
        buf[1] = 1.0;
        buf[2] = 2.0;
        buf[3] = 3.0;
        buf[4] = 10.0;
        buf[5] = -20.0;
        buf[6] = 30.0;
        buf[7] = 2.0;
        buf[8] = 5.0;

        // Inside the band: non-zero field.
        let (h1, h2, h3) = dipole(&buf, &ibuf, 4.0, 6.0, 3.0);
        assert!(h1 != 0.0 && h2 != 0.0 && h3 != 0.0);

        // Too close (RMR1 = 0 < MIN) and too far (RMR1 = 9 > MAX).
        assert_eq!(dipole(&buf, &ibuf, 1.0, 2.0, 3.0), (0.0, 0.0, 0.0));
        assert_eq!(dipole(&buf, &ibuf, 10.0, 2.0, 3.0), (0.0, 0.0, 0.0));

        // MIN/MAX <= 1.E-6 disable the corresponding bound.
        buf[7] = 0.0;
        buf[8] = 0.0;
        let (h1, h2, h3) = dipole(&buf, &ibuf, 10.0, 2.0, 3.0);
        assert!(h1 != 0.0 && h2 != 0.0 && h3 != 0.0);
    }
}
