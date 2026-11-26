      PROGRAM FORNUMDR
C     Fixed-input driver for FORNUM.  Prints deterministic, fixed-format
C     text only (no dates, timing, memory, or uninitialized values).
C
C     FORNUM ( FORM, ICHAR, IMULT ) scans FORM(200) starting at ICHAR:
C     it skips CHARACTER*1 blanks, then accumulates consecutive digits
C     into IMULT = IMULT*10 + digit, advancing ICHAR, and returns with
C     ICHAR pointing at the first non-digit.  IMULT starts at 0.
      CHARACTER*1 FORM(200)
      INTEGER     ICHAR, IMULT
      EXTERNAL    FORNUM
C
C     CASE 1: leading blanks, then a multi-digit field terminated by a
C     non-digit.  ICHAR must come back pointing at the terminator.
      CALL FILL(FORM)
      FORM(1) = ' '
      FORM(2) = ' '
      FORM(3) = ' '
      FORM(4) = '1'
      FORM(5) = '2'
      FORM(6) = '3'
      FORM(7) = 'X'
      ICHAR = 1
      CALL FORNUM(FORM,ICHAR,IMULT)
      WRITE(6,900) 'CASE1', IMULT, ICHAR
C
C     CASE 2: multi-digit field starting at the first character, with
C     no leading blanks at all.
      CALL FILL(FORM)
      FORM(1) = '9'
      FORM(2) = '8'
      FORM(3) = '7'
      FORM(4) = '6'
      FORM(5) = '5'
      FORM(6) = ' '
      ICHAR = 1
      CALL FORNUM(FORM,ICHAR,IMULT)
      WRITE(6,900) 'CASE2', IMULT, ICHAR
C
C     CASE 3: zero-length field -- the very first character is already
C     a non-digit, so IMULT stays 0 and ICHAR is unchanged.
      CALL FILL(FORM)
      FORM(1) = 'A'
      FORM(2) = '4'
      FORM(3) = '2'
      ICHAR = 1
      CALL FORNUM(FORM,ICHAR,IMULT)
      WRITE(6,900) 'CASE3', IMULT, ICHAR
C
C     CASE 4: field at the end of the buffer.  The digits run up to the
C     last element FORM(200), which is the non-digit terminator, so the
C     scan never reads past the end of FORM.
      CALL FILL(FORM)
      FORM(196) = '1'
      FORM(197) = '2'
      FORM(198) = '3'
      FORM(199) = '4'
      FORM(200) = 'Z'
      ICHAR = 196
      CALL FORNUM(FORM,ICHAR,IMULT)
      WRITE(6,900) 'CASE4', IMULT, ICHAR
      STOP
  900 FORMAT(A5,I8,I8)
      END
C
      SUBROUTINE FILL(FORM)
C     Blank out the whole 200-character form buffer.
      CHARACTER*1 FORM(200)
      INTEGER     I
      DO 10 I = 1, 200
         FORM(I) = ' '
   10 CONTINUE
      RETURN
      END
