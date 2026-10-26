# Handset

[![ci](https://github.com/davidamunga/handset/actions/workflows/ci.yml/badge.svg)](https://github.com/davidamunga/handset/actions/workflows/ci.yml)

Mirror and control an Android phone from your Mac, inside a realistic device frame.

Handset is a small native Rust app. It talks to the unmodified
[scrcpy](https://github.com/Genymobile/scrcpy) server on the phone and replaces the desktop client,
so it can draw things a plain window cannot: phone bezels on a transparent window, a toolbar, and
framed screenshots and recordings with audio.

> **Status: early work in progress.** It works on the author's Mac with one phone. Expect rough edges.
> macOS is the only platform that has been tested.

## Features

- Live video and device audio over adb (USB), decoded in software and drawn with wgpu
- Realistic phone frames (`obsidian`, `porcelain`, `hazel`, or your own PNG) on a transparent window,
  switchable while running
- Touch, scroll, keyboard and text input; landscape rotation
- Framed screenshots (transparent PNG) and recordings (MP4 with audio, or GIF) via ffmpeg
- A toolbar beside the phone and a native right-click menu
- A clickable "saved" toast, a recent captures menu and automatic clipboard copy
- A small binary (about 5 MB) with the scrcpy server embedded

## Requirements

- macOS on Apple Silicon or Intel (developed on a recent macOS)
- An Android phone with USB debugging enabled (audio needs Android 11 or later)
- [`adb`](https://developer.android.com/tools/releases/platform-tools) (Android platform-tools)
- [`ffmpeg`](https://ffmpeg.org) for recording (optional)
- Rust (stable) to build from source

## Run it

```sh
git clone https://github.com/davidamunga/handset.git
cd handset/app
cargo run --release -- --audio
```

Or build the app bundle and open it:

```sh
scripts/package-macos.sh
open dist/Handset.app
```

The bundle is ad-hoc signed, so the first time you may need to right-click it and choose Open. To
install the command line tool instead: `cargo install --path app --locked`.

## Using it

- **Right-click** the phone, or use the toolbar beside it: frames, screenshot, recording, recent
  captures, rotate, quit.
- **Shortcuts** (Cmd or Ctrl): `S` screenshot, `E` start or stop recording, `R` rotate the phone.
  `Esc` is Back, `Home` is Home, `End` is Recents.
- **Move** the window by dragging the bezel. **Resize** with Cmd or Ctrl and scroll.
- Screenshots are saved to `~/Pictures/handset`, recordings to `~/Movies/handset`, and both are copied to
  the clipboard. Click the toast to show the file in Finder.

## Options

| Option | What it does |
|---|---|
| `--frame <name\|path\|none>` | Built-in frame, your own PNG (with a `.frame` file), or no frame |
| `--max-size <px>` | Limit the video size (default 1920) |
| `--audio` | Capture the phone's audio and play it here |
| `--mute` | With `--audio`, record the audio but do not play it |
| `--gain <dB>`, `--no-normalize` | Recorded audio is normalised to about -16 LUFS; adjust or turn off |
| `--no-clipboard` | Do not copy captures to the clipboard |
| `--record-for <seconds>` | Record from the first frame, then quit |
| `--record-format mp4\|gif` | Recording format |
| `--record-size <px>` | Longest side of recordings (default 1280) |
| `--record-bg #rrggbb` | Recording background colour |
| `[serial]` | Pick a device when several are connected |

With no arguments, `~/.config/handset/args` is used if it exists. `HANDSET_DEBUG=1` prints diagnostics.
`ADB` and `FFMPEG` override where the tools are found.

## How it works

adb forwards a port to the scrcpy server on the phone. Handset reads the video, audio and control
sockets, decodes H.264 on a video thread, converts and draws the frame on the GPU, and writes touch and key
messages straight back. The reasons behind each choice are in [docs/adr](docs/adr); procedures for
rebuilding the server, releasing and debugging are in [docs/runbooks](docs/runbooks).

## Roadmap

Ideas, not promises:

- Keyboard-to-touch mapping profiles (a long-standing request in the scrcpy world)
- Wireless pairing, a device picker and a connection "doctor" that explains common failures
- OBS-friendly output (Syphon or a virtual camera)
- Back, Home and Recents on the toolbar, and saved settings
- Microphone to phone, Linux and Windows

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md). Please use Conventional Commits.

## Credits and license

Handset embeds the scrcpy server by Genymobile and Romain Vimont (Apache-2.0); see [NOTICE](NOTICE).
Handset is not affiliated with Genymobile. Licensed under Apache-2.0, see [LICENSE](LICENSE).
