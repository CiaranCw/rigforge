"""Independent FBX name/parent scan for POC-FBX-01 cross-check.

RESEARCH ONLY / W0-P / NON-PRODUCTION.

Not ufbx. Not Canonical. Not a production parser.

Reads ASCII FBX Model/C records and binary FBX 7.4 (32-bit records) /
7.5+ (64-bit records) Objects/Model + Connections/C.

This scanner is a small independent structural cross-check only.
It is not Canonical authority. A mismatch with ufbx is evidence.
"""
from __future__ import annotations

import io
import json
import re
import struct
import sys
from pathlib import Path

FBX_MAGIC = b"Kaydara FBX Binary  "


def _decode_fbx_name(raw) -> str:
    if raw is None:
        return ""
    if isinstance(raw, bytes):
        s = raw.decode("utf-8", "replace")
    else:
        s = str(raw)
    if "\x00\x01" in s:
        s = s.split("\x00\x01", 1)[0]
    if s.startswith("Model::"):
        s = s[7:]
    return s


def scan_ascii(data: bytes) -> list[dict]:
    text = data.decode("utf-8", "replace")
    models: dict[int, dict] = {}
    for m in re.finditer(
        r'Model:\s*(-?\d+)\s*,\s*"Model::([^"]*)"\s*,\s*"([^"]*)"',
        text,
    ):
        models[int(m.group(1))] = {
            "id": int(m.group(1)),
            "name": m.group(2),
            "attrib": m.group(3),
            "parent_id": None,
        }
    for m in re.finditer(r'C:\s*"OO"\s*,\s*(-?\d+)\s*,\s*(-?\d+)', text):
        child, parent = int(m.group(1)), int(m.group(2))
        if child in models:
            models[child]["parent_id"] = parent
            if parent in models:
                models[child]["parent_name"] = models[parent]["name"]
            elif parent == 0:
                models[child]["parent_name"] = "(scene_root)"
    return [models[k] for k in sorted(models)]


def _read_prop(f: io.BytesIO):
    ptype = f.read(1)
    if not ptype:
        return None
    t = ptype.decode("ascii", "replace")
    if t == "Y":
        return struct.unpack("<h", f.read(2))[0]
    if t == "C":
        return f.read(1)[0]
    if t == "I":
        return struct.unpack("<i", f.read(4))[0]
    if t == "F":
        return struct.unpack("<f", f.read(4))[0]
    if t == "D":
        return struct.unpack("<d", f.read(8))[0]
    if t == "L":
        return struct.unpack("<q", f.read(8))[0]
    if t in ("S", "R"):
        n = struct.unpack("<I", f.read(4))[0]
        raw = f.read(n)
        if t == "S":
            return raw.decode("utf-8", "replace")
        return "<%s %d>" % (t, n)
    if t in ("b", "c", "f", "d", "l", "i"):
        array_n, encoding, clen = struct.unpack("<III", f.read(12))
        elem = {"f": 4, "d": 8, "i": 4, "l": 8, "b": 1, "c": 1}[t]
        if encoding:
            f.read(clen)
        else:
            f.read(elem * array_n)
        return "<%s-array %d>" % (t, array_n)
    return "<?%s>" % t


PARSE_PROPS = {"Model", "C"}


def _read_node(f: io.BytesIO, wide: bool, limit: int) -> dict | None:
    start = f.tell()
    if start >= limit:
        return None
    header_len = 25 if wide else 13
    raw = f.read(header_len)
    if len(raw) < header_len:
        return None
    if raw == b"\x00" * header_len:
        return None
    if wide:
        end, nprops, plen = struct.unpack_from("<QQQ", raw, 0)
        namelen = raw[24]
    else:
        end, nprops, plen = struct.unpack_from("<III", raw, 0)
        namelen = raw[12]
    if end == 0:
        return None
    name = f.read(namelen).decode("utf-8", "replace")
    props_start = f.tell()
    props = []
    if name in PARSE_PROPS:
        for _ in range(int(nprops)):
            try:
                props.append(_read_prop(f))
            except (struct.error, IndexError, KeyError):
                break
        # Re-sync to the documented property-list end even if a type was unknown.
        f.seek(props_start + int(plen))
    else:
        f.seek(props_start + int(plen))
    children = []
    while f.tell() < end:
        ch = _read_node(f, wide, end)
        if ch is None:
            break
        children.append(ch)
    if f.tell() != end:
        f.seek(end)
    return {"name": name, "props": props, "children": children}


