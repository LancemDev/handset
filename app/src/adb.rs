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

/// Serials of devices in the `device` state.
pub fn devices() -> io::Result<Vec<String>> {
    let out = Command::new(adb_path()).arg("devices").output()?;
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(text
        .lines()
        .skip(1)
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            match (it.next(), it.next()) {
                (Some(s), Some("device")) => Some(s.to_string()),
                _ => None,
            }
        })
        .collect())
}
