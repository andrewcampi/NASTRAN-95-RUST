      PROGRAM DBMRLBDR
C     Fixed-input driver for DBMRLB.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     DBMRLB ( INDEX ) releases the in-memory block INDEX back to the free
C     chain.  MEM is the /ZZZZZZ/ open-core array; word MEM(I) is the
C     "previous" pointer of a block and MEM(I+1) its "next" pointer.
C
C     The seeded state is a fixed 3-block chain 1 -> 5 -> 9 (each block
C     occupies four words: prev, next, data, data) plus a fixed 2-block
C     free chain 13 -> 17.  MEM is reset to that state before every case.
C
C       CASE 1: release the tail block 9 with an EMPTY free chain
C               (IDBFRE = 0)  ->  the block becomes the whole free chain.
C       CASE 2: release the tail block 9 with a NON-EMPTY free chain
C               (IDBFRE = 13) ->  the block is spliced in front of it.
C       CASE 3: release the HEAD block 1 of the 3-block chain, which
C               forces the 110 forward walk to the tail block 9 before
C               the same splice happens.
C
C     IDBFRE and MEM(1..24) are printed after each case.
      INTEGER MEM(24)
      INTEGER IDBBAS, IDBFRE, IDBDIR, INDBAS, INDCLR, INDCBP
      INTEGER NBLOCK, LENALC, IOCODE, IFILEX, NAME, MAXALC
      INTEGER MAXBLK, MAXDSK, IDBLEN, IDBADR, IBASBF, INDDIR
      INTEGER NUMOPN, NUMCLS, NUMWRI, NUMREA, LENOPC
      INTEGER ISYSBF, IWR
      INTEGER I, K, INDEX
      EXTERNAL DBMRLB
      COMMON /ZZZZZZ/ MEM
      COMMON /DBM   / IDBBAS, IDBFRE, IDBDIR, INDBAS, INDCLR, INDCBP
     *,                NBLOCK, LENALC, IOCODE, IFILEX, NAME,   MAXALC
     *,                MAXBLK, MAXDSK, IDBLEN, IDBADR, IBASBF, INDDIR
     *,                NUMOPN, NUMCLS, NUMWRI, NUMREA, LENOPC
      COMMON /SYSTEM/ ISYSBF, IWR
C
      DO 200 K = 1, 3
C
C       RESET MEM TO THE FIXED SEED STATE
C
        DO 10 I = 1, 24
          MEM(I) = 0
   10   CONTINUE
C
C       ALLOCATED CHAIN 1 -> 5 -> 9
C
        MEM( 1) = 0
        MEM( 2) = 5
        MEM( 3) = 101
        MEM( 4) = 102
        MEM( 5) = 1
        MEM( 6) = 9
        MEM( 7) = 103
        MEM( 8) = 104
        MEM( 9) = 5
        MEM(10) = 0
        MEM(11) = 105
        MEM(12) = 106
C
C       FREE CHAIN 13 -> 17
C
        MEM(13) = 0
        MEM(14) = 17
        MEM(15) = 201
        MEM(16) = 202
        MEM(17) = 13
        MEM(18) = 0
        MEM(19) = 203
        MEM(20) = 204
C
C       PER-CASE ARGUMENTS
C
        IF (K .EQ. 1) THEN
          IDBFRE = 0
          INDEX  = 9
        ELSE IF (K .EQ. 2) THEN
          IDBFRE = 13
          INDEX  = 9
        ELSE
          IDBFRE = 13
          INDEX  = 1
        END IF
C
        CALL DBMRLB(INDEX)
C
        WRITE(6,900) K, IDBFRE
        DO 20 I = 1, 24
          WRITE(6,901) I, MEM(I)
   20   CONTINUE
  200 CONTINUE
      STOP
C
  900 FORMAT('CASE ',I1,' IDBFRE=',I12)
  901 FORMAT('  MEM',I3,'=',I12)
      END
