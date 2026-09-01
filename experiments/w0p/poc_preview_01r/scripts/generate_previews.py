# RESEARCH ONLY / W0-P / POC-PREVIEW-01R
# Generate Character / Motion / Derived Preview Artifacts. Not a product service.

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import time
from pathlib import Path

POC = Path(__file__).resolve().parents[1]
GENERATOR = POC / "generator" / "blender_preview_generator.py"
CONFIG_PATH = POC / "config" / "preview_generation.json"
FIXTURE_PATH = POC / "config" / "product_fixture.json"

BLENDER = Path(r"F:\NewResearch\rigforge_w0p_work\toolchains\blender-5.2.1-windows-x64\blender.exe")
CHAR_PATH = Path(
    r"F:\NewResearch\rigforge_w0p_assets\poc_fbx_01\qchar_extract"
    r"\Ultimate Animated Character Pack - Nov 2019\FBX\Knight_Male.fbx"
)
MOTION_PATH = (
    Path(r"F:\NewResearch\rigforge_w0p_assets\poc_blender_e2e_01\ual2_extract")
    / "Universal Animation Library 2[Standard]"
    / "Unity"
    / "UAL2_Standard.fbx"
)
BLEND_PATH = Path(
    r"F:\NewResearch\rigforge_w0p_work\poc_blender_e2e_01\run_1\artifacts\derived_result.blend"
)
WORK = Path(r"F:\NewResearch\rigforge_w0p_work\poc_preview_01r")
EVIDENCE = Path(r"F:\NewResearch\rigforge_w0p_evidence\poc_preview_01r")


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while True:
            b = f.read(1024 * 1024)
            if not b:
                break
            h.update(b)
    return h.hexdigest()


