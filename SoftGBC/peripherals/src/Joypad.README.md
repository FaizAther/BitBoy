# Joypad

`P1` (`FF00`) — d-pad and buttons.

**Refs:** [Pan Docs — Joypad](https://gbdev.io/pandocs/Joypad_Input.html) · stub [`joypad.rs`](joypad.rs)  

## Status *(2026-11)*

**Stub.** Wire after headless PPU + a window.

```text
FF00 write: choose group
  bit4 = 0 → d-pad
  bit5 = 0 → A/B/Select/Start

FF00 read (lower nibble), selected group only:
  pressed → 0    released → 1     (active-low)
```

## Checklist

What → Structure → Min test → Done when (pseudocode).

---

- [ ] **Button state from host**

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
  **Done when:** host can change keys and reads reflect them.

---

- [ ] **Select lines (directions vs buttons)**

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

- [ ] **Active-low bits**

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

- [ ] **Wire into bus**

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

- [ ] **Manual: move in a simple ROM**

  **What:** windowed host + SoftGbc; character moves.  
  **Structure:** host maps keys → `set_from_host` each frame.  
  **Min test:** play homebrew / boot screen by hand.  
  **Done when:** input feels correct in-game.
