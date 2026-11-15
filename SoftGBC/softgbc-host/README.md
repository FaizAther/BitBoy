# softgbc-host

Host **display / remote video** for SoftGBC. The PPU never depends on this crate.

## Architecture

```text
PpuController (SoftGbc / Demo)     ← drive emulation + expose ShadeFrame
        │
        ▼
   run_driver / run_loop
        │
        ▼
   FrameSink (pluggable environment)
     ├─ HostWindow     local GUI on this server (DISPLAY) or SSH -X
     ├─ ChannelSink    owned frames → another thread / process
     ├─ FanOutSink     local **and** remote at once
     └─ NullSink       tests
            │
            ▼
   RemoteEgress        WebRTC / webapp / custom
     └─ WebRtcEgressStub (placeholder; swap for real webrtc)
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
# Demo stripes on $DISPLAY (works with `ssh -X` if X11 forwarding is on):
cargo run -p softgbc-host --bin softgbc-view -- --demo

# Local window + broadcast channel (stub WebRTC consumer drains the channel):
cargo run -p softgbc-host --bin softgbc-view -- --demo --broadcast --frames 120

# Headless broadcast only (e.g. CI / future web bridge):
cargo run -p softgbc-host --bin softgbc-view -- --demo --no-window --broadcast --frames 30
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
