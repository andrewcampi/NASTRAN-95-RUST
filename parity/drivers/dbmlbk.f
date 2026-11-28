      PROGRAM DBMLBKDR
C     Fixed-input driver for DBMLBK.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     DBMLBK ( LASBLK ) returns the last block number allocated to the
C     unit IFILEX:
C
C       LASBLK = FCB( 6, IFILEX )
C       IF ( LASBLK .NE. 0 ) RETURN
C       INDEX  = FCB( 10, IFILEX )
C       IF ( INDEX .EQ. 0 ) LASBLK = 0
C       ELSE LASBLK = MEM( INDEX+3 )
C
C     IFILEX is 1 in every case.  FCB is seeded with the sentinel -777
C     and MEM with the sentinel -888, so the printed LASBLK proves which
C     word was actually read:
C
C       CASE 1: FCB(6,1) = 42 (non-zero) -> LASBLK = 42.  FCB(10,1) and
C               MEM are never consulted; FCB(10,1) stays at -777.
C       CASE 2: FCB(6,1) = 0 and FCB(10,1) = 0 -> LASBLK = 0.  MEM(3)
C               is seeded to 5555, so a stray MEM(INDEX+3) read with
C               INDEX = 0 would show up as 5555 instead of 0.
C       CASE 3: FCB(6,1) = 0 and FCB(10,1) = 5 -> LASBLK = MEM(8) = 1234,
C               i.e. the 1-based MEM(INDEX+3) offset.
C
C     The neighbouring column FCB(6,2)/FCB(10,2) is seeded with different
C     values, so reading the wrong FCB column would also show up.
      INTEGER MAXPRI, MAXFCB
      PARAMETER ( MAXPRI = 80, MAXFCB = 89 )
      INTEGER IEOR, IOERR, IPRVOP, IRETRN, IRWORD, IDATAD
      INTEGER IDSN, LCW, LWORDS, MASKH1, MASKH2
      INTEGER MASKE1, MASKE2, MASKE3, MASKE4, MAXDSN, NWORDS
      INTEGER NBUFF, IOBLK, NBFZ, NLR
      INTEGER MASKQ1, MASKQ2, MASKQ3, MASKQ4, IDSX, IDSP
      INTEGER IDSC, IDSRH, IDSRT, IDSSB, IDSSE, IDSCH
      INTEGER IDSCT, IDSSH, IDSST, IDSSD, IDSEB, IDSEF
      INTEGER IBLOCK, LASNAM, MCBMAS, MULQ1, MULQ2, MULQ3
      INTEGER LHALF
      INTEGER LENDSP, LENWPB, NWRDEL
      INTEGER IDBBAS, IDBFRE, IDBDIR, INDBAS, INDCLR, INDCBP
      INTEGER NBLOCK, LENALC, IOCODE, IFILEX, NAME, MAXALC
      INTEGER MAXBLK, MAXDSK, IDBLEN, IDBADR, IBASBF, INDDIR
      INTEGER NUMOPN, NUMCLS, NUMWRI, NUMREA, LENOPC
      INTEGER FCB
      INTEGER MEM
      INTEGER I, J, K, LASBLK
      EXTERNAL DBMLBK
      COMMON / DSIO /  IEOR
     *,         IOERR , IPRVOP, IRETRN, IRWORD, IDATAD
     *,         IDSN  , LCW   , LWORDS, MASKH1, MASKH2
     *,         MASKE1, MASKE2, MASKE3, MASKE4, MAXDSN, NWORDS
     *,         NBUFF , IOBLK , NBFZ  , NLR
     *,         MASKQ1, MASKQ2, MASKQ3, MASKQ4, IDSX  , IDSP
     *,         IDSC  , IDSRH , IDSRT , IDSSB , IDSSE , IDSCH
     *,         IDSCT , IDSSH , IDSST , IDSSD , IDSEB , IDSEF
     *,         IBLOCK, LASNAM, MCBMAS, MULQ1 , MULQ2 , MULQ3
     *,         LHALF
     *,         LENDSP, LENWPB, NWRDEL(4)
      COMMON /DBM   / IDBBAS, IDBFRE, IDBDIR, INDBAS, INDCLR, INDCBP
     *,                 NBLOCK, LENALC, IOCODE, IFILEX, NAME,   MAXALC
     *,                 MAXBLK, MAXDSK, IDBLEN, IDBADR, IBASBF, INDDIR
     *,                 NUMOPN, NUMCLS, NUMWRI, NUMREA, LENOPC
      COMMON / FCB  / FCB(17,MAXFCB)
      COMMON /ZZZZZZ/ MEM(10)
C
      DO 200 K = 1, 3
C
C       RESET FCB AND MEM TO THE FIXED SEED STATE
C
        DO 10 J = 1, MAXFCB
          DO 5 I = 1, 17
            FCB(I,J) = -777
    5     CONTINUE
   10   CONTINUE
        DO 20 I = 1, 10
          MEM(I) = -888
   20   CONTINUE
C
C       MEM(3) IS THE WORD A BUGGY INDEX=0 PATH WOULD READ;
C       MEM(8) IS MEM(INDEX+3) FOR INDEX = 5.
C
        MEM(3) = 5555
        MEM(8) = 1234
C
C       PER-CASE ARGUMENTS (IFILEX = 1 THROUGHOUT)
C
        IFILEX = 1
        IF (K .EQ. 1) THEN
          FCB( 6,1) = 42
          FCB(10,1) = -777
        ELSE IF (K .EQ. 2) THEN
          FCB( 6,1) = 0
          FCB(10,1) = 0
        ELSE
          FCB( 6,1) = 0
          FCB(10,1) = 5
        END IF
C
C       NEIGHBOURING COLUMN: A WRONG COLUMN WOULD RETURN -777
C
        FCB( 6,2) = 99
        FCB(10,2) = 7
C
        CALL DBMLBK(LASBLK)
C
        WRITE(6,900) K, LASBLK
  200 CONTINUE
      STOP
C
  900 FORMAT('CASE ',I1,' LASBLK=',I12)
      END
