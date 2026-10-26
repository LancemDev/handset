# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project aims to follow
[Semantic Versioning](https://semver.org/) once it has a stable release.

## [Unreleased]

### Added

- Mirror and control an Android phone over adb using the embedded, unmodified scrcpy server
- Video decoded with openh264 and drawn with wgpu, with YUV to RGB conversion on the GPU
- Phone frames (`obsidian`, `porcelain`, `hazel` or a custom PNG) on a transparent window, switchable at run time
- Touch, scroll, keyboard and text input, and landscape rotation
- Device audio captured as raw PCM and played locally (`--audio`, `--mute`)
- Framed screenshots and MP4 or GIF recordings with audio through ffmpeg, with loudness normalisation
- Toolbar, native right-click menu, REC indicator and clickable "saved" toast
- Recent captures menu and automatic clipboard copy (`--no-clipboard` to disable)
- `--record-for` for hands-off recordings and `~/.config/handset/args` for default options
- macOS app bundle packaging with a generated icon
- Repository setup: license, contribution and security docs, commit linting, issue and pull request
  templates, CI, ADRs and runbooks
