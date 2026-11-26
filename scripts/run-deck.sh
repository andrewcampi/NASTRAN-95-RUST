#!/bin/bash
# Run one demo deck through the rebuilt baseline.
# usage: scripts/run-deck.sh d01000a
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DECK="${1:?usage: run-deck.sh <deck-name>}"
VENDOR="$ROOT/vendor/nastran95"
BIN="$ROOT/build/nastran"
OUT="$ROOT/build/run/$DECK"

[ -x "$BIN" ] || { echo "build/nastran missing; run scripts/build-baseline.sh first" >&2; exit 1; }
[ -f "$VENDOR/inp/$DECK.inp" ] || { echo "no such deck: $DECK" >&2; exit 1; }

rm -rf "$OUT"
mkdir -p "$OUT"
# READFILE <name> searches the working directory; all alt packages are small.
cp "$VENDOR/alt/"* "$OUT/" 2>/dev/null || true

cd "$OUT"
env RFDIR="$VENDOR/rf" DIRCTY="$OUT" LOGNM="$OUT/run.log" \
    DICTNM=none PUNCHNM=none OPTPNM=none NPTPNM=none PLTNM=none \
    FTN11=none FTN12=none FTN13=none FTN14=none FTN15=none FTN16=none \
    FTN17=none FTN18=none FTN19=none FTN20=none FTN21=none \
    DBMEM=12000000 OCMEM=2000000 \
    "$BIN" < "$VENDOR/inp/$DECK.inp" > "$OUT/out.txt" 2> "$OUT/err.txt"

echo "$OUT/out.txt"
