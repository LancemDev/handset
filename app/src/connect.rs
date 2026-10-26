//! The landing screen. While no phone is ready it is drawn inside the phone frame, as if it were the
//! phone's own screen. It says what Handset sees (nothing yet, a phone waiting for permission, a stale
//! connection), what to do about it, and offers the other ways in: Wi-Fi (phones found on the network, a
//! typed address, switching a USB phone) and QR code pairing. It goes away by itself once a phone is
//! ready. The picture is rasterized on the CPU with the overlay's bitmap font, with clickable buttons
//! whose rectangles are returned alongside it, and handed to the renderer like a video frame.

use crate::adb;
use crate::overlay::glyph;
use crate::render::Yuv;
use crate::stream::UserEvent;
use qrcode::{Color, QrCode};
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

/// Is this adb serial a phone reached over the network (`ip:port` or an mDNS name)?
pub fn is_wireless(serial: &str) -> bool {
    serial.contains(':') || serial.contains("_adb-tls")
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Home,
    Wifi,
    Qr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Open(View),
    Back,
    /// Connect to `ip:port`.
    Connect(String),
    TypeAddress,
    /// Switch this USB phone to adb over Wi-Fi.
    SwitchToWifi(String),
}

pub struct Button {
    /// x, y, width, height in the picture's pixels.
    pub rect: [u32; 4],
    pub label: String,
    pub action: Action,
}

/// Results of work done off the main thread.
#[derive(Debug)]
pub enum LandingEvent {
    /// Phones announcing themselves on the network.
    Nearby(Vec<adb::Service>),
    /// `adb pair` finished: the phone's IP, or why not.
    Paired(Result<String, String>),
    /// `adb connect` or the Wi-Fi switch finished: the serial to mirror, or why not.
    Connected(Result<String, String>),
    /// The address typed into the prompt, or `None` if it was cancelled.
    Typed(Option<String>),
}

static STOP: AtomicBool = AtomicBool::new(false);
static MDNS: AtomicBool = AtomicBool::new(false);

/// Ask adb for the device list every second or so and send it to the app, until `stop_poll`. While the
/// Wi-Fi or QR views are open it also asks which phones are announcing themselves on the network.
pub fn spawn_poll(proxy: EventLoopProxy<UserEvent>) {
    STOP.store(false, Ordering::SeqCst);
    std::thread::spawn(move || {
        while !STOP.load(Ordering::SeqCst) {
            let result = adb::list().map_err(|e| e.to_string());
            if proxy.send_event(UserEvent::Adb(result)).is_err() {
                break;
            }
            if MDNS.load(Ordering::SeqCst)
                && let Ok(services) = adb::mdns_services()
                && proxy.send_event(UserEvent::Landing(LandingEvent::Nearby(services))).is_err()
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(1200));
        }
    });
}

pub fn stop_poll() {
    STOP.store(true, Ordering::SeqCst);
}

/// Turn the network lookup on or off (only the Wi-Fi and QR views need it).
pub fn want_mdns(on: bool) {
    MDNS.store(on, Ordering::SeqCst);
}

fn send(proxy: &EventLoopProxy<UserEvent>, e: LandingEvent) {
    let _ = proxy.send_event(UserEvent::Landing(e));
}

pub fn connect_async(proxy: EventLoopProxy<UserEvent>, addr: String) {
    std::thread::spawn(move || send(&proxy, LandingEvent::Connected(adb::connect(&addr).map_err(|e| e.to_string()))));
}

pub fn wifi_async(proxy: EventLoopProxy<UserEvent>, usb_serial: String) {
    std::thread::spawn(move || {
        send(&proxy, LandingEvent::Connected(adb::wifi(&usb_serial).map_err(|e| e.to_string())))
    });
}

pub fn pair_async(proxy: EventLoopProxy<UserEvent>, addr: String, password: String) {
    std::thread::spawn(move || {
        let ip = addr.split(':').next().unwrap_or_default().to_string();
        send(&proxy, LandingEvent::Paired(adb::pair(&addr, &password).map(|_| ip).map_err(|e| e.to_string())));
    });
}

/// Ask for a phone address with a native prompt (the landing screen has no text field).
pub fn ask_address_async(proxy: EventLoopProxy<UserEvent>) {
    std::thread::spawn(move || {
        let typed = ask_address();
        send(&proxy, LandingEvent::Typed(typed));
    });
}

