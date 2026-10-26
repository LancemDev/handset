# 0008. Record through an ffmpeg sidecar

- Status: accepted
- Date: 2026-10-26

## Context

Recordings should show the framed phone and include the phone's audio, as MP4 or GIF, without
making Handset itself large.

## Decision

Render the framed scene offscreen at a fixed 30 fps (repeating frames while the screen is static) and pipe
the raw RGBA frames to an `ffmpeg` child process on stdin. Audio goes to the same process as a second
input over a local TCP socket, queued from the moment recording starts so audio and video begin together.

- MP4 uses libx264 with AAC audio at 48 kHz; the audio is loudness-normalised to about -16 LUFS by
  default (`--gain` and `--no-normalize` adjust that), and `-shortest` trims the padding the normaliser adds.
- GIF uses a palette pass and has no audio.
- If the phone rotates mid-recording the file is finished and a new one starts at the new size.
- If ffmpeg cannot keep up, frames are dropped rather than queued without bound.
- Screenshots are PNGs with a transparent background, rendered through the same offscreen path.
- `ffmpeg` is an optional dependency, found on PATH or in the usual Homebrew locations.

## Alternatives considered

- **Encode in process** (openh264 encoder plus an MP4 muxer): no external tool, but more code and slower
  software encoding.
- **Link FFmpeg libraries:** a large dependency that goes against ADR 0006.
- **Re-mux the phone's H.264 stream:** cheap, but it would not contain the frame.

## Consequences

- Recording needs `ffmpeg` installed; Handset reports a clear error if it is missing.
- Audio and video sync is as good as the start-together queueing allows (checked to within a video frame
  on one phone, not guaranteed in general).
- Each captured frame is read back from the GPU, which costs some CPU and memory bandwidth.
