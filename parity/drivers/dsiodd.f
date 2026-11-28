      PROGRAM DSIODDDR
C     Fixed-input driver for DSIODD.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     DSIODD ( mds/dsiodd.f ) is the GINO/DSIOF state initializer.  It has
C     no dummy arguments: it writes module-level state held in
C     COMMON /GINOX/ (LGINOX) and COMMON /DSIO/ (everything else), using
C     NUMFCB and NUMSOF from the PARAMETER statement in GINOX.COM.
C
C     The driver therefore includes the very same COMMON blocks, pre-fills
C     every field DSIODD is supposed to set with a sentinel (-1), calls
C     DSIODD exactly once, and prints the resulting state image.  A field
C     that DSIODD fails to store shows up as -1 in the golden file.
C
C     Build (the .COM includes live next to the Fortran source):
C       gfortran -O0 -ffp-contract=off -fno-fast-math -std=legacy -fdec \
C         -ffixed-line-length-none -I vendor/nastran95/mds \
C         -o /tmp/dsiodd_drv parity/drivers/dsiodd.f \
C         vendor/nastran95/mds/dsiodd.f
C
      INCLUDE 'GINOX.COM'
      INCLUDE 'DSIOF.COM'
      INTEGER I
      EXTERNAL DSIODD
C
C     Sentinel every field DSIODD is expected to store.
      LGINOX = -1
      LHALF  = -1
      LENDSP = -1
      LENWPB = -1
      MASKH1 = -1
      MASKH2 = -1
      MASKE1 = -1
      MASKE2 = -1
      MASKE3 = -1
      MASKE4 = -1
      MCBMAS = -1
      MAXDSN = -1
      MASKQ1 = -1
      MASKQ2 = -1
      MASKQ3 = -1
      MASKQ4 = -1
      MULQ1  = -1
      MULQ2  = -1
      MULQ3  = -1
      IDSX   = -1
      IDSP   = -1
      IDSC   = -1
      IDSRH  = -1
      IDSRT  = -1
      IDSSB  = -1
      IDSSE  = -1
      IDSCH  = -1
      IDSCT  = -1
      IDSSH  = -1
      IDSST  = -1
      IDSSD  = -1
      IDSEB  = -1
      IDSEF  = -1
      DO 10 I = 1, 4
         NWRDEL(I) = -1
   10 CONTINUE
C
C     The single call under test.
      CALL DSIODD
C
C     Fixed inputs, so the golden pins the NUMFCB/NUMSOF the state was
C     built from.
      WRITE(6,900) 'NUMFCB', NUMFCB, NUMFCB
      WRITE(6,900) 'NUMSOF', NUMSOF, NUMSOF
C
C     The state image, in source order of mds/dsiodd.f.
      WRITE(6,900) 'LGINOX', LGINOX, LGINOX
      WRITE(6,900) 'LHALF',  LHALF,  LHALF
      WRITE(6,900) 'LENDSP', LENDSP, LENDSP
      WRITE(6,900) 'LENWPB', LENWPB, LENWPB
      WRITE(6,900) 'MASKH1', MASKH1, MASKH1
      WRITE(6,900) 'MASKH2', MASKH2, MASKH2
      WRITE(6,900) 'MASKE1', MASKE1, MASKE1
      WRITE(6,900) 'MASKE2', MASKE2, MASKE2
      WRITE(6,900) 'MASKE3', MASKE3, MASKE3
      WRITE(6,900) 'MASKE4', MASKE4, MASKE4
      WRITE(6,900) 'MCBMAS', MCBMAS, MCBMAS
      WRITE(6,900) 'MAXDSN', MAXDSN, MAXDSN
      WRITE(6,900) 'MASKQ1', MASKQ1, MASKQ1
      WRITE(6,900) 'MASKQ2', MASKQ2, MASKQ2
      WRITE(6,900) 'MASKQ3', MASKQ3, MASKQ3
      WRITE(6,900) 'MASKQ4', MASKQ4, MASKQ4
      WRITE(6,900) 'MULQ1',  MULQ1,  MULQ1
      WRITE(6,900) 'MULQ2',  MULQ2,  MULQ2
      WRITE(6,900) 'MULQ3',  MULQ3,  MULQ3
      WRITE(6,900) 'IDSX',   IDSX,   IDSX
      WRITE(6,900) 'IDSP',   IDSP,   IDSP
      WRITE(6,900) 'IDSC',   IDSC,   IDSC
      WRITE(6,900) 'IDSRH',  IDSRH,  IDSRH
      WRITE(6,900) 'IDSRT',  IDSRT,  IDSRT
      WRITE(6,900) 'IDSSB',  IDSSB,  IDSSB
      WRITE(6,900) 'IDSSE',  IDSSE,  IDSSE
      WRITE(6,900) 'IDSCH',  IDSCH,  IDSCH
      WRITE(6,900) 'IDSCT',  IDSCT,  IDSCT
      WRITE(6,900) 'IDSSH',  IDSSH,  IDSSH
      WRITE(6,900) 'IDSST',  IDSST,  IDSST
      WRITE(6,900) 'IDSSD',  IDSSD,  IDSSD
      WRITE(6,900) 'IDSEB',  IDSEB,  IDSEB
      WRITE(6,900) 'IDSEF',  IDSEF,  IDSEF
      DO 20 I = 1, 4
         WRITE(6,901) 'NWRDEL', I, NWRDEL(I), NWRDEL(I)
   20 CONTINUE
      STOP
  900 FORMAT(A8,1X,Z8.8,1X,I12)
  901 FORMAT(A8,I1,1X,Z8.8,1X,I12)
      END
