# Joypad

`P1` (`FF00`) — d-pad and buttons.

**Implementation stub:** [`joypad.rs`](joypad.rs) (`peripherals::joypad`).

## Current status *(2026-11)*

**Stub.** Wire after headless PPU; pair with a windowed frontend.

## Checklist

- [ ] Button state from host (keyboard / gamepad)
- [ ] Select lines for directions vs buttons
- [ ] Correct active-low bit behaviour
- [ ] Joypad IRQ on button press (when enabled)
- [ ] Wire into bus IO
- [ ] Manual test: move character in a simple homebrew / commercial ROM
