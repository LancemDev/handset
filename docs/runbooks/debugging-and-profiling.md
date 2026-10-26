# Debugging and profiling

## Debug output

```sh
HANDSET_DEBUG=1 cargo run --release -- --audio
```

prints input events (clicks, moves, keys, scrolls, toolbar actions), orientation changes and, with
audio, a line every two seconds: milliseconds buffered for playback, milliseconds received, underruns and
dropped audio. A healthy stream shows tens of milliseconds buffered, zero underruns and zero dropped.

## Hands-free recording

`--record-for <seconds>` starts recording on the first frame, stops after that time and quits. It is the
easiest way to test the video and audio path without touching the keyboard or mouse:

```sh
handset --audio --mute --no-clipboard --record-for 5
```

Check the result with `ffprobe` and, for audio, `ffmpeg -i file.mp4 -vn -af volumedetect -f null -`.

## Looking at the window without capturing the whole screen

Capture only the app's window, never the full screen. Find its window id (for example with
`CGWindowListCopyWindowInfo` in a small Swift script keyed on the process id) and run:

```sh
screencapture -x -l <window id> window.png
```

## Memory and CPU

```sh
footprint <pid>        # physical footprint, close to Activity Monitor's Memory column
ps -o rss=,pcpu= -p <pid>
```

Resident memory (`rss`) counts shared system pages and is higher than the footprint; graphics memory
in `footprint` varies from run to run, so compare several readings. Reference numbers on an Apple Silicon
Mac with a 720x1600 phone: footprint about 65 to 70 MB, resident about 110 MB, and about 15 to 20 percent
of one core while the screen scrolls continuously.

## Tests

```sh
cd app
cargo test
cargo test -- --ignored clipboard    # overwrites the real clipboard, run on purpose
```

## Rules for automated testing on a developer's machine

- Do not synthesise mouse or keyboard events or capture the full screen without the owner's say-so.
- Restore anything you change on the phone (rotation, volume, apps) and delete test captures from
  `~/Pictures/handset` and `~/Movies/handset`.
