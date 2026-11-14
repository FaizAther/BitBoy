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
./serve.sh                 # http://127.0.0.1:8000/
./serve.sh --build-only
```

**Rule:** edit Markdown / checklists under SoftGBC; rebuild HTML. Never sync HTML back into MD.
