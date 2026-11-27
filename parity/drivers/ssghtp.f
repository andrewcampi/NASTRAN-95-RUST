      PROGRAM SSGHTPDRV
C     Fixed-input driver for SSGHTP.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     SSGHTP(ORDER,Z,LZ) applies the permutation ORDER in place to Z by
C     walking the pointer chains, marking visited slots with the
C     1000000 sentinel, and finally clearing every flag.
C
C     CASE 1: LZ=8 permutation with one 4-hop chain (1->3->2->5->1), one
C     3-hop chain (4->7->8->4) and one fixed point (6->6).
      INTEGER ORDER(8), Z(8)
      EXTERNAL SSGHTP
      LZ = 8
      ORDER(1) = 3
      ORDER(2) = 5
      ORDER(3) = 2
      ORDER(4) = 7
      ORDER(5) = 1
      ORDER(6) = 6
      ORDER(7) = 8
      ORDER(8) = 4
      Z(1) = 10
      Z(2) = 20
      Z(3) = 30
      Z(4) = 40
      Z(5) = 50
      Z(6) = 60
      Z(7) = 70
      Z(8) = 80
      CALL SSGHTP(ORDER,Z,LZ)
      WRITE(6,900) 'CASE1', LZ, (ORDER(I),I=1,LZ)
      WRITE(6,901) 'Z1   ', LZ, (Z(I),I=1,LZ)
C
C     CASE 2: LZ=8 permutation whose first slot is a fixed point and whose
C     chains are entered from the middle (2->8->3->2 and 4->5->7->6->4).
      ORDER(1) = 1
      ORDER(2) = 8
      ORDER(3) = 2
      ORDER(4) = 5
      ORDER(5) = 7
      ORDER(6) = 4
      ORDER(7) = 6
      ORDER(8) = 3
      Z(1) = -1
      Z(2) = -2
      Z(3) = -3
      Z(4) = -4
      Z(5) = -5
      Z(6) = -6
      Z(7) = -7
      Z(8) = -8
      CALL SSGHTP(ORDER,Z,LZ)
      WRITE(6,900) 'CASE2', LZ, (ORDER(I),I=1,LZ)
      WRITE(6,901) 'Z2   ', LZ, (Z(I),I=1,LZ)
      STOP
  900 FORMAT(A5,I3,8I4)
  901 FORMAT(A5,I3,8I4)
      END
