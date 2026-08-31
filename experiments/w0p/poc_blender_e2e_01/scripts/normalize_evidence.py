# RESEARCH ONLY / POC-BLENDER-E2E-01
# Semantic JSON hash helper (sort keys; no timestamps).

from __future__ import annotations

import hashlib
import json
from pathlib import Path


def canonical_json(obj) -> bytes:
    return json.dumps(obj, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode(
        "utf-8"
    )


def semantic_hash(obj) -> str:
    return hashlib.sha256(canonical_json(obj)).hexdigest()


def main() -> int:
    print("normalization: json.dumps(sort_keys=True, separators=(',', ':'))")
    print("excluded from run summaries: timestamps, absolute temp PIDs, raw container bytes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
