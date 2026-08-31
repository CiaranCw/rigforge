#!/usr/bin/env python3
"""
POC-CORE-01 Python ctypes caller.

RESEARCH ONLY / W0-P / NON-PRODUCTION

The SAME script is used against the C++ DLL, the Rust raw-C DLL, and the
POC-CORE-01S official-binding DLL.
Do not use pybind11 / nanobind / PyO3 here.
"""

from __future__ import annotations

import argparse
import base64
import ctypes
import json
import os
import subprocess
import sys
from ctypes import (
    POINTER,
    c_char_p,
    c_double,
    c_int32,
    c_size_t,
    c_void_p,
)

RF_POC_OK = 0
RF_POC_ERR_NULL = -1
RF_POC_ERR_INVALID_INDEX = -2
RF_POC_ERR_IO = -3
RF_POC_ERR_PARSE = -4
RF_POC_ERR_BUFFER = -5
RF_POC_ERR_INVALID_HANDLE = -6

EPS = 1e-9

EXPECTED_NAMES = [
    "Root",
    "Hips",
    "Spine",
    "Head",
    "LeftShoulder_helper_POC",
]
EXPECTED_PARENTS = [-1, 0, 1, 2, 2]
EXPECTED_T = [
    (0.0, 0.0, 0.0),
    (0.0, 100.0, 0.0),
    (0.0, 20.0, 0.0),
    (0.0, 30.0, 0.0),
    (15.0, 10.0, 0.0),
]


class Fail(Exception):
    pass


def near(a: float, b: float) -> bool:
    return abs(a - b) <= EPS


def bind(lib) -> None:
    lib.rf_poc_load.argtypes = [c_char_p, POINTER(c_int32)]
    lib.rf_poc_load.restype = c_void_p
    lib.rf_poc_destroy.argtypes = [c_void_p]
    lib.rf_poc_destroy.restype = None
    lib.rf_poc_asset_id.argtypes = [c_void_p, c_char_p, c_size_t, POINTER(c_size_t)]
    lib.rf_poc_asset_id.restype = c_int32
    lib.rf_poc_joint_count.argtypes = [c_void_p]
    lib.rf_poc_joint_count.restype = c_int32
    lib.rf_poc_joint_parent.argtypes = [c_void_p, c_int32, POINTER(c_int32)]
    lib.rf_poc_joint_parent.restype = c_int32
    lib.rf_poc_joint_name.argtypes = [c_void_p, c_int32, c_char_p, c_size_t, POINTER(c_size_t)]
    lib.rf_poc_joint_name.restype = c_int32
    lib.rf_poc_joint_rest_translation.argtypes = [c_void_p, c_int32, POINTER(c_double)]
    lib.rf_poc_joint_rest_translation.restype = c_int32
    lib.rf_poc_joint_rest_rotation.argtypes = [c_void_p, c_int32, POINTER(c_double)]
    lib.rf_poc_joint_rest_rotation.restype = c_int32
    lib.rf_poc_joint_rest_scale.argtypes = [c_void_p, c_int32, POINTER(c_double)]
    lib.rf_poc_joint_rest_scale.restype = c_int32
    lib.rf_poc_motion_count.argtypes = [c_void_p]
    lib.rf_poc_motion_count.restype = c_int32
    lib.rf_poc_motion_name.argtypes = [c_void_p, c_int32, c_char_p, c_size_t, POINTER(c_size_t)]
    lib.rf_poc_motion_name.restype = c_int32
    lib.rf_poc_motion_time_range.argtypes = [c_void_p, c_int32, POINTER(c_double), POINTER(c_double)]
    lib.rf_poc_motion_time_range.restype = c_int32
    lib.rf_poc_track_count.argtypes = [c_void_p, c_int32]
    lib.rf_poc_track_count.restype = c_int32
    lib.rf_poc_track_joint.argtypes = [c_void_p, c_int32, c_int32, POINTER(c_int32)]
    lib.rf_poc_track_joint.restype = c_int32
    lib.rf_poc_track_translation_key_count.argtypes = [c_void_p, c_int32, c_int32]
    lib.rf_poc_track_translation_key_count.restype = c_int32
    lib.rf_poc_track_rotation_key_count.argtypes = [c_void_p, c_int32, c_int32]
    lib.rf_poc_track_rotation_key_count.restype = c_int32
    lib.rf_poc_track_translation_key.argtypes = [
        c_void_p, c_int32, c_int32, c_int32, POINTER(c_double), POINTER(c_double)
    ]
    lib.rf_poc_track_translation_key.restype = c_int32
    lib.rf_poc_track_rotation_key.argtypes = [
        c_void_p, c_int32, c_int32, c_int32, POINTER(c_double), POINTER(c_double)
    ]
    lib.rf_poc_track_rotation_key.restype = c_int32
    lib.rf_poc_diagnostic_count.argtypes = [c_void_p]
    lib.rf_poc_diagnostic_count.restype = c_int32
    lib.rf_poc_diagnostic.argtypes = [
        c_void_p, c_int32, POINTER(c_int32),
        c_char_p, c_size_t, POINTER(c_size_t),
        c_char_p, c_size_t, POINTER(c_size_t),
        c_char_p, c_size_t, POINTER(c_size_t),
    ]
    lib.rf_poc_diagnostic.restype = c_int32


