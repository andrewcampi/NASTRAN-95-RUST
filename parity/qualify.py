#!/usr/bin/env python3
"""Final NASTRAN-95 baseline qualification report.

Policy:
  PASS      normalized output identical to the 1995 golden
  KNOWN_DIFF same structure, numeric values within atol=1e-9 / rtol=1e-3
  MISSING   deck needs an RSCARDS / INP1 / SOF artifact not in the repo
  RESTART   PARAM IRES deck whose 1995 golden came from a checkpoint
  FAIL      structural mismatch or numeric outside tolerance
"""
from __future__ import annotations

import difflib
import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import classify as C  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
VENDOR = ROOT / "vendor/nastran95"
GOLD = VENDOR / "demoout"
INP = VENDOR / "inp"
ALT = {p.name for p in (VENDOR / "alt").iterdir()}
OUT = ROOT / "build/qual"
ATOL = 1e-9
RTOL = 1e-3


def numeric_ok(g, o):
    sm = difflib.SequenceMatcher(None, [C.masked(x) for x in g], [C.masked(x) for x in o])
    structural = 0
    bad = 0
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag != "equal":
            structural += (i2 - i1) + (j2 - j1)
            continue
        for a, b in zip(g[i1:i2], o[j1:j2]):
            na, nb = C.numbers(a), C.numbers(b)
            if len(na) != len(nb):
                structural += 1
                continue
            for x, y in zip(na, nb):
                if x == y:
                    continue
                if abs(x - y) > ATOL + RTOL * max(abs(x), abs(y)):
                    bad += 1
    return structural, bad


def dependencies(name: str):
    text = (INP / f"{name}.inp").read_text(errors="replace")
    files = [m.upper() for m in re.findall(r"READFILE\s*(?:,\s*NOPRINT)?\s+([A-Za-z0-9_]+)", text, re.I)]
    missing = [f for f in files if f not in ALT]
    has_inp1 = bool(re.search(r"FILES\s*=\s*\(?[^)\n]*INP1", text, re.I))
    ires = bool(re.search(r"PARAM\s+IRES", text, re.I))
    return files, missing, has_inp1, ires


def main():
    old = {r["deck"]: r for r in json.load(open(OUT / "report.json"))}
    rows = []
    for gold in sorted(GOLD.glob("*.out")):
        name = gold.stem
        meta = old.get(name, {})
        got_path = OUT / name / "out.txt"
        got = got_path.read_text(errors="replace") if got_path.exists() else ""
        g = C.clean(gold.read_text(errors="replace"))
        o = C.clean(got)
        files, missing, has_inp1, ires = dependencies(name)
        structural, bad = numeric_ok(g, o)
        exact = g == o

        if meta.get("exit", 0) != 0 or not got:
            status, reason = "FAIL", f"exit {meta.get('exit') or 'no output'}"
        elif exact:
            status, reason = "PASS", "exact after normalization"
        elif structural == 0 and bad == 0:
            status, reason = "KNOWN_DIFF", "numeric within 1e-9 / 1e-3"
        elif missing or has_inp1:
            status, reason = "MISSING", "needs " + (", ".join(missing) if missing else "INP1")
        elif ires:
            status, reason = "RESTART", "1995 golden came from a checkpoint"
        elif structural:
            status, reason = "FAIL", f"{structural} structural line(s)"
        else:
            status, reason = "FAIL", f"{bad} numeric value(s) outside tolerance"

        rows.append(
            {
                "deck": name,
                "status": status,
                "reason": reason,
                "structural": structural,
                "numeric_out": bad,
                "gold_lines": len(g),
                "our_lines": len(o),
                "readfile": files,
                "missing": missing,
                "inp1": has_inp1,
                "ires": ires,
                "first_diff": C.compare(g, o)[3],
            }
        )

    (OUT / "final_report.json").write_text(json.dumps(rows, indent=2) + "\n")
    from collections import Counter

    counts = Counter(r["status"] for r in rows)
    print("QUALIFICATION", dict(counts), "total", len(rows))
    for r in rows:
        if r["status"] not in ("PASS",):
            print(
                f"{r['deck']:<10} {r['status']:<11} {r['reason']:<34} "
                f"struct={r['structural']:<4} num={r['numeric_out']:<4} | {r['first_diff'][:80]}"
            )


if __name__ == "__main__":
    main()
