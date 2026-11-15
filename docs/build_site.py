#!/usr/bin/env python3
"""Generate static HTML from Markdown only.

Sources:
  - SoftGBC crate READMEs + SoftGBC/*/docs/*.md
  - docs/qemu/*.md (Chunk E QEMU SM83 checklists)
  - docs/llvm/*.md (Chunks C-D compiler checklists)
  - docs/sdk/*.md (agent-first GBC game SDK checklists)
  - docs/datasheets, docs/REFERENCES.md
  - docs/vendor/gbedg (and pandocs when listed)

Edit Markdown; never hand-edit docs/site/**/*.html.
"""

from __future__ import annotations

import os
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
    # Visual lab page is generated in build_visual_lab() (not from Markdown).
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

# QEMU SM83 Chunk E (nav section separate from SoftGBC crates)
QEMU_PAGES: list[tuple[str, Path, str]] = [
    ("QEMU approach", DOCS / "qemu" / "README.md", "qemu/overview.html"),
    ("Semantics contract", DOCS / "qemu" / "semantics.md", "qemu/semantics.html"),
    ("00 Harness", DOCS / "qemu" / "00-harness.md", "qemu/00-harness.html"),
    ("01 Skeleton", DOCS / "qemu" / "01-skeleton.md", "qemu/01-skeleton.html"),
    ("02 Helper ISA", DOCS / "qemu" / "02-helper-isa.md", "qemu/02-helper-isa.html"),
    ("03 Interrupts", DOCS / "qemu" / "03-interrupts.md", "qemu/03-interrupts.html"),
    ("04 Board", DOCS / "qemu" / "04-board.md", "qemu/04-board.html"),
    ("05 Parity", DOCS / "qemu" / "05-parity.md", "qemu/05-parity.html"),
    ("06 TCG / gdb", DOCS / "qemu" / "06-tcg-debug.md", "qemu/06-tcg-debug.html"),
]

LLVM_PAGES: list[tuple[str, Path, str]] = [
    ("LLVM approach", DOCS / "llvm" / "README.md", "llvm/overview.html"),
    ("00 Contracts", DOCS / "llvm" / "00-contracts.md", "llvm/00-contracts.html"),
    ("01 MC layer", DOCS / "llvm" / "01-mc.md", "llvm/01-mc.html"),
    ("02 Object / link", DOCS / "llvm" / "02-object-link.md", "llvm/02-object-link.html"),
    ("03 Code generation", DOCS / "llvm" / "03-codegen.md", "llvm/03-codegen.html"),
    ("04 ABI", DOCS / "llvm" / "04-abi.md", "llvm/04-abi.html"),
    ("05 Runtime / driver", DOCS / "llvm" / "05-runtime-driver.md", "llvm/05-runtime-driver.html"),
    ("06 Validation", DOCS / "llvm" / "06-validation.md", "llvm/06-validation.html"),
]

