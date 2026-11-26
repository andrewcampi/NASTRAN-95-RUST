      PROGRAM DAXBDRV
C     Fixed-input driver for DAXB.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
      DOUBLE PRECISION A(3), B(3), C(3), X(3), Y(3)
      EXTERNAL DAXB
C
C     CASE 1: plain integer-valued components.
      A(1) = 1.0D0
      A(2) = 2.0D0
      A(3) = 3.0D0
      B(1) = 4.0D0
      B(2) = 5.0D0
      B(3) = 6.0D0
      CALL DAXB(A,B,C)
      WRITE(6,900) 'CASE1', C(1), C(2), C(3)
C
C     CASE 2: negative and zero components.
      A(1) =  0.0D0
      A(2) = -2.5D0
      A(3) =  1.25D0
      B(1) = -3.0D0
      B(2) =  0.0D0
      B(3) =  4.5D0
      CALL DAXB(A,B,C)
      WRITE(6,900) 'CASE2', C(1), C(2), C(3)
C
C     CASE 3: documented overlap path -- the same array is passed as
C     both A and C.  DAXB must compute all of D before storing to C.
      X(1) = 1.0D0
      X(2) = 2.0D0
      X(3) = 3.0D0
      B(1) = 4.0D0
      B(2) = 5.0D0
      B(3) = 6.0D0
      CALL DAXB(X,B,X)
      WRITE(6,900) 'CASE3', X(1), X(2), X(3)
C
C     CASE 4: overlap path with the same array passed as both B and C.
      A(1) = 1.0D0
      A(2) = 2.0D0
      A(3) = 3.0D0
      Y(1) = 4.0D0
      Y(2) = 5.0D0
      Y(3) = 6.0D0
      CALL DAXB(A,Y,Y)
      WRITE(6,900) 'CASE4', Y(1), Y(2), Y(3)
C
C     CASE 5: inexact decimal components (rounding-sensitive).
      A(1) = 0.1D0
      A(2) = 0.2D0
      A(3) = 0.3D0
      B(1) = 0.7D0
      B(2) = 1.1D0
      B(3) = -0.3D0
      CALL DAXB(A,B,C)
      WRITE(6,900) 'CASE5', C(1), C(2), C(3)
      STOP
  900 FORMAT(A5,3ES24.16E3)
      END
