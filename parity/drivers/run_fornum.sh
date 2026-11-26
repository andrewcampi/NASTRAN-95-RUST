#!/bin/bash
# Unit-driver parity gate for FORNUM (GAG-7).
#
# Compiles parity/drivers/fornum.f against the pinned Fortran source
# vendor/nastran95/mis/fornum.f with the pinned flags, runs it, and diffs its
# stdout against the committed golden parity/golden/fornum.txt.  The Fortran
# remains the oracle; the golden is never regenerated from Rust.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

DRIVER=parity/drivers/fornum.f
SOURCE=vendor/nastran95/mis/fornum.f
GOLDEN=parity/golden/fornum.txt
BIN=/tmp/fornum_drv
OUT=/tmp/fornum_drv.out

for f in "$DRIVER" "$SOURCE" "$GOLDEN"; do
    if [ ! -f "$f" ]; then
        echo "run_fornum.sh: missing $f" >&2
        exit 1
    fi
done

gfortran -O0 -ffp-contract=off -fno-fast-math -std=legacy -fdec \
    -ffixed-line-length-none -o "$BIN" "$DRIVER" "$SOURCE"

"$BIN" > "$OUT"

if ! diff -u "$GOLDEN" "$OUT"; then
    echo "run_fornum.sh: FORNUM driver output differs from $GOLDEN" >&2
    exit 1
fi

echo "run_fornum.sh: FORNUM parity OK (empty diff vs $GOLDEN)"
