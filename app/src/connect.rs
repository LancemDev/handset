//! The "connect a phone" screen. While no phone is ready it is drawn inside the phone frame, as if it
//! were the phone's own screen: it says what Handset sees (nothing yet, a phone waiting for
//! permission, a stale connection) and what to do about it, and goes away by itself once a phone is
//! ready. The picture is rasterized on the CPU with the overlay's bitmap font and handed to the
//! renderer like a video frame.

use crate::adb;
use crate::overlay::glyph;
use crate::render::Yuv;
use crate::stream::UserEvent;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use winit::event_loop::EventLoopProxy;

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    /// Nothing to talk to yet.
    Looking,
    /// A phone is plugged in but has not allowed this Mac.
    Unauthorized(String),
    Offline(String),
    Permissions(String),
    AdbMissing,
    /// A phone is ready and mirroring is starting.
    Connecting(String),
    /// Starting failed; the text says why.
    Failed(String),
}

/// The phone to mirror: the one named `want` if given, else the first in the `device` state.
pub fn pick(list: &[(String, String)], want: Option<&str>) -> Option<String> {
    list.iter()
        .find(|(serial, state)| state == "device" && want.is_none_or(|w| w == serial))
        .map(|(serial, _)| serial.clone())
}

/// What to tell the person when no phone in `list` is ready.
pub fn status_from(list: &[(String, String)], want: Option<&str>) -> Status {
    let relevant = |serial: &String| want.is_none_or(|w| w == serial);
    let find =
        |pred: &dyn Fn(&str) -> bool| list.iter().find(|(s, st)| relevant(s) && pred(st)).map(|(s, _)| s.clone());
    if let Some(s) = find(&|st| st == "unauthorized") {
        Status::Unauthorized(s)
    } else if let Some(s) = find(&|st| st == "offline") {
        Status::Offline(s)
    } else if let Some(s) = find(&|st| st.contains("permissions")) {
        Status::Permissions(s)
    } else {
        Status::Looking
    }
}

static STOP: AtomicBool = AtomicBool::new(false);

/// Ask adb for the device list every second or so and send it to the app, until `stop_poll`.
pub fn spawn_poll(proxy: EventLoopProxy<UserEvent>) {
    STOP.store(false, Ordering::SeqCst);
    std::thread::spawn(move || {
        while !STOP.load(Ordering::SeqCst) {
            let result = adb::list().map_err(|e| e.to_string());
            if proxy.send_event(UserEvent::Adb(result)).is_err() {
                break;
            }
            std::thread::sleep(Duration::from_millis(1200));
        }
    });
}

pub fn stop_poll() {
    STOP.store(true, Ordering::SeqCst);
}

const BG: [u8; 3] = [16, 20, 26];
const WHITE: [u8; 3] = [245, 245, 247];
const MUTED: [u8; 3] = [150, 158, 170];
const AMBER: [u8; 3] = [255, 179, 0];
const RED: [u8; 3] = [255, 84, 73];
const GREEN: [u8; 3] = [48, 221, 129];

struct Canvas {
    w: u32,
    h: u32,
    px: Vec<u8>,
}

impl Canvas {
    fn new(w: u32, h: u32) -> Self {
        let mut px = Vec::with_capacity((w * h * 3) as usize);
        for _ in 0..w * h {
            px.extend_from_slice(&BG);
        }
        Self { w, h, px }
    }

    fn put(&mut self, x: u32, y: u32, c: [u8; 3]) {
        if x < self.w && y < self.h {
            let i = ((y * self.w + x) * 3) as usize;
            self.px[i..i + 3].copy_from_slice(&c);
        }
    }

    fn text(&mut self, x: u32, y: u32, s: u32, text: &str, c: [u8; 3]) {
        for (n, ch) in text.chars().enumerate() {
            let gx = x + n as u32 * 6 * s;
            for (row, bits) in glyph(ch).iter().enumerate() {
                for col in 0..5u32 {
                    if bits & (1 << (4 - col)) != 0 {
                        for dy in 0..s {
                            for dx in 0..s {
                                self.put(gx + col * s + dx, y + row as u32 * s + dy, c);
                            }
                        }
                    }
                }
            }
        }
    }

    fn dot(&mut self, cx: u32, cy: u32, r: u32, c: [u8; 3]) {
        for y in cy.saturating_sub(r)..=cy + r {
            for x in cx.saturating_sub(r)..=cx + r {
                let (dx, dy) = (x as f32 - cx as f32, y as f32 - cy as f32);
                if dx * dx + dy * dy <= (r * r) as f32 {
                    self.put(x, y, c);
                }
            }
        }
    }
}

