# Joypad

`P1` (`FF00`) — d-pad and buttons.

**Refs:** [Pan Docs — Joypad](https://gbdev.io/pandocs/Joypad_Input.html) · stub [`joypad.rs`](joypad.rs)  

## Status *(2026-11)*

**P1 select + pressed state wired.** Host play loop: `run_play_loop` polls
`InputSource` → `Joypad::set_pressed` each frame, then SoftGbc advances and blits.
Window keyboard via `SharedKeyboardInput` (arrows / X=A / Z=B / Enter / Backspace).
IRQ still open.

```text
FF00 write: choose group
  bit4 = 0 → d-pad
  bit5 = 0 → A/B/Select/Start

FF00 read (lower nibble), selected group only:
  pressed → 0    released → 1     (active-low)

Host: InputSource::poll() → Buttons → Joypad::set_pressed
```

## Checklist

What → Structure → Min test → Done when (pseudocode).

---

- [x] **Button state from host**

  **What:** host keyboard/gamepad updates which keys are pressed.  
  **Structure:**
```text
Joypad:
  keys: Right Left Up Down A B Select Start   # pressed? yes/no
  select_dpad, select_buttons

  set_from_host(keys):
    map arrows → d-pad, Z/X → A/B, …
```
  **Min test:**
```text
set A pressed
select button group
read FF00 → A bit is 0
```
  **Done when:** host can change keys and reads reflect them (`set_pressed` + `ManualInput`).

---

- [x] **Select lines (directions vs buttons)**

  **What:** bits 4–5 of `FF00` choose which nibble is visible.  
  **Structure:**
```text
write FF00:
  select_dpad    = (bit4 == 0)
  select_buttons = (bit5 == 0)

read lower nibble:
  if select_buttons → from A/B/Select/Start
  else if select_dpad → from Right/Left/Up/Down
  else → 0xF
```
  **Min test:**
```text
only A pressed
select buttons (not d-pad)
read FF00 → only A bit low
```
  **Done when:** polarity matches Pan Docs for a known select value.

---

- [x] **Active-low bits**

  **What:** pressed = 0, released = 1.  
  **Structure:** same read path as above.  
  **Min test:**
```text
no keys, group selected → nibble 0xF
one key pressed         → that bit 0, others 1
```
  **Done when:** both cases pass.

---

- [ ] **Joypad IRQ (IF bit 4)**

  **What:** high→low on a selected line can request joypad IRQ (`$0060`).  
  **Structure:**
```text
on press edge while group selected:
  request IF bit 4
bus: IF = IF | 0x10
```
  **Min test:**
```text
group selected, A released → then A pressed
assert (IF & 0x10) != 0
```
  **Done when:** edge sets bit once; see [`../../interrupt/README.md`](../../interrupt/README.md).

---

- [x] **Wire into bus**

  **What:** `FF00` routes to Joypad like timer IO.  
  **Structure:**
```text
bus read/write FF00 → Joypad
```
  **Min test:**
```text
bus.write(FF00, select_buttons)
bus.read(FF00) matches pressed keys
```
  **Done when:** SoftGbc can read joypad without a special bus.

---

- [x] **Host play loop (blit + poll)**

  **What:** each frame: poll keys → joypad → SoftGbc frame → present.  
  **Structure:**
```text
run_play_loop(SoftGbcController, InputSource, FrameSink):
  buttons = input.poll()
  joypad.set_pressed(buttons)
  advance_frame()   # CPU + DMA ticks + render
  sink.present(shade_frame)
```
  **Min test:**
```text
ManualInput(A) + NullSink → 2 frames; joypad.pressed has A
keys_to_buttons(Left|A) → Buttons LEFT|A
```
  **Done when:** `play_loop_applies_input_then_presents` + `softgbc-view --rom` path green.

---

- [ ] **Manual: move in a simple ROM**

  **What:** windowed host + SoftGbc; character moves on a real cart.  
  **Structure:** `softgbc-view --rom game.gb --window` (needs APU mute / MBC as required).  
  **Min test:** play homebrew / boot screen by hand.  
  **Done when:** input feels correct in-game.
