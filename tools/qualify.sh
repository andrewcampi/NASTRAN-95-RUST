#!/bin/bash
# Run the 132-deck baseline qualification and fail if the status counts regress.
# The baseline has one occasionally flaky deck, so a failed run is retried once
# from a clean output directory before the gate fails.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

run_once() {
    rm -rf build/qual
    python3 parity/run_all.py > /tmp/gaggle-parity.log 2>&1 || return 1
    python3 parity/qualify.py > /tmp/gaggle-qualify.log 2>&1 || return 1
    python3 - "$ROOT" <<'PY' || return 1
import json, sys
from collections import Counter
from pathlib import Path
root=Path(sys.argv[1])
want=json.loads((root/'qualification/baseline.json').read_text())['counts']
rows=json.loads((root/'build/qual/final_report.json').read_text())
got=dict(Counter(r['status'] for r in rows))
if got != want:
    print('QUALIFICATION REGRESSION', got, 'expected', want)
    raise SystemExit(1)
print('QUALIFICATION OK', got)
PY
}

if ! run_once; then
    echo "qualification retry after a failed run" >&2
    run_once
fi
