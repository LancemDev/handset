# Handset

Mirror and control an Android phone from your Mac, inside a realistic device frame.

Handset is a small native Rust app. It talks to the unmodified
[scrcpy](https://github.com/Genymobile/scrcpy) server on the phone and replaces the desktop
client, so it can draw things a plain window cannot: phone bezels, a transparent window,
overlays and framed screenshots and recordings.

> **Status: early work in progress.** Expect rough edges and missing pieces. macOS is the only
> platform being tested.

## Planned

- [ ] Connect over adb and start the scrcpy server
- [ ] Decode and render the video stream
- [ ] Phone frames (bezels) around the screen
- [ ] Touch, scroll and keyboard input
- [ ] Device audio, played locally and added to recordings
- [ ] Framed screenshots and recordings
- [ ] Native menu and toolbar
- [ ] macOS app bundle

## License

Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
