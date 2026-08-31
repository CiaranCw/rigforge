# RESEARCH ONLY / POC-BLENDER-E2E-01
# Inspect is blender_worker.py mode=inspect, invoked by run_e2e.py.
# Not Canonical. Not a language selection.

from __future__ import annotations

from pathlib import Path

WORKER = Path(__file__).resolve().with_name("blender_worker.py")


def main() -> int:
    print("inspect implementation:", WORKER)
    print("mode: inspect")
    print("invoke via: python experiments/w0p/poc_blender_e2e_01/scripts/run_e2e.py")
    print("direct: blender --background --factory-startup --disable-autoexec")
    print("        --python-exit-code 1 --python", WORKER, "-- inspect <inspect_job.json>")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
