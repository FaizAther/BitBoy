(() => {
  let id = null;
  let reloading = false;
  async function poll() {
    if (reloading) return;
    try {
      const r = await fetch("/api/dev/build?t=" + Date.now(), {
        cache: "no-store",
      });
      if (r.ok) {
        const j = await r.json();
        if (id === null) id = j.id;
        else if (j.id !== id && !j.busy) {
          reloading = true;
          location.reload();
          return;
        }
      }
    } catch (_) {
      /* server restarting */
    }
    setTimeout(poll, 900);
  }
  poll();
})();
