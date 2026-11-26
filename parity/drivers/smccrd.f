      PROGRAM SMCCRDDRV
C
C     Fixed-input parity driver for SMCCRD (mis/smccrd.f).
C
C     SMCCRD ( DTEMP, ZIL, ILIM, ZOL ) accumulates a vector scaled by a
C     scalar:  DTEMP(I) = DTEMP(I) + ZIL(I) * ZOL  for I = 1..ILIM.
C
C     Exercises: plain integer-valued data, negative/zero data, inexact
C     decimals (rounding-sensitive), a partial update where ILIM is
C     smaller than the declared array length, and ILIM = 0.
C
C     Output is deterministic, fixed-format text only: no dates, timing,
C     memory addresses, or uninitialized values.
C
      IMPLICIT NONE
      INTEGER ILIM
      DOUBLE PRECISION ZOL
      DOUBLE PRECISION DTEMP(4), ZIL(4)
C
C     CASE 1: plain integer-valued data, ZOL = 2.5.
      DTEMP(1) = 1.0D0
      DTEMP(2) = 2.0D0
      DTEMP(3) = 3.0D0
      DTEMP(4) = 4.0D0
      ZIL(1) = 1.0D0
      ZIL(2) = 2.0D0
      ZIL(3) = 3.0D0
      ZIL(4) = 4.0D0
      ILIM = 4
      ZOL = 2.5D0
      CALL SMCCRD(DTEMP, ZIL, ILIM, ZOL)
      WRITE(6,900) 'CASE1', DTEMP(1), DTEMP(2), DTEMP(3), DTEMP(4)
C
C     CASE 2: negative and zero entries.
      DTEMP(1) =  0.0D0
      DTEMP(2) = -2.5D0
      DTEMP(3) =  1.25D0
      DTEMP(4) = -0.0D0
      ZIL(1) = -3.0D0
      ZIL(2) =  0.0D0
      ZIL(3) =  4.5D0
      ZIL(4) = -1.0D0
      ILIM = 4
      ZOL = 2.5D0
      CALL SMCCRD(DTEMP, ZIL, ILIM, ZOL)
      WRITE(6,900) 'CASE2', DTEMP(1), DTEMP(2), DTEMP(3), DTEMP(4)
C
C     CASE 3: inexact decimal data (rounding-sensitive).
      DTEMP(1) = 0.1D0
      DTEMP(2) = 0.2D0
      DTEMP(3) = 0.3D0
      DTEMP(4) = 1.0D0
      ZIL(1) = 0.7D0
      ZIL(2) = 1.1D0
      ZIL(3) = -0.3D0
      ZIL(4) = 0.3333333333333333D0
      ILIM = 4
      ZOL = 2.5D0
      CALL SMCCRD(DTEMP, ZIL, ILIM, ZOL)
      WRITE(6,900) 'CASE3', DTEMP(1), DTEMP(2), DTEMP(3), DTEMP(4)
C
C     CASE 4: partial update, ILIM = 2 -- elements 3 and 4 must be
C     left exactly as they were.
      DTEMP(1) = 10.0D0
      DTEMP(2) = 20.0D0
      DTEMP(3) = 30.0D0
      DTEMP(4) = 40.0D0
      ZIL(1) = 1.0D0
      ZIL(2) = -1.0D0
      ZIL(3) = 100.0D0
      ZIL(4) = 100.0D0
      ILIM = 2
      ZOL = 2.5D0
      CALL SMCCRD(DTEMP, ZIL, ILIM, ZOL)
      WRITE(6,900) 'CASE4', DTEMP(1), DTEMP(2), DTEMP(3), DTEMP(4)
C
C     CASE 5: ILIM = 0 -- the loop body must not execute at all.
      DTEMP(1) = 1.0D0
      DTEMP(2) = 2.0D0
      DTEMP(3) = 3.0D0
      DTEMP(4) = 4.0D0
      ZIL(1) = 5.0D0
      ZIL(2) = 6.0D0
      ZIL(3) = 7.0D0
      ZIL(4) = 8.0D0
      ILIM = 0
      ZOL = 2.5D0
      CALL SMCCRD(DTEMP, ZIL, ILIM, ZOL)
      WRITE(6,900) 'CASE5', DTEMP(1), DTEMP(2), DTEMP(3), DTEMP(4)
      STOP
  900 FORMAT(A5,4ES24.16E3)
      END