def scan_binary(data: bytes) -> tuple[list[dict], dict]:
    meta = {
        "fbx_version": None,
        "record_width_bits": None,
        "header_ok": False,
        "root_node_count": 0,
        "parse_notes": [],
    }
    if not data.startswith(FBX_MAGIC):
        meta["parse_notes"].append("missing Kaydara FBX Binary magic")
        return [], meta
    # 21-byte magic, 0x1A, 0x00, then uint32 version at offset 23.
    if len(data) < 27:
        meta["parse_notes"].append("header too short")
        return [], meta
    version = struct.unpack_from("<I", data, 23)[0]
    wide = version >= 7500
    meta["fbx_version"] = version
    meta["record_width_bits"] = 64 if wide else 32
    meta["header_ok"] = True
    f = io.BytesIO(data)
    f.seek(27)
    roots = []
    while True:
        n = _read_node(f, wide, len(data))
        if n is None:
            break
        roots.append(n)
    meta["root_node_count"] = len(roots)

    models: list[dict] = []
    connections: list[tuple] = []

    def walk(node: dict) -> None:
        if node["name"] == "Model":
            ps = node["props"]
            pid = ps[0] if ps else None
            raw_name = ps[1] if len(ps) > 1 else ""
            attrib = ps[2] if len(ps) > 2 else ""
            models.append(
                {
                    "id": pid,
                    "name": _decode_fbx_name(raw_name),
                    "attrib": _decode_fbx_name(attrib) if not isinstance(attrib, str) or "\x00" in attrib else attrib,
                    "parent_id": None,
                }
            )
        elif node["name"] == "C":
            ps = node["props"]
            if len(ps) >= 3:
                connections.append((ps[0], ps[1], ps[2]))
        for ch in node["children"]:
            walk(ch)

    for n in roots:
        walk(n)

    by_id = {m["id"]: m for m in models if m["id"] is not None}
    # Hierarchy only: OO where both ends are Model ids (or parent is scene root 0).
    # Other OO links (Model↔Geometry, Model↔NodeAttribute) are not parents.
    for ctype, child, parent in connections:
        if ctype != "OO":
            continue
        if child not in by_id:
            continue
        if parent in by_id or parent in (0, 0.0):
            by_id[child]["parent_id"] = parent
            if parent in by_id:
                by_id[child]["parent_name"] = by_id[parent]["name"]
            else:
                by_id[child]["parent_name"] = "(scene_root)"
    meta["model_count"] = len(models)
    meta["connection_oo_applied"] = sum(1 for m in models if m.get("parent_id") is not None)
    return models, meta


def scan_file(path: Path) -> dict:
    data = path.read_bytes()
    ascii_mode = not data.startswith(FBX_MAGIC)
    binary_meta = {
        "fbx_version": None,
        "record_width_bits": None,
        "header_ok": None,
        "root_node_count": None,
        "parse_notes": ["ascii"],
    }
    if ascii_mode:
        models = scan_ascii(data)
    else:
        models, binary_meta = scan_binary(data)
    return {
        "scanner": "poc_fbx_01_fbx_name_scan",
        "classification": "INDEPENDENT STRUCTURAL CROSS-CHECK. Not ufbx. Not Canonical.",
        "not": "ufbx",
        "basename": path.name,
        "ascii": ascii_mode,
        "binary_meta": binary_meta,
        "models": models,
    }


def main() -> int:
    if len(sys.argv) < 3:
        print("usage: fbx_name_scan.py INPUT.fbx OUT.json", file=sys.stderr)
        return 2
    src = Path(sys.argv[1])
    dst = Path(sys.argv[2])
    result = scan_file(src)
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
