      PROGRAM MAKMCBDR
C     Fixed-input driver for MAKMCB.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     MAKMCB ( MCB, IFILE, IROW, IF, IT ) fills the 7-word MCB block:
C     MCB(1)=IFILE, MCB(2)=0, MCB(3)=IROW, MCB(4)=IF, MCB(5)=IT,
C     MCB(6)=0, MCB(7)=0.  Pure argument-only routine, no COMMON, no I/O.
      INTEGER MCB(7), I, IFILE, IROW, IFLAG, IT
      EXTERNAL MAKMCB
C
C     CASE 1: the fixed inputs from the ticket.
      DO 10 I = 1, 7
         MCB(I) = 0
   10 CONTINUE
      IFILE = 101
      IROW  = 7
      IFLAG = 3
      IT    = 9
      CALL MAKMCB(MCB,IFILE,IROW,IFLAG,IT)
      WRITE(6,900) 'CASE1'
      WRITE(6,901) (MCB(I), I = 1, 7)
C
C     CASE 2: MCB pre-filled with sentinels, so every one of the seven
C     words must be overwritten by MAKMCB -- including the three zeros,
C     which prove MCB(2), MCB(6) and MCB(7) are stored, not left alone.
      DO 20 I = 1, 7
         MCB(I) = -999
   20 CONTINUE
      IFILE = -5
      IROW  = 0
      IFLAG = 12
      IT    = -3
      CALL MAKMCB(MCB,IFILE,IROW,IFLAG,IT)
      WRITE(6,900) 'CASE2'
      WRITE(6,901) (MCB(I), I = 1, 7)
      STOP
  900 FORMAT(A5)
  901 FORMAT(I8)
      END
