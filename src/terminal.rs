mod linux;
mod windows;

pub use linux::*;
pub use windows::*;

fn send_escape_sequence_csi(code: &str) {
    print!("\x1B[{}", code);
}
