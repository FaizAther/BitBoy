#!/usr/bin/env python3
"""Generate static HTML from Markdown only.

Sources:
  - SoftGBC crate READMEs + SoftGBC/*/docs/*.md
  - docs/datasheets, docs/REFERENCES.md
  - docs/vendor/gbedg (and pandocs when listed)

Edit Markdown; never hand-edit docs/site/**/*.html.
"""

from __future__ import annotations

import re
import shutil
from pathlib import Path

import markdown

DOCS = Path(__file__).resolve().parent
REPO = DOCS.parent
SOFTGBC = REPO / "SoftGBC"
SITE = DOCS / "site"
VENDOR_GBEDG = DOCS / "vendor" / "gbedg"
VENDOR_PANDOCS = DOCS / "vendor" / "pandocs"

# (nav label, source path, site-relative html)
LEARN_AND_CRATE: list[tuple[str, Path, str]] = [
    ("Overview / checklist", SOFTGBC / "README.md", "softgbc/overview.html"),
    ("T / M cycles", SOFTGBC / "soc" / "docs" / "t-and-m-cycles.md", "softgbc/t-and-m-cycles.html"),
    ("SoC / machine", SOFTGBC / "soc" / "README.md", "softgbc/soc.html"),
    ("DMG PPU checklist", SOFTGBC / "dmg-ppu" / "README.md", "softgbc/dmg-ppu.html"),
    ("LY fixture", SOFTGBC / "dmg-ppu" / "fixtures" / "01_ly_only.expected.txt", "softgbc/ly-fixture.html"),
    ("SM83 ISS", SOFTGBC / "sm83-iss" / "README.md", "softgbc/sm83-iss.html"),
    ("SM83 fixtures", SOFTGBC / "sm83-iss" / "fixtures" / "README.md", "softgbc/sm83-fixtures.html"),
    ("Bus", SOFTGBC / "softgbc-core" / "src" / "Bus.README.md", "softgbc/bus.html"),
    ("Memory", SOFTGBC / "softgbc-core" / "src" / "Memory.README.md", "softgbc/memory.html"),
    ("Peripherals", SOFTGBC / "peripherals" / "README.md", "softgbc/peripherals.html"),
    ("Timer checklist", SOFTGBC / "peripherals" / "src" / "Timer.README.md", "softgbc/timer.html"),
    ("Serial checklist", SOFTGBC / "peripherals" / "src" / "Serial.README.md", "softgbc/serial.html"),
    ("DMA", SOFTGBC / "peripherals" / "src" / "Dma.README.md", "softgbc/dma.html"),
    ("Joypad", SOFTGBC / "peripherals" / "src" / "Joypad.README.md", "softgbc/joypad.html"),
    ("Interrupts", SOFTGBC / "interrupt" / "README.md", "softgbc/interrupt.html"),
    ("Cartridge", SOFTGBC / "cartridge" / "README.md", "softgbc/cartridge.html"),
    ("Boot", SOFTGBC / "boot" / "README.md", "softgbc/boot.html"),
    ("APU", SOFTGBC / "apu" / "README.md", "softgbc/apu.html"),
    ("Test ROMs", SOFTGBC / "tests" / "roms" / "README.md", "softgbc/test-roms.html"),
    ("References", DOCS / "REFERENCES.md", "references.html"),
    ("Datasheets note", DOCS / "datasheets" / "README.md", "datasheets/readme.html"),
]