def read_str(fn, handle, index=None) -> str:
    n = c_size_t(0)
    if index is None:
        st = fn(handle, None, 0, ctypes.byref(n))
    else:
        st = fn(handle, index, None, 0, ctypes.byref(n))
    if st not in (RF_POC_OK, RF_POC_ERR_BUFFER):
        raise Fail(f"size query failed status={st}")
    buf = ctypes.create_string_buffer(n.value + 1)
    if index is None:
        st = fn(handle, buf, n.value + 1, ctypes.byref(n))
    else:
        st = fn(handle, index, buf, n.value + 1, ctypes.byref(n))
    if st != RF_POC_OK:
        raise Fail(f"copy failed status={st}")
    return buf.value.decode("utf-8")


def expect(cond: bool, msg: str) -> None:
    if not cond:
        raise Fail(msg)


def run_abi(lib, fixture: str) -> None:
    st = c_int32(0)
    expect(lib.rf_poc_load(None, ctypes.byref(st)) is None, "null path should return NULL")
    expect(st.value == RF_POC_ERR_NULL, f"null path status {st.value}")

    missing = fixture + ".does_not_exist"
    st = c_int32(0)
    expect(
        lib.rf_poc_load(missing.encode("utf-8"), ctypes.byref(st)) is None,
        "missing file should return NULL",
    )
    expect(st.value == RF_POC_ERR_IO, f"missing file status {st.value}")

    st = c_int32(0)
    h = lib.rf_poc_load(fixture.encode("utf-8"), ctypes.byref(st))
    expect(h not in (None, 0), "load should succeed")
    expect(st.value == RF_POC_OK, f"load status {st.value}")

    expect(read_str(lib.rf_poc_asset_id, h) == "poc_core_01_asset", "asset id")
    jc = lib.rf_poc_joint_count(h)
    expect(jc == 5, f"joint count {jc}")

    names = []
    for i in range(jc):
        names.append(read_str(lib.rf_poc_joint_name, h, i))
        parent = c_int32(-99)
        expect(lib.rf_poc_joint_parent(h, i, ctypes.byref(parent)) == RF_POC_OK, "parent")
        expect(parent.value == EXPECTED_PARENTS[i], f"parent[{i}]={parent.value}")
        t = (c_double * 3)()
        r = (c_double * 4)()
        s = (c_double * 3)()
        expect(lib.rf_poc_joint_rest_translation(h, i, t) == RF_POC_OK, "t")
        expect(lib.rf_poc_joint_rest_rotation(h, i, r) == RF_POC_OK, "r")
        expect(lib.rf_poc_joint_rest_scale(h, i, s) == RF_POC_OK, "s")
        et = EXPECTED_T[i]
        expect(all(near(t[k], et[k]) for k in range(3)), f"t[{i}] {[t[k] for k in range(3)]}")
        expect(near(r[0], 0) and near(r[1], 0) and near(r[2], 0) and near(r[3], 1), f"r[{i}]")
        expect(near(s[0], 1) and near(s[1], 1) and near(s[2], 1), f"s[{i}]")
    expect(names == EXPECTED_NAMES, f"names {names}")

    n = c_size_t(0)
    stn = lib.rf_poc_joint_name(h, 0, None, 0, ctypes.byref(n))
    expect(stn == RF_POC_OK, "null buf size query")
    expect(n.value == len("Root"), f"Root len {n.value}")
    tiny = ctypes.create_string_buffer(2)
    stn = lib.rf_poc_joint_name(h, 0, tiny, 2, ctypes.byref(n))
    expect(stn == RF_POC_ERR_BUFFER, "tiny buffer")

    expect(lib.rf_poc_joint_parent(h, 99, ctypes.byref(c_int32())) == RF_POC_ERR_INVALID_INDEX, "bad index")

    # sequential queries / "internal work"
    names2 = [read_str(lib.rf_poc_joint_name, h, i) for i in range(jc)]
    expect(names2 == names, "sequential names")

    expect(lib.rf_poc_motion_count(h) == 1, "motion count")
    expect(read_str(lib.rf_poc_motion_name, h, 0) == "poc_walk", "motion name")
    t0 = c_double()
    t1 = c_double()
    expect(lib.rf_poc_motion_time_range(h, 0, ctypes.byref(t0), ctypes.byref(t1)) == RF_POC_OK, "range")
    expect(near(t0.value, 0.0) and near(t1.value, 1.0), "time range")

    tc = lib.rf_poc_track_count(h, 0)
    expect(tc == 5, f"track count {tc}")
    last = jc - 1
    for i in range(tc):
        jid = c_int32(-1)
        expect(lib.rf_poc_track_joint(h, 0, i, ctypes.byref(jid)) == RF_POC_OK, "track joint")
        expect(jid.value == i, f"track joint {jid.value}")
        tk = lib.rf_poc_track_translation_key_count(h, 0, i)
        rk = lib.rf_poc_track_rotation_key_count(h, 0, i)
        expect(tk == 2, f"t keys {tk}")
        if i == last:
            expect(rk == 0, "last joint empty rotation keys")
            expect(
                lib.rf_poc_track_rotation_key(h, 0, i, 0, ctypes.byref(c_double()), (c_double * 4)())
                == RF_POC_ERR_INVALID_INDEX,
                "empty rot key query",
            )
        else:
            expect(rk == 2, f"r keys {rk}")
        time = c_double()
        xyz = (c_double * 3)()
        expect(lib.rf_poc_track_translation_key(h, 0, i, 0, ctypes.byref(time), xyz) == RF_POC_OK, "k0")
        expect(near(time.value, 0.0), "k0 time")
        et = EXPECTED_T[i]
        expect(all(near(xyz[k], et[k]) for k in range(3)), "k0 t")
        expect(lib.rf_poc_track_translation_key(h, 0, i, 1, ctypes.byref(time), xyz) == RF_POC_OK, "k1")
        expect(near(time.value, 1.0), "k1 time")
        expect(near(xyz[0], et[0] + 0.1 * i) and near(xyz[1], et[1]) and near(xyz[2], et[2]), "k1 t")

    dc = lib.rf_poc_diagnostic_count(h)
    expect(dc == 4, f"diag count {dc}")
    codes = []
    for i in range(dc):
        sev = c_int32()
        cl = c_size_t()
        ml = c_size_t()
        ll = c_size_t()
        lib.rf_poc_diagnostic(
            h, i, ctypes.byref(sev),
            None, 0, ctypes.byref(cl),
            None, 0, ctypes.byref(ml),
            None, 0, ctypes.byref(ll),
        )
        cb = ctypes.create_string_buffer(cl.value + 1)
        mb = ctypes.create_string_buffer(ml.value + 1)
        lb = ctypes.create_string_buffer(ll.value + 1)
        lib.rf_poc_diagnostic(
            h, i, ctypes.byref(sev),
            cb, cl.value + 1, ctypes.byref(cl),
            mb, ml.value + 1, ctypes.byref(ml),
            lb, ll.value + 1, ctypes.byref(ll),
        )
        codes.append(cb.value.decode("utf-8"))
    expect(codes == ["POC_LOAD", "POC_JOINTS", "POC_SYNTH", "POC_EMPTY_ROT"], f"diag codes {codes}")

    # empty collections already covered by last-joint rotation keys

    dead = h
    lib.rf_poc_destroy(h)
    expect(lib.rf_poc_joint_count(dead) == -1, "use after destroy count")
    expect(
        lib.rf_poc_joint_parent(dead, 0, ctypes.byref(c_int32())) == RF_POC_ERR_INVALID_HANDLE,
        "use after destroy",
    )
    expect(lib.rf_poc_joint_count(None) == -1, "null handle count")


