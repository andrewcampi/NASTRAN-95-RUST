      PROGRAM GETURNDR
C     Fixed-input driver for GETURN.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     GETURN ( NAMFIL ) resolves a DMAP file name to a GINO unit and
C     refreshes the DSIOF open-state.  It reads /XFIST/, /XFIAT/,
C     /XXFIAT/, /DSUNIT/ and the DSIOF fields (IFILEX, LASNAM, IPRVOP,
C     NLR, NBLOCK, FCB) and writes IFILEX, LASNAM, IPRVOP, NLR, NBLOCK
C     and IUNIT(NAMFIL-100) on branch (a).
C
C     Six fixed NAMFIL values cover: the fast path, branch (a), branch
C     (b), branch (c), no-match, and the MAXPRI rejection path.
      INCLUDE 'DSIOF.COM'
        COMMON / XFIST  / IFSTMX, IFSTCA, IFIST( 100 )
        COMMON / XFIAT  / IFATUF, IFATMX, IFATCA, IFIAT( 640 )
        COMMON / XXFIAT / IXFIAT( 19 )
        INTEGER*2         IUNIT
        COMMON / DSUNIT / IUNIT( 220 )
      INTEGER NAMFIL, J
      EXTERNAL GETURN
C
C     ---- seed the FIST/FIAT tables with fixed values -----------------
C     IFSTCA = 6 name entries, so LIM = 2*IFSTCA-1 = 11 and the scan
C     visits IFST = 1,3,5,7,9,11.
      IFSTMX = 100
      IFSTCA = 6
      DO 5 J = 1, 100
         IFIST(J) = 0
    5 CONTINUE
C     pair 1: name 101, value 5   -> branch (a), IFIAT(5-2)=IFIAT(3)
      IFIST(1) = 101
      IFIST(2) = 5
C     pair 2: name 205, value 9   -> branch (b), IFIAT(9-2)=IFIAT(7)
      IFIST(3) = 205
      IFIST(4) = 9
C     pair 3: name 400, value -3  -> branch (c), IXFIAT(IABS(-3)+1)=IXFIAT(4).
C                                   NAMFIL 400 is outside 101..320, so the
C                                   branch (a) test falls through.
      IFIST(5) = 400
      IFIST(6) = -3
C     pair 4: name 104, value 13  -> branch (a), IFIAT(13-2)=IFIAT(11),
C                                   masked value exceeds MAXPRI.  NAMFIL 104
C                                   maps to IUNIT(4), which is printed, so
C                                   the skipped store is observable.
      IFIST(7) = 104
      IFIST(8) = 13
C     pair 5: name 500, value 1   -> branch (b), never matched below
      IFIST(9) = 500
      IFIST(10) = 1
C     pair 6: name 600, value -1  -> branch (c), never matched below
      IFIST(11) = 600
      IFIST(12) = -1
C
      IFATUF = 0
      IFATMX = 640
      IFATCA = 640
      DO 6 J = 1, 640
         IFIAT(J) = 0
    6 CONTINUE
C     high bits set on purpose: IAND with '00007FFF'X must clear them
      IFIAT(3)  = 65541
      IFIAT(7)  = 131082
      IFIAT(11) = 255
C
      DO 7 J = 1, 19
         IXFIAT(J) = 0
    7 CONTINUE
      IXFIAT(4) = 12
C
C     ---- seed the FCB so IPRVOP/NLR/NBLOCK are identifiable ----------
      DO 8 J = 1, MAXFCB
         FCB(1,J) = 100 + J
         FCB(2,J) = 0
         FCB(3,J) = 200 + J
         FCB(4,J) = 300 + J
    8 CONTINUE
C     unit 10 has FCB(1,10) = 2, which GETURN must report as IPRVOP = 0
      FCB(1,10) = 2
C
C     ---- CASE 1: fast path -------------------------------------------
C     NAMFIL .EQ. LASNAM and IFILEX .NE. 0, so the /XFIST/ scan is
C     skipped.  IFILEX stays 7 (not the 5 that branch (a) would give).
      CALL RESET(IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
      LASNAM = 101
      IFILEX = 7
      NAMFIL = 101
      CALL GETURN(NAMFIL)
      CALL REPORT('CASE1', IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
C
C     ---- CASE 2: branch (a), NAMFIL in 101..320 ----------------------
C     IFIAT(3) = 65541, IAND '00007FFF'X -> 5, stored in IUNIT(1).
      CALL RESET(IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
      NAMFIL = 101
      CALL GETURN(NAMFIL)
      CALL REPORT('CASE2', IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
C
C     ---- CASE 3: branch (b), IFIST(IFST+1) .GT. 0 --------------------
C     IFIAT(7) = 131082, IAND '00007FFF'X -> 10.  FCB(1,10) = 2, so
C     IPRVOP must come back as 0.  No IUNIT store on this branch.
      CALL RESET(IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
      NAMFIL = 205
      CALL GETURN(NAMFIL)
      CALL REPORT('CASE3', IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
C
C     ---- CASE 4: branch (c), IFIST(IFST+1) .LE. 0 --------------------
C     NAMFIL 400 is outside 101..320 and IFIST(6) = -3, so
C     IFILEX = IXFIAT(IABS(-3)+1) = IXFIAT(4) = 12.
      CALL RESET(IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
      NAMFIL = 400
      CALL GETURN(NAMFIL)
      CALL REPORT('CASE4', IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
C
C     ---- CASE 5: no match --------------------------------------------
C     NAMFIL 999 is not in /XFIST/, so IFILEX is zeroed and the DSIOF
C     open-state fields keep their sentinels.
      CALL RESET(IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
      NAMFIL = 999
      CALL GETURN(NAMFIL)
      CALL REPORT('CASE5', IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
C
C     ---- CASE 6: MAXPRI rejection on branch (a) ----------------------
C     IFIAT(11) = 255, IAND '00007FFF'X -> 255 .GT. MAXPRI (80), so
C     IFILEX is zeroed and IUNIT(104-100) = IUNIT(4) is NOT stored.
      CALL RESET(IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
      NAMFIL = 104
      CALL GETURN(NAMFIL)
      CALL REPORT('CASE6', IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
      STOP
      END
C
      SUBROUTINE RESET(IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
C     Restore the mutable DSIOF fields and IUNIT(1..4) to fixed
C     sentinels before each GETURN call.
      INTEGER IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, J
      INTEGER*2 IUNIT(220)
      IFILEX = 0
      LASNAM = 0
      IPRVOP = -1
      NLR    = -1
      NBLOCK = -1
      DO 10 J = 1, 4
         IUNIT(J) = -1
   10 CONTINUE
      RETURN
      END
C
      SUBROUTINE REPORT(TAG, IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, IUNIT)
C     Print the six DSIOF fields plus IUNIT(1..4) in fixed I12 format.
      CHARACTER*5 TAG
      INTEGER IFILEX, LASNAM, IPRVOP, NLR, NBLOCK, J
      INTEGER*2 IUNIT(220)
      WRITE(6,900) TAG
      WRITE(6,901) IFILEX
      WRITE(6,901) LASNAM
      WRITE(6,901) IPRVOP
      WRITE(6,901) NLR
      WRITE(6,901) NBLOCK
      DO 10 J = 1, 4
         WRITE(6,901) IUNIT(J)
   10 CONTINUE
      RETURN
  900 FORMAT(A5)
  901 FORMAT(I12)
      END
