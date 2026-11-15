(() => {
  const root = document.getElementById("visual-lab");
  if (!root) return;

  const listEl = document.getElementById("lab-list");
  const statusEl = document.getElementById("lab-status");

  /** 1×1 transparent PNG — avoids broken-image icons before a run. */
  const PLACEHOLDER =
    "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

  function setStatus(msg, kind) {
    statusEl.textContent = msg || "";
    statusEl.dataset.kind = kind || "";
  }

  function screenBox(label) {
    const wrap = document.createElement("div");
    wrap.className = "lab-screen";
    const cap = document.createElement("div");
    cap.className = "lab-screen-cap";
    cap.textContent = label;
    const img = document.createElement("img");
    img.width = 160;
    img.height = 144;
    img.alt = label;
    img.className = "lab-lcd lab-lcd--empty";
    img.src = PLACEHOLDER;
    wrap.append(cap, img);
    return { wrap, img };
  }

  function cardFor(t) {
    const card = document.createElement("article");
    card.className = "lab-card";
    card.dataset.id = t.id;
    card.dataset.state = "idle";

    const head = document.createElement("header");
    const h = document.createElement("h2");
    h.textContent = t.title;
    const desc = document.createElement("p");
    desc.className = "lab-desc";
    desc.textContent = t.description;
    head.append(h, desc);

    const expected = document.createElement("div");
    expected.className = "lab-expected";
    expected.innerHTML =
      "<strong>Expected outcome</strong>" +
      `<div class="lab-crc">CRC <code>${t.expected_crc}</code></div>` +
      `<p class="lab-notes"></p>` +
      `<pre class="lab-fixture muted">Loading fixture…</pre>`;
    expected.querySelector(".lab-notes").textContent = t.expected_notes || "";

    const actions = document.createElement("div");
    actions.className = "lab-actions";
    const play = document.createElement("button");
    play.type = "button";
    play.className = "lab-play";
    play.textContent = "Play";
    const resultBadge = document.createElement("span");
    resultBadge.className = "lab-badge";
    resultBadge.hidden = true;
    actions.append(play, resultBadge);

    const screens = document.createElement("div");
    screens.className = "lab-screens";
    const actual = screenBox("Actual");
    const ref = screenBox("Reference scene");
    screens.append(actual.wrap, ref.wrap);

    const meta = document.createElement("p");
    meta.className = "lab-meta muted";
    meta.textContent = "Runs when this card is on screen (or press Play).";

    card.append(head, expected, actions, screens, meta);

    if (t.fixture) {
      fetch("/api/fixtures/" + encodeURIComponent(t.fixture))
        .then((r) => (r.ok ? r.text() : Promise.reject()))
        .then((text) => {
          expected.querySelector(".lab-fixture").textContent = text.trim();
          expected.querySelector(".lab-fixture").classList.remove("muted");
        })
        .catch(() => {
          expected.querySelector(".lab-fixture").textContent =
            "(fixture file not available)";
        });
    } else {
      expected.querySelector(".lab-fixture").textContent =
        "(no checked-in fixture — lab baseline CRC)";
    }

    let inFlight = null;

    async function runTest(reason) {
      if (card.dataset.state === "running") return inFlight;
      if (card.dataset.state === "done" && reason === "auto") return null;

      card.dataset.state = "running";
      play.disabled = true;
      play.textContent = "Running…";
      resultBadge.hidden = true;
      meta.textContent = "Rendering…";
      setStatus("Running " + t.id + "…");

      inFlight = (async () => {
        try {
          const r = await fetch(
            "/api/visual-tests/" + encodeURIComponent(t.id) + "/run",
            { method: "POST" }
          );
          const data = await r.json();
          if (!r.ok) throw new Error(data.error || r.statusText);

          actual.img.src = "data:image/png;base64," + data.actual_png_base64;
          ref.img.src = "data:image/png;base64," + data.expected_png_base64;
          actual.img.classList.remove("lab-lcd--empty");
          ref.img.classList.remove("lab-lcd--empty");
          meta.textContent =
            "actual " +
            data.actual_crc +
            " · expected " +
            data.expected_crc +
            (data.fixture ? " · " + data.fixture : "");

          resultBadge.hidden = false;
          resultBadge.textContent = data.passed ? "PASS" : "FAIL";
          resultBadge.dataset.pass = data.passed ? "1" : "0";
          card.dataset.state = "done";
          setStatus(
            data.passed
              ? t.title + " passed"
              : t.title + " failed (CRC mismatch)",
            data.passed ? "ok" : "err"
          );
        } catch (e) {
          card.dataset.state = "error";
          setStatus(String(e.message || e), "err");
          resultBadge.hidden = false;
          resultBadge.textContent = "ERROR";
          resultBadge.dataset.pass = "0";
          meta.textContent = "Run failed — press Play to retry.";
        } finally {
          play.disabled = false;
          play.textContent = "Play";
          inFlight = null;
        }
      })();

      return inFlight;
    }

    play.addEventListener("click", () => {
      card.dataset.state = "idle"; // allow re-run
      runTest("manual");
    });

    card._runTest = runTest;
    return card;
  }

  setStatus("Loading tests…");
  fetch("/api/visual-tests")
    .then((r) => r.json().then((j) => ({ ok: r.ok, j })))
    .then(({ ok, j }) => {
      if (!ok) throw new Error(j.error || "failed to list tests");
      listEl.replaceChildren();
      const cards = j.map((t) => cardFor(t));
      cards.forEach((c) => listEl.appendChild(c));
      setStatus(
        j.length + " visual tests — auto-run as each card comes into view."
      );

      const io = new IntersectionObserver(
        (entries) => {
          entries.forEach((entry) => {
            if (!entry.isIntersecting) return;
            const card = entry.target;
            if (typeof card._runTest === "function") {
              card._runTest("auto");
            }
          });
        },
        { root: null, rootMargin: "80px 0px", threshold: 0.2 }
      );
      cards.forEach((c) => io.observe(c));
    })
    .catch((e) => {
      setStatus(
        "Could not load visual tests. Is ./serve.sh running with the lab binary? " +
          (e.message || e),
        "err"
      );
    });
})();
