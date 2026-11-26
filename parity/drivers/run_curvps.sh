#!/bin/bash
# Compile parity/drivers/curvps.f against the pinned Fortran baseline, run it,
# and diff its stdout against parity/golden/curvps.txt. The Fortran is the
# oracle; the golden is never regenerated from Rust.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

gfortran -O0 -ffp-contract=off -fno-fast-math -std=legacy -fdec \
  -ffixed-line-length-none -o "$OUT/curvps_drv" \
  parity/drivers/curvps.f vendor/nastran95/mis/curvps.f

"$OUT/curvps_drv" > "$OUT/curvps.out"

diff -u parity/golden/curvps.txt "$OUT/curvps.out"
echo "CURVPS parity OK (empty diff vs parity/golden/curvps.txt)"
