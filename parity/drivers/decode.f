      PROGRAM DECODEDR
C     Fixed-input driver for DECODE.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     DECODE ( CODE, LIST, N ) reads the /TWO/ COMMON block, which holds
C     the powers of two TWO(I) = 2**(I-1), and scans I = 1..32 testing
C     TWO(33-I) against CODE.  On a hit it increments N and stores I-1
C     into LIST(N).  The scan therefore runs from the high-order bit
C     (I=1, TWO(32)) down to the low-order bit (I=32, TWO(1)), and the
C     stored value is I-1, i.e. 32 minus the bit number.  LIST ordering
C     is part of the contract and is reproduced verbatim.
C
C     Build (the Fortran is the oracle):
C       gfortran -O0 -ffp-contract=off -fno-fast-math -std=legacy -fdec \
C         -ffixed-line-length-none -o /tmp/decode_drv \
C         parity/drivers/decode.f vendor/nastran95/mis/decode.f \
C         vendor/nastran95/mds/mapfns.f
C     (mapfns.f supplies the ANDF entry, which is IAND.)
      INTEGER     CODE, LIST(32), N, I, J
      COMMON /TWO/ TWO(32)
      INTEGER     TWO
      EXTERNAL    DECODE
C
C     Initialize the /TWO/ table exactly as the production code does:
C     TWO(I) = 2**(I-1).  ISHFT is used so that the top entry (the sign
C     bit, 2**31) is formed without integer overflow.
      DO 10 I = 1, 32
         TWO(I) = ISHFT(1, I-1)
   10 CONTINUE
C
C     CASE 1: no bits set at all -- N must come back 0 and LIST is
C     untouched.
      CODE = 0
      CALL RUN(CODE, LIST, N, 'CASE1')
C
C     CASE 2: a single low-order bit (bit 0).  It is found at I=32, so
C     LIST(1) = 31.
      CODE = 1
      CALL RUN(CODE, LIST, N, 'CASE2')
C
C     CASE 3: a single high-order bit (bit 31, the sign bit).  It is
C     found at I=1, so LIST(1) = 0.
      CODE = ISHFT(1, 31)
      CALL RUN(CODE, LIST, N, 'CASE3')
C
C     CASE 4: alternating pattern 0101...01 (bits 0,2,...,30 set).
C     Sixteen hits, stored in descending scan order.
      CODE = 1431655765
      CALL RUN(CODE, LIST, N, 'CASE4')
C
C     CASE 5: alternating pattern 1010...10 (bits 1,3,...,31 set).
      CODE = -1431655766
      CALL RUN(CODE, LIST, N, 'CASE5')
C
C     CASE 6: every bit set -- all 32 entries, LIST(J) = J-1.
      CODE = -1
      CALL RUN(CODE, LIST, N, 'CASE6')
C
C     CASE 7: mixed pattern with both end bits and a gap in the middle.
      CODE = IOR(ISHFT(1, 31), 1056)
      CALL RUN(CODE, LIST, N, 'CASE7')
      STOP
      END
C
      SUBROUTINE RUN(CODE, LIST, N, TAG)
C     Calls DECODE once and writes N followed by LIST(1..N).  Only the
C     first N entries are written, so no uninitialized value is printed.
      INTEGER     CODE, LIST(32), N, J
      CHARACTER*5 TAG
      EXTERNAL    DECODE
C
      CALL DECODE(CODE, LIST, N)
      WRITE(6,900) TAG, CODE, N
      DO 20 J = 1, N
         WRITE(6,901) J, LIST(J)
   20 CONTINUE
      RETURN
  900 FORMAT(A5,' CODE=',I12,' N=',I4)
  901 FORMAT(4X,'LIST(',I2,')=',I4)
      END
