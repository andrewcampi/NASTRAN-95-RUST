#!/bin/bash
# Unit-driver parity gate for SMCCRD (mis/smccrd.f).
#
# Compiles parity/drivers/smccrd.f against the pinned Fortran baseline source
# with the pinned flags, runs it, and diffs stdout against the committed
# golden. Exits 0 only when the diff is empty.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

DRV=parity/drivers/smccrd.f
SRC=vendor/nastran95/mis/smccrd.f
GOLD=parity/golden/smccrd.txt
BIN="${TMPDIR:-/tmp}/smccrd_drv"
OUT="${TMPDIR:-/tmp}/smccrd_fortran.out"

gfortran -O0 -ffp-contract=off -fno-fast-math -std=legacy -fdec \
  -ffixed-line-length-none -o "$BIN" "$DRV" "$SRC"
"$BIN" > "$OUT"

if ! diff -u "$GOLD" "$OUT"; then
  echo "SMCCRD PARITY FAIL: Fortran driver output differs from $GOLD" >&2
  exit 1
fi
echo "SMCCRD PARITY OK: $OUT matches $GOLD"