GBEDG_PAGES: list[tuple[str, Path, str]] = [
    ("GBEDG home", VENDOR_GBEDG / "README.md", "gbedg/index.html"),
    ("Concepts", VENDOR_GBEDG / "concepts" / "index.md", "gbedg/concepts.html"),
    ("PPU", VENDOR_GBEDG / "ppu" / "index.md", "gbedg/ppu.html"),
    ("Timers", VENDOR_GBEDG / "timers" / "index.md", "gbedg/timers.html"),
    ("DMA", VENDOR_GBEDG / "dma" / "index.md", "gbedg/dma.html"),
    ("MBCs", VENDOR_GBEDG / "mbcs" / "index.md", "gbedg/mbcs.html"),
    ("MBC1", VENDOR_GBEDG / "mbcs" / "mbc1" / "index.md", "gbedg/mbc1.html"),
    ("MBC2", VENDOR_GBEDG / "mbcs" / "mbc2" / "index.md", "gbedg/mbc2.html"),
    ("MBC3", VENDOR_GBEDG / "mbcs" / "mbc3" / "index.md", "gbedg/mbc3.html"),
    ("MBC5", VENDOR_GBEDG / "mbcs" / "mbc5" / "index.md", "gbedg/mbc5.html"),
    ("Common bugs", VENDOR_GBEDG / "bugs" / "index.md", "gbedg/bugs.html"),
]

# High-value Pan Docs chapters (CC0) — optional if vendor present
PANDOCS_PAGES: list[tuple[str, Path, str]] = [
    ("About", VENDOR_PANDOCS / "src" / "About.md", "pandocs/about.html"),
    ("Memory map", VENDOR_PANDOCS / "src" / "Memory_Map.md", "pandocs/memory-map.html"),
    ("Graphics", VENDOR_PANDOCS / "src" / "Graphics.md", "pandocs/graphics.html"),
    ("LCDC", VENDOR_PANDOCS / "src" / "LCDC.md", "pandocs/lcdc.html"),
    ("Rendering", VENDOR_PANDOCS / "src" / "Rendering.md", "pandocs/rendering.html"),
    ("Interrupts", VENDOR_PANDOCS / "src" / "Interrupts.md", "pandocs/interrupts.html"),
    ("Timer", VENDOR_PANDOCS / "src" / "Timer_and_Divider_Registers.md", "pandocs/timer.html"),
    ("Serial", VENDOR_PANDOCS / "src" / "Serial_Data_Transfer_(Link_Cable).md", "pandocs/serial.html"),
    ("Specifications", VENDOR_PANDOCS / "src" / "Specifications.md", "pandocs/specifications.html"),
]

MD = markdown.Markdown(
    extensions=["tables", "fenced_code", "toc", "sane_lists", "smarty"],
    extension_configs={"toc": {"permalink": False}},
)

STYLE = (DOCS / "assets" / "style.css")


def rewrite_md_links(text: str, is_gbedg: bool) -> str:
    def repl_link(m: re.Match[str]) -> str:
        label, url = m.group(1), m.group(2)
        if url.startswith(("http://", "https://", "#", "mailto:")):
            return m.group(0)
        path_part, frag = url, ""
        if "#" in url:
            path_part, frag = url.split("#", 1)
            frag = "#" + frag
        if not path_part:
            return m.group(0)
        if path_part.endswith(".md") and is_gbedg:
            stem = Path(path_part).stem
            if stem == "index":
                parent = Path(path_part).parent.name
                if parent and parent not in (".", ""):
                    return f"[{label}]({parent}.html{frag})"
                return f"[{label}](index.html{frag})"
            return f"[{label}]({stem}.html{frag})"
        return m.group(0)

    return re.sub(r"\[([^\]]+)\]\(([^)]+)\)", repl_link, text)


def render_body(source: Path, is_gbedg: bool = False) -> str:
    raw = source.read_text(encoding="utf-8")
    if source.suffix == ".txt":
        escaped = raw.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
        return f'<pre class="fixture">{escaped}</pre>'
    raw = rewrite_md_links(raw, is_gbedg)
    MD.reset()
    return MD.convert(raw)


