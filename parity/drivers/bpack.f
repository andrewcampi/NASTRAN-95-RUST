      PROGRAM BPACKDR
C     Fixed-input driver for BPACK.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     BPACK ( IG, I, J, L ) packs internal grid number L into the
C     column-wise IG table:
C
C         IPASS = IPASS + 1
C         LOC   = J - 1
C         N1    = II1*LOC + I
C         IG(N1) = L
C
C     Only the IBM/VAX branch of the Fortran is exercised: IG is
C     INTEGER*2, so storing L truncates it to a 16-bit slot.  The
C     commented-out CDC/UNIVAC branch is not ported and not driven.
C
C     /BANDB/ and /BANDS/ are declared here exactly as in bpack.f so the
C     driver and the routine share the same blocks:
C
C         /BANDB/  NBIT, DUM3B(3), IPASS, NW, DUM1B, NBPW   (8 words)
C         /BANDS/  DUM4S(4), II1, DUM5S(5), MASK            (11 words)
C
C     BPACK reads only IPASS from /BANDB/ and only II1 from /BANDS/;
C     the remaining words are set to fixed values so nothing is
C     uninitialized.
      INTEGER*2 IG(16)
      INTEGER NBIT, DUM3B(3), IPASS, NW, DUM1B, NBPW
      INTEGER DUM4S(4), II1, DUM5S(5), MASK
      INTEGER I, J, L, K
      EXTERNAL BPACK
      COMMON /BANDB / NBIT, DUM3B, IPASS, NW, DUM1B, NBPW
      COMMON /BANDS / DUM4S, II1, DUM5S, MASK
C
C     Fixed state: II1 = 4 (column stride), IPASS = 0 (call counter).
      NBIT  = 16
      DO 5 K = 1, 3
         DUM3B(K) = 0
    5 CONTINUE
      IPASS = 0
      NW    = 1
      DUM1B = 0
      NBPW  = 16
      DO 6 K = 1, 4
         DUM4S(K) = 0
    6 CONTINUE
      II1 = 4
      DO 7 K = 1, 5
         DUM5S(K) = 0
    7 CONTINUE
      MASK = 0
C
C     Sentinel-fill IG so the slots BPACK actually writes are visible.
      DO 10 K = 1, 16
         IG(K) = -1
   10 CONTINUE
C
C     CALL 1: J = 1, so LOC = 0 and N1 = I = 1  ->  IG(1).
      I = 1
      J = 1
      L = 5
      CALL BPACK(IG,I,J,L)
C
C     CALL 2: LOC = 2, N1 = 4*2 + 2 = 10  ->  IG(10).
      I = 2
      J = 3
      L = 7
      CALL BPACK(IG,I,J,L)
C
C     CALL 3: LOC = 1, N1 = 4*1 + 3 = 7  ->  IG(7).  L is the largest
C     value an INTEGER*2 slot can hold, i.e. the i16 boundary.
      I = 3
      J = 2
      L = 32767
      CALL BPACK(IG,I,J,L)
C
C     CALL 4: LOC = 3, N1 = 4*3 + 4 = 16  ->  IG(16), the last slot.
      I = 4
      J = 4
      L = 12345
      CALL BPACK(IG,I,J,L)
C
      WRITE(6,900) 'IPASS'
      WRITE(6,901) IPASS
      WRITE(6,900) 'IG'
      WRITE(6,901) (IG(K), K = 1, 16)
      STOP
  900 FORMAT(A5)
  901 FORMAT(I8)
      END
