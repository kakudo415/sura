use super::*;
use libc;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub struct Terminal {
    default_termios: libc::termios,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl Terminal {
    pub fn open() -> Self {
        let mut editor = Terminal {
            default_termios: libc::termios {
                c_iflag: 0,
                c_oflag: 0,
                c_cflag: 0,
                c_lflag: 0,
                c_line: 0,
                c_cc: [0u8; 32],
                c_ispeed: 0,
                c_ospeed: 0,
            },
        };
        send_escape_sequence_csi("?1049h");
        editor.enable_raw_mode();
        send_escape_sequence_csi("1;1H");
        editor
    }

    pub fn close(&mut self) {
        self.disable_raw_mode();
        send_escape_sequence_csi("?1049l");
    }

    fn enable_raw_mode(&mut self) {
        unsafe {
            let mut ptr = &mut self.default_termios;
            libc::tcgetattr(0, ptr);
        }
        let mut raw_mode_termios = self.default_termios;
        raw_mode_termios.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG);
        raw_mode_termios.c_cc[libc::VMIN] = 1;
        raw_mode_termios.c_cc[libc::VTIME] = 0;
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &raw_mode_termios);
        }
        unsafe {
            libc::fcntl(0, libc::F_SETFL, libc::O_NONBLOCK);
        }
    }

    fn disable_raw_mode(&self) {
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &self.default_termios);
        }
    }
}
