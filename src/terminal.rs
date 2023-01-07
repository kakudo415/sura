mod rawmode;

pub use rawmode::*;

use std::mem;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::UnboundedSender;

use crate::message::Event;
use crate::message::KeyPress;

pub fn send_escape_sequence_csi(code: &str) {
    print!("\x1B[{}", code);
}

pub fn clear_line() {
    send_escape_sequence_csi("2K");
}

pub fn move_cursor(row: usize, column: usize) {
    send_escape_sequence_csi(format!("{};{}H", row, column).as_str());
}

pub fn open() {
    send_escape_sequence_csi("?1049h");
    raw_mode();
}

pub fn close() {
    canonical_mode();
    send_escape_sequence_csi("?1049l");
}

pub async fn listen(tx: UnboundedSender<Event>) {
    let mut buf = [0];
    loop {
        tokio::io::stdin().read(&mut buf).await.unwrap();
        if buf[0] > 0x7F {
            todo!(); // UTF-8
        }
        let event = match buf[0] {
            0x08 => Event::KeyPress(KeyPress::BS),
            0x0A => Event::KeyPress(KeyPress::LF),
            0x0D => Event::KeyPress(KeyPress::CR),
            0x7F => Event::KeyPress(KeyPress::Delete),
            0x1B => Event::KeyPress(read_escape_sequence().await),
            0x01..=0x1A => Event::KeyPress(KeyPress::Control((buf[0] + 0x40) as char)),
            0x20..=0x7E => Event::KeyPress(KeyPress::Character(buf[0] as char)),
            _ => {
                panic!("UNKNOWN CHARACTER");
            }
        };
        tx.send(event).unwrap();
    }
}

async fn read_escape_sequence() -> KeyPress {
    let mut buf = [0];
    tokio::io::stdin().read(&mut buf).await.unwrap();
    match buf[0] {
        0x5B => {
            tokio::io::stdin().read(&mut buf).await.unwrap();
            match buf[0] {
                0x41 => KeyPress::CursorUp,
                0x42 => KeyPress::CursorDown,
                0x43 => KeyPress::CursorForward,
                0x44 => KeyPress::CursorBack,
                _ => {
                    panic!("UNKNOWN ESCAPE SEQUENCE (CURSOR MOVING?)")
                }
            }
        }
        _ => {
            panic!("UNKNOWN ESCAPE SEQUENCE");
        }
    }
}

pub fn size() -> (usize, usize) {
    let mut winsize: nix::libc::winsize = unsafe { mem::zeroed() };
    unsafe {
        nix::libc::ioctl(
            nix::libc::STDOUT_FILENO,
            nix::libc::TIOCGWINSZ,
            &mut winsize,
        );
    }
    (winsize.ws_row.into(), winsize.ws_col.into())
}
