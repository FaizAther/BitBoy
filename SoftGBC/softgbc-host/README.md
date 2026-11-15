# softgbc-host

Host **display / remote video** for SoftGBC. The PPU never depends on this crate.

## Architecture

```text
SoftGbcController / Demo
        │
   run_play_loop (ROM) / run_loop (demo)
     InputSource → Joypad          (keyboard / manual / web stub)
     advance_frame → ShadeFrame
        │
        ▼
   FrameSink
     ├─ HostWindow (+ SharedKeyboardInput)
     ├─ HubSink / HTTP preview
     ├─ ChannelSink → RemoteEgress
     └─ NullSink
```

| Layer | Crate | Knows about windows? |
|---|---|---|
| Shade buffer | `softgbc-utils::video` | No |
| PPU | `dmg-ppu` | No — only `framebuffer` / `shade_frame()` |
| Controller | `softgbc-host` | No GUI; steps SoftGBC / demo |
| Sink / driver | `softgbc-host` | Yes — minifb, channels, remotes |

## Run (server or remote X)

```bash
cd SoftGBC
# Demo stripes on $DISPLAY:
cargo run -p softgbc-host --bin softgbc-view -- --demo --window

# Play a ROM (blit + keyboard → joypad):
cargo run -p softgbc-host --bin softgbc-view -- --rom path/to/game.gb --window
# Keys: arrows  X=A  Z=B  Enter=Start  Backspace=Select  Esc=quit

# HTTP preview (default port 8765) without a local window:
cargo run -p softgbc-host --bin softgbc-view -- --rom path/to/game.gb --http 8765 --frames 120
```

## Adding WebRTC / a webapp

1. Keep using [`ChannelSink`](src/sink.rs) (or `DriverConfig::local_and_broadcast`).
2. Implement [`RemoteEgress`](src/remote.rs) with your encoder + `webrtc` / WS signaling.
3. `forward_to_remote(rx, &mut your_egress)` on a worker thread (or async task).

The PPU and SoftGBC machine stay unchanged.

## Tests

```bash
cargo test -p softgbc-host
cargo test -p softgbc-utils video
```
