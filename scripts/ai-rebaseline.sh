#!/usr/bin/env bash
set -euo pipefail
export $(grep -v '^#' .env.determinism 2>/dev/null | xargs -d '\n' -I {} echo {}) || true
mkdir -p artifacts/ai
python tooling/python/ai_golden_test.py --rebaseline
echo "✔ Rebaseline complete. Review seeds/ai/goldens.json and commit."
