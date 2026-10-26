//! Locate helper programs (`adb`, `ffmpeg`). An app started from Finder gets
//! a bare PATH without Homebrew, so well-known install locations are tried too.

use std::path::PathBuf;

/// Find `name`: `$NAME` override first, then PATH, then common install places.
/// Falls back to the bare name so the OS error message names the program.
pub fn find(name: &str) -> PathBuf {
    if let Some(p) = std::env::var_os(name.to_ascii_uppercase()) {
        return p.into();
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let mut dirs: Vec<PathBuf> =
        std::env::var_os("PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    dirs.extend(["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"].map(PathBuf::from));
    dirs.extend(
        ["Library/Android/sdk/platform-tools", "Android/Sdk/platform-tools", ".local/bin"].map(|r| home.join(r)),
    );
    dirs.into_iter().map(|d| d.join(name)).find(|p| p.is_file()).unwrap_or_else(|| name.into())
}

/// Print an error and quit. When there is no terminal to read it (launched
/// from Finder), show it in a dialog instead.
pub fn die(msg: &str) -> ! {
    use std::io::IsTerminal;
    eprintln!("{msg}");
    #[cfg(target_os = "macos")]
    if !std::io::stderr().is_terminal() {
        let text = msg.replace('\\', "\\\\").replace('"', "\\\"");
        let _ = std::process::Command::new("osascript")
            .args(["-e", &format!("display alert \"Handset\" message \"{text}\" as critical")])
            .output();
    }
    std::process::exit(1)
}