#[cfg(target_os = "macos")]
fn ask_address() -> Option<String> {
    let script = "text returned of (display dialog \"Phone address, as ip or ip:port (shown under Wireless debugging):\" \
                  default answer \"192.168.\" buttons {\"Cancel\", \"Connect\"} default button \"Connect\" with title \"Handset\")";
    let out = std::process::Command::new("osascript").args(["-e", script]).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string()).filter(|s| !s.is_empty())
}

#[cfg(not(target_os = "macos"))]
fn ask_address() -> Option<String> {
    None
}

/// A QR code that pairs a phone: scanning it on the phone (Wireless debugging > Pair device with QR code)
/// makes the phone announce a pairing service called `name`, which `password` unlocks.
pub struct Qr {
    pub name: String,
    pub password: String,
    pub width: usize,
    pub dark: Vec<bool>,
}

impl Qr {
    pub fn new() -> Result<Qr, String> {
        let name = format!("handset-{}", random_alnum(6).to_lowercase());
        let password = random_alnum(10);
        let code = QrCode::new(payload(&name, &password).as_bytes()).map_err(|e| e.to_string())?;
        let dark = code.to_colors().into_iter().map(|c| c == Color::Dark).collect();
        Ok(Qr { name, password, width: code.width(), dark })
    }
}

/// What the phone's QR scanner reads.
fn payload(name: &str, password: &str) -> String {
    format!("WIFI:T:ADB;S:{name};P:{password};;")
}

fn random_alnum(n: usize) -> String {
    use std::io::Read;
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnpqrstuvwxyz23456789";
    let mut bytes = vec![0u8; n];
    if std::fs::File::open("/dev/urandom").and_then(|mut f| f.read_exact(&mut bytes)).is_err() {
        // no system randomness: fall back to the clock, which is enough for a one-off pairing code
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = (t >> (i * 5 % 100)) as u8 ^ (i as u8).wrapping_mul(37);
        }
    }
    bytes.iter().map(|b| CHARS[*b as usize % CHARS.len()] as char).collect()
}

const BG: [u8; 3] = [16, 20, 26];
const WHITE: [u8; 3] = [245, 245, 247];
const MUTED: [u8; 3] = [150, 158, 170];
const AMBER: [u8; 3] = [255, 179, 0];
const RED: [u8; 3] = [255, 84, 73];
const GREEN: [u8; 3] = [48, 221, 129];
const BUTTON: [u8; 3] = [30, 38, 50];
const BUTTON_HOVER: [u8; 3] = [48, 60, 80];
const BUTTON_EDGE: [u8; 3] = [90, 104, 126];

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

    fn rect(&mut self, x: u32, y: u32, w: u32, h: u32, c: [u8; 3]) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.put(xx, yy, c);
            }
        }
    }

    /// A rounded button with a border and a centred label.
    fn button(&mut self, rect: [u32; 4], s: u32, label: &str, hover: bool) {
        let [x, y, w, h] = rect;
        let r = 3 * s;
        let fill = if hover { BUTTON_HOVER } else { BUTTON };
        let edge = if hover { WHITE } else { BUTTON_EDGE };
        for yy in 0..h {
            for xx in 0..w {
                // how far outside the rounded corner this pixel's corner square reaches, 0 away from corners
                let cx = if xx < r {
                    r - xx
                } else if xx >= w - r {
                    xx - (w - r) + 1
                } else {
                    0
                };
                let cy = if yy < r {
                    r - yy
                } else if yy >= h - r {
                    yy - (h - r) + 1
                } else {
                    0
                };
                let corner = cx > 0 && cy > 0;
                if corner && cx * cx + cy * cy > r * r {
                    continue;
                }
                let border =
                    xx < s || yy < s || xx >= w - s || yy >= h - s || (corner && cx * cx + cy * cy > (r - s) * (r - s));
                self.put(x + xx, y + yy, if border { edge } else { fill });
            }
        }
        // the label shrinks (down to scale 3) until it fits, then is cut
        let n = label.chars().count() as u32;
        let ls = (3..=s).rev().find(|ls| (n * 6 * ls).saturating_sub(*ls) + 8 * s <= w).unwrap_or(3.min(s));
        let fit = ((w.saturating_sub(4 * s)) / (6 * ls)).max(1) as usize;
        let label: String = label.chars().take(fit).collect();
        let tw = (label.chars().count() as u32 * 6 * ls).saturating_sub(ls);
        self.text(x + w.saturating_sub(tw) / 2, y + h.saturating_sub(7 * ls) / 2, ls, &label, WHITE);
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
    /// Smaller print, in grey.
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
            "",
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

/// Everything the screen shows.
pub struct Model<'a> {
    pub status: &'a Status,
    pub view: View,
    /// Animation step, one per half second.
    pub tick: u32,
    /// The button under the cursor.
    pub hover: Option<usize>,
    /// Phones announcing themselves that can be connected to.
    pub nearby: &'a [adb::Service],
    /// A USB phone that is ready but waiting in the Wi-Fi view, to offer to switch it.
    pub usb: Option<&'a str>,
    pub qr: Option<&'a Qr>,
    /// A line of progress or trouble, shown in the Wi-Fi and QR views.
    pub note: Option<&'a str>,
}

