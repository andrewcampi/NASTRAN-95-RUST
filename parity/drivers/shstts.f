      PROGRAM SHSTTSDRV
C     Fixed-input driver for SHSTTS.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     SHSTTS(TAB,UAB,VAB) builds the shell stress-tensor transformation
C     matrices from a row-loaded 3x3 orthogonal inplane rotation TAB.
C
C     CASE 1: 30-degree inplane rotation about the shell normal, row-loaded
C     as TAB(1..3) = row 1, TAB(4..6) = row 2, TAB(7..9) = row 3.  TAB(9) is
C     deliberately set to 0.5 (not 1.0) so that the VAB products, which all
C     carry a TAB(9) factor, are exercised with a non-unit scale.
      REAL TAB(9), UAB(9), VAB(4)
      EXTERNAL SHSTTS
      TAB(1) = 0.86602540
      TAB(2) = 0.5
      TAB(3) = 0.0
      TAB(4) = -0.5
      TAB(5) = 0.86602540
      TAB(6) = 0.0
      TAB(7) = 0.0
      TAB(8) = 0.0
      TAB(9) = 0.5
      CALL SHSTTS(TAB,UAB,VAB)
      WRITE(6,900) 'CASE1', (UAB(I),I=1,9)
      WRITE(6,901) 'VAB1', (VAB(I),I=1,4)
C
C     CASE 2: identity rotation (TAB(9) = 1.0, zero inplane rotation) to pin
C     the identity path of both transformation matrices.
      TAB(1) = 1.0
      TAB(2) = 0.0
      TAB(3) = 0.0
      TAB(4) = 0.0
      TAB(5) = 1.0
      TAB(6) = 0.0
      TAB(7) = 0.0
      TAB(8) = 0.0
      TAB(9) = 1.0
      CALL SHSTTS(TAB,UAB,VAB)
      WRITE(6,900) 'CASE2', (UAB(I),I=1,9)
      WRITE(6,901) 'VAB2', (VAB(I),I=1,4)
      STOP
  900 FORMAT(A5,9ES24.16E3)
  901 FORMAT(A5,4ES24.16E3)
      END
