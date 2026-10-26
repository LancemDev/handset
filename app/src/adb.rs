//! Thin wrapper around the `adb` executable.

use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn adb_path() -> PathBuf {
    crate::tools::find("adb")
}

pub fn command(serial: &str) -> Command {
    let mut c = Command::new(adb_path());
    c.args(["-s", serial]);
    c
}

pub fn run(serial: &str, args: &[&str]) -> io::Result<String> {
    let out = command(serial).args(args).stdin(Stdio::null()).output()?;
    if !out.status.success() {
        return Err(io::Error::other(format!(
            "adb {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Every device adb lists, with its state (`device`, `unauthorized`, `offline`, ...).
pub fn list() -> io::Result<Vec<(String, String)>> {
    let out = Command::new(adb_path()).arg("devices").output()?;
    Ok(parse_devices(&String::from_utf8_lossy(&out.stdout)))
}

/// Serials of devices in the `device` state.
pub fn devices() -> io::Result<Vec<String>> {
    Ok(list()?.into_iter().filter(|(_, state)| state == "device").map(|(serial, _)| serial).collect())
}

fn parse_devices(text: &str) -> Vec<(String, String)> {
    text.lines()
        .skip(1)
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            Some((it.next()?.to_string(), it.next()?.to_string()))
        })
        .collect()
}

/// Why no device could be used, in words for the person at the keyboard.
pub fn no_device_message(listed: &[(String, String)]) -> String {
    let help = "Connect your phone over USB, turn on USB debugging and accept the prompt on the phone, \
                or connect over Wi-Fi with `handset --connect <ip:port>`.";
    if let Some((serial, _)) = listed.iter().find(|(_, s)| s == "unauthorized") {
        return format!(
            "The phone ({serial}) has not authorised this Mac.\n\nUnlock it and tap Allow on the \"Allow USB debugging?\" prompt. If there is no prompt, unplug the cable, \
             turn USB debugging off and on in Developer options, and plug it in again."
        );
    }
    if let Some((serial, _)) = listed.iter().find(|(_, s)| s == "offline") {
        return format!(
            "The phone ({serial}) is listed as offline.\n\nUnplug and replug the cable, or run `adb kill-server` and try again."
        );
    }
    if let Some((serial, _)) = listed.iter().find(|(_, s)| s.contains("permissions")) {
        return format!("This Mac cannot access the phone ({serial}): adb reports a permissions problem.\n\n{help}");
    }
    format!("No Android device found.\n\n{help} Handset needs adb (Android platform-tools) installed.")
}

/// `host` or `host:port` as `host:port`, with adb's default port when none is given.
pub fn normalize_addr(addr: &str) -> String {
    if addr.contains(':') { addr.to_string() } else { format!("{addr}:5555") }
}

/// `adb connect`: returns the serial to use afterwards.
pub fn connect(addr: &str) -> io::Result<String> {
    let addr = normalize_addr(addr);
    let out = Command::new(adb_path()).args(["connect", &addr]).stdin(Stdio::null()).output()?;
    parse_connect(&String::from_utf8_lossy(&out.stdout), &String::from_utf8_lossy(&out.stderr), &addr)
}

fn parse_connect(stdout: &str, stderr: &str, addr: &str) -> io::Result<String> {
    let text = format!("{} {}", stdout.trim(), stderr.trim());
    let lower = text.to_lowercase();
    if lower.contains("connected to") && !lower.contains("failed") && !lower.contains("cannot") {
        return Ok(addr.to_string());
    }
    let mut reason = text.trim().to_string();
    if reason.is_empty() {
        reason = "no answer from adb".to_string();
    }
    Err(io::Error::other(format!(
        "Could not connect to {addr}: {reason}\n\nCheck that the phone and this Mac are on the same Wi-Fi, and that Wireless \
         debugging is on (Developer options). On Android 11 and later, pair first with `handset --pair <ip:port> <code>`."
    )))
}

/// `adb pair host:port code`, as shown under Developer options > Wireless debugging > Pair device.
pub fn pair(addr: &str, code: &str) -> io::Result<String> {
    let out = Command::new(adb_path()).args(["pair", addr, code]).stdin(Stdio::null()).output()?;
    let text =
        format!("{} {}", String::from_utf8_lossy(&out.stdout).trim(), String::from_utf8_lossy(&out.stderr).trim());
    if text.to_lowercase().contains("successfully paired") {
        Ok(text.trim().to_string())
    } else {
        Err(io::Error::other(format!(
            "Pairing with {addr} failed: {}\n\nUse the address and the six digit code from the \"Pair device with pairing code\" \
             screen (the pairing port differs from the connection port), and finish within the time shown.",
            text.trim()
        )))
    }
}

/// Switch a USB-connected phone to adb over Wi-Fi and connect to it. Returns the new serial.
pub fn wifi(usb_serial: &str) -> io::Result<String> {
    let route = run(usb_serial, &["shell", "ip", "route", "get", "1.1.1.1"]).unwrap_or_default();
    let ip = parse_route_src(&route)
        .or_else(|| {
            run(usb_serial, &["shell", "ip", "-f", "inet", "addr", "show", "wlan0"]).ok().and_then(|t| parse_inet(&t))
        })
        .ok_or_else(|| io::Error::other("Could not find the phone's Wi-Fi address. Is it connected to Wi-Fi?"))?;
    run(usb_serial, &["tcpip", "5555"])?;
    // adb restarts on the phone; give it a moment, then retry the connection a few times
    let addr = format!("{ip}:5555");
    let mut last = None;
    for _ in 0..6 {
        std::thread::sleep(std::time::Duration::from_millis(800));
        match connect(&addr) {
            Ok(s) => return Ok(s),
            Err(e) => last = Some(e),
        }
    }
    Err(last.unwrap_or_else(|| io::Error::other("could not connect")))
}

/// `ip route get 1.1.1.1` output: `... src 192.168.1.23 ...`.
fn parse_route_src(text: &str) -> Option<String> {
    let mut it = text.split_whitespace();
    while let Some(w) = it.next() {
        if w == "src" {
            return it.next().filter(|ip| ip.parse::<std::net::Ipv4Addr>().is_ok()).map(str::to_string);
        }
    }
    None
}

/// `ip -f inet addr show wlan0` output: `inet 192.168.1.23/24 ...`.
fn parse_inet(text: &str) -> Option<String> {
    let mut it = text.split_whitespace();
    while let Some(w) = it.next() {
        if w == "inet" {
            return it
                .next()
                .and_then(|a| a.split('/').next())
                .filter(|ip| ip.parse::<std::net::Ipv4Addr>().is_ok())
                .map(str::to_string);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str =
        "List of devices attached\nW82202003817\tdevice\n192.168.1.5:5555\toffline\nR58M\tunauthorized\n\n";

    #[test]
    fn parses_device_states() {
        assert_eq!(
            parse_devices(LIST),
            vec![
                ("W82202003817".into(), "device".into()),
                ("192.168.1.5:5555".into(), "offline".into()),
                ("R58M".into(), "unauthorized".into())
            ]
        );
        assert!(parse_devices("List of devices attached\n\n").is_empty());
    }

    #[test]
    fn explains_each_failure() {
        let unauthorized = no_device_message(&[("R58M".into(), "unauthorized".into())]);
        assert!(unauthorized.contains("Allow"), "{unauthorized}");
        let offline = no_device_message(&[("R58M".into(), "offline".into())]);
        assert!(offline.contains("offline"), "{offline}");
        let none = no_device_message(&[]);
        assert!(none.contains("--connect"), "{none}");
    }

    #[test]
    fn adds_the_default_port() {
        assert_eq!(normalize_addr("192.168.1.5"), "192.168.1.5:5555");
        assert_eq!(normalize_addr("192.168.1.5:37123"), "192.168.1.5:37123");
    }

    #[test]
    fn reads_connect_results() {
        assert_eq!(parse_connect("connected to 10.0.0.2:5555\n", "", "10.0.0.2:5555").unwrap(), "10.0.0.2:5555");
        assert!(parse_connect("already connected to 10.0.0.2:5555", "", "10.0.0.2:5555").is_ok());
        let err =
            parse_connect("failed to connect to '10.0.0.2:5555': Connection refused", "", "10.0.0.2:5555").unwrap_err();
        assert!(err.to_string().contains("same Wi-Fi"), "{err}");
        assert!(parse_connect("", "", "10.0.0.2:5555").is_err());
    }

    #[test]
    fn finds_the_wifi_address() {
        assert_eq!(
            parse_route_src("1.1.1.1 via 192.168.1.1 dev wlan0 src 192.168.1.23 uid 2000 \n    cache"),
            Some("192.168.1.23".into())
        );
        assert_eq!(parse_route_src("RTNETLINK answers: Network is unreachable"), None);
        assert_eq!(
            parse_inet("    inet 192.168.1.23/24 brd 192.168.1.255 scope global wlan0"),
            Some("192.168.1.23".into())
        );
        assert_eq!(parse_inet("Device \"wlan0\" does not exist."), None);
    }
}
