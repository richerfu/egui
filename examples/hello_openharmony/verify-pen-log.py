#!/usr/bin/env python3
"""Check the native pen event boundary, not merely the UITest injector's exit code."""
import argparse
import pathlib
import re
import sys

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("log", type=pathlib.Path, help="hilog capture from the test")
parser.add_argument("run_log", type=pathlib.Path, help="output of the matching aa test command")
args = parser.parse_args()
run_log = args.run_log.read_text(errors="replace")
results = re.findall(r"TestFinished-ResultCode:\s*(-?\d+)", run_log)
if results != ["0"]:
    sys.exit("FAIL: the matching UITest run did not finish successfully")
log = args.log.read_text(errors="replace")
runs = [line.split()[2] for line in log.splitlines() if "/EguiPenTest:" in line and "penSwipe pressure=" in line]
if not runs:
    raise SystemExit("FAIL: no UITest pen run marker")
pid = runs[-1]
log = "\n".join(line for line in log.splitlines() if len(line.split()) >= 3 and line.split()[2] == pid)
print(f"Checking latest UITest process {pid}")
samples = re.findall(r"OHOS native pen: Pen \{ phase: (Start|Move|End), pressure: Some\(([0-9.]+)\)", log)
for pressure in (0.25, 0.5, 0.75, 1.0):
    phases = {phase for phase, value in samples if abs(float(value) - pressure) < 1e-6}
    if not {"Start", "Move", "End"} <= phases:
        raise SystemExit(f"FAIL pressure={pressure}: observed phases {sorted(phases)}")
    print(f"PASS native pen pressure={pressure}: Start, Move, End")
