#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

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