def run_long_name(lib, fixture: str) -> None:
    expected_path = os.path.splitext(fixture)[0] + ".expected.txt"
    if not os.path.isfile(expected_path):
        raise Fail(f"missing expected name file {expected_path}")
    expected = (
        open(expected_path, "rb").read().replace(b"\r\n", b"\n").replace(b"\n", b"").decode("ascii")
    )
    expect(len(expected) > 1023, f"long-name fixture must exceed 1023 bytes, got {len(expected)}")
    st = c_int32(0)
    h = lib.rf_poc_load(fixture.encode("utf-8"), ctypes.byref(st))
    expect(h not in (None, 0), f"long-name load should succeed status={st.value}")
    expect(st.value == RF_POC_OK, f"long-name load status {st.value}")
    jc = lib.rf_poc_joint_count(h)
    expect(jc == 1, f"long-name joint count {jc}")
    n = c_size_t(0)
    stn = lib.rf_poc_joint_name(h, 0, None, 0, ctypes.byref(n))
    expect(stn == RF_POC_OK, f"long-name size query status {stn}")
    expect(n.value == len(expected), f"long-name byte length {n.value} != {len(expected)}")
    name = read_str(lib.rf_poc_joint_name, h, 0)
    expect(name == expected, "long-name string mismatch")
    expect(len(name.encode("utf-8")) == n.value, "long-name utf-8 length")
    lib.rf_poc_destroy(h)


