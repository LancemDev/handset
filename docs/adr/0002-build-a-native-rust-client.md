# 0002. Build a native Rust client instead of forking scrcpy

- Status: accepted
- Date: 2026-10-25

## Context

The goal is a mirroring app that shows the phone inside a realistic device frame, can draw overlays,
and records or screenshots the framed result, while staying small and light.

A prototype patched scrcpy's C client (SDL) to draw a bezel around the video. It worked, but SDL's
renderer cannot clip to rounded shapes, every upstream release would need a rebase, and many requested
features sit unmerged upstream for years.

## Decision

We will write our own client in Rust (winit for the window, wgpu for rendering) that speaks the scrcpy
protocol, and keep using scrcpy's server on the phone (see ADR 0003).

## Alternatives considered

- **Keep patching the C client.** Fast to start, but a permanent rebase burden and limited rendering.
- **A wrapper window around stock scrcpy.** Two windows to keep in sync, and the framed result could
  not be recorded or captured.
- **Electron or Tauri.** Much easier UI work, but a webview and its memory use go against the goal of
  a small app.

## Consequences

- We own the client side of the protocol, including input, decoding and audio.
- Handset can draw anything: bezels, transparency, overlays, framed captures.
- macOS is the first and only tested target. `cfg` guards keep other targets compiling in principle.
- Measured on a Mac, Handset's memory footprint is lower than stock scrcpy's but its resident memory
  (about 110 MB) is not, so "smaller than scrcpy" holds for the binary (about 5 MB) rather than for RAM.
