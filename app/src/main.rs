//! Handset: mirror and control an Android phone inside a realistic device frame.
//!
//! Work in progress: modules are wired up one at a time.
#![allow(dead_code)]

mod adb;
mod audio;
mod capture;
mod control;
mod frame;
mod menu;
mod overlay;
mod render;
mod server;
mod stream;
mod toolbar;
mod tools;

use std::time::{Duration, Instant};

fn main() -> std::io::Result<()> {
    let serial = adb::devices()?.into_iter().next().ok_or_else(|| std::io::Error::other("no adb device found"))?;
    println!("device serial: {serial}");

    let mut session = server::start(&serial, &server::Options::default())?;
    println!("connected: {}", session.device_name);

    let start = Instant::now();
    let (mut frames, mut bytes) = (0u32, 0usize);
    while start.elapsed() < Duration::from_secs(5) {
        match server::read_packet(&mut session.video)? {
            server::Packet::Session { width, height } => println!("session: {width}x{height}"),
            server::Packet::Config(data) => println!("config packet: {} bytes", data.len()),
            server::Packet::Frame { data, .. } => {
                frames += 1;
                bytes += data.len();
            }
        }
    }
    println!("{frames} frames, {bytes} bytes in 5s");
    Ok(())
}
