# Architecture decision records

Short notes that capture why Handset is built the way it is. One decision per file, numbered in
the order they were made. Never rewrite history: if a decision changes, add a new ADR and mark the old
one as superseded.

Statuses: `proposed`, `accepted`, `superseded by NNNN`, `rejected`.

Copy [`0000-template.md`](0000-template.md) to start a new one.

| ADR | Decision | Status |
|---|---|---|
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | accepted |
| [0002](0002-build-a-native-rust-client.md) | Build a native Rust client instead of forking scrcpy | accepted |
| [0003](0003-reuse-the-unmodified-scrcpy-server.md) | Reuse the unmodified scrcpy server | accepted |
| [0004](0004-draw-device-frames-from-png-bezels.md) | Draw device frames from PNG bezels | accepted |
| [0005](0005-render-with-wgpu-in-a-transparent-window.md) | Render with wgpu in a transparent window | accepted |
| [0006](0006-decode-h264-in-software-with-openh264.md) | Decode H.264 in software with openh264 | accepted |
