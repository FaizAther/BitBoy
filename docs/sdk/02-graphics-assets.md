# 02 — Graphics API and asset pipeline

Convert common art files into typed CGB assets and provide bounded rendering
APIs that respect VRAM banks, tile maps, palettes, OAM, and VBlank timing.

## Checklist

What → Structure → Min test → Done when.

---

- [ ] **Deterministic image conversion**

  **What:** Convert indexed/true-color PNG tiles and sprites to CGB 2bpp tile
  bytes plus 15-bit palettes. Palette assignment, deduplication, flips, and
  ordering must be deterministic and configurable.

  **Structure:**
```text
PNG
  → validate dimensions/transparency/colors
  → quantize/map palettes
  → split 8×8 tiles
  → optional exact/flip dedupe
  → 16-byte 2bpp tiles + palette data + metadata
```

  **Min test:**
```text
known 8×8 image produces exact bytes
same pixels in different file metadata produce identical output
too many colors names tile/palette causing overflow
```

  **Done when:** converter has golden byte tests and emits no nondeterministic IDs.

---

- [ ] **Typed generated asset interface**

  **What:** Generate namespaced declarations/descriptors instead of requiring
  games to remember tile indices, bank numbers, dimensions, or palette slots.

  **Structure:**
```c
extern const gbc_tileset_t hero_tiles;
extern const gbc_sprite_sheet_t hero_sprites;
extern const gbc_tilemap_t level_1_map;
extern const gbc_palette_set_t level_1_palettes;
```
```text
generated files:
  build/generated/assets.h
  asset data objects/sections
  JSON inventory for reports/tools
```

  **Min test:**
```text
rename manifest asset → old symbol fails clearly, new symbol compiles
two assets with colliding normalized names are rejected
```

  **Done when:** gameplay source contains no converter-generated magic offsets.

---

- [ ] **VRAM and palette allocator**

  **What:** Track CGB VRAM banks, tile slots, BG maps, and BG/OBJ palette slots.
  Static allocation happens at build/load time; dynamic allocation is explicit
  and bounded. Reports identify fragmentation/owners.

  **Structure:**
```text
VRAM bank 0/1:
  tile ranges
  map ranges

palette:
  8 BG palettes × 4 colors
  8 OBJ palettes × 4 colors

handle → owner/range/bank/generation
```

  **Min test:**
```text
allocate exact capacity
one extra allocation fails with requested/available units
free/reuse invalidates stale debug handle
```

  **Done when:** no API can overwrite another asset’s VRAM silently.

---

- [ ] **VBlank upload queue**

  **What:** Tile/map/palette writes are queued with a measurable byte/cycle
  budget and committed during safe periods. Large uploads span frames or require
  LCD-off loading through an explicit API.

  **Structure:**
```text
game:
  gbc_gfx_upload(asset, destination)
       ↓
fixed queue: source, destination, bytes, progress
       ↓ VBlank
copy up to frame budget; retain remainder
```

  **Min test:**
```text
small upload completes one VBlank
large upload spans expected frames without queue corruption
budget overflow returns status and appears in debug report
```

  **Done when:** API documents completion/fence semantics and visual tests show
  no unsafe partial map exposure.

---

- [ ] **Background and window API**

  **What:** Provide tilemap regions, scrolling, map attributes (VRAM bank,
  palette, flips, priority), and window position through typed operations.
  Separate CPU-side map state from queued hardware commits.

  **Structure:**
```text
gbc_bg_set_tiles(...)
gbc_bg_set_attrs(...)
gbc_bg_scroll(x,y)
gbc_window_show(x,y)

logical map update → dirty region → VBlank queue
```

  **Min test:**
```text
write 2×2 tile/attribute region at map edge
scroll wrapping and window visibility match expected frame CRC/image
```

  **Done when:** common scene rendering requires no direct LCDC/VBK/MMIO writes.

---

- [ ] **Sprite/OAM API**

  **What:** Manage 40 hardware sprites with stable handles, hide/show, position,
  tile, bank, palette, flips, and priority. Build a shadow OAM and commit through
  the runtime’s safe transfer path.

  **Structure:**
```text
game sprite handles
  → shadow OAM entries
  → ordering/culling policy
  → VBlank OAM commit

limits:
  40 total hardware entries
  visible-per-scanline constraint reported/tested
```

  **Min test:**
```text
create/move/hide/reuse sprite
41st allocation fails cleanly
scanline-stress example documents deterministic ordering/flicker policy
```

  **Done when:** sprite state is deterministic and OAM limits appear in reports.

---

- [ ] **Map importer and collision metadata**

  **What:** Import a documented subset of TMX/JSON tile maps, layers, object
  markers, and custom collision properties. Reject unsupported editor features
  rather than approximating silently.

  **Structure:**
```text
map source
  → visual tile/attribute chunks
  → collision/property arrays
  → spawn/object records
  → typed descriptor
```

  **Min test:**
```text
small two-layer map converts to expected tiles + collision bytes
unsupported infinite/rotated object reports exact source item
```

  **Done when:** example game loads map visuals and collision from one source.

---

- [ ] **Visual regression fixtures**

  **What:** Every graphics primitive gets a tiny deterministic ROM and expected
  frame checkpoint through SoftGBC. Compare frame CRC plus saved image on
  failure.

  **Structure:**
```text
asset fixture → SDK ROM → scripted frames → framebuffer CRC/image
```

  **Min test:**
```text
tiles/palettes, background attrs, window, sprites, upload spanning frames
```

  **Done when:** graphics changes fail with a named fixture and inspectable image.
