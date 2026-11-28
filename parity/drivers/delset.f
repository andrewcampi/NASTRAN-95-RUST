      PROGRAM DELSETDR
C     Fixed-input driver for DELSET.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     DELSET populates the /GPTA1/ table entries for the nine dummy
C     element types 53..61 from the packed /SYSTEM/ IDUM words:
C
C        NGRIDS = IDUM(I) / 10000000
C        NC     = MOD( IDUM(I),10000000) / 10000
C        NP     = MOD( IDUM(I),10000) / 10
C        ND     = MOD( IDUM(I), 10)
C        IZERO  = (DUMTYP(I) - 1)*INCR
C
C     INCR is fixed at 1, so IZERO = DUMTYP(I)-1 and the highest word
C     touched is NE(60+15) = NE(75); NE(1..96) is printed so the words
C     DELSET must leave alone are visible too.
C
C     IDUM(1..9) is a fixed mix of zero and non-zero NGRIDS/NC/NP:
C     entries 2 and 6 have NGRIDS = 0, entries 2 and 3 are all zero,
C     and entries 4, 5 and 9 have NP = 0 to pin the "IF(NP.EQ.0) NE=0"
C     special case.  Because INCR = 1 the nine types' word ranges
C     overlap, so the NP = 0 case is only observable in the final table
C     for the last type (61); entry 9 therefore carries NP = 0.
      INTEGER DUMTYP(9)
C
      COMMON/SYSTEM/ NSKIP(45), IDUM(9)
C
      COMMON/GPTA1 / NELEM, LAST, INCR, NE(96)
C
      INTEGER I
C
      INCR  = 1
      NELEM = 0
      LAST  = 0
C
C     Sentinel so that words DELSET does not write are identifiable.
      DO 10 I = 1, 96
         NE(I) = -1
   10 CONTINUE
C
      DO 20 I = 1, 45
         NSKIP(I) = 0
   20 CONTINUE
C
C     IDUM(I) = NGRIDS*10000000 + NC*10000 + NP*10 + ND
C
C     type 53: NGRIDS=1 NC=2 NP=3 ND=4
      IDUM(1) = 10020034
C     type 54: NGRIDS=0 NC=0 NP=0 ND=0   (all zero)
      IDUM(2) = 0
C     type 55: NGRIDS=2 NC=0 NP=5 ND=1
      IDUM(3) = 20000051
C     type 56: NGRIDS=0 NC=3 NP=0 ND=7   (NP = 0 special case)
      IDUM(4) = 30007
C     type 57: NGRIDS=3 NC=1 NP=0 ND=0   (NP = 0 special case)
      IDUM(5) = 30010000
C     type 58: NGRIDS=0 NC=0 NP=9 ND=9
      IDUM(6) = 99
C     type 59: NGRIDS=1 NC=1 NP=1 ND=1
      IDUM(7) = 10010011
C     type 60: NGRIDS=9 NC=9 NP=9 ND=9
      IDUM(8) = 90090099
C     type 61: NGRIDS=0 NC=5 NP=0 ND=3   (NP = 0 special case; type 61
C                                        is the last type, so its
C                                        NE(IZERO+9) = 0 is not
C                                        overwritten by a later type)
      IDUM(9) = 50003
C
      CALL DELSET
C
      DO 30 I = 1, 96
         WRITE(6,900) NE(I)
   30 CONTINUE
      STOP
  900 FORMAT(I12)
      END
