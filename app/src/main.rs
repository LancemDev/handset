mod adb;
mod app;
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

use std::io::Read;
use std::sync::{Arc, Mutex};
use winit::event_loop::EventLoop;

fn usage() -> ! {
    eprintln!(
        "usage: handset [--frame <name|path|none>] [--max-size <px>]\n\
         \x20                  [--audio [--mute] [--gain <dB>] [--no-normalize]] [--no-clipboard] [--record-for <seconds>] [--record-format mp4|gif] [--record-size <px>] [--record-bg #rrggbb] [serial]\n\
         built-in frames: {}\n\
         right-click the phone for the menu; keys: Cmd/Ctrl+S screenshot, Cmd/Ctrl+E record, Cmd/Ctrl+R rotate",
        frame::builtin_names().collect::<Vec<_>>().join(", ")
    );
    std::process::exit(2)
}

fn main() {
    let mut frame_spec = Some("obsidian".to_string());
    let mut opts = server::Options::default();
    let mut serial = None;
    let mut cap = capture::Settings::default();
    let mut mute = false;
    let mut record_for: Option<f32> = None;
    // no arguments: use the saved defaults (and `--audio` inside the app bundle)
    let given: Vec<String> = std::env::args().skip(1).collect();
    let mut args = if given.is_empty() { tools::default_args() } else { given }.into_iter();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--frame" => {
                let v = args.next().unwrap_or_else(|| usage());
                frame_spec = (v != "none").then_some(v);
            }
            "--max-size" => opts.max_size = args.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| usage()),
            "--record-format" => {
                cap.format = args.next().and_then(|v| capture::Format::parse(&v)).unwrap_or_else(|| usage())
            }
            "--record-size" => cap.rec_size = args.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| usage()),
            "--record-bg" => cap.rec_bg = args.next().and_then(|v| capture::parse_color(&v)).unwrap_or_else(|| usage()),
            "--audio" => opts.audio = true,
            "--gain" => cap.gain_db = args.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| usage()),
            "--no-normalize" => cap.normalize = false,
            "--no-clipboard" => cap.clipboard = false,
            "--mute" => mute = true,
            "--record-for" => record_for = Some(args.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| usage())),
            "-h" | "--help" => usage(),
            s if s.starts_with('-') => usage(),
            s => serial = Some(s.to_string()),
        }
    }

    let bezel = frame_spec
        .as_deref()
        .map(|s| frame::load(s).unwrap_or_else(|e| tools::die(&format!("Cannot load the frame: {e}"))));

    let serial = serial.unwrap_or_else(|| {
        adb::devices().ok().and_then(|d| d.into_iter().next()).unwrap_or_else(|| {
            tools::die("No Android device found.\n\nConnect your phone over USB, turn on USB debugging and accept the prompt on the phone. Handset needs adb (Android platform-tools) installed.")
        })
    });

    let session =
        server::start(&serial, &opts).unwrap_or_else(|e| tools::die(&format!("Could not start mirroring: {e}")));
    eprintln!("connected: {}", session.device_name);

    let event_loop = EventLoop::<stream::UserEvent>::with_user_event().build().unwrap();
    menu::install(event_loop.create_proxy());
    let slot = Arc::new(Mutex::new(stream::Slot::default()));
    stream::spawn(session.video.try_clone().unwrap(), slot.clone(), event_loop.create_proxy());

    // The server also writes device messages (clipboard...) on the control
    // socket: drain them so it never blocks.
    let mut drain = session.control.try_clone().unwrap();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        while drain.read(&mut buf).map(|n| n > 0).unwrap_or(false) {}
    });

    // Audio: play it here (unless --mute) and give recordings a copy. The
    // playback stream is not `Send`, so it lives on this thread until exit.
    let audio_sink: audio::RecSink = Arc::new(Mutex::new(None));
    let playback = (session.audio.is_some() && !mute)
        .then(|| audio::Playback::new().map_err(|e| eprintln!("no local playback: {e}")).ok())
        .flatten();
    if let Some(a) = &session.audio {
        audio::spawn(a.try_clone().unwrap(), playback.as_ref().map(|p| p.ring()), audio_sink.clone());
    }

    let control = control::Control::new(session.control.try_clone().unwrap());
    let mut app = app::App::new(app::Init {
        bezel,
        title: session.device_name.clone(),
        slot,
        control,
        cap,
        audio_sink,
        with_audio: session.audio.is_some(),
        frame_spec,
    });
    app.auto_record = record_for.map(std::time::Duration::from_secs_f32);
    event_loop.run_app(&mut app).unwrap();
    drop(playback);
    // `session` drops here: kills the server process and removes the adb forward.
}
