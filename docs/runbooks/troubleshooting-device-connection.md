# Troubleshoot a device connection

Use this when Handset does not show the phone, or shows a black or frozen window.

## 1. Is adb seeing the phone?

```sh
adb devices
```

| Output | Meaning | Fix |
|---|---|---|
| nothing listed | no connection | try another cable or port; enable **USB debugging** in Developer options |
| `unauthorized` | the phone has not trusted this Mac | unlock the phone and accept the prompt; if it never appears, revoke USB debugging authorisations and reconnect |
| `offline` | stale adb session | `adb kill-server && adb start-server`, then replug |
| serial and `device` | fine | go to step 2 |

If `adb` itself is missing, install the Android platform-tools. Handset finds `adb` on `PATH`, in
`/opt/homebrew/bin`, `/usr/local/bin` and the Android SDK folder, and the `ADB` environment variable
overrides the lookup.

## 2. Run from a terminal to see the real error

```sh
HANDSET_DEBUG=1 handset
```

Errors are printed to the terminal. When launched from Finder they appear in an alert instead.

## 3. Common causes

- **Mirroring starts but touch does nothing.** Some manufacturers (for example Xiaomi and Oppo
  phones) have a second switch for controlling the device, often called **USB debugging (Security
  settings)**. Turn it on in Developer options.
- **"Could not start mirroring" or a version error.** The embedded server is pinned to one scrcpy
  version. Check `SERVER_VERSION` in `app/src/server.rs` against the asset
  ([rebuild runbook](rebuild-scrcpy-server.md)).
- **Stale port forwards.** List them with `adb forward --list` and clear with
  `adb forward --remove-all`.
- **Black rectangle inside an app.** Apps that mark their window as secure (banking, some streaming)
  are not captured by the server. That is expected.
- **No audio.** Audio is opt-in (`--audio`) and needs Android 11 or later. Without it Handset prints
  that audio is unavailable and carries on.
- **A recording has video but no audio.** Run `HANDSET_DEBUG=1 handset --audio` and look for ffmpeg
  messages. An `Input contains (near) NaN/+-Inf` error means the loudness normaliser produced bad
  samples; Handset guards against this, so please report it with the log if you still see it. As a
  workaround, `--no-normalize` skips the normaliser. Check the file with
  `ffprobe -show_entries stream=codec_type file.mp4`.
- **Recording fails.** Recording needs `ffmpeg` (`brew install ffmpeg`). The `FFMPEG` environment
  variable overrides the lookup.
- **First launch of the app from Finder.** Right-click the app and choose Open the first time (it is
  ad-hoc signed), and allow access to Pictures and Movies when macOS asks.

## 4. Still stuck?

Open a bug report and include the output of `HANDSET_DEBUG=1 handset`, your phone model and Android
version, the output of `adb version`, and what you tried above.
