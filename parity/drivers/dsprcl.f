      PROGRAM DSPRCLDR
C     Fixed-input driver for DSPRCL.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     DSPRCL ( mds/dsprcl.f ) unpacks a DSIOF block header into BLOCK(15):
C
C       BLOCK(  2 ) = IAND( IBASE( INDCBP ), MASKQ4 )
C       BLOCK(  3 ) = IAND( IBASE( INDCBP ), MASKQ3 )
C       BLOCK(  3 ) = BLOCK( 3 ) / MULQ3
C       BLOCK( 11 ) = NWRDEL( BLOCK( 2 ) )
C       BLOCK( 12 ) = IBASE( INDCBP+1 )
C       BLOCK( 14 ) = IDIV( BLOCK( 2 ) )      DATA IDIV / 1, 2, 1, 2 /
C
C     The driver includes the very same COMMON blocks, sets MASKQ3,
C     MASKQ4, MULQ3 and NWRDEL(1..4) to the DSIODD defaults, seeds
C     IBASE(INDCBP) with fixed hex words chosen so BLOCK(2) lands on
C     1, 2, 3 and 4 across four calls, and prints the five fields DSPRCL
C     stores.  BLOCK is pre-filled with a sentinel so a field DSPRCL
C     fails to store (or stores spuriously) shows up in the golden file.
C
C     Build (the .COM includes live next to the Fortran source):
C       gfortran -O0 -ffp-contract=off -fno-fast-math -std=legacy -fdec \
C         -ffixed-line-length-none -I vendor/nastran95/mds \
C         -o /tmp/dsprcl_drv parity/drivers/dsprcl.f \
C         vendor/nastran95/mds/dsprcl.f
C
      INCLUDE 'DSIOF.COM'
      INCLUDE 'XNSTRN.COM'
      INTEGER BLOCK( 15 )
      INTEGER I, J
      INTEGER SEED( 4 ), SEED2( 4 )
      EXTERNAL DSPRCL
C
C     ---- DSIODD defaults for the state DSPRCL reads -------------------
      MASKQ3 = '0000FF00'X
      MASKQ4 = '000000FF'X
      MULQ3  = 2**8
      NWRDEL( 1 ) = 1
      NWRDEL( 2 ) = 2
      NWRDEL( 3 ) = 2
      NWRDEL( 4 ) = 4
C
C     ---- fixed inputs -------------------------------------------------
C     INDCBP = 5, so DSPRCL reads IBASE(5) and IBASE(6).  IBASE(4) and
C     IBASE(7) are seeded with distinct sentinels so an off-by-one in the
C     1-based indexing is visible in the golden file.
      INDCBP = 5
      DO 5 J = 1, 700000
         IBASE( J ) = 0
    5 CONTINUE
      IBASE( 4 ) = '11111111'X
      IBASE( 7 ) = '22222222'X
C
C     IBASE(INDCBP) low byte selects BLOCK(2) = 1, 2, 3, 4; the second
C     byte selects BLOCK(3) after the MASKQ3 mask and the MULQ3 divide.
C     High bits are set on purpose: the masks must clear them.
      SEED( 1 ) = '1234A501'X
      SEED( 2 ) = 'FFFF0002'X
      SEED( 3 ) = '00000003'X
      SEED( 4 ) = '7FFF7F04'X
C     IBASE(INDCBP+1) becomes BLOCK(12) verbatim.
      SEED2( 1 ) = 100
      SEED2( 2 ) = -200
      SEED2( 3 ) = 300000
      SEED2( 4 ) = 0
C
C     The state DSPRCL reads, printed once so the golden pins it.
      WRITE(6,900) 'MASKQ3', MASKQ3
      WRITE(6,900) 'MASKQ4', MASKQ4
      WRITE(6,900) 'MULQ3 ', MULQ3
      DO 10 I = 1, 4
         WRITE(6,901) 'NWRDEL', I, NWRDEL( I )
   10 CONTINUE
      WRITE(6,902) 'INDCBP', INDCBP
C
C     ---- four calls under test ----------------------------------------
      DO 20 I = 1, 4
         IBASE( INDCBP )   = SEED( I )
         IBASE( INDCBP+1 ) = SEED2( I )
         DO 15 J = 1, 15
            BLOCK( J ) = -999
   15    CONTINUE
         CALL DSPRCL( BLOCK )
         WRITE(6,903) I, IBASE( INDCBP ), IBASE( INDCBP+1 )
         WRITE(6,904) BLOCK( 2 ), BLOCK( 3 ), BLOCK( 11 ), BLOCK( 12 ),
     *                BLOCK( 14 )
C        BLOCK(1) and BLOCK(15) are never written by DSPRCL.
         WRITE(6,905) BLOCK( 1 ), BLOCK( 15 )
   20 CONTINUE
      STOP
  900 FORMAT(A6,1X,Z8.8)
  901 FORMAT(A6,I1,1X,Z8.8)
  902 FORMAT(A6,1X,I12)
  903 FORMAT('CALL',I2,1X,Z8.8,1X,Z8.8)
  904 FORMAT(5I12)
  905 FORMAT(2I12)
      END
