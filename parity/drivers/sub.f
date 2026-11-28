      PROGRAM SUBDRV
C     Fixed-input parity driver for SUB (mis/sub.f).
C
C     SUB forms Y = A*X - B*Y for I = 1..NCOL, where A and B are scalar
C     multipliers and NCOL comes from COMMON /INVPWX/ (XX, NCOL).  All
C     arithmetic is DOUBLE PRECISION.
C
C     Prints deterministic, fixed-format text only (no dates, timing,
C     memory, or uninitialized values).
C
C     CASE 1: NCOL = 0 -- the DO 10 loop body never runs, so SUB is a
C             no-op.  Y(1..4) is printed afterwards to show it is untouched.
C     CASE 2: NCOL = 1 -- single element.
C     CASE 3: NCOL = 4 -- multi-element case with negative B (and a
C             negative X component), so the subtraction is exercised.
C             NCOL = 4 so that Y(1..NCOL) fits the 4D24.16 format exactly
C             and no format reversion (which would print garbage) occurs.
C
C     A = 2.5D0 and B = -1.25D0 for all three cases.  The record format is
C     (A6,1P,4D24.16): a 6-column tag (so 'CASE1U' is not truncated) followed
C     by the D24.16 values.
      IMPLICIT NONE
      DOUBLE PRECISION X(8), Y(8), A, B
      INTEGER NCOL, I
C     SUB declares COMMON /INVPWX/ XX,NCOL without IMPLICIT NONE, so XX is
C     implicitly REAL*4 and NCOL sits at byte offset 4 of the block.  The
C     driver must use the same layout (REAL XX, not DOUBLE PRECISION XX),
C     otherwise SUB reads the wrong word as NCOL.
      REAL XX
      COMMON /INVPWX/ XX, NCOL
      EXTERNAL SUB
C
      A = 2.5D0
      B = -1.25D0
C
C     CASE 1: NCOL = 0 (no-op).
      NCOL = 0
      X(1) = 1.0D0
      X(2) = 2.0D0
      X(3) = 3.0D0
      X(4) = 4.0D0
      Y(1) = 10.0D0
      Y(2) = 20.0D0
      Y(3) = 30.0D0
      Y(4) = 40.0D0
      CALL SUB(X,Y,A,B)
      WRITE(6,900) 'CASE1', (Y(I),I=1,NCOL)
      WRITE(6,900) 'CASE1U', (Y(I),I=1,4)
C
C     CASE 2: NCOL = 1.
      NCOL = 1
      X(1) = 0.1D0
      Y(1) = 3.7D0
      CALL SUB(X,Y,A,B)
      WRITE(6,900) 'CASE2', (Y(I),I=1,NCOL)
C
C     CASE 3: NCOL = 4, multi-element, negative B.
      NCOL = 4
      X(1) = 0.1D0
      X(2) = -2.3D0
      X(3) = 7.0D0
      X(4) = 1.0D-3
      Y(1) = 3.7D0
      Y(2) = -0.25D0
      Y(3) = 0.3333333333333333D0
      Y(4) = 0.1428571428571429D0
      CALL SUB(X,Y,A,B)
      WRITE(6,900) 'CASE3', (Y(I),I=1,NCOL)
      STOP
  900 FORMAT(A6,1P,4D24.16)
      END
