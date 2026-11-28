      PROGRAM DSSDCBDR
C     Fixed-input driver for DSSDCB.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     DSSDCB updates the DSIOF dataset control block from the current
C     /DBM/ state:
C
C       ICLR              = INDCLR - INDBAS + 1
C       FCB( 3, IFILEX )  = ICLR
C       FCB( 4, IFILEX )  = NBLOCK
C       IBASE( INDBAS+1 ) = INDCBP - INDBAS + 1
C       IBASE( INDBAS+2 ) = ICLR
C       LASNAM            = NAME
C
C     FCB and IBASE are seeded with sentinels (-777) so the printed
C     untouched words prove that DSSDCB writes only the five words above,
C     and the printed written words prove the 1-based Fortran indexing
C     (FCB(3,IFILEX), FCB(4,IFILEX), IBASE(INDBAS+1), IBASE(INDBAS+2)).
C
C       CASE 1: the fixed inputs from the ticket --
C               IFILEX=2, NBLOCK=17, INDBAS=5, INDCLR=9, INDCBP=13,
C               NAME=1234567.
C       CASE 2: a different column/offset set --
C               IFILEX=4, NBLOCK=33, INDBAS=2, INDCLR=11, INDCBP=20,
C               NAME=7654321.
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
      INTEGER IBASE
      INTEGER I, J
      EXTERNAL DSSDCB
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
      COMMON / ZZZZZZ / IBASE(700000)
C
C     CASE 1
C
      DO 10 J = 1, MAXFCB
        DO 5 I = 1, 17
          FCB(I,J) = -777
    5   CONTINUE
   10 CONTINUE
      DO 20 I = 1, 40
        IBASE(I) = -777
   20 CONTINUE
      LASNAM = -777
      IFILEX = 2
      NBLOCK = 17
      INDBAS = 5
      INDCLR = 9
      INDCBP = 13
      NAME   = 1234567
      CALL DSSDCB
      WRITE(6,900) 'CASE1'
      WRITE(6,901) FCB(3,2)
      WRITE(6,901) FCB(4,2)
      WRITE(6,901) IBASE(6)
      WRITE(6,901) IBASE(7)
      WRITE(6,901) LASNAM
C     untouched neighbours: FCB(1,2), FCB(3,1), FCB(3,3), IBASE(5),
C     IBASE(8)
      WRITE(6,901) FCB(1,2)
      WRITE(6,901) FCB(3,1)
      WRITE(6,901) FCB(3,3)
      WRITE(6,901) IBASE(5)
      WRITE(6,901) IBASE(8)
C
C     CASE 2
C
      DO 30 J = 1, MAXFCB
        DO 25 I = 1, 17
          FCB(I,J) = -777
   25   CONTINUE
   30 CONTINUE
      DO 40 I = 1, 40
        IBASE(I) = -777
   40 CONTINUE
      LASNAM = -777
      IFILEX = 4
      NBLOCK = 33
      INDBAS = 2
      INDCLR = 11
      INDCBP = 20
      NAME   = 7654321
      CALL DSSDCB
      WRITE(6,900) 'CASE2'
      WRITE(6,901) FCB(3,4)
      WRITE(6,901) FCB(4,4)
      WRITE(6,901) IBASE(3)
      WRITE(6,901) IBASE(4)
      WRITE(6,901) LASNAM
C     untouched neighbours: FCB(1,4), FCB(3,3), FCB(3,5), IBASE(2),
C     IBASE(5)
      WRITE(6,901) FCB(1,4)
      WRITE(6,901) FCB(3,3)
      WRITE(6,901) FCB(3,5)
      WRITE(6,901) IBASE(2)
      WRITE(6,901) IBASE(5)
      STOP
  900 FORMAT(A5)
  901 FORMAT(I12)
      END