def canonical_json(obj) -> bytes:
    return json.dumps(obj, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("utf-8")


def semantic_hash(obj) -> str:
    return hashlib.sha256(canonical_json(obj)).hexdigest()


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def isolated_env(workspace: Path) -> dict:
    env = os.environ.copy()
    user = workspace / "blender_user"
    tmp = workspace / "tmp"
    for p in (user / "config", user / "scripts", tmp):
        p.mkdir(parents=True, exist_ok=True)
    env["TEMP"] = str(tmp)
    env["TMP"] = str(tmp)
    env["TMPDIR"] = str(tmp)
    env["BLENDER_USER_CONFIG"] = str(user / "config")
    env["BLENDER_USER_SCRIPTS"] = str(user / "scripts")
    env["BLENDER_USER_DATAFILES"] = str(user / "datafiles")
    return env


def run_blender(mode: str, job_path: Path, workspace: Path) -> subprocess.CompletedProcess:
    cmd = [
        str(BLENDER),
        "--background",
        "--factory-startup",
        "--disable-autoexec",
        "--python-exit-code",
        "1",
        "--python",
        str(GENERATOR),
        "--",
        mode,
        str(job_path),
    ]
    t0 = time.perf_counter()
    proc = subprocess.run(cmd, env=isolated_env(workspace), cwd=str(workspace), capture_output=True, text=True)
    proc.elapsed_s = round(time.perf_counter() - t0, 3)  # type: ignore[attr-defined]
    (workspace / f"{mode}_stdout.txt").write_text(proc.stdout or "", encoding="utf-8")
    (workspace / f"{mode}_stderr.txt").write_text(proc.stderr or "", encoding="utf-8")
    return proc


def semantic_generation(gen: dict) -> dict:
    return {
        "kind": gen.get("kind"),
        "status": gen.get("status"),
        "clip_observed": gen.get("clip_observed"),
        "default_animation": gen.get("default_animation"),
        "duration_s": gen.get("duration_s"),
        "fps": gen.get("fps"),
        "frame_start": gen.get("frame_start"),
        "frame_end": gen.get("frame_end"),
        "had_source_mesh": gen.get("had_source_mesh"),
        "had_source_renderable_mesh": gen.get("had_source_renderable_mesh"),
        "source_mesh_used_as_preview": gen.get("source_mesh_used_as_preview"),
        "proxy": gen.get("proxy"),
        "presentation": gen.get("presentation"),
        "bounds_valid": (gen.get("bounds") or {}).get("valid"),
        "mesh_object_count": (gen.get("bounds") or {}).get("mesh_object_count"),
        "animation_names": [a.get("name") for a in gen.get("animation_inventory") or []],
        "warnings": gen.get("warnings"),
        "material_preview_fix": gen.get("material_preview_fix"),
        "generator_version": gen.get("generator_version"),
        "blender_build_hash": gen.get("blender_build_hash"),
    }


def semantic_manifest(man: dict) -> dict:
    return {
        "preview_artifact_version": man.get("preview_artifact_version"),
        "preview_artifact_id": man.get("preview_artifact_id"),
        "source_product_kind": man.get("source_product_kind"),
        "source_product_id": man.get("source_product_id"),
        "source_product_version": man.get("source_product_version"),
        "source_binding": man.get("source_binding"),
        "generator_id": man.get("generator_id"),
        "generator_version": man.get("generator_version"),
        "generator_backend": man.get("generator_backend"),
        "generator_build": man.get("generator_build"),
        "payload_type": man.get("payload_type"),
        "payload_mime": man.get("payload_mime"),
        "animation_inventory": man.get("animation_inventory"),
        "default_animation": man.get("default_animation"),
        "duration_s": man.get("duration_s"),
        "scene": {
            "valid": (man.get("scene") or {}).get("valid"),
            "mesh_object_count": (man.get("scene") or {}).get("mesh_object_count"),
        },
        "declared_losses": man.get("declared_losses"),
        "generation_recipe_sha256": man.get("generation_recipe_sha256"),
        "authority": man.get("authority"),
    }


def fixture_product(fixture: dict, kind: str) -> dict:
    for p in fixture["products"]:
        if p["product_kind"] == kind:
            return p
    raise KeyError(kind)


def product_truth_snapshot(fixture: dict) -> dict:
    """Independent of Preview Artifacts. Fixture + live source lineage files."""
    char = fixture_product(fixture, "CHARACTER")
    motion = fixture_product(fixture, "MOTION")
    derived = fixture_product(fixture, "DERIVED_VARIANT")
    bind_d = derived["expected_source_binding"]
    return {
        "source": "product_fixture.json + live source file SHA-256",
        "product_fixture_id": fixture["fixture_id"],
        "product_fixture_sha256": sha256_file(FIXTURE_PATH),
        "character": {
            "product_id": char["product_id"],
            "product_version": char["product_version"],
            "file_sha256": sha256_file(CHAR_PATH),
            "expected_character_sha256": char["expected_source_binding"]["character_sha256"],
        },
        "motion": {
            "product_id": motion["product_id"],
            "product_version": motion["product_version"],
            "file_sha256": sha256_file(MOTION_PATH),
            "expected_motion_sha256": motion["expected_source_binding"]["motion_sha256"],
            "clip_id": motion["expected_source_binding"]["clip_id"],
        },
        "derived_variant": {
            "product_id": derived["product_id"],
            "product_version": derived["product_version"],
            "derived_variant_id": bind_d["derived_variant_id"],
            "persistence_blend_sha256_live": sha256_file(BLEND_PATH),
            "persistence_blend_sha256_expected": bind_d["persistence_blend_sha256"],
            "mapping_sha256": bind_d["mapping_sha256"],
            "retarget_policy_sha256": bind_d["retarget_policy_sha256"],
            "job_spec_semantic_hash": bind_d["job_spec_semantic_hash"],
            "qc_report_sha256": bind_d["qc_report_sha256"],
        },
    }


def catalog_entry_from_fixture(product: dict) -> dict:
    return {
        "product_kind": product["product_kind"],
        "product_id": product["product_id"],
        "product_version": product["product_version"],
        "label": product["label"],
        "preview_artifact_id": product["preview_artifact_id"],
        "expected_source_binding": dict(product["expected_source_binding"]),
    }


def build_manifest(product: dict, gen: dict, payload: Path, recipe_sha: str) -> dict:
    binding = dict(product["expected_source_binding"])
    bounds = gen.get("bounds") or {}
    cam = {
        "target": bounds.get("center"),
        "radius": bounds.get("radius"),
        "orbit": "0deg 75deg auto",
    }
    man = {
        "preview_artifact_version": "poc-preview-01r.v1",
        "preview_artifact_id": product["preview_artifact_id"],
        "classification": "RESEARCH_ONLY / DERIVED / REBUILDABLE / NON-AUTHORITATIVE",
        "source_product_kind": product["product_kind"],
        "source_product_id": product["product_id"],
        "source_product_version": product["product_version"],
        "source_binding": binding,
        "generator_id": gen.get("generator_id"),
        "generator_version": gen.get("generator_version"),
        "generator_backend": "blender",
        "generator_backend_version": gen.get("blender_version"),
        "generator_build": gen.get("blender_build_hash"),
        "payload_type": "GLB",
        "payload_mime": "model/gltf-binary",
        "payload_ref": {"scheme": "preview-payload", "id": payload.name},
        "payload_sha256": sha256_file(payload),
        "payload_size": payload.stat().st_size,
        "scene": bounds,
        "camera_framing_hint": cam,
        "animation_inventory": [a.get("name") for a in gen.get("animation_inventory") or []],
        "default_animation": gen.get("default_animation"),
        "duration_s": gen.get("duration_s"),
        "fps": gen.get("fps"),
        "declared_losses": gen.get("warnings") or [],
        "generation_recipe_sha256": recipe_sha,
        "generated_from": {
            "poc": "POC-PREVIEW-01R",
            "lineage_poc": "POC-BLENDER-E2E-01" if product["product_kind"] != "CHARACTER" else "POC-FBX-01/POC-BLENDER-E2E-01",
            "product_fixture_id": "poc-preview-01r.product-fixture.v1",
        },
        "authority": {
            "preview_is_product_authority": False,
            "preview_defines_asset_identity": False,
            "preview_defines_mapping": False,
            "preview_defines_qc": False,
        },
        "payload_not_selected_as_product_format": True,
        "viewer_library_not_selected": True,
    }
    man["semantic_hash"] = semantic_hash(semantic_manifest(man))
    man["generation_semantic_hash"] = semantic_hash(semantic_generation(gen))
    return man


def generate_one(kind: str, mode: str, source: Path, payload: Path, workspace: Path, extra: dict) -> dict:
    workspace.mkdir(parents=True, exist_ok=True)
    gen_json = workspace / "generation.json"
    clip_id = extra.get("clip_id", "Armature|Armature|Walk_Carry_Loop")
    job = {
        "source_path": str(source),
        "clip_id": clip_id,
        "fps": 30,
        **extra,
        "outputs": {"payload": str(payload), "generation_json": str(gen_json), "workspace": str(workspace)},
    }
    job_path = workspace / "job.json"
    write_json(job_path, job)
    proc = run_blender(mode, job_path, workspace)
    if proc.returncode != 0:
        raise SystemExit(f"{mode} failed rc={proc.returncode}\n{(proc.stderr or '')[-2500:]}")
    gen = load_json(gen_json)
    if gen.get("status") != "SUCCESS":
        raise SystemExit(f"{mode} status {gen.get('status')}")
    gen["wall_s"] = getattr(proc, "elapsed_s", None)
    return gen


def run_generation(label: str, kinds: list, fixture: dict, recipe_sha: str, payload_root: Path, ws_root: Path) -> dict:
    if payload_root.exists():
        shutil.rmtree(payload_root)
    payload_root.mkdir(parents=True)
    if ws_root.exists():
        shutil.rmtree(ws_root)
    ws_root.mkdir(parents=True)
    out = {}
    for kind, mode, source, fname, extra in kinds:
        product = fixture_product(fixture, kind)
        payload = payload_root / fname
        ws = ws_root / mode
        gen = generate_one(kind, mode, source, payload, ws, extra)
        man = build_manifest(product, gen, payload, recipe_sha)
        write_json(ws / "manifest.json", man)
        write_json(ws / "generation.json", gen)
        out[kind] = {
            "manifest": man,
            "generation": gen,
            "payload_sha256": man["payload_sha256"],
            "payload_size": man["payload_size"],
            "manifest_semantic_hash": man["semantic_hash"],
            "generation_semantic_hash": man["generation_semantic_hash"],
        }
    return out


def assert_product_truth_matches_files(fixture: dict) -> None:
    char = fixture_product(fixture, "CHARACTER")["expected_source_binding"]
    motion = fixture_product(fixture, "MOTION")["expected_source_binding"]
    derived = fixture_product(fixture, "DERIVED_VARIANT")["expected_source_binding"]
    if sha256_file(CHAR_PATH) != char["character_sha256"]:
        raise SystemExit("STOP: live Knight hash != product fixture")
    if sha256_file(MOTION_PATH) != motion["motion_sha256"]:
        raise SystemExit("STOP: live Motion hash != product fixture")
    if sha256_file(BLEND_PATH) != derived["persistence_blend_sha256"]:
        raise SystemExit("STOP: live Derived blend hash != product fixture")


def main() -> int:
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    WORK.mkdir(parents=True, exist_ok=True)
    if not BLENDER.is_file():
        raise SystemExit(f"STOP: missing blender {BLENDER}")
    if not CHAR_PATH.is_file():
        raise SystemExit(f"STOP: missing Character {CHAR_PATH}")
    if not MOTION_PATH.is_file():
        raise SystemExit(f"STOP: missing Motion {MOTION_PATH}")
    if not BLEND_PATH.is_file():
        raise SystemExit(f"STOP: missing Derived persistence {BLEND_PATH}")
    if not FIXTURE_PATH.is_file():
        raise SystemExit(f"STOP: missing product fixture {FIXTURE_PATH}")

    fixture = load_json(FIXTURE_PATH)
    assert_product_truth_matches_files(fixture)
    recipe_sha = sha256_file(CONFIG_PATH)
    generator_src_sha = sha256_file(GENERATOR)
    clip_id = fixture_product(fixture, "MOTION")["expected_source_binding"]["clip_id"]

    kinds = [
        ("CHARACTER", "character", CHAR_PATH, "character_preview.glb", {"clip_id": clip_id}),
        ("MOTION", "motion", MOTION_PATH, "motion_preview.glb", {"clip_id": clip_id}),
        ("DERIVED_VARIANT", "derived", BLEND_PATH, "derived_preview.glb", {"clip_id": clip_id}),
    ]

    payloads_a = WORK / "payloads_a"
    payloads_b = WORK / "payloads_b"
    gen_a = WORK / "gen_a"
    gen_b = WORK / "gen_b"

    runs = {}
    runs["A"] = run_generation("A", kinds, fixture, recipe_sha, payloads_a, gen_a)

    product_before = product_truth_snapshot(fixture)
    a_payload_names = sorted(p.name for p in payloads_a.glob("*.glb"))
    if len(a_payload_names) != 3:
        raise SystemExit(f"STOP: generation A missing payloads: {a_payload_names}")

    shutil.rmtree(payloads_a)
    shutil.rmtree(gen_a)
    deleted = {
        "deleted_payloads_a": not payloads_a.exists(),
        "deleted_gen_a": not gen_a.exists(),
        "deleted_payload_names": a_payload_names,
    }
    if payloads_a.exists() or gen_a.exists():
        raise SystemExit("STOP: preview output deletion did not remove A directories")

    product_after_delete = product_truth_snapshot(fixture)
    if product_after_delete != product_before:
        raise SystemExit("STOP: product truth changed after Preview output deletion")

    runs["B"] = run_generation("B", kinds, fixture, recipe_sha, payloads_b, gen_b)

    runtime = WORK / "runtime"
    if runtime.exists():
        shutil.rmtree(runtime)
    (runtime / "payloads").mkdir(parents=True)
    (runtime / "artifacts").mkdir(parents=True)
    for fname in ("character_preview.glb", "motion_preview.glb", "derived_preview.glb"):
        shutil.copy2(payloads_b / fname, runtime / "payloads" / fname)

    catalog_assets = []
    for product in fixture["products"]:
        kind = product["product_kind"]
        man = runs["B"][kind]["manifest"]
        write_json(runtime / "artifacts" / f"{man['preview_artifact_id']}.json", man)
        ev_stem = {"CHARACTER": "character", "MOTION": "motion", "DERIVED_VARIANT": "derived"}[kind]
        write_json(EVIDENCE / f"{ev_stem}_preview_manifest.json", man)
        write_json(EVIDENCE / f"{ev_stem}_generation.json", runs["B"][kind]["generation"])
        catalog_assets.append(catalog_entry_from_fixture(product))

    char_product = fixture_product(fixture, "CHARACTER")
    char_man = runs["B"]["CHARACTER"]["manifest"]

    stale = dict(char_man)
    stale["preview_artifact_id"] = "preview.character.stale-binding.fixture"
    stale["source_binding"] = dict(stale["source_binding"])
    stale["source_binding"]["character_sha256"] = "0" * 64
    stale["source_binding"]["source_asset_sha256"] = "0" * 64
    stale.pop("semantic_hash", None)
    stale["semantic_hash"] = semantic_hash(semantic_manifest(stale))
    write_json(runtime / "artifacts" / f"{stale['preview_artifact_id']}.json", stale)
    write_json(EVIDENCE / "stale_character_manifest.json", stale)

    version_bad = dict(char_man)
    version_bad["preview_artifact_id"] = "preview.character.wrong-version.fixture"
    version_bad["source_product_version"] = "wrong.version.not-frozen"
    version_bad.pop("semantic_hash", None)
    version_bad["semantic_hash"] = semantic_hash(semantic_manifest(version_bad))
    write_json(runtime / "artifacts" / f"{version_bad['preview_artifact_id']}.json", version_bad)
    write_json(EVIDENCE / "stale_version_manifest.json", version_bad)

    payload_bad = dict(char_man)
    payload_bad["preview_artifact_id"] = "preview.character.payload-hash.fixture"
    payload_bad["payload_sha256"] = "0" * 64
    payload_bad.pop("semantic_hash", None)
    payload_bad["semantic_hash"] = semantic_hash(semantic_manifest(payload_bad))
    write_json(runtime / "artifacts" / f"{payload_bad['preview_artifact_id']}.json", payload_bad)
    write_json(EVIDENCE / "stale_payload_manifest.json", payload_bad)

    def fixture_card(fid: str, label: str, artifact_id: str, expected: str) -> dict:
        entry = catalog_entry_from_fixture(char_product)
        entry["id"] = fid
        entry["label"] = label
        entry["preview_artifact_id"] = artifact_id
        entry["expected"] = expected
        return entry

    catalog = {
        "catalog_id": "poc-preview-01r.catalog",
        "classification": "RESEARCH_ONLY; catalog expected binding comes from product_fixture.json, not Preview Manifest",
        "product_fixture_id": fixture["fixture_id"],
        "product_fixture_sha256": sha256_file(FIXTURE_PATH),
        "assets": catalog_assets,
        "fixtures": [
            fixture_card(
                "stale-character-hash",
                "Stale Character hash",
                stale["preview_artifact_id"],
                "BINDING_FAIL",
            ),
            fixture_card(
                "wrong-product-version",
                "Wrong Character version",
                version_bad["preview_artifact_id"],
                "BINDING_FAIL",
            ),
            fixture_card(
                "payload-hash-mismatch",
                "Payload SHA mismatch",
                payload_bad["preview_artifact_id"],
                "PAYLOAD_HASH_FAIL",
            ),
        ],
        "resolver": {
            "artifact_manifest": "/artifacts/{preview_artifact_id}.json",
            "payload": "/payloads/{payload_ref.id}",
        },
    }
    write_json(runtime / "catalog.json", catalog)
    write_json(EVIDENCE / "preview_catalog.json", catalog)
    write_json(EVIDENCE / "product_fixture.json", fixture)

    rebuild = {
        "sequence": [
            "generate A",
            "record product truth from product_fixture.json + live source SHA-256",
            "delete payloads_a and gen_a Preview outputs",
            "verify product truth unchanged",
            "generate B from frozen inputs",
            "viewer loads regenerated B",
        ],
        "literal_delete": deleted,
        "product_truth_source": "product_fixture.json + live Character/Motion/blend files (not Preview Manifest)",
        "product_state_before_delete": product_before,
        "product_state_after_delete": product_after_delete,
        "product_state_unchanged": product_before == product_after_delete,
        "kinds": {},
        "payload_byte_identical": {},
        "semantic_equivalent": True,
    }
    for kind, *_rest in kinds:
        a = runs["A"][kind]
        b = runs["B"][kind]
        same_sem = a["manifest_semantic_hash"] == b["manifest_semantic_hash"]
        same_gen = a["generation_semantic_hash"] == b["generation_semantic_hash"]
        same_pay = a["payload_sha256"] == b["payload_sha256"]
        rebuild["payload_byte_identical"][kind] = same_pay
        rebuild["kinds"][kind] = {
            "run_a_manifest_semantic_hash": a["manifest_semantic_hash"],
            "run_b_manifest_semantic_hash": b["manifest_semantic_hash"],
            "run_a_generation_semantic_hash": a["generation_semantic_hash"],
            "run_b_generation_semantic_hash": b["generation_semantic_hash"],
            "run_a_payload_sha256": a["payload_sha256"],
            "run_b_payload_sha256": b["payload_sha256"],
            "run_a_payload_size": a["payload_size"],
            "run_b_payload_size": b["payload_size"],
            "run_a_animation_inventory": a["manifest"].get("animation_inventory"),
            "run_b_animation_inventory": b["manifest"].get("animation_inventory"),
            "run_a_duration_s": a["manifest"].get("duration_s"),
            "run_b_duration_s": b["manifest"].get("duration_s"),
            "manifest_semantic_match": same_sem,
            "generation_semantic_match": same_gen,
            "payload_sha_match": same_pay,
        }
        if not (same_sem and same_gen):
            rebuild["semantic_equivalent"] = False
    write_json(EVIDENCE / "rebuildability.json", rebuild)

    write_json(
        EVIDENCE / "runtime_independence.json",
        {
            "note": "viewer request audit filled by preview_selftest.py",
            "product_state_unchanged_across_payload_delete": rebuild["product_state_unchanged"],
            "product_state": product_after_delete,
        },
    )

    timing = {
        "character_preview_generation_s": runs["B"]["CHARACTER"]["generation"].get("elapsed_s"),
        "motion_preview_generation_s": runs["B"]["MOTION"]["generation"].get("elapsed_s"),
        "derived_preview_generation_s": runs["B"]["DERIVED_VARIANT"]["generation"].get("elapsed_s"),
        "character_wall_s": runs["B"]["CHARACTER"]["generation"].get("wall_s"),
        "motion_wall_s": runs["B"]["MOTION"]["generation"].get("wall_s"),
        "derived_wall_s": runs["B"]["DERIVED_VARIANT"]["generation"].get("wall_s"),
    }
    write_json(EVIDENCE / "timing_summary.json", timing)
    write_json(
        WORK / "generation_index.json",
        {
            "recipe_sha256": recipe_sha,
            "generator_source_sha256": generator_src_sha,
            "product_fixture_sha256": sha256_file(FIXTURE_PATH),
            "runs": {
                lab: {k: {"payload_sha256": v["payload_sha256"], "semantic": v["manifest_semantic_hash"]} for k, v in kinds_map.items()}
                for lab, kinds_map in runs.items()
            },
        },
    )
    print("rebuild_semantic", rebuild["semantic_equivalent"])
    print("payload_identical", rebuild["payload_byte_identical"])
    print("deleted_A", deleted)
    print("product_unchanged", rebuild["product_state_unchanged"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
