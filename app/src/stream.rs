//! Video thread: read packets from the server, decode H.264, hand the most
//! recent frame to the UI thread.

use crate::render::Yuv;
use crate::server::{self, Packet};
use openh264::decoder::Decoder;
use openh264::formats::YUVSource;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use winit::event_loop::EventLoopProxy;

#[derive(Debug)]
pub enum UserEvent {
    Frame,
    Disconnected,
    /// A native menu item was chosen.
    Menu(String),
}

#[derive(Default)]
pub struct Slot {
    pub yuv: Yuv,
    /// A frame is waiting to be picked up by the UI thread.
    pub fresh: bool,
}

pub fn spawn(mut video: TcpStream, slot: Arc<Mutex<Slot>>, proxy: EventLoopProxy<UserEvent>) {
    std::thread::spawn(move || {
        let mut decoder = match Decoder::new() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("decoder: {e}");
                let _ = proxy.send_event(UserEvent::Disconnected);
                return;
            }
        };
        loop {
            let data = match server::read_packet(&mut video) {
                Ok(Packet::Session { width, height }) => {
                    eprintln!("video session: {width}x{height}");
                    continue;
                }
                Ok(Packet::Config(d)) => d,
                Ok(Packet::Frame { data, .. }) => data,
                Err(e) => {
                    eprintln!("video stream ended: {e}");
                    break;
                }
            };
            match decoder.decode(&data) {
                Ok(Some(img)) => {
                    let (w, h) = img.dimensions();
                    let (sy, su, _) = img.strides();
                    let mut s = slot.lock().unwrap();
                    let was_fresh = s.fresh;
                    s.yuv.width = w as u32;
                    s.yuv.height = h as u32;
                    s.yuv.stride_y = sy as u32;
                    s.yuv.stride_uv = su as u32;
                    s.yuv.y.clear();
                    s.yuv.y.extend_from_slice(img.y());
                    s.yuv.u.clear();
                    s.yuv.u.extend_from_slice(img.u());
                    s.yuv.v.clear();
                    s.yuv.v.extend_from_slice(img.v());
                    s.fresh = true;
                    drop(s);
                    if !was_fresh {
                        let _ = proxy.send_event(UserEvent::Frame);
                    }
                }
                Ok(None) => {}
                Err(e) => eprintln!("decode error: {e}"),
            }
        }
        let _ = proxy.send_event(UserEvent::Disconnected);
    });
}
