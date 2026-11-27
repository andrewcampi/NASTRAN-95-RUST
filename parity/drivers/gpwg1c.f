      PROGRAM GPWG1CDRV
C     Fixed-input driver for GPWG1C.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     GPWG1C(B,E,EIG,IFLAG) is the double-precision 3x3 Jacobi
C     eigensolver: DETB is the sum of squares of all nine B entries,
C     EPSIL = DSQRT(DETB)*1.0D-5, and the iteration stops when the
C     largest off-diagonal magnitude falls below EPSIL (IFLAG=0) or
C     after 21 iterations (IFLAG=1).
C
C     CASE 1: general symmetric 3x3 with all three off-diagonals
C     nonzero, so several rotations are taken before convergence.
C     CASE 2: diagonal B.  DETB is nonzero but BMAX is zero, i.e. below
C     EPSIL, so the routine exits immediately at label 100.
C     CASE 3: B = 0.  DETB = 0, the DETB .EQ. 0 early exit is taken.
C     CASE 4: B(1,1) .EQ. B(2,2), so R = 0 and the |R| .LT. 1.0D-6
C     branch (S = C = DSQRT(.5D0)) is exercised.
      DOUBLE PRECISION B(3,3), E(3,3), EIG(3)
      INTEGER IFLAG, I, J
      EXTERNAL GPWG1C
C
C     CASE 1
C
      B(1,1) = 4.0D0
      B(1,2) = 1.0D0
      B(1,3) = 2.0D0
      B(2,1) = 1.0D0
      B(2,2) = 3.0D0
      B(2,3) = 0.5D0
      B(3,1) = 2.0D0
      B(3,2) = 0.5D0
      B(3,3) = 1.0D0
      CALL GPWG1C(B,E,EIG,IFLAG)
      WRITE(6,900) 'CASE1', IFLAG
      DO 10 I = 1,3
         WRITE(6,901) 'CASE1', I, EIG(I)
   10 CONTINUE
      DO 20 I = 1,3
      DO 20 J = 1,3
         WRITE(6,902) 'CASE1', I, J, E(I,J)
   20 CONTINUE
C
C     CASE 2: diagonal B, DETB nonzero, BMAX below EPSIL
C
      B(1,1) = 2.0D0
      B(1,2) = 0.0D0
      B(1,3) = 0.0D0
      B(2,1) = 0.0D0
      B(2,2) = -3.0D0
      B(2,3) = 0.0D0
      B(3,1) = 0.0D0
      B(3,2) = 0.0D0
      B(3,3) = 5.0D0
      CALL GPWG1C(B,E,EIG,IFLAG)
      WRITE(6,900) 'CASE2', IFLAG
      DO 30 I = 1,3
         WRITE(6,901) 'CASE2', I, EIG(I)
   30 CONTINUE
      DO 40 I = 1,3
      DO 40 J = 1,3
         WRITE(6,902) 'CASE2', I, J, E(I,J)
   40 CONTINUE
C
C     CASE 3: B = 0, DETB = 0
C
      DO 50 I = 1,3
      DO 50 J = 1,3
         B(I,J) = 0.0D0
   50 CONTINUE
      CALL GPWG1C(B,E,EIG,IFLAG)
      WRITE(6,900) 'CASE3', IFLAG
      DO 60 I = 1,3
         WRITE(6,901) 'CASE3', I, EIG(I)
   60 CONTINUE
      DO 70 I = 1,3
      DO 70 J = 1,3
         WRITE(6,902) 'CASE3', I, J, E(I,J)
   70 CONTINUE
C
C     CASE 4: B(1,1) .EQ. B(2,2) so R = 0 (|R| .LT. 1.0D-6 branch)
C
      B(1,1) = 2.0D0
      B(1,2) = 1.0D0
      B(1,3) = 0.0D0
      B(2,1) = 1.0D0
      B(2,2) = 2.0D0
      B(2,3) = 0.0D0
      B(3,1) = 0.0D0
      B(3,2) = 0.0D0
      B(3,3) = 3.0D0
      CALL GPWG1C(B,E,EIG,IFLAG)
      WRITE(6,900) 'CASE4', IFLAG
      DO 80 I = 1,3
         WRITE(6,901) 'CASE4', I, EIG(I)
   80 CONTINUE
      DO 90 I = 1,3
      DO 90 J = 1,3
         WRITE(6,902) 'CASE4', I, J, E(I,J)
   90 CONTINUE
      STOP
  900 FORMAT(A5,' IFLAG=',I2)
  901 FORMAT(A5,' EIG(',I1,') = ',ES24.16E3)
  902 FORMAT(A5,' E(',I1,',',I1,') = ',ES24.16E3)
      END
