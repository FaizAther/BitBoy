/* SoftGBC Environment — shareable session play via /api/play/* */
(function () {
  const W = 160;
  const H = 144;

  const BTN = {
    RIGHT: 1 << 0,
    LEFT: 1 << 1,
    UP: 1 << 2,
    DOWN: 1 << 3,
    A: 1 << 4,
    B: 1 << 5,
    SELECT: 1 << 6,
    START: 1 << 7,
  };

  const KEY_MAP = {
    ArrowRight: BTN.RIGHT,
    ArrowLeft: BTN.LEFT,
    ArrowUp: BTN.UP,
    ArrowDown: BTN.DOWN,
    KeyZ: BTN.A,
    KeyX: BTN.B,
    KeyA: BTN.A,
    KeyS: BTN.B,
    Enter: BTN.START,
    Space: BTN.START,
    ShiftRight: BTN.SELECT,
    ShiftLeft: BTN.SELECT,
  };

  const SCROLL_BLOCK = {
    Space: 1,
    ArrowUp: 1,
    ArrowDown: 1,
    ArrowLeft: 1,
    ArrowRight: 1,
    PageUp: 1,
    PageDown: 1,
    Home: 1,
    End: 1,
  };

  const lcd = document.getElementById("play-lcd");
  const statusEl = document.getElementById("play-status");
  const romSelect = document.getElementById("play-rom-select");
  const fileInput = document.getElementById("play-file");
  const btnPlay = document.getElementById("play-start");
  const btnStop = document.getElementById("play-stop");
  const sessionEl = document.getElementById("play-session");
  const sessionLink = document.getElementById("play-session-link");
  const lockEl = document.getElementById("play-key-lock");
  const liveList = document.getElementById("play-live-list");
  if (!lcd || !btnPlay) return;

  const ctx = lcd.getContext("2d", { alpha: false });
  const imageData = ctx.createImageData(W, H);
  const rgba = imageData.data;

  let bits = 0;
  let playing = false;
  let keysLocked = false;
  let raf = 0;
  let fetching = false;
  let lastInputPost = 0;
  let sessionId = ensureSessionId();

  function ensureSessionId() {
    const u = new URL(window.location.href);
    let s = u.searchParams.get("s");
    if (!s || !/^[A-Za-z0-9_-]{6,64}$/.test(s)) {
      const bytes = new Uint8Array(9);
      crypto.getRandomValues(bytes);
      let bin = "";
      for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
      s = btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
      u.searchParams.set("s", s);
      history.replaceState(null, "", u.pathname + u.search + u.hash);
    }
    return s;
  }

  function sessionHeaders(extra) {
    return Object.assign({ "X-Play-Session": sessionId }, extra || {});
  }

  function setStatus(msg) {
    if (statusEl) statusEl.textContent = msg;
  }

  function updateSessionUi() {
    const url = new URL(window.location.href);
    url.searchParams.set("s", sessionId);
    const share = url.toString();
    if (sessionEl) sessionEl.textContent = sessionId;
    if (sessionLink) {
      sessionLink.href = share;
      sessionLink.textContent = share;
    }
    updateReachHint();
  }

  async function updateReachHint() {
    const hint = document.getElementById("play-reach-hint");
    if (!hint) return;
    try {
      const r = await fetch("/api/play/reach", {
        credentials: "same-origin",
        cache: "no-store",
      });
      const data = await r.json();
      const path =
        "/softgbc/environment.html?s=" + encodeURIComponent(sessionId);
      const port = data.port || 8000;
      const localA = document.createElement("a");
      localA.href = "http://127.0.0.1:" + port + path;
      localA.textContent = "localhost:" + port + path;
      const kids = [
        document.createTextNode(
          "Mac: Cursor Ports → forward " + port + ", then open "
        ),
        localA,
      ];
      if (data.lan_base) {
        const lanA = document.createElement("a");
        lanA.href = data.lan_base + path;
        lanA.textContent = data.lan_base.replace(/^https?:\/\//, "") + path;
        kids.push(document.createTextNode(". Same Wi‑Fi: "), lanA);
      }
      kids.push(document.createTextNode("."));
      hint.replaceChildren.apply(hint, kids);
    } catch (_) {}
  }

  function setKeyLock(on) {
    keysLocked = !!on;
    if (on) {
      lcd.focus({ preventScroll: true });
      lcd.classList.add("play-lcd--locked");
      if (lockEl) {
        lockEl.hidden = false;
        lockEl.textContent = "Keys locked to LCD — Esc to unlock";
      }
    } else {
      lcd.classList.remove("play-lcd--locked");
      if (lockEl) {
        lockEl.hidden = true;
        lockEl.textContent = "";
      }
      if (bits) {
        bits = 0;
        postInput(true);
      }
    }
  }

  function blitRgb(buf) {
    const rgb = new Uint8Array(buf);
    if (rgb.length < W * H * 3) return;
    let si = 0;
    for (let i = 0; i < W * H; i++) {
      const o = i * 4;
      rgba[o] = rgb[si++];
      rgba[o + 1] = rgb[si++];
      rgba[o + 2] = rgb[si++];
      rgba[o + 3] = 255;
    }
    ctx.putImageData(imageData, 0, 0);
    lcd.classList.remove("lab-lcd--empty");
  }

  function pumpFrames() {
    if (!playing) return;
    if (fetching) {
      raf = requestAnimationFrame(pumpFrames);
      return;
    }
    fetching = true;
    fetch(
      "/api/play/frame.rgb?s=" + encodeURIComponent(sessionId) + "&t=" + Date.now(),
      { headers: sessionHeaders(), credentials: "same-origin", cache: "no-store" }
    )
      .then(function (r) {
        if (r.status === 204) return null; // dropped duplicate poll
        if (!r.ok) throw new Error("frame " + r.status);
        return r.arrayBuffer();
      })
      .then(function (buf) {
        if (buf) blitRgb(buf);
      })
      .catch(function () {})
      .finally(function () {
        fetching = false;
        if (playing) raf = requestAnimationFrame(pumpFrames);
      });
  }

  function switchToSession(id) {
    if (!id || id === sessionId) return;
    sessionId = id;
    const u = new URL(window.location.href);
    u.searchParams.set("s", sessionId);
    history.replaceState(null, "", u.pathname + u.search + u.hash);
    updateSessionUi();
    attachIfLive();
  }

  async function refreshLiveList() {
    if (!liveList) return;
    try {
      const r = await fetch("/api/play/sessions", {
        credentials: "same-origin",
        cache: "no-store",
      });
      const data = await r.json();
      const sessions = data.sessions || [];
      liveList.innerHTML = "";
      if (!sessions.length) {
        const li = document.createElement("li");
        li.className = "play-live-empty";
        li.textContent = "No live sessions yet.";
        liveList.appendChild(li);
        return;
      }
      sessions.forEach(function (s) {
        const li = document.createElement("li");
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className =
          "play-live-item" + (s.id === sessionId ? " is-active" : "");
        const title = document.createElement("span");
        title.className = "play-live-title";
        title.textContent = s.title || "(no ROM)";
        const meta = document.createElement("span");
        meta.className = "play-live-meta";
        meta.textContent =
          s.id +
          " · " +
          (s.running ? "running" : "idle") +
          " · " +
          (s.frames || 0) +
          "f";
        btn.appendChild(title);
        btn.appendChild(meta);
        btn.addEventListener("click", function () {
          switchToSession(s.id);
        });
        li.appendChild(btn);
        liveList.appendChild(li);
      });
    } catch (_) {}
  }

  async function attachIfLive() {
    try {
      const r = await fetch(
        "/api/play/status?s=" + encodeURIComponent(sessionId),
        { headers: sessionHeaders(), credentials: "same-origin", cache: "no-store" }
      );
      const data = await r.json();
      if (data.loaded && data.running) {
        playing = true;
        if (btnStop) btnStop.disabled = false;
        setStatus(
          "Watching “" +
            (data.title || "ROM") +
            "” — click LCD to control, or pick another live game on the right."
        );
        cancelAnimationFrame(raf);
        raf = requestAnimationFrame(pumpFrames);
        return true;
      }
    } catch (_) {}
    return false;
  }

  async function postInput(force) {
    const now = Date.now();
    if (!force && now - lastInputPost < 16) return;
    lastInputPost = now;
    try {
      await fetch("/api/play/input?s=" + encodeURIComponent(sessionId), {
        method: "POST",
        headers: sessionHeaders({ "Content-Type": "application/json" }),
        body: JSON.stringify({ bits: bits }),
        credentials: "same-origin",
      });
    } catch (_) {}
  }

  async function loadRoms() {
    try {
      const r = await fetch("/api/play/roms", { credentials: "same-origin" });
      const data = await r.json();
      if (!romSelect) return;
      romSelect.innerHTML = "";
      const opt0 = document.createElement("option");
      opt0.value = "";
      opt0.textContent = "— choose cartridge ROM —";
      romSelect.appendChild(opt0);
      (data.roms || []).forEach(function (name) {
        const o = document.createElement("option");
        o.value = name;
        o.textContent = name;
        romSelect.appendChild(o);
      });
      const tetris = (data.roms || []).find(function (n) {
        return /tetris/i.test(n);
      });
      if (tetris) romSelect.value = tetris;
    } catch (e) {
      setStatus("Could not list ROMs (is ./serve.sh running?)");
    }
  }

  async function startPlay() {
    btnPlay.disabled = true;
    setStatus("Loading…");
    try {
      let r;
      if (fileInput && fileInput.files && fileInput.files[0]) {
        const buf = await fileInput.files[0].arrayBuffer();
        r = await fetch("/api/play/load?s=" + encodeURIComponent(sessionId), {
          method: "POST",
          headers: sessionHeaders({
            "Content-Type": "application/octet-stream",
          }),
          body: buf,
          credentials: "same-origin",
        });
      } else if (romSelect && romSelect.value) {
        r = await fetch(
          "/api/play/load/builtin?s=" + encodeURIComponent(sessionId),
          {
            method: "POST",
            headers: sessionHeaders({ "Content-Type": "application/json" }),
            body: JSON.stringify({ name: romSelect.value }),
            credentials: "same-origin",
          }
        );
      } else {
        setStatus("Pick a cartridge ROM or upload a .gb/.gbc file.");
        btnPlay.disabled = false;
        return;
      }
      const data = await r.json();
      if (!r.ok) {
        setStatus(data.error || "Load failed");
        btnPlay.disabled = false;
        return;
      }
      if (data.session) {
        sessionId = data.session;
        const u = new URL(window.location.href);
        u.searchParams.set("s", sessionId);
        history.replaceState(null, "", u.pathname + u.search + u.hash);
        updateSessionUi();
      }
      playing = true;
      bits = 0;
      setStatus(
        "Playing “" +
          (data.title || "ROM") +
          "” — click LCD to lock keys (arrows / Z X / Space=Start). Esc unlocks."
      );
      if (btnStop) btnStop.disabled = false;
      setKeyLock(true);
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(pumpFrames);
    } catch (e) {
      setStatus("Play failed: " + e);
    } finally {
      btnPlay.disabled = false;
    }
  }

  async function stopPlay() {
    playing = false;
    cancelAnimationFrame(raf);
    fetching = false;
    bits = 0;
    setKeyLock(false);
    try {
      await fetch("/api/play/stop?s=" + encodeURIComponent(sessionId), {
        method: "POST",
        headers: sessionHeaders(),
        credentials: "same-origin",
      });
    } catch (_) {}
    setStatus("Stopped.");
    if (btnStop) btnStop.disabled = true;
    lcd.classList.add("lab-lcd--empty");
  }

  function onKey(down, ev) {
    if (down && ev.code === "Escape") {
      if (keysLocked) {
        ev.preventDefault();
        setKeyLock(false);
        setStatus("Keys unlocked — click the LCD to capture again.");
      }
      return;
    }
    if (!keysLocked) return;
    if (SCROLL_BLOCK[ev.code] || KEY_MAP[ev.code]) {
      ev.preventDefault();
      ev.stopPropagation();
    }
    if (!playing) return;
    const mask = KEY_MAP[ev.code];
    if (!mask) return;
    if (down) {
      if (ev.repeat) return;
      bits |= mask;
    } else {
      bits &= ~mask;
    }
    postInput();
  }

  btnPlay.addEventListener("click", startPlay);
  if (btnStop) {
    btnStop.disabled = true;
    btnStop.addEventListener("click", stopPlay);
  }

  lcd.tabIndex = 0;
  lcd.addEventListener("click", function (e) {
    e.preventDefault();
    setKeyLock(true);
    if (playing) {
      setStatus(
        "Keys locked — arrows / Z X / Space=Start / Enter. Esc unlocks."
      );
    }
  });
  lcd.addEventListener("blur", function () {
    if (keysLocked) setKeyLock(false);
  });

  window.addEventListener("keydown", function (e) { onKey(true, e); }, true);
  window.addEventListener("keyup", function (e) { onKey(false, e); }, true);

  updateSessionUi();
  loadRoms();
  refreshLiveList();
  setInterval(refreshLiveList, 2000);
  attachIfLive().then(function (watching) {
    if (!watching) {
      setStatus(
        "Choose Tetris (or upload), Play, then click the LCD to lock keys. Live games appear on the right."
      );
    }
  });
})();
