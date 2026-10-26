# 0006. Decode H.264 in software with openh264

- Status: accepted
- Date: 2026-10-25

## Context

The phone sends H.264. Handset aims to stay small, and linking FFmpeg would grow the binary and
the build considerably. Hardware decoding (VideoToolbox) needs a lot of unsafe platform code.

## Decision

Decode with the `openh264` crate (Cisco's decoder, built from source) on a dedicated video thread.
The thread writes only the newest frame into a shared slot and wakes the UI thread through an
event-loop proxy, so a slow UI drops stale frames instead of building latency.

Measured on an Apple Silicon Mac mirroring a 720x1600 phone that scrolls continuously: about 15 to 20
percent of one core. Memory is dominated by the GPU stack, not the decoder.

## Alternatives considered

- **FFmpeg (libavcodec).** Hardware paths and more codecs, but a much bigger dependency.
- **VideoToolbox directly.** Would lower CPU use; skipped because the numbers above do not justify the
  unsafe FFI yet.

## Consequences

- Only H.264 is supported (the client rejects any other codec id).
- Higher resolutions and frame rates may need hardware decoding. Revisit if CPU use becomes a problem.
