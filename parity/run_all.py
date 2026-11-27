#!/usr/bin/env python3
from __future__ import annotations

import concurrent.futures
import difflib
import json
import os
import re
import subprocess
import time
import shutil
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
ROOT = REPO / "vendor/nastran95"
GOLD = ROOT / "demoout"
INP = ROOT / "inp"
RF = ROOT / "rf"
ALT = ROOT / "alt"
BIN = REPO / "build/nastran"
WORK = REPO / "build/qual"
WORK.mkdir(parents=True, exist_ok=True)

# The corpus includes decks that are slow under load. Keep the pool small and
# retry a timed-out deck once; six workers plus concurrent review/merge jobs
# made one deck flip between KNOWN_DIFF and FAIL.
TIMEOUT = 480
WORKERS = 3
DECK_TRIES = 2


def norm(text: str) -> list[str]:
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    out: list[str] = []
    for line in text.split("\n"):
        line = line.rstrip()
        # page headers: "1   / 95 SUN SOLARIS NASTRAN / MAY 17, 95 / PAGE 2"
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
        # restart header: date and checkpoint sequence vary by run
        line = re.sub(
            r"(RESTART\s+\S+\s*,\s*\S+\s*,\s*)[0-9/]+(,\s*)\d+",
            r"\1<date>\2<seq>",
            line,
        )
        # the rebuilt baseline omits this release-note diagnostic
        if "TURN DIAG 48 ON" in line or "MODULES AND NEW BULKDATA" in line:
            continue
        if line.strip() == "":
            continue
        out.append(line)
    return out


def run_deck(deck: Path) -> dict:
    name = deck.stem
    d = WORK / name
    d.mkdir(parents=True, exist_ok=True)
    # READFILE <name> looks for <name> in the working directory. The demo
    # corpus keeps those DMAP alter packages in alt/.
    text = deck.read_text(errors="replace")
    for match in re.finditer(r"READFILE\s*(?:,\s*NOPRINT)?\s+([A-Za-z0-9_]+)", text, re.I):
        alt = ALT / match.group(1).upper()
        if alt.exists():
            shutil.copy2(alt, d / match.group(1).upper())
    env = os.environ.copy()
    env.update(
        {
            "RFDIR": str(RF),
            "DIRCTY": str(d),
            "LOGNM": str(d / "run.log"),
            "DICTNM": "none",
            "PUNCHNM": "none",
            "OPTPNM": "none",
            "NPTPNM": "none",
            "PLTNM": "none",
            "FTN11": "none",
            "FTN12": "none",
            "FTN13": "none",
            "FTN14": "none",
            "FTN15": "none",
            "FTN16": "none",
            "FTN17": "none",
            "FTN18": "none",
            "FTN19": "none",
            "FTN20": "none",
            "FTN21": "none",
            "DBMEM": "12000000",
            "OCMEM": "2000000",
        }
    )
    started = time.time()
    exit_code = -1
    for attempt in range(DECK_TRIES):
        try:
            with deck.open("rb") as fin, (d / "out.txt").open("wb") as fout, (
                d / "err.txt"
            ).open("wb") as ferr:
                proc = subprocess.run(
                    [str(BIN)],
                    stdin=fin,
                    stdout=fout,
                    stderr=ferr,
                    env=env,
                    cwd=str(d),
                    timeout=TIMEOUT,
                )
            exit_code = proc.returncode
        except subprocess.TimeoutExpired:
            exit_code = -9
        except Exception as exc:  # noqa: BLE001
            exit_code = -1
            (d / "err.txt").write_text(str(exc))
        if exit_code != -9 or attempt == DECK_TRIES - 1:
            break
        time.sleep(2)

    elapsed = round(time.time() - started, 1)
    out_path = d / "out.txt"
    got = out_path.read_text(errors="replace") if out_path.exists() else ""
    gold_path = GOLD / f"{name}.out"
    want = gold_path.read_text(errors="replace") if gold_path.exists() else ""

    g = norm(want)
    o = norm(got)
    status = "PASS"
    reason = ""
    first = ""
    diff_lines = 0

    if exit_code != 0:
        status = "FAIL"
        reason = f"exit {exit_code}"
    elif not got:
        status = "FAIL"
        reason = "empty output"
    elif re.search(r"USER FATAL MESSAGE", got):
        status = "FAIL"
        reason = "fatal message"
    elif g == o:
        status = "PASS"
    else:
        diff = list(difflib.unified_diff(g, o, "gold", "ours", lineterm="", n=0))
        diff_lines = len(diff)
        sm = difflib.SequenceMatcher(None, g, o)
        reason = f"sim={sm.ratio():.4f}"
        for line in diff:
            if line.startswith(("---", "+++", "@@")):
                continue
            if line.strip():
                first = line[:160]
                break
        # A diff confined to generated SEQGP lines is a known compiler difference.
        changed = [x for x in diff if x[:1] in "+-" and not x.startswith(("---", "+++"))]
        if changed and all(
            "SEQGP" in x or "S Y S T E M  G E N E R A T E D" in x or "SEQG" in x
            for x in changed
        ):
            status = "KNOWN_DIFF"
            reason = "generated SEQGP only"
        else:
            status = "FAIL"

    return {
        "deck": name,
        "status": status,
        "reason": reason,
        "exit": exit_code,
        "seconds": elapsed,
        "gold_lines": len(g),
        "our_lines": len(o),
        "diff_lines": diff_lines,
        "first_diff": first,
        "out_bytes": len(got),
    }


def main() -> None:
    decks = sorted(INP.glob("*.inp"))
    print(f"running {len(decks)} decks with {WORKERS} workers", flush=True)
    results = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=WORKERS) as pool:
        futures = {pool.submit(run_deck, d): d for d in decks}
        for i, fut in enumerate(concurrent.futures.as_completed(futures), 1):
            try:
                r = fut.result()
            except Exception as exc:  # noqa: BLE001
                r = {
                    "deck": futures[fut].stem,
                    "status": "FAIL",
                    "reason": f"harness {exc}",
                    "exit": -1,
                    "seconds": 0,
                    "gold_lines": 0,
                    "our_lines": 0,
                    "diff_lines": 0,
                    "first_diff": "",
                    "out_bytes": 0,
                }
            results.append(r)
            print(
                f"[{i:3}/{len(decks)}] {r['deck']:<10} {r['status']:<11} {r['reason']}",
                flush=True,
            )
    results.sort(key=lambda r: r["deck"])
    (WORK / "report.json").write_text(json.dumps(results, indent=2) + "\n")
    counts = {}
    for r in results:
        counts[r["status"]] = counts.get(r["status"], 0) + 1
    print("SUMMARY", counts)
    for r in results:
        if r["status"] != "PASS":
            print(r["deck"], r["status"], r["reason"], "|", r["first_diff"][:110])


if __name__ == "__main__":
    main()
