"""POC-FBX-01 repository runner.

RESEARCH ONLY / W0-P / NON-PRODUCTION.

Does not download cloud-drive / file-transfer hosts / 网盘.
Level-3 assets must already exist in the operator drop directory.

C is the test harness language only. This is not Core language evidence.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]
EXP = Path(__file__).resolve().parents[1]
WORK = Path(r"F:\NewResearch\rigforge_w0p_work\poc_fbx_01")
EVIDENCE = Path(r"F:\NewResearch\rigforge_w0p_evidence\poc_fbx_01")
ASSETS = Path(r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01")
UFBX_ROOT = Path(r"F:\NewResearch\rigforge_w0p_work\poc_core_01\deps\ufbx-0.23.0")
PY = sys.executable
CORE_MINIMAL = REPO / "experiments" / "w0p" / "poc_core_01" / "fixture" / "minimal_chain.fbx"
DEFAULT_CONFIG = EXP / "l3_run_config.json"

QCHAR_REQUIRED = (
    r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01"
    r"\qchar_extract\Ultimate Animated Character Pack - Nov 2019\FBX\Knight_Male.fbx"
)
QANIMAL_REQUIRED = (
    r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01"
    r"\qanimal_extract\Ultimate Animated Animals - July 2021\FBX\Wolf.fbx"
)


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2) + "\n", encoding="utf-8")


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def build_harness() -> Path:
    WORK.mkdir(parents=True, exist_ok=True)
    (WORK / "build").mkdir(parents=True, exist_ok=True)
    bat = EXP / "scripts" / "build_harness.cmd"
    subprocess.run(["cmd", "/c", str(bat)], check=True)
    exe = WORK / "build" / "inspect_fbx.exe"
    if not exe.exists():
        raise SystemExit("inspect_fbx.exe missing after build")
    return exe


def inspect(
    exe: Path,
    fbx: Path,
    out_json: Path,
    stack_index: int | None = None,
    sample_names: list[str] | None = None,
) -> str:
    out_json.parent.mkdir(parents=True, exist_ok=True)
    cmd = [str(exe), "--input", str(fbx), "--out", str(out_json)]
    if stack_index is not None:
        cmd += ["--anim-stack-index", str(stack_index)]
    if sample_names:
        cmd += ["--sample-names", ",".join(sample_names)]
    cp = subprocess.run(cmd, capture_output=True, text=True, check=True)
    perf = (cp.stderr or "").strip()
    out_json.with_suffix(".perf.txt").write_text(perf + "\n", encoding="utf-8")
    return sha256_file(out_json)


def stop_missing(kind: str, path: Path, expected_sha: str) -> int:
    print("STOP")
    print("Required local file is missing or unreadable.")
    print("kind:", kind)
    print("required_path:", path)
    print("required_sha256:", expected_sha)
    print("Restore this operator-placed file manually.")
    print("Do not fetch a replacement through a third-party file-transfer / cloud-drive / 网盘 host.")
    return 2


def stop_hash(kind: str, path: Path, expected: str, observed: str) -> int:
    print("STOP")
    print("Local file hash does not match the frozen experiment pin.")
    print("kind:", kind)
    print("path:", path)
    print("required_sha256:", expected)
    print("observed_sha256:", observed)
    print("Restore the original operator-placed file manually.")
    print("Do not fetch a replacement through a third-party file-transfer / cloud-drive / 网盘 host.")
    return 2


def resolve_l3_path(cli: Path | None, assets_relative: str, required_fallback: str) -> Path:
    if cli is not None:
        return Path(cli)
    rel = ASSETS / assets_relative
    if rel.is_file():
        return rel
    fb = Path(required_fallback)
    if fb.is_file():
        return fb
    return rel


def parse_names(raw: str | None, default: list[str]) -> list[str]:
    if raw is None or raw.strip() == "":
        return list(default)
    return [p.strip() for p in raw.split(",") if p.strip()]


def token_presence(path: Path, tokens: list[str]) -> dict:
    data = path.read_bytes()
    return {t: (t.encode("utf-8") in data) for t in tokens}


def norm_parent(name) -> str:
    if name in (None, "", "(scene_root)"):
        return "(scene_root)"
    return str(name)


def ufbx_named(doc: dict, name: str) -> list[dict]:
    return [n for n in doc.get("nodes", []) if n.get("source_name") == name]


def scan_named(doc: dict, name: str) -> list[dict]:
    return [m for m in doc.get("models", []) if m.get("name") == name]


def add_row(rows: list, asset: str, fact: str, ufbx_obs, scan_obs, result: str) -> None:
    rows.append(
        {
            "asset": asset,
            "fact": fact,
            "ufbx_observation": ufbx_obs,
            "independent_scanner_observation": scan_obs,
            "result": result,
        }
    )


def compare_asset(
    asset: str,
    names: list[str],
    ufbx_doc: dict,
    scan_doc: dict,
    rows: list,
) -> None:
    ufbx_roots = [r.get("name") for r in ufbx_doc.get("root_nodes", [])]
    scan_roots = [
        m.get("name")
        for m in scan_doc.get("models", [])
        if norm_parent(m.get("parent_name")) == "(scene_root)"
    ]
    add_row(
        rows,
        asset,
        "root_level_organization",
        {"ufbx_root_node_children": ufbx_roots},
        {"scanner_models_parent_scene_root": scan_roots},
        "MATCH" if sorted(ufbx_roots) == sorted(scan_roots) else "MISMATCH",
    )
    for name in names:
        un = ufbx_named(ufbx_doc, name)
        sn = scan_named(scan_doc, name)
        u_exist = [ {"source_name": n.get("source_name"), "attrib_type": n.get("attrib_type"), "parent_name": n.get("parent_name")} for n in un ]
        s_exist = [ {"name": m.get("name"), "attrib": m.get("attrib"), "parent_name": m.get("parent_name")} for m in sn ]
        if not un and not sn:
            result = "NOT_CHECKABLE"
        elif bool(un) == bool(sn):
            result = "MATCH"
        else:
            result = "MISMATCH"
        add_row(rows, asset, "name_exists:" + name, u_exist, s_exist, result)

        u_parents = sorted(norm_parent(n.get("parent_name")) for n in un)
        s_parents = sorted(norm_parent(m.get("parent_name")) for m in sn)
        if not un or not sn:
            presult = "NOT_CHECKABLE" if not un and not sn else "MISMATCH"
        elif u_parents == s_parents:
            presult = "MATCH"
        else:
            presult = "MISMATCH"
        add_row(
            rows,
            asset,
            "parent_relation:" + name,
            {"parent_names": u_parents, "count": len(un)},
            {"parent_names": s_parents, "count": len(sn)},
            presult,
        )


def write_sha256_manifest() -> None:
    lines = ["POC-FBX-01 sha256 manifest", ""]
    extra = [
        ASSETS / "Ultimate Animated Character Pack - Nov 2019-20260831T071517Z-1-001.zip",
        ASSETS / "Ultimate Animated Animals - July 2021-20260831T071530Z-1-001.zip",
        ASSETS / "qchar_extract" / "Ultimate Animated Character Pack - Nov 2019" / "FBX" / "Knight_Male.fbx",
        ASSETS / "qanimal_extract" / "Ultimate Animated Animals - July 2021" / "FBX" / "Wolf.fbx",
        ASSETS / "qchar_License.txt",
        ASSETS / "qanimal_License.txt",
        EXP / "harness" / "inspect_fbx.c",
        EXP / "harness" / "CMakeLists.txt",
        EXP / "scripts" / "run_poc_fbx.py",
        EXP / "scripts" / "fbx_name_scan.py",
        EXP / "scripts" / "build_harness.cmd",
        EXP / "l3_run_config.json",
        EXP / "fixtures" / "ordinary_anim.fbx",
        EXP / "fixtures" / "authored_local.fbx",
        CORE_MINIMAL,
        UFBX_ROOT / "ufbx.c",
        UFBX_ROOT / "ufbx.h",
        UFBX_ROOT / "LICENSE",
        UFBX_ROOT / "misc" / "ufbx_testcases" / "maya_pivots_7500_ascii.fbx",
        REPO / "docs" / "research" / "poc" / "POC_FBX_01.md",
        REPO / "docs" / "research" / "poc" / "README.md",
        REPO / "docs" / "research" / "R1_RESEARCH_BASELINE.md",
        REPO / "docs" / "development" / "ROADMAP.md",
    ]
    seen: set[Path] = set()
    for path in extra:
        if path.is_file() and path not in seen:
            seen.add(path)
            lines.append("%s  %s  %s" % (sha256_file(path), path, path.stat().st_size))
    for path in sorted(EVIDENCE.iterdir()):
        if path.is_file() and path.name != "sha256_manifest.txt":
            lines.append("%s  %s  %s" % (sha256_file(path), path, path.stat().st_size))
    (EVIDENCE / "sha256_manifest.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", type=Path, default=DEFAULT_CONFIG)
    parser.add_argument("--qchar-fbx", type=Path, default=None)
    parser.add_argument("--qanimal-fbx", type=Path, default=None)
    parser.add_argument("--qchar-stack-index", type=int, default=None)
    parser.add_argument("--qanimal-stack-index", type=int, default=None)
    parser.add_argument("--qchar-sample-names", type=str, default=None)
    parser.add_argument("--qanimal-sample-names", type=str, default=None)
    parser.add_argument("--l1-only", action="store_true")
    args = parser.parse_args()

    cfg = load_json(args.config)
    ufbx_pin = cfg["ufbx_pin"]
    qchar_cfg = cfg["qchar"]
    qanimal_cfg = cfg["qanimal"]
    l2_cfg = cfg["l2_maya_pivots"]

    WORK.mkdir(parents=True, exist_ok=True)
    EVIDENCE.mkdir(parents=True, exist_ok=True)

    exe = build_harness()
    print("harness", exe)

    scan_py = EXP / "scripts" / "fbx_name_scan.py"
    l1 = [
        ("minimal_chain", CORE_MINIMAL),
        ("ordinary_anim", EXP / "fixtures" / "ordinary_anim.fbx"),
        ("authored_local", EXP / "fixtures" / "authored_local.fbx"),
    ]
    l1_hashes = {}
    for name, path in l1:
        out = EVIDENCE / ("controlled_%s.json" % name)
        digest = inspect(exe, path, out)
        rel = str(path.relative_to(REPO)) if path.is_relative_to(REPO) else str(path)
        l1_hashes[name] = {
            "file": rel,
            "json_sha256": digest,
            "fbx_sha256": sha256_file(path),
            "size": path.stat().st_size,
        }
        print("L1", name, digest)
        subprocess.run([PY, str(scan_py), str(path), str(EVIDENCE / ("crosscheck_%s.json" % name))], check=True)

    l2_path = Path(l2_cfg["default_path"])
    l2_expected = l2_cfg["sha256"]
    if not l2_path.is_file():
        return stop_missing("L2 maya_pivots_7500_ascii.fbx", l2_path, l2_expected)
    l2_hash = sha256_file(l2_path)
    if l2_hash != l2_expected:
        return stop_hash("L2 maya_pivots_7500_ascii.fbx", l2_path, l2_expected, l2_hash)
    l2_json_hash = inspect(exe, l2_path, EVIDENCE / "l2_maya_pivots.json")
    print("L2 maya_pivots", l2_json_hash)
    l2_record = {
        "basename": l2_cfg["basename"],
        "source_path": str(l2_path),
        "size_bytes": l2_path.stat().st_size,
        "sha256": l2_hash,
        "origin": l2_cfg["origin"],
        "ufbx_pin": ufbx_pin,
        "json_sha256": l2_json_hash,
        "note": "ufbx v0.23.0 upstream testcase; not Canonical; not L3",
    }

    status = {
        "l1": l1_hashes,
        "l2_maya_pivots": l2_record,
        "l3": "NOT_RUN",
        "reason": None,
        "ufbx_pin": ufbx_pin,
    }

    qchar_stack_index = qchar_cfg["stack_index"] if args.qchar_stack_index is None else args.qchar_stack_index
    qanimal_stack_index = qanimal_cfg["stack_index"] if args.qanimal_stack_index is None else args.qanimal_stack_index
    qchar_samples = parse_names(args.qchar_sample_names, qchar_cfg["sample_names"])
    qanimal_samples = parse_names(args.qanimal_sample_names, qanimal_cfg["sample_names"])

    if args.l1_only:
        status["reason"] = "l1_only flag"
        write_json(EVIDENCE / "controlled_fixture_results.json", status)
        print("L3 skipped (--l1-only)")
        write_sha256_manifest()
        return 0

    qchar_path = resolve_l3_path(args.qchar_fbx, qchar_cfg["assets_relative"], QCHAR_REQUIRED)
    qanimal_path = resolve_l3_path(args.qanimal_fbx, qanimal_cfg["assets_relative"], QANIMAL_REQUIRED)

    if not qchar_path.is_file():
        return stop_missing("QCHAR Knight_Male.fbx", Path(QCHAR_REQUIRED), qchar_cfg["sha256"])
    if not qanimal_path.is_file():
        return stop_missing("QANIMAL Wolf.fbx", Path(QANIMAL_REQUIRED), qanimal_cfg["sha256"])

    qchar_hash = sha256_file(qchar_path)
    qanimal_hash = sha256_file(qanimal_path)
    if qchar_hash != qchar_cfg["sha256"]:
        return stop_hash("QCHAR Knight_Male.fbx", qchar_path, qchar_cfg["sha256"], qchar_hash)
    if qanimal_hash != qanimal_cfg["sha256"]:
        return stop_hash("QANIMAL Wolf.fbx", qanimal_path, qanimal_cfg["sha256"], qanimal_hash)

    jobs = [
        (
            "qchar",
            qchar_path,
            qchar_hash,
            qchar_stack_index,
            qchar_cfg["stack_name"],
            qchar_samples,
        ),
        (
            "qanimal",
            qanimal_path,
            qanimal_hash,
            qanimal_stack_index,
            qanimal_cfg["stack_name"],
            qanimal_samples,
        ),
    ]

    effective = {
        "poc": "POC-FBX-01",
        "classification": "RESEARCH_ONLY / W0-P / NON-PRODUCTION",
        "ufbx_pin": ufbx_pin,
        "config_file": str(args.config),
        "l2_maya_pivots": l2_record,
        "qchar": {},
        "qanimal": {},
    }
    l3_summary = {
        "poc": "POC-FBX-01",
        "classification": "RESEARCH_ONLY / W0-P / NON-PRODUCTION",
        "ufbx_pin": ufbx_pin,
    }
    comparison_rows: list = []
    token_report = {
        "classification": "TOKEN-PRESENCE SECONDARY CROSS-CHECK",
        "does_not_prove": [
            "hierarchy",
            "transform",
            "skin",
            "bind",
            "animation evaluation",
        ],
        "note": "UTF-8 needle in file bytes. Supports name/token presence and animation-name presence only.",
        "qchar_fbx": {},
        "qanimal_fbx": {},
        "blend_files": "NOT_PRESENT_IN_EXTRACT_TREE_THIS_REV1; prior blend UTF-8 hits remain historical evidence only",
    }

    for label, path, fhash, stack_index, stack_name, samples in jobs:
        run_hashes = []
        observed_stack = None
        for i in range(1, 4):
            out = EVIDENCE / ("%s_run_%s.json" % (label, i))
            digest = inspect(exe, path, out, stack_index=stack_index, sample_names=samples)
            run_hashes.append(digest)
            print(label, "run", i, digest)
            doc = load_json(out)
            stacks = doc.get("animation", {}).get("stacks", [])
            if 0 <= stack_index < len(stacks):
                observed_stack = stacks[stack_index].get("name")
            sel = doc.get("experiment_selection", {})
            if sel.get("anim_stack_index") != stack_index:
                raise SystemExit("experiment_selection stack_index mismatch for %s" % label)
            if observed_stack != stack_name:
                raise SystemExit(
                    "stack name mismatch for %s: expected %s observed %s"
                    % (label, stack_name, observed_stack)
                )
        if len(set(run_hashes)) != 1:
            raise SystemExit("determinism FAIL %s: %s" % (label, run_hashes))
        (EVIDENCE / ("%s_file.sha256" % label)).write_text(fhash + "\n", encoding="utf-8")
        cross_path = EVIDENCE / ("crosscheck_%s.json" % label)
        subprocess.run([PY, str(scan_py), str(path), str(cross_path)], check=True)
        scan_doc = load_json(cross_path)
        if not scan_doc.get("models"):
            raise SystemExit("independent scanner returned empty models for %s" % label)
        ufbx_doc = load_json(EVIDENCE / ("%s_run_1.json" % label))
        compare_asset(label, cfg["crosscheck_names"][label], ufbx_doc, scan_doc, comparison_rows)
        tokens = list(cfg["crosscheck_names"][label]) + [stack_name, "Walk"]
        token_report["%s_fbx" % label] = token_presence(path, tokens)

        rec = {
            "basename": path.name,
            "operator_path": str(path),
            "sha256": fhash,
            "size_bytes": path.stat().st_size,
            "stack_name": stack_name,
            "stack_index": stack_index,
            "observed_stack_name": observed_stack,
            "sample_names": samples,
            "ufbx_pin": ufbx_pin,
            "run_hashes": run_hashes,
            "json_sha256": run_hashes[0],
            "determinism": "PASS" if len(set(run_hashes)) == 1 else "FAIL",
        }
        effective[label] = rec
        l3_summary[label] = {
            "asset_hash": fhash,
            "effective_experiment_config": {
                "stack_name": stack_name,
                "stack_index": stack_index,
                "sample_names": samples,
                "ufbx_pin": ufbx_pin,
            },
            "run_hashes": run_hashes,
            "determinism": rec["determinism"],
        }
        status.setdefault("l3_files", {})[label] = rec

    qchar_det = l3_summary["qchar"]["determinism"]
    qanimal_det = l3_summary["qanimal"]["determinism"]
    l3_summary["determinism"] = (
        "PASS" if qchar_det == "PASS" and qanimal_det == "PASS" else "FAIL"
    )
    status["l3"] = "RAN"
    write_json(EVIDENCE / "controlled_fixture_results.json", status)
    write_json(EVIDENCE / "run_config.json", effective)
    write_json(EVIDENCE / "l3_run_summary.json", l3_summary)
    write_json(
        EVIDENCE / "crosscheck_comparison.json",
        {
            "classification": (
                "INDEPENDENT STRUCTURAL CROSS-CHECK. "
                "The scanner is not Canonical authority. A mismatch is evidence."
            ),
            "scanner": "poc_fbx_01_fbx_name_scan",
            "ufbx": "inspect_fbx.c / pinned ufbx v0.23.0",
            "byte_search": token_report,
            "rows": comparison_rows,
        },
    )
    write_sha256_manifest()
    print("L3 determinism", l3_summary["determinism"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
