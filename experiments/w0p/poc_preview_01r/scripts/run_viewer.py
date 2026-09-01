# RESEARCH ONLY / POC-PREVIEW-01R
# Local static server: viewer + external payloads/vendor. Transport only.

from __future__ import annotations

import argparse
from functools import partial
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from pathlib import Path

POC = Path(__file__).resolve().parents[1]
VIEWER = POC / "viewer"
WORK = Path(r"F:\NewResearch\rigforge_w0p_work\poc_preview_01r")
RUNTIME = WORK / "runtime"
VENDOR = WORK / "vendor"


class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(VIEWER), **kwargs)

    def log_message(self, fmt, *args):
        return

    def translate_path(self, path: str) -> str:
        clean = path.split("?", 1)[0].split("#", 1)[0]
        if clean.startswith("/vendor/"):
            rel = clean[len("/vendor/") :]
            return str((VENDOR / rel).resolve())
        if clean.startswith("/payloads/"):
            rel = clean[len("/payloads/") :]
            return str((RUNTIME / "payloads" / rel).resolve())
        if clean.startswith("/artifacts/"):
            rel = clean[len("/artifacts/") :]
            return str((RUNTIME / "artifacts" / rel).resolve())
        if clean == "/catalog.json":
            return str(RUNTIME / "catalog.json")
        return super().translate_path(path)

    def guess_type(self, path):
        if str(path).endswith(".js"):
            return "application/javascript"
        if str(path).endswith(".json"):
            return "application/json"
        if str(path).endswith(".glb"):
            return "model/gltf-binary"
        return super().guess_type(path)


def make_server(host: str = "127.0.0.1", port: int = 8765) -> ThreadingHTTPServer:
    return ThreadingHTTPServer((host, port), Handler)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=8765)
    args = parser.parse_args()
    httpd = make_server(port=args.port)
    print(f"http://127.0.0.1:{args.port}/")
    print("transport only; no product backend")
    httpd.serve_forever()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
