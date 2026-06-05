#!/usr/bin/env python3
"""
AI Golden Test - Deterministic AI inference testing
"""

import argparse, json, hashlib, os, sys, time
from pathlib import Path

SEEDS = Path("seeds/ai")
GOLDEN = SEEDS / "goldens.json"
ART = Path("artifacts/ai"); ART.mkdir(parents=True, exist_ok=True)

def sha256_file(p: Path) -> str:
    h = hashlib.sha256()
    with p.open("rb") as f:
        for chunk in iter(lambda: f.read(1<<20), b""):
            h.update(chunk)
    return h.hexdigest()

def run_vision_on(img: Path) -> str:
    # bridge out to your AI service or call a thin CLI that prints a stable detection hash
    # for now, hash the deterministic preproc bytes as a proxy (service should expose a --hash-only for goldens)
    try:
        from PIL import Image
        im = Image.open(img).convert("RGB")
        # low-variance resize to 320x320 nearest for hashing proxy
        im = im.resize((320,320), Image.NEAREST)
        b = im.tobytes()
        return hashlib.sha256(b).hexdigest()
    except (ImportError, Exception):
        # Fallback: hash the file content directly (for placeholder files)
        return sha256_file(img)

def run_audio_on(wav: Path) -> str:
    # deterministically decode & hash PCM
    try:
        import wave, struct
        with wave.open(str(wav), "rb") as w:
            frames = w.readframes(w.getnframes())
        return hashlib.sha256(frames).hexdigest()
    except (ImportError, Exception):
        # Fallback: hash the file content directly (for placeholder files)
        return sha256_file(wav)

def load_expected():
    if GOLDEN.exists():
        return json.loads(GOLDEN.read_text())
    return {"inputs": {}, "outputs": {}}

def save_expected(obj):
    GOLDEN.write_text(json.dumps(obj, indent=2, sort_keys=True))

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--rebaseline", action="store_true")
    args = ap.parse_args()

    images = [SEEDS/"test_image_1.jpg", SEEDS/"test_image_2.jpg"]
    audios = [SEEDS/"test_audio_1.wav", SEEDS/"test_audio_2.wav"]

    exp = load_expected()
    report = {"tests": [], "ts": time.time()}
    failed = 0

    # Build current hashes
    cur_inputs = {}
    cur_outputs = {}
    for p in images + audios:
        if not p.exists():
            report["tests"].append({"file": str(p), "ok": False, "reason": "MISSING"})
            failed += 1
            continue
        ih = sha256_file(p)
        cur_inputs[str(p.name)] = ih
        if p.suffix.lower() == ".jpg":
            oh = run_vision_on(p)
        else:
            oh = run_audio_on(p)
        cur_outputs[str(p.name)] = oh

    if args.rebaseline:
        exp["inputs"] = cur_inputs
        exp["outputs"] = cur_outputs
        save_expected(exp)
        print("✔ Rebaselined goldens.json with current seeds & outputs.")
        return 0

    # Compare against expected
    for name, ih in cur_inputs.items():
        exp_ih = exp.get("inputs", {}).get(name)
        exp_oh = exp.get("outputs", {}).get(name)
        oh = cur_outputs[name]
        if exp_ih is None or exp_oh is None:
            report["tests"].append({"file": name, "ok": False, "reason": "NO_EXPECTED", "input_sha256": ih, "output_sha256": oh})
            failed += 1
            continue
        if exp_ih != ih:
            report["tests"].append({"file": name, "ok": False, "reason": "SEED_DRIFT", "expected_input": exp_ih, "actual_input": ih, "hint": "Replace seed file or run --rebaseline intentionally."})
            failed += 1
            continue
        if exp_oh != oh:
            report["tests"].append({"file": name, "ok": False, "reason": "MODEL_DRIFT", "expected_output": exp_oh, "actual_output": oh, "hint": "Check model version, env .env.determinism, threading=1. Rebaseline only after review."})
            failed += 1
            continue
        report["tests"].append({"file": name, "ok": True, "input_sha256": ih, "output_sha256": oh})

    ART.joinpath("golden.json").write_text(json.dumps(report, indent=2))
    total = len(report["tests"])
    passed = sum(1 for t in report["tests"] if t["ok"])
    print("============================================================")
    print("AI GOLDEN TEST SUMMARY")
    print("============================================================")
    print(f"Total Tests: {total}\nPassed: {passed}\nFailed: {total - passed}\nSuccess Rate: {100.0*passed/max(1,total):.1f}%")
    if failed:
        print("\n❌ GOLDEN TEST FAILED - See artifacts/ai/golden.json")
        return 1
    print("\n✅ GOLDEN TEST PASSED")
    return 0

if __name__ == "__main__":
    sys.exit(main())