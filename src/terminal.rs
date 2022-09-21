mod linux;
mod macos;
mod windows;

pub use linux::*;
pub use macos::*;
pub use windows::*;

fn send_escape_sequence_csi(code: &str) {
    print!("\x1B[{}", code);
}

pub fn clear() {
    send_escape_sequence_csi("2J");
}

pub fn clear_line() {
    send_escape_sequence_csi("2K");
}

pub fn move_cursor(row: usize, column: usize) {
    send_escape_sequence_csi(format!("{};{}H", row, column).as_str());
}
