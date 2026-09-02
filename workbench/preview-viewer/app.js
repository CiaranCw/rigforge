/* V1-6 Preview viewer. Payload is shown only after Application-validated session
   plus local size/SHA-256. Viewer state is presentation-only. */
(function () {
  const statusEl = document.getElementById("status-pre");
  const viewer = document.getElementById("viewer");
  const blockEl = document.getElementById("preview-block");
  const seek = document.getElementById("seek");

  let session = null;
  let lastError = null;
  let loaded = false;
  let blobUrl = null;
  let previousValid = false;

  function hexSha256(buffer) {
    return crypto.subtle.digest("SHA-256", buffer).then(function (digest) {
      const bytes = new Uint8Array(digest);
      let hex = "";
      for (let i = 0; i < bytes.length; i++) {
        hex += bytes[i].toString(16).padStart(2, "0");
      }
      return hex;
    });
  }

  function cameraObs() {
    const orbit = viewer.getCameraOrbit && viewer.getCameraOrbit();
    const target = viewer.getCameraTarget && viewer.getCameraTarget();
    return {
      cameraOrbit: orbit && orbit.toString ? orbit.toString() : String(viewer.cameraOrbit || ""),
      fieldOfView: viewer.getFieldOfView ? viewer.getFieldOfView() : viewer.fieldOfView || null,
      cameraTarget: target && target.toString ? target.toString() : String(viewer.cameraTarget || ""),
    };
  }

  function renderStatus() {
    const failed = !!lastError;
    const text = {
      selected_product_kind: session && session.selected_product_kind,
      selected_product_version_id: session && session.selected_product_version_id,
      preview_artifact_id: session && session.preview_artifact_id,
      preview_status: lastError ? lastError : loaded ? "LOADED" : "IDLE",
      valid: !failed && loaded,
      previous_valid_cleared: !previousValid,
      duration_s: failed ? null : viewer.duration || null,
      animation: failed ? null : viewer.animationName || null,
      available_animations: failed ? [] : viewer.availableAnimations || [],
      currentTime: failed ? null : typeof viewer.currentTime === "number" ? viewer.currentTime : null,
      paused: failed ? null : viewer.paused,
      camera: failed ? null : cameraObs(),
      failure_kind: session && session.failure_kind,
      failure_detail: session && session.failure_detail,
      payload_is_not_product_identity: true,
      viewer_library: "@google/model-viewer",
      viewer_version: "4.3.1",
    };
    statusEl.textContent = JSON.stringify(text, null, 2);
  }

  function showBlock(message) {
    blockEl.textContent = message;
    blockEl.classList.remove("hidden");
  }

  function hideBlock() {
    blockEl.classList.add("hidden");
  }

  function clearViewer() {
    viewer.removeAttribute("src");
    viewer.src = "";
    loaded = false;
    previousValid = false;
    if (blobUrl) {
      URL.revokeObjectURL(blobUrl);
      blobUrl = null;
    }
  }

  function fail(reason) {
    lastError = reason;
    loaded = false;
    previousValid = false;
    clearViewer();
    const detail = (session && session.failure_detail) || reason;
    showBlock("Preview unavailable\n" + detail);
    renderStatus();
  }

  async function loadSession() {
    lastError = null;
    loaded = false;
    clearViewer();
    const resp = await fetch("/session.json", { cache: "no-store" });
    if (!resp.ok) {
      fail("Preview metadata missing");
      return;
    }
    session = await resp.json();
    if (!session || session.valid !== true || !session.payload_filename) {
      fail(session && session.failure_kind ? session.failure_kind : "no Preview generated");
      return;
    }
    const payloadResp = await fetch("/" + session.payload_filename, { cache: "no-store" });
    if (!payloadResp.ok) {
      fail("payload missing");
      return;
    }
    const buf = await payloadResp.arrayBuffer();
    const size = buf.byteLength;
    const expectedSize = Number(session.payload_size);
    if (!(expectedSize > 0) || size !== expectedSize) {
      fail("payload size mismatch");
      return;
    }
    const digest = await hexSha256(buf);
    if (digest !== String(session.payload_sha256 || "").toLowerCase()) {
      fail("payload digest mismatch");
      return;
    }
    blobUrl = URL.createObjectURL(new Blob([buf], { type: "model/gltf-binary" }));
    if (session.default_animation) {
      viewer.setAttribute("animation-name", session.default_animation);
    } else {
      viewer.removeAttribute("animation-name");
    }
    viewer.autoplay = false;
    const onLoad = () => {
      const animations = viewer.availableAnimations || [];
      if (session.selected_product_kind !== "CharacterAssetVersion") {
        if (!animations.length) {
          fail("animation missing from loaded Preview payload");
          return;
        }
      }
      loaded = true;
      previousValid = true;
      if (session.default_animation && animations.includes(session.default_animation)) {
        viewer.animationName = session.default_animation;
      } else if (animations.length) {
        viewer.animationName = animations[0];
      }
      viewer.pause && viewer.pause();
      hideBlock();
      renderStatus();
    };
    viewer.addEventListener("load", onLoad, { once: true });
    viewer.addEventListener("error", () => {
      fail("viewer load failure");
    }, { once: true });
    viewer.setAttribute("src", blobUrl);
  }

  document.getElementById("btn-play").onclick = () => {
    viewer.play && viewer.play();
    renderStatus();
  };
  document.getElementById("btn-pause").onclick = () => {
    viewer.pause && viewer.pause();
    renderStatus();
  };
  document.getElementById("btn-restart").onclick = () => {
    viewer.currentTime = 0;
    viewer.play && viewer.play();
    renderStatus();
  };
  document.getElementById("btn-reset-cam").onclick = () => {
    try {
      viewer.resetTurntableRotation && viewer.resetTurntableRotation();
    } catch (e) {}
    viewer.jumpCameraToGoal && viewer.jumpCameraToGoal();
    try {
      viewer.cameraOrbit = "0deg 75deg 105%";
      viewer.fieldOfView = "30deg";
      viewer.jumpCameraToGoal && viewer.jumpCameraToGoal();
    } catch (e) {}
    renderStatus();
  };
  document.getElementById("btn-orbit").onclick = () => {
    const orbit = viewer.getCameraOrbit && viewer.getCameraOrbit();
    const theta = orbit && typeof orbit.theta === "number" ? orbit.theta : 0;
    const phi = orbit && typeof orbit.phi === "number" ? orbit.phi : 1.2;
    const radius = orbit && typeof orbit.radius === "number" ? orbit.radius : 1;
    const next = (theta + 0.45) + "rad " + phi + "rad " + radius + "m";
    viewer.cameraOrbit = next;
    viewer.jumpCameraToGoal && viewer.jumpCameraToGoal();
    renderStatus();
  };
  document.getElementById("btn-zoom").onclick = () => {
    const orbit = viewer.getCameraOrbit && viewer.getCameraOrbit();
    const theta = orbit && typeof orbit.theta === "number" ? orbit.theta : 0;
    const phi = orbit && typeof orbit.phi === "number" ? orbit.phi : 1.2;
    const radius = orbit && typeof orbit.radius === "number" ? orbit.radius : 1;
    const nextRadius = Math.max(radius * 0.7, 0.15);
    viewer.cameraOrbit = theta + "rad " + phi + "rad " + nextRadius + "m";
    viewer.jumpCameraToGoal && viewer.jumpCameraToGoal();
    renderStatus();
  };
  seek.addEventListener("input", () => {
    const dur = viewer.duration || 0;
    if (dur > 0) {
      viewer.currentTime = (Number(seek.value) / 1000) * dur;
      renderStatus();
    }
  });

  window.__rigforgePreview = {
    play: function () { viewer.play && viewer.play(); },
    pause: function () { viewer.pause && viewer.pause(); },
    restart: function () {
      viewer.currentTime = 0;
      viewer.play && viewer.play();
    },
    seek: function (t) { viewer.currentTime = t; },
    orbit: function () { document.getElementById("btn-orbit").click(); },
    zoom: function () { document.getElementById("btn-zoom").click(); },
    reset: function () { document.getElementById("btn-reset-cam").click(); },
    refresh: function () { renderStatus(); },
    camera: cameraObs,
    state: function () {
      return {
        selected_product_kind: session && session.selected_product_kind,
        selected_product_version_id: session && session.selected_product_version_id,
        preview_artifact_id: session && session.preview_artifact_id,
        preview_status: lastError ? "ERROR" : loaded ? "LOADED" : "IDLE",
        error: lastError,
        valid: !lastError && loaded,
        loaded: !!viewer.loaded,
        modelIsVisible: !!viewer.modelIsVisible,
        available_animations: viewer.availableAnimations || [],
        animation_name: viewer.animationName || null,
        duration: viewer.duration || 0,
        currentTime: viewer.currentTime || 0,
        paused: viewer.paused,
        camera: cameraObs(),
        session_valid: !!(session && session.valid),
      };
    },
  };

  const ready = window.customElements && customElements.whenDefined
    ? customElements.whenDefined("model-viewer")
    : Promise.resolve();
  ready.then(function () {
    return loadSession();
  }).catch(function (err) {
    fail(String(err && err.message ? err.message : err));
  });
})();
