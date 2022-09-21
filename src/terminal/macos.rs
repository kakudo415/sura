use super::*;
use libc;

#[cfg(target_os = "macos")]
pub struct Terminal {
    default_termios: libc::termios,
}

#[cfg(target_os = "macos")]
impl Terminal {
    pub fn open() -> Self {
        let mut editor = Terminal {
            default_termios: libc::termios {
                c_iflag: 0,
                c_oflag: 0,
                c_cflag: 0,
                c_lflag: 0,
                c_cc: [0u8; 20],
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

    pub fn size(&self) -> (usize, usize) {
        let mut winsize = libc::winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        unsafe {
            libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut winsize);
        }
        (winsize.ws_row.into(), winsize.ws_col.into())
    }

    fn enable_raw_mode(&mut self) {
        unsafe {
            libc::tcgetattr(0, &mut self.default_termios);
        }

        let mut raw_mode_termios = self.default_termios;
        unsafe {
            libc::cfmakeraw(&mut raw_mode_termios);
        }
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
