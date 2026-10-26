//! Handset: mirror and control an Android phone inside a realistic device frame.
//!
//! Work in progress: modules are wired up one at a time.
#![allow(dead_code)]

mod adb;
mod tools;

fn main() -> std::io::Result<()> {
    let devices = adb::devices()?;
    if devices.is_empty() {
        eprintln!("no adb device found");
        std::process::exit(1);
    }
    for serial in devices {
        println!("{serial}");
    }
    Ok(())
}
