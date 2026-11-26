      PROGRAM SCALEXDRV
C     Fixed-input driver for SCALEX.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     SCALEX declares L(1) but writes L(1..6); the caller must supply six
C     words, so L is dimensioned 6 here.
      INTEGER L(6), ILVAL, CODE
      EXTERNAL SCALEX
C
C     CASE 1: CODE < 0 -- the arithmetic IF IF(CODE) 102,102,103 sends
C     negative CODE to the same branch as CODE = 0 (L(1) = ILVAL).
      ILVAL = 10
      CODE = -3
      CALL SCALEX(ILVAL,CODE,L)
      WRITE(6,900) 'CASE1', L(1), L(2), L(3), L(4), L(5), L(6)
C
C     CASE 2: CODE = 0 -- the middle arm of the arithmetic IF, same
C     branch as CASE 1.
      ILVAL = 10
      CODE = 0
      CALL SCALEX(ILVAL,CODE,L)
      WRITE(6,900) 'CASE2', L(1), L(2), L(3), L(4), L(5), L(6)
C
C     CASE 3: six distinct nonzero digits -- J = 6, every slot filled.
      ILVAL = 10
      CODE = 123456
      CALL SCALEX(ILVAL,CODE,L)
      WRITE(6,900) 'CASE3', L(1), L(2), L(3), L(4), L(5), L(6)
C
C     CASE 4: repeated digits -- only the first occurrence of each
C     nonzero digit is kept, so J = 3 and L(4..6) stay zero.
      ILVAL = 10
      CODE = 112233
      CALL SCALEX(ILVAL,CODE,L)
      WRITE(6,900) 'CASE4', L(1), L(2), L(3), L(4), L(5), L(6)
C
C     CASE 5: zeros interleaved with nonzero digits -- zeros are
C     skipped, J = 3, L(4..6) stay zero.
      ILVAL = 10
      CODE = 102030
      CALL SCALEX(ILVAL,CODE,L)
      WRITE(6,900) 'CASE5', L(1), L(2), L(3), L(4), L(5), L(6)
      STOP
  900 FORMAT(A5,6I6)
      END
