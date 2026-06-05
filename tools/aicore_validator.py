#!/usr/bin/env python3
import json
import subprocess
import sys

BIN = sys.argv[1] if len(sys.argv) > 1 else 'aicore'
GOAL = sys.argv[2] if len(sys.argv) > 2 else 'test goal'

def run_submit(goal: str):
    proc = subprocess.run([BIN, 'submit', goal], capture_output=True, text=True)
    if proc.returncode != 0:
        print(f"CLI failed: {proc.stderr}")
        sys.exit(proc.returncode)
    return json.loads(proc.stdout)

if __name__ == '__main__':
    # Initialize
    init = subprocess.run([BIN, 'init'], capture_output=True, text=True)
    if init.returncode != 0:
        print(f"Init failed: {init.stderr}")
        sys.exit(init.returncode)

    a = run_submit(GOAL)
    b = run_submit(GOAL)

    # Determinism check: allow exact JSON equality
    if json.dumps(a, sort_keys=True) == json.dumps(b, sort_keys=True):
        print("Determinism OK")
        sys.exit(0)
    else:
        print("Determinism FAILED")
        print("A:", json.dumps(a, indent=2, sort_keys=True))
        print("B:", json.dumps(b, indent=2, sort_keys=True))
        sys.exit(1)