/// Break `text` into lines of at most `max` characters, on spaces.
fn wrap(text: &str, max: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > max {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

struct Words {
    /// Status line, its colour, and whether it ends in animated dots.
    status: String,
    colour: [u8; 3],
    animate: bool,
    /// Numbered or plain paragraphs, in white.
    body: Vec<String>,
    /// Smaller print at the bottom, in grey.
    hint: String,
    /// Which phone this is about, if any.
    phone: Option<String>,
}

fn words(status: &Status) -> Words {
    let w = |status: &str, colour, animate, body: &[&str], hint: &str, phone: Option<&String>| Words {
        status: status.to_string(),
        colour,
        animate,
        body: body.iter().map(|s| s.to_string()).collect(),
        hint: hint.to_string(),
        phone: phone.cloned(),
    };
    match status {
        Status::Looking => w(
            "LOOKING FOR A PHONE",
            AMBER,
            true,
            &[
                "1 PLUG IT IN WITH A USB CABLE",
                "2 TURN ON USB DEBUGGING IN DEVELOPER OPTIONS",
                "3 TAP ALLOW ON THE PHONE",
            ],
            "NO CABLE? RUN HANDSET --WIFI WITH THE PHONE ON USB, OR HANDSET --CONNECT IP:PORT.",
            None,
        ),
        Status::Unauthorized(s) => w(
            "ALLOW THIS MAC ON THE PHONE",
            AMBER,
            false,
            &["UNLOCK THE PHONE AND TAP ALLOW ON THE ALLOW USB DEBUGGING? PROMPT."],
            "NO PROMPT? UNPLUG THE CABLE, TURN USB DEBUGGING OFF AND ON, AND PLUG IT BACK IN.",
            Some(s),
        ),
        Status::Offline(s) => w(
            "THE PHONE IS OFFLINE",
            RED,
            false,
            &["UNPLUG AND REPLUG THE CABLE."],
            "IF IT STAYS OFFLINE, RUN ADB KILL-SERVER AND TRY AGAIN.",
            Some(s),
        ),
        Status::Permissions(s) => w(
            "NO ACCESS TO THE PHONE",
            RED,
            false,
            &["ADB REPORTS A PERMISSIONS PROBLEM.", "TRY ANOTHER CABLE OR USB PORT."],
            "",
            Some(s),
        ),
        Status::AdbMissing => w(
            "ADB IS MISSING",
            RED,
            false,
            &["HANDSET NEEDS ANDROID PLATFORM-TOOLS."],
            "INSTALL IT WITH: BREW INSTALL --CASK ANDROID-PLATFORM-TOOLS",
            None,
        ),
        Status::Connecting(s) => w("CONNECTING", GREEN, true, &["STARTING MIRRORING."], "", Some(s)),
        Status::Failed(msg) => {
            let short: String = msg.chars().take(140).collect();
            w("COULD NOT START MIRRORING", RED, false, &[&short], "REPLUG THE PHONE TO TRY AGAIN.", None)
        }
    }
}

/// The screen as RGB, `w` by `h` pixels. `tick` drives the animation (one step every half second).
pub fn render(w: u32, h: u32, status: &Status, tick: u32) -> Vec<u8> {
    let mut c = Canvas::new(w, h);
    let s = (w / 115).max(1);
    let margin = 9 * s;
    let max_chars = ((w.saturating_sub(2 * margin)) / (6 * s)).max(8) as usize;
    let line_h = 11 * s;
    let words = words(status);

    let mut y = h / 8;
    c.text(margin, y, s, "CONNECT A PHONE", WHITE);
    y += 2 * line_h;

    let dots = if words.animate { ".".repeat((tick % 4) as usize) } else { String::new() };
    let lit = !words.animate || tick.is_multiple_of(2);
    let dot_colour = if lit { words.colour } else { [words.colour[0] / 3, words.colour[1] / 3, words.colour[2] / 3] };
    c.dot(margin + 5 * s / 2, y + 7 * s / 2, 5 * s / 2, dot_colour);
    let status_text = format!("{}{}", words.status, dots);
    let status_chars = max_chars.saturating_sub(3);
    for (i, line) in wrap(&words.status, status_chars).iter().enumerate() {
        let shown =
            if i == 0 && wrap(&words.status, status_chars).len() == 1 { status_text.clone() } else { line.clone() };
        c.text(margin + 8 * s, y + i as u32 * line_h, s, &shown, words.colour);
    }
    y += wrap(&words.status, status_chars).len() as u32 * line_h + line_h;

    for para in &words.body {
        for line in wrap(para, max_chars) {
            c.text(margin, y, s, &line, WHITE);
            y += line_h;
        }
        y += line_h / 2;
    }
    if let Some(phone) = &words.phone {
        y += line_h / 2;
        for line in wrap(&format!("PHONE {phone}"), max_chars) {
            c.text(margin, y, s, &line, MUTED);
            y += line_h;
        }
    }

    let hint = wrap(&words.hint, max_chars);
    let mut hy = h.saturating_sub(margin * 3 + hint.len() as u32 * line_h);
    for line in &hint {
        c.text(margin, hy, s, line, MUTED);
        hy += line_h;
    }
    c.px
}

/// RGB to I420 the way the renderer reads it: BT.709, limited range, chroma averaged over 2x2.
pub fn to_yuv(rgb: &[u8], w: u32, h: u32) -> Yuv {
    let (w, h) = (w & !1, h & !1);
    let src = |x: u32, y: u32| {
        let i = ((y * w + x) * 3) as usize;
        [rgb[i] as f32, rgb[i + 1] as f32, rgb[i + 2] as f32]
    };
    let mut yuv = Yuv { width: w, height: h, stride_y: w, stride_uv: w / 2, ..Default::default() };
    yuv.y = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let [r, g, b] = src(x, y);
            let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            yuv.y.push((16.0 + luma * 219.0 / 255.0).round().clamp(16.0, 235.0) as u8);
        }
    }
    for y in (0..h).step_by(2) {
        for x in (0..w).step_by(2) {
            let mut avg = [0.0f32; 3];
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let p = src(x + dx, y + dy);
                for k in 0..3 {
                    avg[k] += p[k] / 4.0;
                }
            }
            let [r, g, b] = avg;
            let cb = -0.1146 * r - 0.3854 * g + 0.5 * b;
            let cr = 0.5 * r - 0.4542 * g - 0.0458 * b;
            yuv.u.push((128.0 + cb * 224.0 / 255.0).round().clamp(16.0, 240.0) as u8);
            yuv.v.push((128.0 + cr * 224.0 / 255.0).round().clamp(16.0, 240.0) as u8);
        }
    }
    yuv
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(items: &[(&str, &str)]) -> Vec<(String, String)> {
        items.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
    }

    #[test]
    fn picks_a_ready_phone() {
        let list = l(&[("A", "unauthorized"), ("B", "device"), ("C", "device")]);
        assert_eq!(pick(&list, None), Some("B".into()));
        assert_eq!(pick(&list, Some("C")), Some("C".into()));
        assert_eq!(pick(&list, Some("A")), None);
        assert_eq!(pick(&[], None), None);
    }

    #[test]
    fn describes_what_is_wrong() {
        assert_eq!(status_from(&[], None), Status::Looking);
        assert_eq!(status_from(&l(&[("A", "unauthorized")]), None), Status::Unauthorized("A".into()));
        assert_eq!(status_from(&l(&[("A", "offline")]), None), Status::Offline("A".into()));
        assert_eq!(status_from(&l(&[("A", "no permissions (x)")]), None), Status::Permissions("A".into()));
        // a phone the person did not ask for is not relevant
        assert_eq!(status_from(&l(&[("A", "unauthorized")]), Some("B")), Status::Looking);
    }

    #[test]
    fn wraps_on_spaces() {
        assert_eq!(wrap("ONE TWO THREE", 7), vec!["ONE TWO", "THREE"]);
        assert_eq!(wrap("TOOLONGWORD", 4), vec!["TOOLONGWORD"]);
        assert!(wrap("", 10).is_empty());
    }

    #[test]
    fn white_converts_to_limited_range_white() {
        let (w, h) = (4, 4);
        let y = to_yuv(&vec![255u8; (w * h * 3) as usize], w, h);
        assert!(y.y.iter().all(|&v| v == 235));
        assert!(y.u.iter().all(|&v| v == 128) && y.v.iter().all(|&v| v == 128));
        assert_eq!((y.u.len(), y.stride_uv), (4, 2));
    }

    /// Writes every state to `$HANDSET_TEST_OUT/connect-<n>.png` (if set) so they can be looked at.
    #[test]
    fn renders_every_state() {
        let states = [
            Status::Looking,
            Status::Unauthorized("W82202003817".into()),
            Status::Offline("W82202003817".into()),
            Status::Permissions("W82202003817".into()),
            Status::AdbMissing,
            Status::Connecting("W82202003817".into()),
            Status::Failed("Could not start mirroring: adb push: no such file".into()),
        ];
        for (n, s) in states.iter().enumerate() {
            let (w, h) = (720, 1560);
            let rgb = render(w, h, s, 1);
            assert_eq!(rgb.len(), (w * h * 3) as usize);
            if let Some(dir) = std::env::var_os("HANDSET_TEST_OUT") {
                let path = std::path::Path::new(&dir).join(format!("connect-{n}.png"));
                let mut enc = png::Encoder::new(std::fs::File::create(path).unwrap(), w, h);
                enc.set_color(png::ColorType::Rgb);
                enc.set_depth(png::BitDepth::Eight);
                enc.write_header().unwrap().write_image_data(&rgb).unwrap();
            }
        }
    }
}
