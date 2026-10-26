# 0007. Capture raw PCM audio and play it with cpal

- Status: accepted
- Date: 2026-10-26

## Context

Mirroring audio matters for demos and recordings. The server can send Opus, AAC, FLAC or raw PCM.
Handset avoids heavy dependencies (see ADR 0006).

## Decision

Ask the server for `audio_codec=raw`: 48 kHz, stereo, signed 16-bit little-endian. No decoder is needed.
The audio thread keeps a short ring buffer for local playback through `cpal` (about 40 ms prebuffer,
capped at about 120 ms, dropping the oldest audio rather than letting latency grow) and hands a copy of the
stream to the recorder when one is running. Audio is opt-in (`--audio`), and `--mute` records without
playing locally.

## Alternatives considered

- **Opus.** Smaller on the wire, but needs a decoder crate for no benefit over USB.
- **Always on.** Capturing audio changes how the phone behaves, so it is off by default.

## Consequences

- Needs Android 11 or later on the phone; older phones report that audio is unavailable and mirroring
  continues without it.
- About 192 KB/s of extra traffic over adb, which is fine over USB.
- Playback health (buffered milliseconds, underruns, dropped audio) is printed with `HANDSET_DEBUG=1`.
- The phone's own speaker may go quiet while audio is captured; this is server behaviour and has not been
  verified here.
