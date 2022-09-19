use libc;
use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::thread;
use std::time;

fn send_escape_sequence_csi(code: &str) {
    print!("\x1B[{}", code);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct Editor {
    saved_term: libc::termios,
    is_closing: bool,
}

#[cfg(target_os = "windows")]
struct Editor {
    is_closing: bool,
}

impl Editor {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn new() -> Self {
        send_escape_sequence_csi("?1049h");
        let mut editor = Editor {
            saved_term: libc::termios {
                c_iflag: 0,
                c_oflag: 0,
                c_cflag: 0,
                c_lflag: 0,
                c_line: 0,
                c_cc: [0u8; 32],
                c_ispeed: 0,
                c_ospeed: 0,
            },
            is_closing: false,
        };
        editor.enable_raw_mode();
        send_escape_sequence_csi("1;1H");
        editor
    }

    #[cfg(target_os = "windows")]
    fn new() -> Self {
        todo!()
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn close(&mut self) {
        self.disable_raw_mode();
        send_escape_sequence_csi("?1049l");
        self.is_closing = true;
    }

    #[cfg(target_os = "windows")]
    fn close(&mut self) {
        todo!()
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn enable_raw_mode(&mut self) {
        unsafe {
            let mut ptr = &mut self.saved_term;
            libc::tcgetattr(0, ptr);
        }
        let mut termattr = self.saved_term;
        termattr.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG);
        termattr.c_cc[libc::VMIN] = 1;
        termattr.c_cc[libc::VTIME] = 0;
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &termattr);
        }
        unsafe {
            libc::fcntl(0, libc::F_SETFL, libc::O_NONBLOCK);
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn disable_raw_mode(&self) {
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &self.saved_term);
        }
    }

    fn routine(&mut self) {
        let mut buf = [0; 1];
        let ptr = &mut buf;

        let r = unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 1) };
        if r > 0 {
            let c = char::from_u32((*ptr)[0]).unwrap();
            print!("{}", c);
            // Exit
            if c == 'E' {
                self.is_closing = true;
            }
        }
        thread::sleep(time::Duration::from_millis(16));
        std::io::stdout().flush().unwrap();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("ERROR: Select the input file");
    }

    let mut editor = Editor::new();

    loop {
        if editor.is_closing {
            break;
        }
        editor.routine();
    }

    editor.close();
}
