      PROGRAM SORTDGDRV
C     Fixed-input parity driver for SORTDG (mis/sortdg.f).
C
C     SORTDG sorts STK2 by the degree of the node (NDEG) and appends it to
C     the end of STK1, lowest degree first.  X1 and X2 are the number of
C     nodes in STK1 and STK2 respectively; X1 is updated in place.
C
C     The sort is a specific bubble-pass structure: IND starts at X2 and is
C     decremented once per outer pass; the inner loop I=1..IND compares
C     NDEG(STK2(I)) with NDEG(STK2(I+1)) and swaps only when strictly
C     greater, setting ITEST=1; the passes repeat while ITEST=1 and
C     IND .GE. 1.  Equal degrees are therefore never swapped.
C
C     Prints deterministic, fixed-format text only (no dates, timing,
C     memory, or uninitialized values).  After each case the driver prints
C     the tag, the updated X1, and STK1(1..X1), all in I12 fields.
C
C     CASE 1: X2 = 0  -- nothing to sort, nothing appended, X1 unchanged.
C     CASE 2: X2 = 1  -- single element, no inner loop iterations.
C     CASE 3: already-sorted STK2 (ascending degree), X1 = 0.
C     CASE 4: reverse-sorted STK2 (descending degree), X1 = 2.
C     CASE 5: duplicate degrees in a scrambled order, to pin the tie
C             behaviour (strict-greater comparison, so equal degrees keep
C             their relative order and the pass loop stops early).
C
C     NDEG(1..12) is fixed for all cases:
C       node :  1  2  3  4  5  6  7  8  9 10 11 12
C       deg  :  5  1  3  1  7  2  9  4  1  6  2  8
      IMPLICIT NONE
      INTEGER NDEG(12), STK1(40), STK2(40)
      INTEGER X1, X2, I
      EXTERNAL SORTDG
C
      NDEG(1)  = 5
      NDEG(2)  = 1
      NDEG(3)  = 3
      NDEG(4)  = 1
      NDEG(5)  = 7
      NDEG(6)  = 2
      NDEG(7)  = 9
      NDEG(8)  = 4
      NDEG(9)  = 1
      NDEG(10) = 6
      NDEG(11) = 2
      NDEG(12) = 8
C
C     CASE 1: X2 = 0.
      X1 = 3
      STK1(1) = 11
      STK1(2) = 12
      STK1(3) = 10
      X2 = 0
      CALL SORTDG(STK1,STK2,X1,X2,NDEG)
      WRITE(6,900) 'CASE1', X1, (STK1(I),I=1,X1)
C
C     CASE 2: X2 = 1.
      X1 = 2
      STK1(1) = 1
      STK1(2) = 2
      X2 = 1
      STK2(1) = 5
      CALL SORTDG(STK1,STK2,X1,X2,NDEG)
      WRITE(6,900) 'CASE2', X1, (STK1(I),I=1,X1)
C
C     CASE 3: already sorted (ascending degree), X1 = 0.
      X1 = 0
      X2 = 12
      STK2(1)  = 2
      STK2(2)  = 4
      STK2(3)  = 9
      STK2(4)  = 6
      STK2(5)  = 11
      STK2(6)  = 3
      STK2(7)  = 8
      STK2(8)  = 1
      STK2(9)  = 10
      STK2(10) = 5
      STK2(11) = 12
      STK2(12) = 7
      CALL SORTDG(STK1,STK2,X1,X2,NDEG)
      WRITE(6,900) 'CASE3', X1, (STK1(I),I=1,X1)
C
C     CASE 4: reverse sorted (descending degree), X1 = 2.
      X1 = 2
      STK1(1) = 3
      STK1(2) = 7
      X2 = 12
      STK2(1)  = 7
      STK2(2)  = 12
      STK2(3)  = 5
      STK2(4)  = 10
      STK2(5)  = 1
      STK2(6)  = 8
      STK2(7)  = 3
      STK2(8)  = 11
      STK2(9)  = 6
      STK2(10) = 9
      STK2(11) = 4
      STK2(12) = 2
      CALL SORTDG(STK1,STK2,X1,X2,NDEG)
      WRITE(6,900) 'CASE4', X1, (STK1(I),I=1,X1)
C
C     CASE 5: duplicate degrees, scrambled, X1 = 1.
      X1 = 1
      STK1(1) = 6
      X2 = 12
      STK2(1)  = 6
      STK2(2)  = 4
      STK2(3)  = 11
      STK2(4)  = 9
      STK2(5)  = 2
      STK2(6)  = 3
      STK2(7)  = 8
      STK2(8)  = 1
      STK2(9)  = 10
      STK2(10) = 5
      STK2(11) = 12
      STK2(12) = 7
      CALL SORTDG(STK1,STK2,X1,X2,NDEG)
      WRITE(6,900) 'CASE5', X1, (STK1(I),I=1,X1)
      STOP
  900 FORMAT(A6,21I12)
      END
