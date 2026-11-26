      PROGRAM PERMUTDRV
C     Fixed-input driver for PERMUT.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     PERMUT(IA,ID,N,ISW) copies IA(1..N) into IC, sets IB(I)=I, exchange
C     sorts IC ascending while carrying IB (strictly-less swaps only, so
C     equal elements keep their original order), then rotates IB so that
C     the sequence starts at the first I with IC(I) >= ISW.
      DIMENSION IA(10), ID(10)
      EXTERNAL PERMUT
C
C     CASE 1: distinct values, ISW equal to the minimum.
      N = 6
      IA(1) = 5
      IA(2) = 3
      IA(3) = 9
      IA(4) = 1
      IA(5) = 7
      IA(6) = 2
      ISW = 1
      CALL PERMUT(IA,ID,N,ISW)
      WRITE(6,900) 'CASE1', N, (ID(I),I=1,N)
C
C     CASE 2: duplicate values -- the tie-break must keep the original
C     order of equal elements (stable permutation, not a plain sort).
      N = 7
      IA(1) = 3
      IA(2) = 1
      IA(3) = 3
      IA(4) = 2
      IA(5) = 1
      IA(6) = 3
      IA(7) = 2
      ISW = 1
      CALL PERMUT(IA,ID,N,ISW)
      WRITE(6,900) 'CASE2', N, (ID(I),I=1,N)
C
C     CASE 3: ISW below the minimum -- every IC(I) >= ISW, so the
C     rotation starts at position 1.
      N = 5
      IA(1) = 4
      IA(2) = 2
      IA(3) = 8
      IA(4) = 6
      IA(5) = 0
      ISW = -5
      CALL PERMUT(IA,ID,N,ISW)
      WRITE(6,900) 'CASE3', N, (ID(I),I=1,N)
C
C     CASE 4: ISW above the maximum -- the DO 50 loop falls through,
C     K=1 and the GOTO 71 path copies IB unchanged.
      N = 5
      IA(1) = 4
      IA(2) = 2
      IA(3) = 8
      IA(4) = 6
      IA(5) = 0
      ISW = 100
      CALL PERMUT(IA,ID,N,ISW)
      WRITE(6,900) 'CASE4', N, (ID(I),I=1,N)
C
C     CASE 5: ISW equal to an interior value -- the rotation starts at
C     that position and wraps to the front of IB.
      N = 4
      IA(1) = 4
      IA(2) = 2
      IA(3) = 8
      IA(4) = 6
      ISW = 6
      CALL PERMUT(IA,ID,N,ISW)
      WRITE(6,900) 'CASE5', N, (ID(I),I=1,N)
      STOP
  900 FORMAT(A5,I3,10I4)
      END