pub struct Screen {
    pub rgb: Vec<u8>,
    pub buttons: Vec<Button>,
}

/// The screen as RGB, `w` by `h` pixels, and where its buttons are.
pub fn render(w: u32, h: u32, m: &Model) -> Screen {
    let mut c = Canvas::new(w, h);
    let s = (w / 115).max(1);
    let margin = 9 * s;
    let max_chars = ((w.saturating_sub(2 * margin)) / (6 * s)).max(8) as usize;
    let line_h = 11 * s;
    let (btn_h, gap) = (15 * s, 4 * s);
    let btn_w = w.saturating_sub(2 * margin);
    let mut buttons: Vec<Button> = Vec::new();

    let mut y = h / 11;
    let title = match m.view {
        View::Home => "CONNECT A PHONE",
        View::Wifi => "PHONES ON WI-FI",
        View::Qr => "SCAN TO PAIR",
    };
    c.text(margin, y, s, title, WHITE);
    y += 2 * line_h;

    // bottom buttons are stacked upwards from the bottom edge; `bottom` is the top of the stack
    let mut bottom = h.saturating_sub(margin * 3);
    let mut stack = |label: &str, action: Action, buttons: &mut Vec<Button>| {
        bottom = bottom.saturating_sub(btn_h);
        buttons.push(Button { rect: [margin, bottom, btn_w, btn_h], label: label.to_string(), action });
        bottom = bottom.saturating_sub(gap);
    };

    match m.view {
        View::Home => {
            stack("QR CODE", Action::Open(View::Qr), &mut buttons);
            stack("WI-FI", Action::Open(View::Wifi), &mut buttons);
            let words = words(m.status);
            let dots = if words.animate { ".".repeat((m.tick % 4) as usize) } else { String::new() };
            let lit = !words.animate || m.tick.is_multiple_of(2);
            let dot_colour =
                if lit { words.colour } else { [words.colour[0] / 3, words.colour[1] / 3, words.colour[2] / 3] };
            c.dot(margin + 5 * s / 2, y + 7 * s / 2, 5 * s / 2, dot_colour);
            let status_lines = wrap(&words.status, max_chars.saturating_sub(3));
            for (i, line) in status_lines.iter().enumerate() {
                let shown = if i + 1 == status_lines.len() { format!("{line}{dots}") } else { line.clone() };
                c.text(margin + 8 * s, y + i as u32 * line_h, s, &shown, words.colour);
            }
            y += status_lines.len() as u32 * line_h + line_h;
            for para in &words.body {
                for line in wrap(para, max_chars) {
                    c.text(margin, y, s, &line, WHITE);
                    y += line_h;
                }
                y += line_h / 2;
            }
            if let Some(phone) = &words.phone {
                for line in wrap(&format!("PHONE {phone}"), max_chars) {
                    c.text(margin, y, s, &line, MUTED);
                    y += line_h;
                }
            }
            // small print only while it clears the buttons and their lead-in
            let room = bottom.saturating_sub(line_h * 2 + 7 * s);
            for line in wrap(&words.hint, max_chars) {
                if y + line_h > room {
                    break;
                }
                c.text(margin, y, s, &line, MUTED);
                y += line_h;
            }
            c.text(margin, bottom.saturating_sub(line_h / 2 + 7 * s), s, "NO CABLE?", MUTED);
        }
        View::Wifi => {
            stack("BACK", Action::Back, &mut buttons);
            if let Some(usb) = m.usb {
                let short: String = usb.chars().take(max_chars.saturating_sub(10)).collect();
                stack(&format!("SWITCH {short} TO WI-FI"), Action::SwitchToWifi(usb.to_string()), &mut buttons);
            }
            stack("TYPE AN ADDRESS", Action::TypeAddress, &mut buttons);
            for line in wrap("TAP A PAIRED PHONE TO CONNECT:", max_chars) {
                c.text(margin, y, s, &line, MUTED);
                y += line_h;
            }
            y += line_h / 2;
            let connectable: Vec<&adb::Service> =
                m.nearby.iter().filter(|n| n.kind == adb::ServiceKind::Connect).take(2).collect();
            if connectable.is_empty() {
                let dots = ".".repeat((m.tick % 4) as usize);
                c.text(margin, y, s, &format!("NONE FOUND YET{dots}"), AMBER);
                y += line_h + line_h / 2;
                for line in wrap(
                    "TURN ON WIRELESS DEBUGGING IN DEVELOPER OPTIONS. A NEW PHONE NEEDS PAIRING ONCE: USE QR CODE.",
                    max_chars,
                ) {
                    c.text(margin, y, s, &line, MUTED);
                    y += line_h;
                }
            }
            for n in connectable {
                buttons.push(Button {
                    rect: [margin, y, btn_w, btn_h],
                    label: n.addr.clone(),
                    action: Action::Connect(n.addr.clone()),
                });
                y += btn_h + gap;
            }
            // notes sit just above the buttons: progress or trouble first, then the USB hint above it
            let mut ny = bottom;
            if let Some(note) = m.note {
                let lines: Vec<String> = wrap(note, max_chars).into_iter().take(3).collect();
                ny = ny.saturating_sub(lines.len() as u32 * line_h + line_h / 2);
                for (k, line) in lines.iter().enumerate() {
                    c.text(margin, ny + k as u32 * line_h, s, line, AMBER);
                }
            }
            if m.usb.is_some() {
                let hint = wrap("A USB PHONE IS PLUGGED IN.", max_chars);
                ny = ny.saturating_sub(hint.len() as u32 * line_h + line_h / 2);
                for (k, line) in hint.iter().enumerate() {
                    c.text(margin, ny + k as u32 * line_h, s, line, MUTED);
                }
            }
        }
        View::Qr => {
            stack("BACK", Action::Back, &mut buttons);
            if let Some(qr) = m.qr {
                let quiet = 3;
                let modules = qr.width as u32 + 2 * quiet;
                // leave room below for the steps and the status
                let side = w.saturating_sub(2 * margin).min(bottom.saturating_sub(y + 10 * line_h));
                let px = (side / modules).max(1);
                let size = px * modules;
                let x0 = (w - size) / 2;
                c.rect(x0, y, size, size, WHITE);
                for my in 0..qr.width {
                    for mx in 0..qr.width {
                        if qr.dark[my * qr.width + mx] {
                            c.rect(x0 + (mx as u32 + quiet) * px, y + (my as u32 + quiet) * px, px, px, [0, 0, 0]);
                        }
                    }
                }
                y += size + line_h;
            } else {
                c.text(margin, y, s, "COULD NOT MAKE A QR CODE", RED);
                y += 2 * line_h;
            }
            for line in wrap("PHONE: DEVELOPER OPTIONS, WIRELESS DEBUGGING, PAIR WITH QR CODE.", max_chars) {
                c.text(margin, y, s, &line, MUTED);
                y += line_h;
            }
            y += line_h / 2;
            let (text, colour) = match m.note {
                Some(n) => (n.to_string(), AMBER),
                None => (format!("WAITING FOR THE PHONE TO SCAN{}", ".".repeat((m.tick % 4) as usize)), WHITE),
            };
            for line in wrap(&text, max_chars) {
                c.text(margin, y, s, &line, colour);
                y += line_h;
            }
        }
    }

    // read top to bottom, and indexed that way for hover
    buttons.sort_by_key(|b| b.rect[1]);
    for (i, b) in buttons.iter().enumerate() {
        c.button(b.rect, s, &b.label, m.hover == Some(i));
    }
    Screen { rgb: c.px, buttons }
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

    fn svc(kind: adb::ServiceKind, addr: &str) -> adb::Service {
        adb::Service { name: "adb-X".into(), kind, addr: addr.into() }
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
    fn tells_wireless_from_usb() {
        assert!(is_wireless("192.168.0.20:5555"));
        assert!(is_wireless("adb-W82202003817-MPn68n._adb-tls-connect._tcp"));
        assert!(!is_wireless("W82202003817"));
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

    #[test]
    fn the_qr_payload_is_what_android_scans() {
        assert_eq!(payload("handset-abc123", "Secret42"), "WIFI:T:ADB;S:handset-abc123;P:Secret42;;");
        let qr = Qr::new().unwrap();
        assert!(qr.name.starts_with("handset-") && qr.name.len() == 14);
        assert_eq!(qr.password.len(), 10);
        assert_eq!(qr.dark.len(), qr.width * qr.width);
        assert!(qr.dark.iter().any(|&d| d) && qr.dark.iter().any(|&d| !d));
        // a second code is different
        assert_ne!(Qr::new().unwrap().password, qr.password);
    }

    fn model<'a>(
        status: &'a Status,
        view: View,
        nearby: &'a [adb::Service],
        usb: Option<&'a str>,
        qr: Option<&'a Qr>,
        note: Option<&'a str>,
    ) -> Model<'a> {
        Model { status, view, tick: 1, hover: None, nearby, usb, qr, note }
    }

    #[test]
    fn buttons_stay_inside_the_picture_and_do_not_overlap() {
        let (w, h) = (720u32, 1560u32);
        let qr = Qr::new().unwrap();
        let nearby = [
            svc(adb::ServiceKind::Connect, "192.168.0.20:41789"),
            svc(adb::ServiceKind::Connect, "192.168.0.31:40000"),
            svc(adb::ServiceKind::Pairing, "192.168.0.9:1"),
        ];
        let looking = Status::Looking;
        for m in [
            model(&looking, View::Home, &[], None, None, None),
            model(&looking, View::Wifi, &[], None, None, None),
            model(&looking, View::Wifi, &nearby, Some("W82202003817"), None, Some("CONNECTING TO 192.168.0.20...")),
            model(&looking, View::Qr, &[], None, Some(&qr), None),
            model(&looking, View::Qr, &[], None, None, Some("PAIRING...")),
        ] {
            let screen = render(w, h, &m);
            assert_eq!(screen.rgb.len(), (w * h * 3) as usize);
            assert!(!screen.buttons.is_empty());
            for (i, b) in screen.buttons.iter().enumerate() {
                let [x, y, bw, bh] = b.rect;
                assert!(x + bw <= w && y + bh <= h, "{:?} outside", b.action);
                for o in &screen.buttons[i + 1..] {
                    let [ox, oy, ow, oh] = o.rect;
                    let apart = y + bh <= oy || oy + oh <= y || x + bw <= ox || ox + ow <= x;
                    assert!(apart, "{:?} overlaps {:?}", b.action, o.action);
                }
            }
        }
        // the Wi-Fi view lists the connectable phones, not the pairing entry
        let screen = render(w, h, &model(&looking, View::Wifi, &nearby, None, None, None));
        let connects: Vec<_> = screen.buttons.iter().filter(|b| matches!(b.action, Action::Connect(_))).collect();
        assert_eq!(connects.len(), 2);
    }

    /// Writes screens to `$HANDSET_TEST_OUT/connect-<n>.png` (if set) so they can be looked at.
    #[test]
    fn renders_every_screen() {
        let qr = Qr::new().unwrap();
        let nearby = [svc(adb::ServiceKind::Connect, "192.168.0.20:41789")];
        let looking = Status::Looking;
        let unauth = Status::Unauthorized("W82202003817".into());
        let failed = Status::Failed("Could not start mirroring: adb push: no such file".into());
        let screens = [
            model(&looking, View::Home, &[], None, None, None),
            model(&unauth, View::Home, &[], None, None, None),
            model(&failed, View::Home, &[], None, None, None),
            model(&looking, View::Wifi, &[], None, None, None),
            model(&looking, View::Wifi, &nearby, Some("W82202003817"), None, Some("CONNECTING TO 192.168.0.20...")),
            model(&looking, View::Qr, &[], None, Some(&qr), None),
        ];
        for (n, m) in screens.iter().enumerate() {
            let (w, h) = (720, 1560);
            let screen = render(w, h, m);
            if let Some(dir) = std::env::var_os("HANDSET_TEST_OUT") {
                let path = std::path::Path::new(&dir).join(format!("connect-{n}.png"));
                let mut enc = png::Encoder::new(std::fs::File::create(path).unwrap(), w, h);
                enc.set_color(png::ColorType::Rgb);
                enc.set_depth(png::BitDepth::Eight);
                enc.write_header().unwrap().write_image_data(&screen.rgb).unwrap();
            }
        }
    }
}