def run_worker(worker: str, fixture: str) -> None:
    req = json.dumps(
        {"op": "load", "payload_b64": base64.b64encode(fixture.encode("utf-8")).decode("ascii")},
        separators=(",", ":"),
    )
    proc = subprocess.run(
        [worker],
        input=req + "\n",
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=False,
    )
    line = (proc.stdout or "").strip().splitlines()[-1]
    obj = json.loads(line)
    expect(obj.get("ok") is True, f"worker ok {line}")
    inner = json.loads(base64.b64decode(obj["payload_b64"]).decode("utf-8"))
    expect(inner["joint_count"] == 5, f"worker joints {inner}")
    expect(inner["motion_count"] == 1, "worker motions")
    expect(inner["diag_count"] == 4, "worker diags")
    expect(abs(inner["t0"] - 0.0) <= EPS and abs(inner["t1"] - 1.0) <= EPS, "worker range")
    expect(len(obj["diag"]) == 4, "outer diag")

    missing = fixture + ".does_not_exist"
    req2 = json.dumps(
        {"op": "load", "payload_b64": base64.b64encode(missing.encode("utf-8")).decode("ascii")},
        separators=(",", ":"),
    )
    proc2 = subprocess.run(
        [worker],
        input=req2 + "\n",
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=False,
    )
    obj2 = json.loads((proc2.stdout or "").strip().splitlines()[-1])
    expect(obj2.get("ok") is False, "worker missing should fail")
    expect(obj2["diag"][0]["code"] == "POC_IO", f"worker missing code {obj2}")


def main() -> int:
    p = argparse.ArgumentParser(description="POC-CORE-01 ctypes runner (research-only)")
    p.add_argument("--lib", required=True)
    p.add_argument("--worker", required=True)
    p.add_argument("--fixture", required=True)
    p.add_argument(
        "--long-name-fixture",
        required=True,
        help="Project-authored fixture with a node name longer than 1023 bytes",
    )
    args = p.parse_args()
    if not os.path.isfile(args.lib):
        print(f"FAIL missing lib {args.lib}", file=sys.stderr)
        return 2
    if not os.path.isfile(args.worker):
        print(f"FAIL missing worker {args.worker}", file=sys.stderr)
        return 2
    if not os.path.isfile(args.fixture):
        print(f"FAIL missing fixture {args.fixture}", file=sys.stderr)
        return 2
    if not os.path.isfile(args.long_name_fixture):
        print(f"FAIL missing long-name fixture {args.long_name_fixture}", file=sys.stderr)
        return 2
    lib = ctypes.WinDLL(args.lib) if os.name == "nt" else ctypes.CDLL(args.lib)
    bind(lib)
    try:
        run_abi(lib, os.path.abspath(args.fixture))
        run_long_name(lib, os.path.abspath(args.long_name_fixture))
        run_worker(args.worker, os.path.abspath(args.fixture))
    except Fail as e:
        print(f"FAIL {e}")
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
