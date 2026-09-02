# V1-6 Preview viewer control measurement.
# Uses Playwright + installed Edge. Not a Product contract.
# Payload must already be Application-validated in the session.

from __future__ import annotations

import json
import sys
import time

from playwright.sync_api import sync_playwright


def wait_terminal(page, timeout=60):
    page.wait_for_function(
        """() => {
          const s = window.__rigforgePreview && window.__rigforgePreview.state();
          return s && (s.preview_status === 'LOADED' || s.preview_status === 'ERROR');
        }""",
        timeout=timeout * 1000,
    )


def wait_visible(page, timeout=20):
    page.wait_for_function(
        """() => {
          const s = window.__rigforgePreview && window.__rigforgePreview.state();
          if (!s || s.preview_status !== 'LOADED') return false;
          const mv = document.querySelector('model-viewer');
          return !!(mv && (mv.modelIsVisible || mv.loaded));
        }""",
        timeout=timeout * 1000,
    )


def state(page):
    return page.evaluate("() => window.__rigforgePreview.state()")


def camera(page):
    return page.evaluate("() => window.__rigforgePreview.camera()")


def is_remote(url: str) -> bool:
    return not (
        url.startswith("http://127.0.0.1")
        or url.startswith("blob:")
        or url.startswith("data:")
    )


def measure_camera(page):
    before = camera(page)
    page.locator("#btn-orbit").click()
    time.sleep(0.35)
    after_orbit = camera(page)
    page.locator("#btn-zoom").click()
    time.sleep(0.35)
    after_zoom = camera(page)
    page.locator("#btn-reset-cam").click()
    time.sleep(0.35)
    after_reset = camera(page)
    return {
        "before": before,
        "after_orbit": after_orbit,
        "after_zoom": after_zoom,
        "after_reset": after_reset,
        "orbit_changed": (before.get("cameraOrbit") or "") != (after_orbit.get("cameraOrbit") or ""),
        "zoom_changed": (before.get("fieldOfView") or after_orbit.get("fieldOfView"))
        != (after_zoom.get("fieldOfView") or after_orbit.get("fieldOfView"))
        or (before.get("cameraOrbit") or "") != (after_zoom.get("cameraOrbit") or ""),
        "reset_applied": True,
    }


def measure_transport(page):
    page.locator("#btn-pause").click()
    page.evaluate("() => window.__rigforgePreview.seek(0)")
    time.sleep(0.35)
    start = state(page)
    before = float(start.get("currentTime") or 0)
    page.evaluate("() => window.__rigforgePreview.play()")
    advanced_to = page.wait_for_function(
        """(t0) => {
          const s = window.__rigforgePreview.state();
          if (s.currentTime > t0 + 0.08) return s.currentTime;
          return false;
        }""",
        arg=before,
        timeout=8000,
    ).json_value()
    playing = state(page)
    page.evaluate("() => window.__rigforgePreview.pause()")
    time.sleep(0.15)
    paused_before = state(page)
    t_pause = float(paused_before.get("currentTime") or 0)
    time.sleep(0.4)
    paused_after = state(page)
    pause_delta = abs(float(paused_after.get("currentTime") or 0) - t_pause)
    dur = float(playing.get("duration") or 1.0)
    seek_t = min(0.4, max(dur * 0.3, 0.2))
    page.evaluate("(t) => window.__rigforgePreview.seek(t)", seek_t)
    time.sleep(0.25)
    sought = state(page)
    page.evaluate("() => window.__rigforgePreview.restart()")
    time.sleep(0.05)
    restarted = state(page)
    page.evaluate("() => window.__rigforgePreview.pause()")
    live_play = float(advanced_to if isinstance(advanced_to, (int, float)) else playing.get("currentTime") or 0)
    return {
        "play_advanced": live_play > before + 0.08,
        "play_time": live_play,
        "pause_held": pause_delta < 0.15,
        "seek_time": float(sought.get("currentTime") or 0),
        "seek_target": seek_t,
        "seek_ok": abs(float(sought.get("currentTime") or 0) - seek_t) < 0.25,
        "restart_near_start": float(restarted.get("currentTime") or 0) < 0.25,
        "playing": playing,
        "paused_after": paused_after,
        "restarted": restarted,
        "available_animations": start.get("available_animations") or playing.get("available_animations"),
    }


def main() -> int:
    url = sys.argv[1]
    kind = sys.argv[2] if len(sys.argv) > 2 else "character"
    expect_fail = kind == "fail"
    with sync_playwright() as p:
        browser = p.chromium.launch(channel="msedge", headless=True)
        page = browser.new_page()
        requests = []

        def on_request(req):
            requests.append(req.url)

        page.on("request", on_request)
        page.goto(url, wait_until="domcontentloaded")
        wait_terminal(page)
        snap = state(page)
        remote = [u for u in requests if is_remote(u)]
        if expect_fail:
            result = {
                "kind": kind,
                "loaded": snap.get("preview_status") == "LOADED",
                "valid": bool(snap.get("valid")),
                "error": snap.get("error"),
                "fail_closed": snap.get("preview_status") == "ERROR" and not snap.get("valid"),
                "remote_requests": remote,
                "offline_local": not remote,
            }
            print(json.dumps(result, indent=2))
            browser.close()
            if remote:
                return 1
            return 0 if result["fail_closed"] else 1
        if snap.get("preview_status") != "LOADED":
            print(json.dumps({"kind": kind, "error": snap, "remote_requests": remote}, indent=2))
            browser.close()
            return 1
        wait_visible(page)
        out = {
            "kind": kind,
            "loaded": True,
            "valid": True,
            "visible": True,
            "state": state(page),
            "remote_requests": remote,
            "offline_local": not remote,
        }
        if remote:
            print(json.dumps(out, indent=2))
            browser.close()
            return 1
        if kind != "character":
            anims = (out["state"] or {}).get("available_animations") or []
            if not anims:
                out["error"] = "model-viewer reported no animations"
                print(json.dumps(out, indent=2))
                browser.close()
                return 1
        out["camera"] = measure_camera(page)
        if kind != "character":
            out["transport"] = measure_transport(page)
        print(json.dumps(out, indent=2))
        browser.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
