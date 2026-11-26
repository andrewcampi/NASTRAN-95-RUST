      PROGRAM DNORMDRV
C
C     Fixed-input parity driver for DNORM (mis/dnorm.f).
C     Exercises: general non-unit vector, negative components,
C     already-unit vector, and the zero vector (early-return path).
C
      IMPLICIT NONE
      DOUBLE PRECISION X(3), MAG
      DOUBLE PRECISION CASES(3,4)
      INTEGER I, J
      DATA CASES / 3.0D0, 4.0D0, 0.0D0,
     &            -1.5D0, 2.25D0, -7.0D0,
     &             0.6D0, 0.8D0, 0.0D0,
     &             0.0D0, 0.0D0, 0.0D0 /
      DO 20 J = 1, 4
        DO 10 I = 1, 3
          X(I) = CASES(I,J)
   10   CONTINUE
        CALL DNORM(X, MAG)
        WRITE(*,900) J, MAG, X(1), X(2), X(3)
  20  CONTINUE
  900 FORMAT('CASE ',I1,' MAG=',ES24.16E3,' X=',ES24.16E3,1X,ES24.16E3,
     &       1X,ES24.16E3)
      END
