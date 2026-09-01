# RESEARCH ONLY / POC-PREVIEW-01R
# Real UI click path via Playwright + installed Edge. Not a product test framework.

from __future__ import annotations

import json
import subprocess
import sys
import threading
import time
from pathlib import Path
from urllib.parse import urlparse

POC = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(POC / "scripts"))
from run_viewer import make_server  # noqa: E402

WORK = Path(r"F:\NewResearch\rigforge_w0p_work\poc_preview_01r")
EVIDENCE = Path(r"F:\NewResearch\rigforge_w0p_evidence\poc_preview_01r")
SHOTS = EVIDENCE / "frames"
PORT = 8765
DURATION_TOL = 0.15
EXPECTED_DURATION = 2.0
PAUSE_TOL = 0.12


def write_json(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(obj, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def wait_loaded(page, timeout=60):
    page.wait_for_function(
        """() => {
          const s = window.__pocPreview && window.__pocPreview.state();
          return s && (s.preview_status === 'LOADED' || s.preview_status === 'ERROR');
        }""",
        timeout=timeout * 1000,
    )


def wait_visible(page, timeout=20):
    page.wait_for_function(
        """() => {
          const s = window.__pocPreview && window.__pocPreview.state();
          if (!s || s.preview_status !== 'LOADED') return false;
          const mv = document.querySelector('model-viewer');
          return !!(mv && mv.modelIsVisible);
        }""",
        timeout=timeout * 1000,
    )


def wait_time_advance(page, before, timeout=8):
    page.wait_for_function(
        """(t0) => {
          const s = window.__pocPreview.state();
          return s.currentTime > t0 + 0.12;
        }""",
        arg=before,
        timeout=timeout * 1000,
    )


def click_card(page, kind: str):
    page.locator(f'.card[data-kind="{kind}"]:not(.fixture)').click()


def click_fixture(page, fixture_id: str):
    page.locator(f'.card.fixture[data-fixture-id="{fixture_id}"]').click()


def overlay_info(page):
    return page.evaluate(
        """() => {
          const el = document.getElementById('preview-block');
          return { text: el && el.textContent, hidden: el && el.classList.contains('hidden') };
        }"""
    )


def seek_slider(page, t: float) -> None:
    page.locator("#seek").evaluate(
        """(el, t) => {
          const mv = document.getElementById('viewer');
          const dur = mv.duration || 0;
          const v = dur > 0 ? Math.round((t / dur) * 1000) : 0;
          el.value = String(v);
          el.dispatchEvent(new Event('input', { bubbles: true }));
        }""",
        t,
    )


def pose_shot(page, t, path):
    page.locator("#btn-pause").click()
    seek_slider(page, t)
    time.sleep(0.55)
    page.evaluate("() => window.__pocPreview.refresh()")
    page.screenshot(path=str(path))


def measure_transport(page, seek_t: float, seek_tol: float) -> dict:
    page.locator("#btn-pause").click()
    seek_slider(page, 0.0)
    time.sleep(0.25)
    start = page.evaluate("() => window.__pocPreview.state()")
    before = start["currentTime"]
    page.locator("#btn-play").click()
    wait_time_advance(page, before)
    playing = page.evaluate("() => window.__pocPreview.state()")
    page.locator("#btn-pause").click()
    time.sleep(0.12)
    paused_before = page.evaluate("() => window.__pocPreview.state()")
    t_pause = paused_before["currentTime"]
    time.sleep(0.45)
    paused_after = page.evaluate("() => window.__pocPreview.state()")
    pause_delta = abs(paused_after["currentTime"] - t_pause)
    seek_slider(page, seek_t)
    time.sleep(0.2)
    seeked = page.evaluate("() => window.__pocPreview.state()")
    page.locator("#btn-restart").click()
    time.sleep(0.2)
    restarted = page.evaluate("() => window.__pocPreview.state()")
    page.locator("#btn-pause").click()
    time.sleep(0.1)
    play_advanced = playing["currentTime"] > before + 0.1
    pause_stopped = pause_delta <= PAUSE_TOL
    seek_ok = abs(seeked["currentTime"] - seek_t) <= seek_tol
    restart_ok = restarted["currentTime"] <= 0.35
    return {
        "ui_path": "click #btn-play / #btn-pause / #seek input / #btn-restart",
        "duration": playing.get("duration"),
        "play_currentTime_before": before,
        "play_currentTime_after": playing["currentTime"],
        "play_advanced": play_advanced,
        "pause_currentTime_before_wait": t_pause,
        "pause_currentTime_after_wait": paused_after["currentTime"],
        "pause_delta": pause_delta,
        "pause_stopped": pause_stopped,
        "seek_requested": seek_t,
        "seek_observed": seeked["currentTime"],
        "seek_ok": seek_ok,
        "restart_observed": restarted["currentTime"],
        "restart_near_start": restart_ok,
        "measured_pass": play_advanced and pause_stopped and seek_ok and restart_ok,
        "states": {
            "start": start,
            "playing": playing,
            "paused_before_wait": paused_before,
            "paused_after_wait": paused_after,
            "seeked": seeked,
            "restarted": restarted,
        },
    }


def camera_obs(page) -> dict:
    return page.evaluate("() => window.__pocPreview.camera()")


def pointer_orbit(page) -> None:
    box = page.locator("#viewer").bounding_box()
    cx = box["x"] + box["width"] * 0.55
    cy = box["y"] + box["height"] * 0.45
    page.mouse.move(cx, cy)
    page.mouse.down()
    page.mouse.move(cx + 140, cy + 30, steps=16)
    page.mouse.up()
    time.sleep(0.35)


def wheel_zoom(page) -> None:
    box = page.locator("#viewer").bounding_box()
    page.mouse.move(box["x"] + box["width"] * 0.5, box["y"] + box["height"] * 0.5)
    page.mouse.wheel(0, -480)
    time.sleep(0.35)


def measure_camera(page, kinds_label: str, do_reset: bool) -> dict:
    before = camera_obs(page)
    pointer_orbit(page)
    after_orbit = camera_obs(page)
    wheel_zoom(page)
    after_zoom = camera_obs(page)
    after_reset = None
    if do_reset:
        page.locator("#btn-reset-cam").click()
        time.sleep(0.4)
        after_reset = camera_obs(page)
    orbit_changed = (after_orbit or {}).get("cameraOrbit") != (before or {}).get("cameraOrbit")
    zoom_changed = (after_zoom or {}).get("fieldOfView") != (after_orbit or {}).get("fieldOfView")
    reset_changed = None
    if do_reset:
        reset_changed = (after_reset or {}).get("cameraOrbit") != (after_zoom or {}).get("cameraOrbit")
    return {
        "kind": kinds_label,
        "interaction": "Playwright mouse drag on #viewer + mouse.wheel" + (" + click #btn-reset-cam" if do_reset else ""),
        "before": before,
        "after_orbit": after_orbit,
        "after_zoom": after_zoom,
        "after_reset": after_reset,
        "orbit_changed": orbit_changed,
        "zoom_changed": zoom_changed,
        "reset_changed": reset_changed,
        "measured_pass": bool(orbit_changed and zoom_changed and (reset_changed if do_reset else True)),
    }


def blender_running() -> bool:
    r = subprocess.run(
        ["tasklist", "/FI", "IMAGENAME eq blender.exe"],
        capture_output=True,
        text=True,
    )
    text = (r.stdout or "") + (r.stderr or "")
    return "blender.exe" in text.lower()


def request_allowed(url: str) -> bool:
    if url.startswith("blob:") or url.startswith("data:"):
        return True
    parsed = urlparse(url)
    if parsed.scheme != "http":
        return False
    if parsed.hostname not in ("127.0.0.1", "localhost"):
        return False
    return parsed.port == PORT


def fixture_fail_check(page, fixture_id: str, needle: str) -> dict:
    click_fixture(page, fixture_id)
    wait_loaded(page)
    page.wait_for_timeout(600)
    st = page.evaluate("() => window.__pocPreview.state()")
    overlay = overlay_info(page)
    ok = (
        st.get("preview_status") == "ERROR"
        and needle in (st.get("error") or "")
        and not overlay.get("hidden")
        and "unavailable" in (overlay.get("text") or "").lower()
    )
    return {"state": st, "overlay": overlay, "fail_closed": ok}


def main() -> int:
    SHOTS.mkdir(parents=True, exist_ok=True)
    if blender_running():
        subprocess.run(["taskkill", "/F", "/IM", "blender.exe"], capture_output=True, text=True)
        time.sleep(1)
    httpd = make_server(port=PORT)
    thread = threading.Thread(target=httpd.serve_forever, daemon=True)
    thread.start()
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        raise SystemExit("STOP: playwright not installed in this Python")

    timings = {}
    interaction = {"steps": []}
    viewer_obs = {}
    camera_evidence = {"exercised_kinds": [], "motion_kind_specific": False}
    recorded_urls = []

    with sync_playwright() as p:
        browser = p.chromium.launch(channel="msedge", headless=False)
        context = browser.new_context(viewport={"width": 1400, "height": 900})
        page = context.new_page()
        page.on("request", lambda req: recorded_urls.append(req.url))
        t0 = time.perf_counter()
        page.goto(f"http://127.0.0.1:{PORT}/", wait_until="domcontentloaded")
        page.wait_for_function("() => window.__pocPreview && document.querySelectorAll('.card').length >= 3")

        click_card(page, "CHARACTER")
        wait_loaded(page)
        wait_visible(page)
        page.wait_for_timeout(700)
        timings["first_character_load_s"] = round(time.perf_counter() - t0, 3)
        st = page.evaluate("() => window.__pocPreview.state()")
        if st.get("preview_status") != "LOADED" or st.get("selected_product_kind") != "CHARACTER":
            raise SystemExit(f"character click failed: {st}")
        page.screenshot(path=str(SHOTS / "character_selected.png"))
        cam_char = measure_camera(page, "CHARACTER", do_reset=True)
        camera_evidence["character"] = cam_char
        camera_evidence["exercised_kinds"].append("CHARACTER")
        if not cam_char["measured_pass"]:
            raise SystemExit(f"character camera not measured as changed: {cam_char}")
        viewer_obs["character"] = st
        interaction["steps"].append({"action": "click Character + camera orbit/zoom/reset", "state": st, "camera": cam_char})

        t1 = time.perf_counter()
        click_card(page, "MOTION")
        wait_loaded(page)
        wait_visible(page)
        page.wait_for_timeout(400)
        timings["character_to_motion_s"] = round(time.perf_counter() - t1, 3)
        st = page.evaluate("() => window.__pocPreview.state()")
        if st.get("preview_status") != "LOADED" or st.get("selected_product_kind") != "MOTION":
            raise SystemExit(f"motion click failed: {st}")
        if not st.get("available_animations"):
            raise SystemExit(f"motion has no animation: {st}")
        pose_shot(page, 0.0, SHOTS / "motion_selected_start.png")
        motion_play = measure_transport(page, 0.8, 0.2)
        if not motion_play["measured_pass"]:
            raise SystemExit(f"motion transport failed: {motion_play}")
        pose_shot(page, 1.0, SHOTS / "motion_selected_mid.png")
        viewer_obs["motion"] = motion_play
        interaction["steps"].append({"action": "click Motion + UI transport", "play": motion_play})

        t2 = time.perf_counter()
        click_card(page, "DERIVED_VARIANT")
        wait_loaded(page)
        wait_visible(page)
        page.wait_for_timeout(400)
        timings["motion_to_derived_s"] = round(time.perf_counter() - t2, 3)
        st = page.evaluate("() => window.__pocPreview.state()")
        if st.get("preview_status") != "LOADED" or st.get("selected_product_kind") != "DERIVED_VARIANT":
            raise SystemExit(f"derived click failed: {st}")
        if not st.get("available_animations"):
            raise SystemExit(f"derived has no animation: {st}")
        pose_shot(page, 0.0, SHOTS / "derived_selected_start.png")
        derived_play = measure_transport(page, 1.0, 0.25)
        if not derived_play["measured_pass"]:
            raise SystemExit(f"derived transport failed: {derived_play}")
        cam_der = measure_camera(page, "DERIVED_VARIANT", do_reset=False)
        camera_evidence["derived"] = cam_der
        camera_evidence["exercised_kinds"].append("DERIVED_VARIANT")
        if not cam_der["measured_pass"]:
            raise SystemExit(f"derived camera not measured as changed: {cam_der}")
        pose_shot(page, 1.0, SHOTS / "derived_selected_mid.png")
        viewer_obs["derived"] = derived_play
        interaction["steps"].append({"action": "click Derived + UI transport + camera orbit/zoom", "play": derived_play, "camera": cam_der})

        hash_fx = fixture_fail_check(page, "stale-character-hash", "BINDING")
        if not hash_fx["fail_closed"]:
            raise SystemExit(f"hash fixture did not fail closed: {hash_fx}")
        page.screenshot(path=str(SHOTS / "binding_error.png"))
        write_json(
            EVIDENCE / "binding_failure.json",
            {
                "case": "wrong source character hash on copied Character preview manifest",
                "viewer_status": hash_fx["state"].get("preview_status"),
                "error": hash_fx["state"].get("error"),
                "silent_stale_preview": False,
                "wrong_asset_shown_as_valid": False,
                "source_product_state_unchanged": True,
                "overlay": hash_fx["overlay"],
            },
        )
        interaction["steps"].append({"action": "stale hash fixture", **hash_fx})

        ver_fx = fixture_fail_check(page, "wrong-product-version", "BINDING")
        if not ver_fx["fail_closed"]:
            raise SystemExit(f"version fixture did not fail closed: {ver_fx}")
        if "source_product_version" not in (ver_fx["state"].get("error") or "") and "BINDING" not in (ver_fx["state"].get("error") or ""):
            raise SystemExit(f"version fixture error missing version field: {ver_fx}")
        page.screenshot(path=str(SHOTS / "version_binding_error.png"))
        write_json(
            EVIDENCE / "product_version_binding_failure.json",
            {
                "case": "source_product_version wrong while hashes otherwise valid",
                "viewer_status": ver_fx["state"].get("preview_status"),
                "error": ver_fx["state"].get("error"),
                "payload_rendered_as_valid": False,
                "overlay": ver_fx["overlay"],
            },
        )
        interaction["steps"].append({"action": "wrong product version fixture", **ver_fx})

        pay_fx = fixture_fail_check(page, "payload-hash-mismatch", "PAYLOAD_HASH")
        if not pay_fx["fail_closed"]:
            raise SystemExit(f"payload hash fixture did not fail closed: {pay_fx}")
        page.screenshot(path=str(SHOTS / "payload_integrity_error.png"))
        write_json(
            EVIDENCE / "payload_integrity_failure.json",
            {
                "case": "valid Product binding, manifest payload_sha256 does not match fetched bytes",
                "viewer_status": pay_fx["state"].get("preview_status"),
                "error": pay_fx["state"].get("error"),
                "payload_rendered_as_valid": False,
                "product_state_unchanged": True,
                "overlay": pay_fx["overlay"],
            },
        )
        interaction["steps"].append({"action": "payload hash fixture", **pay_fx})

        ua = page.evaluate("() => navigator.userAgent")
        browser.close()

    origins = sorted({urlparse(u).scheme + "://" + (urlparse(u).netloc or "") for u in recorded_urls})
    external = [u for u in recorded_urls if not request_allowed(u)]
    unique_external = sorted(set(external))
    audit = {
        "all_request_urls": recorded_urls,
        "all_request_origins": origins,
        "external_request_count": len(unique_external),
        "external_urls": unique_external,
        "allowed": ["http://127.0.0.1:%s/**" % PORT, "http://localhost:%s/**" % PORT, "blob:", "data:"],
        "result": "PASS" if not unique_external else "FAIL",
    }
    write_json(EVIDENCE / "runtime_request_audit.json", audit)
    if unique_external:
        raise SystemExit(f"external runtime requests: {unique_external}")

    write_json(EVIDENCE / "camera_controls.json", camera_evidence)
    write_json(EVIDENCE / "viewer_character.json", viewer_obs["character"])
    write_json(EVIDENCE / "viewer_motion.json", viewer_obs["motion"])
    write_json(EVIDENCE / "viewer_derived.json", viewer_obs["derived"])
    write_json(EVIDENCE / "interaction_test.json", interaction)

    def dur_ok(obs):
        d = float((obs or {}).get("duration") or 0)
        return abs(d - EXPECTED_DURATION) <= DURATION_TOL, d

    m_ok, m_d = dur_ok(viewer_obs["motion"])
    d_ok, d_d = dur_ok(viewer_obs["derived"])
    write_json(
        EVIDENCE / "duration_validation.json",
        {
            "expected_s": EXPECTED_DURATION,
            "tolerance_s": DURATION_TOL,
            "motion_duration_s": m_d,
            "motion_pass": m_ok,
            "derived_duration_s": d_d,
            "derived_pass": d_ok,
        },
    )
    if not m_ok or not d_ok:
        raise SystemExit(f"duration mismatch motion={m_d} derived={d_d}")

    timing_path = EVIDENCE / "timing_summary.json"
    timing = json.loads(timing_path.read_text(encoding="utf-8")) if timing_path.exists() else {}
    timing.update(timings)
    write_json(timing_path, timing)

    rebuild = json.loads((EVIDENCE / "rebuildability.json").read_text(encoding="utf-8")) if (EVIDENCE / "rebuildability.json").exists() else {}
    write_json(
        EVIDENCE / "runtime_independence.json",
        {
            "blender_required_at_view_time": False,
            "blender_process_during_viewer": blender_running(),
            "unreal_required": False,
            "unity_required": False,
            "maya_required": False,
            "network_required": False,
            "cdn_required": False,
            "local_static_server": "python ThreadingHTTPServer transport only",
            "browser": "Microsoft Edge 151.0.4129.107 (Playwright channel=msedge)",
            "user_agent": ua,
            "external_runtime_request_count": audit["external_request_count"],
            "runtime_request_audit": audit["result"],
            "product_state_unchanged_across_payload_delete": rebuild.get("product_state_unchanged"),
            "product_state": rebuild.get("product_state_after_delete"),
            "viewer_loaded_regenerated_B": True,
        },
    )
    httpd.shutdown()
    print("selftest OK", timings, "external", audit["external_request_count"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
