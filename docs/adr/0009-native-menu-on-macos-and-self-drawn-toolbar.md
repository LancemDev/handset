# 0009. Native menu on macOS and a self-drawn toolbar

- Status: accepted
- Date: 2026-10-26

## Context

Users need a way to switch frames, record, take screenshots and change options without remembering
command-line flags. winit has no menu support, and the window is borderless, so there is no title bar to hang
controls on.

## Decision

- Right-click opens a **native** context menu built with the `muda` crate: frame choice, screenshot,
  recording and its options, recent captures, mute, rotate, toolbar toggle and quit. Selections come back as
  events through the event-loop proxy.
- A small **toolbar** beside the phone gives one-click access to the common actions. It is drawn by our
  own renderer (ADR 0005), only in the window.
- After a screenshot or recording, a **clickable toast** shows the file in Finder, and the capture is
  copied to the clipboard unless turned off.
- The menu is macOS only for now; on other platforms the right-click does nothing.

## Alternatives considered

- **Draw the menu ourselves:** consistent everywhere, but a lot of work to match native behaviour.
- **A separate settings window:** more to build and more to keep in sync.

## Consequences

- The menu blocks the window while it is open, as native menus do.
- Other platforms will need their own menu implementation or a fallback.
- Settings are not persisted yet apart from the optional `~/.config/handset/args` file.
