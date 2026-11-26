      PROGRAM DCDRV
C
C     FIXED-INPUT UNIT DRIVER FOR DCROSS
C     PRINTS ONLY DETERMINISTIC FIXED-FORMAT TEXT
C
      DOUBLE PRECISION X(3),Y(3),Z(3)
      INTEGER I
C
C     CASE 1: ALL POSITIVE, NON-DEGENERATE
C
      X(1) = 1.0D0
      X(2) = 2.0D0
      X(3) = 3.0D0
      Y(1) = 4.0D0
      Y(2) = 5.0D0
      Y(3) = 6.0D0
      CALL DCROSS(X,Y,Z)
      DO 10 I = 1,3
         WRITE(6,900) I,Z(I)
   10 CONTINUE
C
C     CASE 2: NEGATIVE AND ZERO COMPONENTS
C
      X(1) = -1.5D0
      X(2) = 0.0D0
      X(3) = 2.25D0
      Y(1) = 3.0D0
      Y(2) = -4.0D0
      Y(3) = 0.0D0
      CALL DCROSS(X,Y,Z)
      DO 20 I = 1,3
         WRITE(6,900) I,Z(I)
   20 CONTINUE
C
  900 FORMAT('Z(',I1,') = ',ES24.16E3)
      STOP
      END