def nav_block(root: str) -> str:
    soft = "\n".join(
        f'        <li><a href="{root}/{html}">{label}</a></li>'
        for label, src, html in LEARN_AND_CRATE
        if src.exists()
    )
    gbedg = "\n".join(
        f'        <li><a href="{root}/{html}">{label}</a></li>'
        for label, src, html in GBEDG_PAGES
        if src.exists()
    )
    pandocs = "\n".join(
        f'        <li><a href="{root}/{html}">{label}</a></li>'
        for label, src, html in PANDOCS_PAGES
        if src.exists()
    )
    pan_section = ""
    if any(src.exists() for _, src, _ in PANDOCS_PAGES):
        pan_section = f"""
      <p class="nav-section">Pan Docs <span class="muted">(CC0)</span></p>
      <ul>
{pandocs}
      </ul>"""
    return f"""
      <p class="nav-section">SoftGBC</p>
      <ul>
{soft}
      </ul>
      <p class="nav-section">GBEDG <span class="muted">(MIT)</span></p>
      <ul>
{gbedg}
      </ul>{pan_section}
      <p class="nav-section">Other</p>
      <ul>
        <li><a href="{root}/datasheets/index.html">PDF datasheets</a></li>
        <li><a href="{root}/attribution.html">Attribution</a></li>
      </ul>
"""


def page_shell(title: str, body: str, page_html: str, crumb: str) -> str:
    depth = page_html.count("/")
    root = "/".join([".."] * depth) if depth else "."
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title} — SoftGBC docs</title>
  <link rel="stylesheet" href="{root}/assets/style.css">
</head>
<body>
  <div class="layout">
    <nav class="sidebar">
      <a class="brand" href="{root}/index.html">SoftGBC docs</a>
{nav_block(root)}
    </nav>
    <main>
      <p class="crumb">{crumb}</p>
      <article class="content">
{body}
      </article>
    </main>
  </div>
