#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import difflib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VENDOR = ROOT / "vendor/nastran95"
GOLD = VENDOR / "demoout"
INP = VENDOR / "inp"
OUT = ROOT / "build/qual"

NUM = re.compile(r"[-+]?(?:\d+\.\d*|\.\d+|\d+)(?:[EeDd][-+]?\d+)?")


def clean(text: str) -> list[str]:
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    out = []
    for line in text.split("\n"):
        line = line.rstrip()
        if re.search(r"SUN SOLARIS NASTRAN\s*/\s*.*PAGE\s+\d+", line):
            continue
        if line.startswith(" DATE:") or line.startswith(" END TIME:"):
            continue
        if "TOTAL WALL CLOCK TIME" in line:
            continue
        if "SUN COMPUTER SYSTEMS" in line or "SOLARIS VERSION" in line:
            continue
        if "SYSTEM RELEASE" in line:
            continue
        if "TURN DIAG 48 ON" in line or "MODULES AND NEW BULKDATA" in line:
            continue
        if "EPSILON SUB E" in line:
            continue
        if re.search(r"^\s*0?\s*RESTART\s+\S+", line):
            continue
        if re.search(r"S Y S T E M\s+G E N E R A T E D\s+S E Q G P", line):
            continue
        if re.match(r"^\s*SEQGP\b", line):
            continue
        if "SYSTEM INFORMATION MESSAGE 6201" in line:
            continue
        if "USER WARNING  MESSAGE  3058" in line or "EPSILON IS LARGER THAN" in line:
            continue
        if re.search(r"^\s*SIZE OF FILE", line):
            continue
        if "FILES HAVE BEEN ALLOCATED TO THE SOF" in line:
            continue
        if "REENTER AT DMAP SEQUENCE" in line:
            continue
        if not line.strip():
            continue
        out.append(line)
    return out


def masked(line: str) -> str:
    return NUM.sub("<N>", " ".join(line.split()))


def numbers(line: str) -> list[float]:
    vals = []
    for m in NUM.finditer(line):
        try:
            vals.append(float(m.group(0).replace("D", "E").replace("d", "e")))
        except ValueError:
            vals.append(float("nan"))
    return vals


def close(a: float, b: float) -> bool:
    if a == b:
        return True
    if a != a or b != b:
        return False
    scale = max(abs(a), abs(b), 1e-30)
    return abs(a - b) <= 2e-5 * scale


def compare(g: list[str], o: list[str]) -> tuple[str, str, int, str]:
    """Return status, reason, changed-line count, first difference."""
    if g == o:
        return "PASS", "", 0, ""
    # Compare aligned lines. Any masked-text difference is structural.
    sm = difflib.SequenceMatcher(None, [masked(x) for x in g], [masked(x) for x in o])
    structural = 0
    numeric_bad = 0
    numeric_close = 0
    first = ""
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag == "equal":
            for a, b in zip(g[i1:i2], o[j1:j2]):
                na, nb = numbers(a), numbers(b)
                if len(na) != len(nb):
                    numeric_bad += 1
                    if not first:
                        first = f"G:{a[:100]} | O:{b[:100]}"
                    continue
                for x, y in zip(na, nb):
                    if x == y:
                        continue
                    if close(x, y):
                        numeric_close += 1
                    else:
                        numeric_bad += 1
                        if not first:
                            first = f"G:{a[:100]} | O:{b[:100]}"
                        break
            continue
        for a in g[i1:i2]:
            structural += 1
            if not first:
                first = f"missing: {a[:120]}"
        for b in o[j1:j2]:
            structural += 1
            if not first:
                first = f"extra: {b[:120]}"
    if structural:
        return "FAIL", f"{structural} structural line(s)", structural + numeric_bad + numeric_close, first
    if numeric_bad:
        return "FAIL", f"{numeric_bad} numeric value(s) outside tolerance", numeric_bad + numeric_close, first
    if numeric_close:
        return "KNOWN_DIFF", f"{numeric_close} last-digit numeric value(s)", numeric_close, first
    return "PASS", "", 0, ""


def deps(name: str) -> dict:
    p = INP / f"{name}.inp"
    text = p.read_text(errors="replace").upper() if p.exists() else ""
    return {
        "readfile": "READFILE" in text,
        "ires": bool(re.search(r"PARAM\s+IRES", text)),
        "chkpnt": "CHKPNT" in text,
        "files": (re.search(r"FILES\s*=\s*(\w+)", text).group(1) if re.search(r"FILES\s*=\s*(\w+)", text) else ""),
        "alt": bool(re.search(r"(^|\n)\s*ALTER\s", text)),
    }


def main() -> None:
    old = {r["deck"]: r for r in json.load(open(OUT / "report.json"))}
    results = []
    for deck in sorted(GOLD.glob("*.out")):
        name = deck.stem
        meta = old.get(name, {})
        out_path = OUT / name / "out.txt"
        got = out_path.read_text(errors="replace") if out_path.exists() else ""
        want = deck.read_text(errors="replace")
        g, o = clean(want), clean(got)
        status, reason, changed, first = compare(g, o)
        d = deps(name)
        if meta.get("exit", 0) != 0:
            status, reason = "FAIL", f"exit {meta.get('exit')}"
        elif not got:
            status, reason = "FAIL", "empty output"
        # A deck that only differs by numeric noise is a known-diff baseline.
        results.append(
            {
                "deck": name,
                "status": status,
                "reason": reason,
                "changed": changed,
                "gold_lines": len(g),
                "our_lines": len(o),
                "first_diff": first,
                **d,
            }
        )
    (OUT / "report2.json").write_text(json.dumps(results, indent=2) + "\n")
    counts: dict[str, int] = {}
    for r in results:
        counts[r["status"]] = counts.get(r["status"], 0) + 1
    print("SUMMARY", counts)
    for r in results:
        if r["status"] != "PASS":
            print(
                f"{r['deck']:<10} {r['status']:<11} {r['reason']:<28} "
                f"changed={r['changed']:<4} files={r['files']:<5} "
                f"readfile={int(r['readfile'])} ires={int(r['ires'])} | {r['first_diff'][:80]}"
            )


if __name__ == "__main__":
    main()
