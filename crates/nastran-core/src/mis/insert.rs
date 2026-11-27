//! Port of `mis/insert.f` — scatter an IS2D8 matrix partition into open core.
//!
//! Literal transcription of the Fortran oracle:
//!
//! ```text
//!       SUBROUTINE INSERT(NCOL,NROW,NDOF,NGRID,JCORE,Z,DZ,TEMP,DTEMP,IPR)
//!       DIMENSION Z(1),TEMP(9)
//!       DOUBLE PRECISION DZ(1),DTEMP(9)
//!       IS1=NGRID*NDOF**2
//!       IZ1=IS1*(NROW-1)+NDOF*(NCOL-1)+JCORE-1
//!       IZ2=IS1*(NCOL-1)+NDOF*(NROW-1)+JCORE-1
//!       I1=IZ1
//!       I2=IZ2
//!       IF (IPR.EQ.2) GO TO 20
//!       IF(NDOF.EQ.1)GO TO 10
//!       Z(I1+1)=TEMP(1)
//!       Z(I2+1)=TEMP(1)
//!       Z(I1+2)=TEMP(2)
//!       Z(I2+25)=TEMP(2)
//!       ...
//!    10 Z(I1+1)=TEMP(1)
//!       Z(I2+1)=TEMP(1)
//!    20 IF (NDOF.EQ.1) GO TO 30
//!       DZ(I1+ 1)=DTEMP(1)
//!       DZ(I2+ 1)=DTEMP(1)
//!       ...
//!    30 DZ(I1+1)=DTEMP(1)
//!       DZ(I2+1)=DTEMP(1)
//!   100 RETURN
//!       END
//! ```
//!
//! The transpose offsets are deliberately asymmetric (`Z(I1+2)` pairs with
//! `Z(I2+25)`, `Z(I1+3)` with `Z(I2+49)`, ...) and are transcribed literally
//! below; they are *not* folded into a loop that assumes symmetry. When
//! `I1 == I2` half of the entries are written twice, which the Fortran comment
//! states is acceptable because there are no additions — the sequential
//! writes below reproduce that behaviour exactly (last write wins).
//!
//! `INSERT` is pure: it reads its arguments and writes only into `Z`/`DZ`.
//! There is no COMMON or saved state.

/// 0-based index of the Fortran slot `base + off`, where `base` is a 1-based
/// Fortran index (`I1`/`I2`) and `off` is one of the literal offsets
/// `1,2,3,25,26,27,49,50,51` from the oracle.
#[inline]
fn slot(base: i32, off: i32) -> usize {
    (base + off - 1) as usize
}

/// Insert the `TEMP`/`DTEMP` partition and its transpose into `Z`/`DZ`.
///
/// `ipr == 2` selects the double-precision branch (`DZ`/`DTEMP`), anything
/// else the single-precision branch (`Z`/`TEMP`). `ndof == 1` selects the
/// 1x1 partition, otherwise the 3x3 partition.
pub fn insert(
    ncol: i32,
    nrow: i32,
    ndof: i32,
    ngrid: i32,
    jcore: i32,
    z: &mut [f32],
    dz: &mut [f64],
    temp: &[f32; 9],
    dtemp: &[f64; 9],
    ipr: i32,
) {
    // IS1=NGRID*NDOF**2
    let is1 = ngrid * ndof * ndof;

    // COMPUTE STARTING POINTS INTO OPEN CORE FOR THIS PARTITION AND ITS TRAN
    let iz1 = is1 * (nrow - 1) + ndof * (ncol - 1) + jcore - 1;
    let iz2 = is1 * (ncol - 1) + ndof * (nrow - 1) + jcore - 1;

    // IZ1 GETS TEMP,  IZ2 GETS THE TRANSPOSE
    let i1 = iz1;
    let i2 = iz2;

    if ipr == 2 {
        // DO THE SAME IN DOUBLE PRECISION
        if ndof == 1 {
            // 1 X 1 PARTITION
            dz[slot(i1, 1)] = dtemp[0];
            dz[slot(i2, 1)] = dtemp[0];
        } else {
            // 3 X 3 PARTITION
            dz[slot(i1, 1)] = dtemp[0];
            dz[slot(i2, 1)] = dtemp[0];
            dz[slot(i1, 2)] = dtemp[1];
            dz[slot(i2, 25)] = dtemp[1];
            dz[slot(i1, 3)] = dtemp[2];
            dz[slot(i2, 49)] = dtemp[2];
            dz[slot(i1, 25)] = dtemp[3];
            dz[slot(i2, 2)] = dtemp[3];
            dz[slot(i1, 26)] = dtemp[4];
            dz[slot(i2, 26)] = dtemp[4];
            dz[slot(i1, 27)] = dtemp[5];
            dz[slot(i2, 50)] = dtemp[5];
            dz[slot(i1, 49)] = dtemp[6];
            dz[slot(i2, 3)] = dtemp[6];
            dz[slot(i1, 50)] = dtemp[7];
            dz[slot(i2, 27)] = dtemp[7];
            dz[slot(i1, 51)] = dtemp[8];
            dz[slot(i2, 51)] = dtemp[8];
        }
        return;
    }

    if ndof == 1 {
        // 1 X 1 PARTITION
        z[slot(i1, 1)] = temp[0];
        z[slot(i2, 1)] = temp[0];
        return;
    }

    // 3 X 3 PARTITION
    // I1 GETS TEMP. I2 GETS THE TRANSPOSE
    // IF I1=I2, THEN HALF OF THE ENTRIES WILL BE DUPLICATED
    // THAT-S OK SINCE THERE ARE NO ADDITIONS
    z[slot(i1, 1)] = temp[0];
    z[slot(i2, 1)] = temp[0];
    z[slot(i1, 2)] = temp[1];
    z[slot(i2, 25)] = temp[1];
    z[slot(i1, 3)] = temp[2];
    z[slot(i2, 49)] = temp[2];
    z[slot(i1, 25)] = temp[3];
    z[slot(i2, 2)] = temp[3];
    z[slot(i1, 26)] = temp[4];
    z[slot(i2, 26)] = temp[4];
    z[slot(i1, 27)] = temp[5];
    z[slot(i2, 50)] = temp[5];
    z[slot(i1, 49)] = temp[6];
    z[slot(i2, 3)] = temp[6];
    z[slot(i1, 50)] = temp[7];
    z[slot(i2, 27)] = temp[7];
    z[slot(i1, 51)] = temp[8];
    z[slot(i2, 51)] = temp[8];
}

