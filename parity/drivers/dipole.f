      PROGRAM DIPDRV
C
C     FIXED-INPUT PARITY DRIVER FOR DIPOLE (mis/dipole.f)
C     PRINTS ONLY DETERMINISTIC FIXED-FORMAT TEXT.
C
C     BUF(1) IS UNUSED BY DIPOLE (ICID COMES FROM IBUF(1)); BUF(2..4) ARE
C     THE DIPOLE COORDS CX,CY,CZ, BUF(5..7) THE MOMENT MX,MY,MZ AND
C     BUF(8..9) THE MIN/MAX BAND.  THE WHOLE 50-WORD BUF/IBUF IS ZEROED
C     FIRST SO NOTHING UNINITIALIZED CAN LEAK INTO THE OUTPUT.
C
      REAL BUF(50), HC1, HC2, HC3
      INTEGER IBUF(50)
      REAL C(11,8)
      INTEGER I, J
C
C     COLUMNS: CX CY CZ MX MY MZ MIN MAX XX YY ZZ
C
      DATA C /
     &  1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 0.0, 0.0, 4.0, 6.0, 3.0,
     &  0.0, 0.0, 0.0, 1.5, -2.25, 0.75, 0.0, 0.0, 0.5, -1.25, 7.5,
     &  1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 2.0, 5.0, 4.0, 6.0, 3.0,
     &  1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 2.0, 5.0, 2.0, 2.0, 3.0,
     &  1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 2.0, 5.0, 11.0, 2.0, 3.0,
     &  1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 2.0, 5.0, 3.0, 2.0, 3.0,
     &  1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 2.0, 0.0, 11.0, 2.0, 3.0,
     &  1.0, 2.0, 3.0, 10.0, -20.0, 30.0, 0.0, 5.0, 2.0, 2.0, 3.0 /
C
      DO 100 J = 1, 8
        DO 10 I = 1, 50
          BUF(I) = 0.0
          IBUF(I) = 0
   10   CONTINUE
        BUF(2) = C(1,J)
        BUF(3) = C(2,J)
        BUF(4) = C(3,J)
        BUF(5) = C(4,J)
        BUF(6) = C(5,J)
        BUF(7) = C(6,J)
        BUF(8) = C(7,J)
        BUF(9) = C(8,J)
        CALL DIPOLE(BUF,IBUF,C(9,J),C(10,J),C(11,J),HC1,HC2,HC3)
        WRITE(6,900) J, HC1, HC2, HC3
  100 CONTINUE
      STOP
  900 FORMAT('CASE ',I1,' HC=',ES24.16E3,1X,ES24.16E3,1X,ES24.16E3)
      END
