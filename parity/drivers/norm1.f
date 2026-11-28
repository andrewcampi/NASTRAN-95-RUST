      PROGRAM NORM1DRV
C
C     Fixed-input parity driver for NORM1 (mis/norm1.f).
C     Prints deterministic, fixed-format text only (no dates, timing,
C     memory, or uninitialized values).
C
C     NORM1 takes NCOL from COMMON /INVPWX/ via EQUIVALENCE
C     (NCOL,FILEK(2)), so the driver sets FILEK(2) = 4.
C
C     IND1 is DATA-initialized inside NORM1 and persists across calls, so
C     NORM1 is called three times in sequence: the sign of DIV on each call
C     depends on the IND1 left behind by the previous one.
C
C     HAZARD (deliberately not "fixed"): if every X(I) is 0.0, IND is never
C     assigned and the Fortran reads an undefined value.  Every input below
C     is non-zero.
C
      IMPLICIT NONE
      DOUBLE PRECISION X(4), DIV
      INTEGER FILEK, NCOL
      COMMON /INVPWX/ FILEK(7)
      EQUIVALENCE (NCOL, FILEK(2))
      EXTERNAL NORM1
C
      NCOL = 4
C
C     CALL 1: IND1 = 1 initially, so I = 1 and XX = X(1) = 2.0 > 0.
C     MAX = 8.0 at I = 2; X(2) < 0 so IND = -2; DIV = +8.0;
C     IND1 becomes -2.
      X(1) = 2.0D0
      X(2) = -8.0D0
      X(3) = 4.0D0
      X(4) = 1.0D0
      CALL NORM1(X,DIV)
      WRITE(6,900) 'CALL1 DIV', DIV
      WRITE(6,901) 'CALL1 X  ', X(1), X(2), X(3), X(4)
C
C     CALL 2: IND1 = -2 carried over, so I = 2 and XX = X(2) = 1.5 > 0.
C     MAX = 6.0 at I = 3; X(3) > 0 so IND = 3; DIV = -6.0 (sign flipped
C     by ISIGN(1,IND1)); IND1 becomes -3.
      X(1) = -3.0D0
      X(2) = 1.5D0
      X(3) = 6.0D0
      X(4) = -0.5D0
      CALL NORM1(X,DIV)
      WRITE(6,900) 'CALL2 DIV', DIV
      WRITE(6,901) 'CALL2 X  ', X(1), X(2), X(3), X(4)
C
C     CALL 3: IND1 = -3 carried over, so I = 3 and XX = X(3) = -2.0 < 0.
C     MAX = 3.0 at I = 4; X(4) > 0 so IND = 4; DIV = +3.0; IND1 becomes 4.
C     X(2) = 1.0D-40 normalizes to 3.33D-41, which is below 1.E-36 and is
C     therefore flushed to zero by the IF (ABS(XI) .LT. 1.E-36) XI=0. line.
      X(1) = 1.0D0
      X(2) = 1.0D-40
      X(3) = -2.0D0
      X(4) = 3.0D0
      CALL NORM1(X,DIV)
      WRITE(6,900) 'CALL3 DIV', DIV
      WRITE(6,901) 'CALL3 X  ', X(1), X(2), X(3), X(4)
      STOP
  900 FORMAT(A9,1P,1D24.16)
  901 FORMAT(A9,1P,4D24.16)
      END