</body>
</html>
"""


def write_page(html_rel: str, title: str, body: str, crumb: str) -> None:
    out = SITE / html_rel
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(page_shell(title, body, html_rel, crumb), encoding="utf-8")


DEFAULT_CSS = """
:root {
  --bg: #f7f4ef; --panel: #fffdf8; --ink: #1c1a16; --muted: #5c574e;
  --line: #d9d2c4; --accent: #0b5f4b; --code-bg: #efe8db;
}
* { box-sizing: border-box; }
html, body { margin: 0; padding: 0; }
body {
  font-family: "Iowan Old Style", "Palatino Linotype", Palatino, Georgia, serif;
  color: var(--ink);
  background: linear-gradient(160deg, #efe7d8 0%, var(--bg) 40%, #e7eef0 100%);
  min-height: 100vh;
}
.layout { display: grid; grid-template-columns: minmax(220px, 280px) 1fr; min-height: 100vh; }
.sidebar {
  background: var(--panel); border-right: 1px solid var(--line);
  padding: 1.25rem 1rem 2rem; position: sticky; top: 0; height: 100vh; overflow: auto;
}
.brand { display: block; font-weight: 700; font-size: 1.15rem; color: var(--accent); text-decoration: none; margin-bottom: 1rem; }
.nav-section { margin: 1.1rem 0 0.35rem; font-size: 0.75rem; letter-spacing: 0.06em; text-transform: uppercase; color: var(--muted); }
.sidebar ul { list-style: none; margin: 0; padding: 0; }
.sidebar li { margin: 0.15rem 0; }
.sidebar a { color: var(--ink); text-decoration: none; font-size: 0.95rem; }
.sidebar a:hover { color: var(--accent); text-decoration: underline; }
.muted { text-transform: none; letter-spacing: 0; font-size: 0.85em; }
main { padding: 1.5rem 2rem 3rem; max-width: 52rem; }
.crumb { color: var(--muted); font-size: 0.9rem; margin-top: 0; }
.content {
  background: color-mix(in srgb, var(--panel) 88%, transparent);
  border: 1px solid var(--line); border-radius: 10px; padding: 1.25rem 1.5rem 2rem;
}
.content h1 { margin-top: 0; }
.content a { color: var(--accent); }
.content img { max-width: 100%; height: auto; background: #fff; border: 1px solid var(--line); border-radius: 6px; padding: 0.35rem; }
.content pre, .fixture {
  background: var(--code-bg); border: 1px solid var(--line); border-radius: 6px;
  padding: 0.85rem 1rem; overflow-x: auto;
  font-family: "IBM Plex Mono", "Consolas", monospace; font-size: 0.88rem;
}
.content code { font-family: "IBM Plex Mono", "Consolas", monospace; font-size: 0.9em; background: var(--code-bg); padding: 0.1em 0.3em; border-radius: 3px; }
.content pre code { background: none; padding: 0; }
.content table { border-collapse: collapse; width: 100%; margin: 1rem 0; font-size: 0.95rem; }
.content th, .content td { border: 1px solid var(--line); padding: 0.4rem 0.55rem; text-align: left; vertical-align: top; }
.content th { background: #efe8db; }
.cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 0.85rem; margin: 1.25rem 0; }
.card {
  display: block; padding: 1rem; border: 1px solid var(--line); border-radius: 8px;
  background: var(--panel); text-decoration: none; color: inherit;
}
.card:hover { border-color: var(--accent); }
.card strong { display: block; color: var(--accent); margin-bottom: 0.35rem; }
.card span { color: var(--muted); font-size: 0.92rem; }
@media (max-width: 860px) {
  .layout { grid-template-columns: 1fr; }
  .sidebar { position: relative; height: auto; border-right: none; border-bottom: 1px solid var(--line); }
  main { padding: 1rem; }
}
"""


def fix_gbedg_images(html: str, section: str) -> str:
    asset_dir = section.replace("/", "_")

    def repl(m: re.Match[str]) -> str:
        src = m.group(1)
        if src.startswith(("http://", "https://", "data:", "../", "gbedg/")):
            return m.group(0)
        return f'src="{asset_dir}/{Path(src).name}"'

    return re.sub(r'src="([^"]+)"', repl, html)


def build_index() -> None:
    body = """
<h1>SoftGBC documentation</h1>
<p><strong>Markdown is the source of truth.</strong> SoftGBC explainers live next to their crates;
this site (<code>docs/site/</code>) is generated. Datasheets and third-party guides live under repo <code>docs/</code>.</p>
<pre><code>cd docs &amp;&amp; ./serve.sh
# → http://127.0.0.1:8000/</code></pre>
<div class="cards">
  <a class="card" href="softgbc/overview.html"><strong>Roadmap checklist</strong><span>SoftGBC/README — driver + min tests</span></a>
  <a class="card" href="softgbc/dmg-ppu.html"><strong>PPU checklist</strong><span>LY → VBlank → SoftGBC wire → BG</span></a>
  <a class="card" href="softgbc/timer.html"><strong>Timer checklist</strong><span>Overflow IRQ + accuracy follow-ups</span></a>
  <a class="card" href="softgbc/serial.html"><strong>Serial checklist</strong><span>When IF bit 3 fires</span></a>
  <a class="card" href="gbedg/ppu.html"><strong>GBEDG PPU</strong><span>Hardware deep dive</span></a>
  <a class="card" href="datasheets/index.html"><strong>PDF datasheets</strong><span>gbctr, TCAGBD, Sharp 1996</span></a>
  <a class="card" href="references.html"><strong>References</strong><span>How SoftGBC uses each source</span></a>
</div>
"""
    write_page("index.html", "Home", body, "Home")


def build_attribution() -> None:
    parts = ["<h1>Attribution</h1>"]
    lic = VENDOR_GBEDG / "LICENSE"
    if lic.exists():
        text = lic.read_text(encoding="utf-8").replace("&", "&amp;").replace("<", "&lt;")
        parts.append(
            "<h2>GBEDG</h2><p>Vendored from "
            '<a href="https://github.com/Ashiepaws/GBEDG">Ashiepaws/GBEDG</a> (MIT).</p>'
            f"<pre>{text}</pre>"
        )
    parts.append(
        "<h2>Pan Docs</h2><p>Optional vendored copy from "
        '<a href="https://github.com/gbdev/pandocs">gbdev/pandocs</a> (CC0).</p>'
    )
    parts.append(
        "<h2>SoftGBC</h2><p>Project Markdown under <code>SoftGBC/</code> crates; site builder under <code>docs/</code>.</p>"
    )
    write_page("attribution.html", "Attribution", "\n".join(parts), "Attribution")


def build_datasheets_index() -> None:
    ds = DOCS / "datasheets"
    dest = SITE / "datasheets"
    dest.mkdir(parents=True, exist_ok=True)
    links = []
    for f in sorted(ds.iterdir()):
        if f.is_file() and f.suffix.lower() in {".pdf", ".md", ".txt"}:
            shutil.copy2(f, dest / f.name)
            links.append(f'<li><a href="{f.name}">{f.name}</a></li>')
    body = (
        "<h1>Datasheets</h1>"
        "<p>From repo <code>docs/datasheets/</code>. See also "
        '<a href="readme.html">datasheets README</a>.</p><ul>\n'
        + "\n".join(links)
        + "\n</ul>"
    )
    write_page("datasheets/index.html", "Datasheets", body, "Datasheets")


def copy_gbedg_images() -> None:
    for folder in ("ppu", "dma", "bugs", "timers", "concepts", "mbcs", "mbcs/mbc1", "mbcs/mbc2", "mbcs/mbc3", "mbcs/mbc5"):
        src_dir = VENDOR_GBEDG / folder
        if not src_dir.is_dir():
            continue
        dest = SITE / "gbedg" / folder.replace("/", "_")
        dest.mkdir(parents=True, exist_ok=True)
        for img in src_dir.iterdir():
            if img.suffix.lower() in {".png", ".gif", ".jpg", ".jpeg", ".svg", ".webp"}:
                shutil.copy2(img, dest / img.name)


def main() -> None:
    if SITE.exists():
        shutil.rmtree(SITE)
    SITE.mkdir(parents=True)
    assets = SITE / "assets"
    assets.mkdir()
    css_src = DOCS / "assets" / "style.css"
    if not css_src.exists():
        css_src.parent.mkdir(parents=True, exist_ok=True)
        css_src.write_text(DEFAULT_CSS, encoding="utf-8")
    shutil.copy2(css_src, assets / "style.css")

    build_index()
    build_attribution()
    build_datasheets_index()

    for label, src, html_rel in LEARN_AND_CRATE:
        if not src.exists():
            print(f"skip {src}")
            continue
        write_page(html_rel, label, render_body(src), f"SoftGBC / {label}")
        print(f"wrote {html_rel}")

    section_for = {
        "gbedg/ppu.html": "ppu",
        "gbedg/dma.html": "dma",
        "gbedg/bugs.html": "bugs",
        "gbedg/mbc5.html": "mbcs/mbc5",
        "gbedg/mbc1.html": "mbcs/mbc1",
        "gbedg/mbc2.html": "mbcs/mbc2",
        "gbedg/mbc3.html": "mbcs/mbc3",
        "gbedg/mbcs.html": "mbcs",
        "gbedg/timers.html": "timers",
        "gbedg/concepts.html": "concepts",
    }
    for label, src, html_rel in GBEDG_PAGES:
        if not src.exists():
            print(f"skip {src}")
            continue
        body = render_body(src, is_gbedg=True)
        section = section_for.get(html_rel, "")
        if section:
            body = fix_gbedg_images(body, section)
        write_page(html_rel, f"GBEDG — {label}", body, f"GBEDG / {label}")
        print(f"wrote {html_rel}")

    for label, src, html_rel in PANDOCS_PAGES:
        if not src.exists():
            continue
        write_page(html_rel, f"Pan Docs — {label}", render_body(src), f"Pan Docs / {label}")
        print(f"wrote {html_rel}")

    copy_gbedg_images()
    print(f"\nDone → {SITE}")
    print("Serve:  cd docs && ./serve.sh")


if __name__ == "__main__":
    main()
