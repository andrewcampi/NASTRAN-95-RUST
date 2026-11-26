#!/bin/bash
# Build the reproducible NASTRAN-95 baseline from the pinned vendor tree.
# Run from the repository root (or inside the pinned container with /work mounted).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENDOR="$ROOT/vendor/nastran95"
BUILD="$ROOT/build"
SRC="$BUILD/nastran95"

rm -rf "$SRC"
mkdir -p "$BUILD" "$SRC"
cp -a "$VENDOR/mis" "$VENDOR/mds" "$VENDOR/bd" "$VENDOR/rf" "$VENDOR/alt" "$VENDOR/utility" "$SRC/"
mkdir -p "$SRC/bin"
cp "$VENDOR/bin/nastrn.f" "$SRC/bin/nastrn.f"

cd "$SRC"

# Normalize legacy files: strip DOS EOF bytes, then apply the reviewed
# portability patch set. Keep the pristine CRLF layout except for the files the
# patch touches; Hollerith continuations depend on that byte layout.
find . -type f \( -name '*.f' -o -name '*.COM' \) -print0 \
  | xargs -0 perl -pi -e 's/\x1a//g'
for f in mis/endsys.f mis/pexit.f mis/sofut.f mis/strscn.f mis/xdcode.f \
         mds/cputim.f mds/nastim.f; do
  perl -pi -e 's/\r$//' "$f"
done
patch -p1 < "$ROOT/patches/nastran95.patch"

mkdir -p obj
: > "$BUILD/build-errors.txt"
: > "$BUILD/build-failures.txt"
export BUILD

build_one() {
  f="${1#./}"
  o="obj/$(echo "$f" | tr '/' '_').o"
  if [ "$f" = "mis/strscn.f" ]; then
    # Legacy data in the sequence field; fixed 72 columns is correct.
    gfortran -std=legacy -fdec -fallow-invalid-boz -fno-automatic \
      -ffixed-line-length-72 \
      -I. -Imds -Imis -c "$f" -o "$o" >>"$BUILD/build-errors.txt" 2>&1 \
      || echo "$f" >>"$BUILD/build-failures.txt"
  else
    gfortran -std=legacy -fdec -fallow-invalid-boz -fno-automatic \
      -ffixed-line-length-none -ffree-line-length-none \
      -I. -Imds -Imis -c "$f" -o "$o" >>"$BUILD/build-errors.txt" 2>&1 \
      || echo "$f" >>"$BUILD/build-failures.txt"
  fi
}
export -f build_one

find . -name '*.f' -not -path './bin/nastrn.f' -not -path './utility/nastplot.f' \
  | sort | xargs -P 8 -I{} bash -c 'build_one "$@"' _ {}

if [ -s "$BUILD/build-failures.txt" ]; then
  echo "source files failed to compile:" >&2
  cat "$BUILD/build-failures.txt" >&2
  exit 1
fi

gfortran -std=legacy -fdec -fallow-invalid-boz -fno-automatic \
  -ffixed-line-length-none -I. -Imds -Imis -c bin/nastrn.f -o nastrn.o

# The original bin/linknas puts the bd/ block-data objects before the library
# so their COMMON block definitions win. Reproduce that order.
BD_OBJS="$(find obj -maxdepth 1 -name 'bd_*.o' | sort)"
find obj -maxdepth 1 -name '*.o' ! -name 'bd_*.o' -print0 | xargs -0 ar rcs libnastran.a
gfortran -o "$BUILD/nastran" nastrn.o $BD_OBJS libnastran.a

echo "baseline: $BUILD/nastran"
ls -lh "$BUILD/nastran"
