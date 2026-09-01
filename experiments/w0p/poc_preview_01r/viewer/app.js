/* RESEARCH ONLY / POC-PREVIEW-01R. Not product frontend. */
(function () {
  const cardsEl = document.getElementById("cards");
  const statusEl = document.getElementById("status-pre");
  const viewer = document.getElementById("viewer");
  const blockEl = document.getElementById("preview-block");
  const seek = document.getElementById("seek");

  let catalog = null;
  let selected = null;
  let lastError = null;
  let loadedKind = null;
  let blobUrl = null;

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

  function validateBinding(entry, manifest) {
    if (!manifest || !entry) {
      return { ok: false, reason: "MISSING_BINDING" };
    }
    if (manifest.source_product_kind !== entry.product_kind) {
      return { ok: false, reason: "BINDING_MISMATCH", field: "source_product_kind" };
    }
    if (manifest.source_product_id !== entry.product_id) {
      return { ok: false, reason: "BINDING_MISMATCH", field: "source_product_id" };
    }
    if (manifest.source_product_version !== entry.product_version) {
      return { ok: false, reason: "BINDING_MISMATCH", field: "source_product_version" };
    }
    const expected = entry.expected_source_binding || {};
    const got = manifest.source_binding || {};
    const keys = Object.keys(expected);
    if (!keys.length) {
      return { ok: false, reason: "MISSING_BINDING", field: "expected_source_binding" };
    }
    for (let i = 0; i < keys.length; i++) {
      const k = keys[i];
      if (got[k] !== expected[k]) {
        return { ok: false, reason: "BINDING_MISMATCH", field: k };
      }
    }
    return { ok: true, reason: "BINDING_OK" };
  }

  function payloadUrl(manifest) {
    const id = (manifest.payload_ref || {}).id;
    if (!id) return null;
    return "/payloads/" + id;
  }

  function renderStatus(extra) {
    const mv = viewer;
    const failed = !!lastError;
    const text = {
      selected_product_id: selected && selected.product_id,
      selected_product_kind: selected && selected.product_kind,
      selected_product_version: selected && selected.product_version,
      preview_artifact_id: selected && selected.preview_artifact_id,
      preview_status: lastError ? lastError : loadedKind ? "LOADED" : "IDLE",
      preview_type: extra && extra.preview_type,
      source: extra && extra.source,
      duration_s: failed ? null : mv.duration || null,
      animation: failed ? null : mv.animationName || null,
      available_animations: failed ? [] : mv.availableAnimations || [],
      currentTime: failed ? null : typeof mv.currentTime === "number" ? mv.currentTime : null,
      paused: failed ? null : mv.paused,
      generator: extra && extra.generator,
      warnings: extra && extra.warnings,
      payload_is_not_product_identity: true,
      viewer_library: "RESEARCH HARNESS ONLY / NOT SELECTED",
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
    loadedKind = null;
    if (blobUrl) {
      URL.revokeObjectURL(blobUrl);
      blobUrl = null;
    }
    showBlock("Preview unavailable");
  }

  function fail(reason, extra) {
    lastError = reason;
    loadedKind = null;
    showBlock("Preview unavailable / stale\nRegeneration required");
    renderStatus(extra || { preview_type: "UNAVAILABLE", source: selected && selected.product_id, warnings: [reason] });
  }

  async function selectEntry(entry) {
    selected = entry;
    lastError = null;
    loadedKind = null;
    document.querySelectorAll(".card").forEach((el) => {
      el.classList.toggle("selected", el.dataset.artifactId === entry.preview_artifact_id);
    });
    clearViewer();
    const resp = await fetch("/artifacts/" + entry.preview_artifact_id + ".json");
    if (!resp.ok) {
      fail("MANIFEST_UNAVAILABLE");
      return;
    }
    const manifest = await resp.json();
    const check = validateBinding(entry, manifest);
    if (!check.ok) {
      fail(check.reason + " — Preview unavailable / stale. Regeneration required. field=" + (check.field || ""));
      return;
    }
    const url = payloadUrl(manifest);
    if (!url) {
      fail("PAYLOAD_REF_MISSING");
      return;
    }
    const payloadResp = await fetch(url);
    if (!payloadResp.ok) {
      fail("PAYLOAD_UNAVAILABLE");
      return;
    }
    const buf = await payloadResp.arrayBuffer();
    const size = buf.byteLength;
    const expectedSize = Number(manifest.payload_size);
    if (!(expectedSize > 0) || size !== expectedSize) {
      fail("PAYLOAD_SIZE_MISMATCH — Preview unavailable. Regeneration required.");
      return;
    }
    const digest = await hexSha256(buf);
    if (digest !== String(manifest.payload_sha256 || "").toLowerCase()) {
      fail("PAYLOAD_HASH_MISMATCH — Preview unavailable. Regeneration required.");
      return;
    }
    blobUrl = URL.createObjectURL(new Blob([buf], { type: "model/gltf-binary" }));
    viewer.removeAttribute("camera-target");
    viewer.removeAttribute("camera-orbit");
    if (manifest.default_animation) {
      viewer.setAttribute("animation-name", manifest.default_animation);
    } else {
      viewer.removeAttribute("animation-name");
    }
    viewer.autoplay = false;
    const onLoad = () => {
      loadedKind = manifest.source_product_kind;
      if (manifest.default_animation && (viewer.availableAnimations || []).includes(manifest.default_animation)) {
        viewer.animationName = manifest.default_animation;
      } else if ((viewer.availableAnimations || []).length) {
        viewer.animationName = viewer.availableAnimations[0];
      }
      viewer.pause && viewer.pause();
      hideBlock();
      renderStatus({
        preview_type: manifest.source_product_kind,
        source: manifest.source_product_id,
        generator: (manifest.generator_backend_version || "") + " " + (manifest.generator_build || ""),
        warnings: manifest.declared_losses,
      });
    };
    viewer.addEventListener("load", onLoad, { once: true });
    viewer.addEventListener("error", () => {
      fail("PAYLOAD_LOAD_ERROR");
    }, { once: true });
    viewer.setAttribute("src", blobUrl);
  }

  function addCard(entry, fixture) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "card" + (fixture ? " fixture" : "");
    btn.dataset.artifactId = entry.preview_artifact_id;
    btn.dataset.kind = entry.product_kind;
    btn.dataset.productId = entry.product_id;
    if (fixture) btn.dataset.fixtureId = entry.id;
    btn.textContent =
      "[" + entry.product_kind + "] " + (entry.label || entry.product_id) +
      (fixture ? "\n(fixture — must fail)" : "");
    btn.addEventListener("click", () => selectEntry(entry));
    cardsEl.appendChild(btn);
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
    renderStatus();
  };
  seek.addEventListener("input", () => {
    const dur = viewer.duration || 0;
    if (dur > 0) {
      viewer.currentTime = (Number(seek.value) / 1000) * dur;
      renderStatus();
    }
  });

  function cameraObs() {
    const orbit = viewer.getCameraOrbit && viewer.getCameraOrbit();
    const target = viewer.getCameraTarget && viewer.getCameraTarget();
    return {
      cameraOrbit: orbit && orbit.toString ? orbit.toString() : String(viewer.cameraOrbit || ""),
      fieldOfView: viewer.getFieldOfView ? viewer.getFieldOfView() : viewer.fieldOfView || null,
      cameraTarget: target && target.toString ? target.toString() : String(viewer.cameraTarget || ""),
    };
  }

  window.__pocPreview = {
    selectKind: function (kind) {
      const asset = catalog.assets.find((a) => a.product_kind === kind);
      return selectEntry(asset);
    },
    selectFixture: function (id) {
      const fx = (catalog.fixtures || []).find((f) => f.id === id) || catalog.fixtures[0];
      return selectEntry(fx);
    },
    play: function () {
      viewer.play && viewer.play();
    },
    pause: function () {
      viewer.pause && viewer.pause();
    },
    restart: function () {
      viewer.currentTime = 0;
      viewer.play && viewer.play();
    },
    seek: function (t) {
      viewer.currentTime = t;
    },
    refresh: function () {
      renderStatus();
    },
    camera: cameraObs,
    state: function () {
      return {
        selected_product_id: selected && selected.product_id,
        selected_product_kind: selected && selected.product_kind,
        selected_product_version: selected && selected.product_version,
        preview_artifact_id: selected && selected.preview_artifact_id,
        preview_status: lastError ? "ERROR" : loadedKind ? "LOADED" : "IDLE",
        error: lastError,
        loaded: !!viewer.loaded,
        available_animations: viewer.availableAnimations || [],
        animation_name: viewer.animationName || null,
        duration: viewer.duration || 0,
        currentTime: viewer.currentTime || 0,
        paused: viewer.paused,
        camera: cameraObs(),
      };
    },
  };

  fetch("/catalog.json")
    .then((r) => r.json())
    .then((cat) => {
      catalog = cat;
      cat.assets.forEach((a) => addCard(a, false));
      (cat.fixtures || []).forEach((f) => addCard(f, true));
      statusEl.textContent = "Catalog loaded. Click an asset card.";
    });
})();
