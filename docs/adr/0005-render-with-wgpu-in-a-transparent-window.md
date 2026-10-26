# 0005. Render with wgpu in a transparent window

- Status: accepted
- Date: 2026-10-25

## Context

Handset needs a borderless, transparent window with a bezel drawn over live video, small overlays that
must not appear in screenshots or recordings, and captures that look like the window.

## Decision

- Use winit for the window and wgpu (Metal on macOS) for drawing. The window is borderless and
  transparent whenever a frame is active; the surface uses a premultiplied alpha mode and a
  non-sRGB format, because the video is already gamma-encoded.
- Upload the decoded I420 planes as three R8 textures and convert to RGB in the fragment shader
  (BT.709, limited range, which is what the phone's encoder produces). No CPU colour conversion.
- Draw in this order: black screen backing, video, bezel, then (window only) toolbar and overlay pill.
- Screenshots and recordings render the same scene offscreen at a chosen size without the window-only
  layers and read the pixels back, so captures never contain the toolbar, toast or REC pill.
- Prefer the low-power adapter.

## Alternatives considered

- **SDL's renderer** (as the prototype used): no rounded clipping, awkward overlays.
- **Plain softbuffer or CPU compositing:** simple, but converting and blending 1080p frames on the CPU
  would waste battery for no benefit.
- **Skia or egui for everything:** heavier, and we only need a handful of quads.

## Consequences

- Rendering is a few hundred lines of our own code against a fast-moving crate: wgpu 30 differs from
  most tutorials, so read the crate source.
- Offscreen readback stalls briefly per captured frame, which is acceptable at 30 fps for recording.
- Everything is GPU-bound, so window size has little effect on CPU use.