SDK_PAGES: list[tuple[str, Path, str]] = [
    ("SDK approach", DOCS / "sdk" / "README.md", "sdk/overview.html"),
    ("00 Project / CLI", DOCS / "sdk" / "00-project-cli.md", "sdk/00-project-cli.html"),
    ("01 Runtime", DOCS / "sdk" / "01-runtime.md", "sdk/01-runtime.html"),
    ("02 Graphics / assets", DOCS / "sdk" / "02-graphics-assets.md", "sdk/02-graphics-assets.html"),
    ("03 Input / loop", DOCS / "sdk" / "03-input-loop.md", "sdk/03-input-loop.html"),
    ("04 Audio / save / banks", DOCS / "sdk" / "04-audio-save-banking.md", "sdk/04-audio-save-banking.html"),
    ("05 Agent workflow", DOCS / "sdk" / "05-agent-workflow.md", "sdk/05-agent-workflow.html"),
    ("06 Examples / validation", DOCS / "sdk" / "06-validation-examples.md", "sdk/06-validation-examples.html"),
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

SOURCE_TO_HTML = {
    (REPO / "README.md").resolve(): "index.html",
    **{
        source.resolve(): html
        for pages in (LEARN_AND_CRATE, QEMU_PAGES, LLVM_PAGES, SDK_PAGES, GBEDG_PAGES, PANDOCS_PAGES)
        for _, source, html in pages
    },
}

MD = markdown.Markdown(
    extensions=["tables", "fenced_code", "toc", "sane_lists", "smarty"],
    extension_configs={"toc": {"permalink": False}},
)

STYLE = (DOCS / "assets" / "style.css")


def rewrite_md_links(text: str, source: Path, html_rel: str, is_gbedg: bool) -> str:
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
        source_target = (source.parent / path_part).resolve()
        site_target = SOURCE_TO_HTML.get(source_target)
        if site_target:
            relative = os.path.relpath(site_target, start=Path(html_rel).parent).replace(os.sep, "/")
            return f"[{label}]({relative}{frag})"
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


def render_body(source: Path, html_rel: str = "", is_gbedg: bool = False) -> str:
    raw = source.read_text(encoding="utf-8")
    if source.suffix == ".txt":
        escaped = raw.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
        return f'<pre class="fixture">{escaped}</pre>'
    raw = rewrite_md_links(raw, source, html_rel, is_gbedg)
    MD.reset()
    return MD.convert(raw)


def _nav_link(root: str, html: str, label: str, current: str) -> str:
    href = f"{root}/{html}"
    cls = ' class="nav-active"' if current == html else ""
    return f'        <li><a href="{href}"{cls}>{label}</a></li>'


def nav_block(root: str, current: str = "") -> str:
    soft = "\n".join(
        _nav_link(root, html, label, current)
        for label, src, html in LEARN_AND_CRATE
        if src.exists()
    )
    qemu = "\n".join(
        _nav_link(root, html, label, current)
        for label, src, html in QEMU_PAGES
        if src.exists()
    )
    llvm = "\n".join(
        _nav_link(root, html, label, current)
        for label, src, html in LLVM_PAGES
        if src.exists()
    )
    sdk = "\n".join(
        _nav_link(root, html, label, current)
        for label, src, html in SDK_PAGES
        if src.exists()
    )
    gbedg = "\n".join(
        _nav_link(root, html, label, current)
        for label, src, html in GBEDG_PAGES
        if src.exists()
    )
    pandocs = "\n".join(
        _nav_link(root, html, label, current)
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
    qemu_section = ""
    if any(src.exists() for _, src, _ in QEMU_PAGES):
        qemu_section = f"""
      <p class="nav-section">QEMU SM83</p>
      <ul>
{qemu}
      </ul>"""
    llvm_section = ""
    if any(src.exists() for _, src, _ in LLVM_PAGES):
        llvm_section = f"""
      <p class="nav-section">LLVM SM83</p>
      <ul>
{llvm}
      </ul>"""
    sdk_section = ""
    if any(src.exists() for _, src, _ in SDK_PAGES):
        sdk_section = f"""
      <p class="nav-section">GBC SDK</p>
      <ul>
{sdk}
      </ul>"""
    lab = "\n".join(
        [
            _nav_link(root, "softgbc/environment.html", "Environment", current),
            _nav_link(root, "softgbc/visual-tests.html", "Visual tests", current),
        ]
    )
    return f"""
      <p class="nav-section">Lab</p>
      <ul>
{lab}
      </ul>
      <p class="nav-section">SoftGBC</p>
      <ul>
{soft}
      </ul>{llvm_section}{qemu_section}{sdk_section}
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


def page_shell(
    title: str,
    body: str,
    page_html: str,
    crumb: str,
    extra_head: str = "",
    extra_scripts: str = "",
) -> str:
    depth = page_html.count("/")
    root = "/".join([".."] * depth) if depth else "."
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title} — GBC platform docs</title>
  <link rel="stylesheet" href="{root}/assets/style.css">
{extra_head}
</head>
<body>
  <div class="layout">
    <nav class="sidebar">
      <a class="brand" href="{root}/index.html">GBC platform docs</a>
{nav_block(root, page_html)}
    </nav>
    <main>
      <p class="crumb">{crumb}</p>
      <article class="content">
{body}
      </article>
    </main>
  </div>
{extra_scripts}
  <script src="{root}/assets/live_reload.js"></script>
</body>
</html>
"""


def write_page(
    html_rel: str,
    title: str,
    body: str,
    crumb: str,
    extra_head: str = "",
    extra_scripts: str = "",
) -> None:
    out = SITE / html_rel
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(
        page_shell(title, body, html_rel, crumb, extra_head, extra_scripts),
        encoding="utf-8",
    )


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
<h1>GBC platform documentation</h1>
<p><strong>Markdown is the source of truth.</strong> This generated site covers SoftGBC,
the SM83 LLVM and QEMU ports, and the agent-first GBC game SDK. Datasheets and
third-party guides also live under repo <code>docs/</code>.</p>
<pre><code>cd docs &amp;&amp; ./serve.sh
# → http://127.0.0.1:8000/</code></pre>
<div class="cards">
  <a class="card" href="llvm/overview.html"><strong>LLVM SM83</strong><span>Compiler, ABI, linker, runtime, and validation</span></a>
  <a class="card" href="qemu/overview.html"><strong>QEMU SM83</strong><span>CPU target, test machine, traces, and GDB</span></a>
  <a class="card" href="sdk/overview.html"><strong>GBC game SDK</strong><span>Runtime, assets, APIs, and agent workflow</span></a>
  <a class="card" href="softgbc/overview.html"><strong>Roadmap checklist</strong><span>SoftGBC/README — driver + min tests</span></a>
  <a class="card" href="softgbc/dmg-ppu.html"><strong>PPU checklist</strong><span>LY → VBlank → SoftGBC wire → BG</span></a>
  <a class="card" href="softgbc/environment.html"><strong>Environment</strong><span>Video sinks + joypad InputSource</span></a>
  <a class="card" href="softgbc/visual-tests.html"><strong>Visual tests</strong><span>Screen + Play + expected CRC</span></a>
  <a class="card" href="softgbc/timer.html"><strong>Timer checklist</strong><span>Overflow IRQ + accuracy follow-ups</span></a>
  <a class="card" href="softgbc/serial.html"><strong>Serial checklist</strong><span>When IF bit 3 fires</span></a>
  <a class="card" href="gbedg/ppu.html"><strong>GBEDG PPU</strong><span>Hardware deep dive</span></a>
  <a class="card" href="datasheets/index.html"><strong>PDF datasheets</strong><span>gbctr, TCAGBD, Sharp 1996</span></a>
  <a class="card" href="references.html"><strong>References</strong><span>How SoftGBC uses each source</span></a>
</div>
"""
    write_page("index.html", "Home", body, "Home")


def build_visual_lab() -> None:
    body = """
<h1>Visual tests</h1>
<p>Run SoftGBC PPU scenes in the browser: each card has an LCD screen, expected CRC/fixture notes,
and a <strong>Play</strong> button. Tests <strong>auto-run</strong> when their card scrolls into view.</p>
<p class="muted">Requires <code>./serve.sh</code> (lab API). See also
<a href="environment.html">Environment</a> for video sinks + joypad input sources.</p>
<div id="visual-lab">
  <p id="lab-status">…</p>
  <div id="lab-list"></div>
</div>
"""
    write_page(
        "softgbc/visual-tests.html",
        "Visual tests",
        body,
        "SoftGBC / Visual tests",
        extra_scripts='  <script src="../assets/visual_lab.js"></script>\n',
    )
    print("wrote softgbc/visual-tests.html")


def build_environment_page() -> None:
    body = """
<h1>Environment</h1>
<p>How SoftGBC talks to the outside world: <strong>video out</strong> and
<strong>joypad in</strong>. The PPU and Joypad MMIO stay host-agnostic —
pluggable backends live in <code>softgbc-host</code>.</p>

<section class="play-panel" id="play-panel">
  <div class="play-layout">
    <div class="play-main">
      <h2>Play a ROM</h2>
      <p>Load a cart from <code>SoftGBC/cartridge/roms/</code> (Tetris DX is there) or upload
      a <code>.gb</code> / <code>.gbc</code>. Requires <code>./serve.sh</code> (spawns
      <code>softgbc-play</code>). MBC1 carts work; MBC3 (e.g. Pokémon Crystal) later.</p>
      <p class="play-share">Session <code id="play-session">…</code> —
      share <a id="play-session-link" href="#">this link</a> to watch the same play instance.</p>
      <p class="play-share play-share--hint" id="play-reach-hint">
        Watching from a Mac: Cursor <strong>Ports</strong> → forward <code>8000</code>
        (then Mac <code>localhost:8000</code> works), or open the LAN link filled in below.
      </p>
      <div class="play-controls">
        <label class="play-field">
          <span>Cartridge</span>
          <select id="play-rom-select"></select>
        </label>
        <label class="play-field">
          <span>Upload</span>
          <input id="play-file" type="file" accept=".gb,.gbc,.bin,application/octet-stream" />
        </label>
        <div class="lab-actions">
          <button type="button" class="lab-play" id="play-start">Play</button>
          <button type="button" class="lab-play lab-play--stop" id="play-stop">Stop</button>
        </div>
      </div>
      <div class="play-screen">
        <div class="lab-screen-cap">LCD <span id="play-key-lock" class="play-key-lock" hidden></span></div>
        <canvas id="play-lcd" class="lab-lcd lab-lcd--empty play-lcd" width="160" height="144" tabindex="0" role="img" aria-label="Game Boy LCD"></canvas>
      </div>
      <p class="lab-meta" id="play-status">…</p>
    </div>
    <aside class="play-live" aria-label="Live games">
      <h3>Live games</h3>
      <p class="play-live-hint">Click a session to watch.</p>
      <ul id="play-live-list" class="play-live-list">
        <li class="play-live-empty">No live sessions yet.</li>
      </ul>
    </aside>
  </div>
</section>

<h2>Video (FrameSink)</h2>
<pre><code>PpuController → run_loop → FrameSink(s)
  ├─ HostWindow       local minifb / X11
  ├─ HubSink          HTTP /frame.png preview
  ├─ ChannelSink      → RemoteEgress (WebRTC stub …)
  └─ NullSink         tests / CI</code></pre>
<p>Fan-out can drive several sinks at once (window <em>and</em> browser preview).</p>

<h2>Joypad (InputSource)</h2>
<pre><code>InputSource::poll() → Buttons → Joypad::set_pressed → FF00
  ├─ ManualInput / NullInput     tests &amp; scripted presses
  ├─ ChannelInput                worker thread → emu
  ├─ WebInputStub                future browser / data channel
  ├─ BluetoothInputStub          future BLE gamepad
  └─ FanInInput                  OR several sources</code></pre>
<p>Same pattern as video: add a backend later without touching <code>dmg-ppu</code> or
<code>peripherals</code> select/read logic.</p>

<h2>Docs server (<code>./serve.sh</code>)</h2>
<ul>
  <li>Builds static HTML + <code>softgbc-lab</code> + <code>softgbc-play</code></li>
  <li>Serves site with <code>/api/visual-tests</code> and <code>/api/play/*</code></li>
  <li>Fixture text under <code>/api/fixtures/…</code></li>
</ul>
<p><a href="visual-tests.html"><strong>Open visual tests →</strong></a>
(screen + Play + expected outcome per scene)</p>

<h2>Quick commands</h2>
<pre><code>cd docs &amp;&amp; ./serve.sh
# → http://127.0.0.1:8000/softgbc/environment.html
# → http://127.0.0.1:8000/softgbc/visual-tests.html

cargo run -p softgbc-host --bin softgbc-view -- --demo --http 8765
cargo run -p softgbc-host --bin softgbc-lab -- list
cargo run -p softgbc-host --bin softgbc-play -- --port 8766</code></pre>
"""
    write_page(
        "softgbc/environment.html",
        "Environment",
        body,
        "SoftGBC / Environment",
        extra_scripts='  <script src="../assets/play.js"></script>\n',
    )
    print("wrote softgbc/environment.html")


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
    build_environment_page()
    build_visual_lab()
    build_attribution()
    build_datasheets_index()
    for name in ("visual_lab.js", "live_reload.js", "play.js"):
        js_src = DOCS / "assets" / name
        if js_src.exists():
            shutil.copy2(js_src, assets / name)

    for label, src, html_rel in LEARN_AND_CRATE:
        if not src.exists():
            print(f"skip {src}")
            continue
        write_page(html_rel, label, render_body(src, html_rel), f"SoftGBC / {label}")
        print(f"wrote {html_rel}")

    for label, src, html_rel in QEMU_PAGES:
        if not src.exists():
            print(f"skip {src}")
            continue
        write_page(html_rel, label, render_body(src, html_rel), f"QEMU SM83 / {label}")
        print(f"wrote {html_rel}")

    for label, src, html_rel in LLVM_PAGES:
        if not src.exists():
            print(f"skip {src}")
            continue
        write_page(html_rel, label, render_body(src, html_rel), f"LLVM SM83 / {label}")
        print(f"wrote {html_rel}")

    for label, src, html_rel in SDK_PAGES:
        if not src.exists():
            print(f"skip {src}")
            continue
        write_page(html_rel, label, render_body(src, html_rel), f"GBC SDK / {label}")
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
        body = render_body(src, html_rel, is_gbedg=True)
        section = section_for.get(html_rel, "")
        if section:
            body = fix_gbedg_images(body, section)
        write_page(html_rel, f"GBEDG — {label}", body, f"GBEDG / {label}")
        print(f"wrote {html_rel}")

    for label, src, html_rel in PANDOCS_PAGES:
        if not src.exists():
            continue
        write_page(html_rel, f"Pan Docs — {label}", render_body(src, html_rel), f"Pan Docs / {label}")
        print(f"wrote {html_rel}")

    copy_gbedg_images()
    print(f"\nDone → {SITE}")
    print("Serve:  cd docs && ./serve.sh")


if __name__ == "__main__":
    main()
