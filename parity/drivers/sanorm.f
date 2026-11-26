      PROGRAM SANORMDRV
C
C     Fixed-input parity driver for SANORM (mis/sanorm.f).
C     SANORM has an alternate return:  SUBROUTINE SANORM (*,A)
C     The driver pins both exits:
C       case 1: A = (3.0, 4.0, 0.0)  -> normal return, A normalized
C       case 2: A = (0.0, 0.0, 0.0)  -> alternate return 1, A untouched
C     ISEL records which exit was taken (0 = normal, 1 = alternate).
C
      IMPLICIT NONE
      REAL A(3), CASES(3,2)
      INTEGER ISEL, I, J
      DATA CASES / 3.0, 4.0, 0.0,
     &             0.0, 0.0, 0.0 /
      DO 20 J = 1, 2
        DO 10 I = 1, 3
          A(I) = CASES(I,J)
   10   CONTINUE
        ISEL = 0
        CALL SANORM(*100, A)
        GO TO 30
  100   CONTINUE
        ISEL = 1
   30   CONTINUE
        WRITE(*,900) J, ISEL, A(1), A(2), A(3)
   20 CONTINUE
  900 FORMAT('CASE ',I1,' ISEL=',I1,' A=',ES15.7,1X,ES15.7,1X,ES15.7)
      END