#[cfg(test)]
mod tests {
    use super::insert;

    /// Emit exactly the text produced by `parity/drivers/insert.f`.
    ///
    /// Four calls cover the four branches; `Z`/`DZ` are cleared before each
    /// call and only the non-zero slots are printed, so the output pins both
    /// the set of slots `INSERT` touches and the value written into each.
    fn driver_output() -> String {
        // (IPR, NDOF, NGRID, NCOL, NROW, JCORE) per case.
        let cases: [(i32, i32, i32, i32, i32, i32); 4] = [
            (1, 3, 2, 2, 3, 1),
            (1, 1, 3, 2, 3, 2),
            (2, 3, 2, 3, 2, 1),
            (2, 1, 3, 3, 2, 2),
        ];

        let mut out = String::new();
        for (k, &(ipr, ndof, ngrid, ncol, nrow, jcore)) in cases.iter().enumerate() {
            let k = k + 1;

            // CLEAR THE OPEN CORE WINDOW AND THE PARTITION
            let mut z = [0.0_f32; 200];
            let mut dz = [0.0_f64; 200];
            let mut temp = [0.0_f32; 9];
            let mut dtemp = [0.0_f64; 9];
            for i in 0..9 {
                temp[i] = (i + 1) as f32;
                dtemp[i] = (i + 1) as f64 + 0.25;
            }

            insert(
                ncol, nrow, ndof, ngrid, jcore, &mut z, &mut dz, &temp, &dtemp, ipr,
            );

            out.push_str(&format!(
                "CASE {} IPR={} NDOF={} NGRID={} NCOL={} NROW={} JCORE={}\n",
                k, ipr, ndof, ngrid, ncol, nrow, jcore
            ));
            for (i, v) in z.iter().enumerate() {
                if *v != 0.0 {
                    out.push_str(&format!("CASE {} Z {:3} {:8.1}\n", k, i + 1, v));
                }
            }
            for (i, v) in dz.iter().enumerate() {
                if *v != 0.0 {
                    out.push_str(&format!("CASE {} DZ {:3} {:10.2}\n", k, i + 1, v));
                }
            }
        }
        out
    }

    #[test]
    fn insert_parity() {
        let golden = include_str!("../../../../parity/golden/insert.txt");
        let got = driver_output();
        // Printed so the parity diff can be reproduced from the test run.
        print!("{}", got);
        assert_eq!(got, golden, "INSERT output diverges from Fortran golden");
    }

    #[test]
    fn insert_duplicate_write_when_i1_equals_i2() {
        // NCOL == NROW makes IZ1 == IZ2, so the 3x3 scatter writes each slot
        // twice; the last write wins and no addition happens.
        let mut z = [0.0_f32; 200];
        let mut dz = [0.0_f64; 200];
        let mut temp = [0.0_f32; 9];
        let dtemp = [0.0_f64; 9];
        for i in 0..9 {
            temp[i] = (i + 1) as f32;
        }
        insert(2, 2, 3, 2, 1, &mut z, &mut dz, &temp, &dtemp, 1);
        // IS1 = 2*9 = 18, IZ1 = IZ2 = 18*(2-1) + 3*(2-1) + 1 - 1 = 21.
        // Each slot below is written twice; the second write wins.
        assert_eq!(z[21], 1.0); // Z(I1+1) then Z(I2+1)
        assert_eq!(z[22], 4.0); // Z(I1+2)=2 then Z(I2+2)=4
        assert_eq!(z[23], 7.0); // Z(I1+3)=3 then Z(I2+3)=7
        assert_eq!(z[45], 4.0); // Z(I2+25)=2 then Z(I1+25)=4
        assert_eq!(z[46], 5.0); // Z(I1+26) then Z(I2+26)
        assert_eq!(z[47], 8.0); // Z(I1+27)=6 then Z(I2+27)=8
        assert_eq!(z[69], 7.0); // Z(I2+49)=3 then Z(I1+49)=7
        assert_eq!(z[70], 8.0); // Z(I2+50)=6 then Z(I1+50)=8
        assert_eq!(z[71], 9.0); // Z(I1+51) then Z(I2+51)
    }
}
