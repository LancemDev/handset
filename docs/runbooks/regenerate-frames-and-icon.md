# Regenerate the frames and the app icon

The built-in frames and the app icon are generated, then the results are committed.

## Frames

The generator needs Python 3 and Pillow.

```sh
python3 -m venv .venv && .venv/bin/pip install pillow
.venv/bin/python frames/make_frames.py frames/out
```

This writes `frame-<name>.png` and `frame-<name>.frame` for each colourway into `frames/out/`
(ignored by git). The `.frame` file holds `x y w h` of the transparent screen area in pixels.

To ship the result, copy the pairs into `app/assets/`. They are embedded by `app/src/frame.rs`, so a
new built-in name also needs an entry in the `BUILTIN` table there and in the README.

### Making your own frame

1. Draw a PNG (RGBA, 8-bit) of the phone with a fully transparent screen area.
2. Measure the screen rectangle in pixels and save it next to the PNG as `<name>.frame`, containing
   `x y w h` on one line.
3. Run Handset with `--frame path/to/name.png`.

The screen's aspect ratio should match the phone's; otherwise the video is letterboxed inside it.

## App icon

Needs Pillow and macOS (`iconutil`).

```sh
python3 frames/make_icon.py
```

It reads the obsidian frame from `app/assets/` and writes `app/packaging/AppIcon.icns`. The output is
deterministic, so re-running it on an unchanged frame produces an identical file.
