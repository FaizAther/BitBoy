# Documentation (external hub)

Hosts the **static HTML site** and **reference PDFs**. SoftGBC work is **checklist-driven**.

| Path | Role |
|---|---|
| Item pattern | What → Structure → Min test → Done when — in **softgbc-hands-off** Cursor skill |
| SoftGBC checklists | [`../SoftGBC/README.md`](../SoftGBC/README.md), [`dmg-ppu`](../SoftGBC/dmg-ppu/README.md), [`interrupt`](../SoftGBC/interrupt/README.md), … |
| [`datasheets/`](datasheets/) | PDFs (`gbctr`, `TCAGBD`, Sharp 1996) |
| [`REFERENCES.md`](REFERENCES.md) | How SoftGBC uses each source |
| [`vendor/`](vendor/) | GBEDG, Pan Docs |
| [`serve.sh`](serve.sh) | MD → HTML + local server |
| [`site/`](site/) | **Generated** — do not edit by hand |

## Build and run

```bash
cd docs
./serve.sh                 # http://localhost:8000/  (watch + live reload)
./serve.sh --no-watch
./serve.sh --build-only
```

Sidebar **Lab** section → Environment + Visual tests.

- http://localhost:8000/softgbc/environment.html  
- http://localhost:8000/softgbc/visual-tests.html  

`./serve.sh` watches docs assets / Markdown / `softgbc-lab` sources, rebuilds, and the browser refreshes via `live_reload.js`.

**Rule:** edit Markdown / checklists under SoftGBC; rebuild HTML. Never sync HTML back into MD.
