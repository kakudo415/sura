use std::sync::Mutex;

use nix::sys::termios;

static STORED_TERMIOS: Mutex<Option<termios::Termios>> = Mutex::new(None);

pub fn raw_mode() {
    let mut stored_termios = STORED_TERMIOS.lock().unwrap();
    match &*stored_termios {
        Some(_) => {}
        None => {
            let prev_termios = termios::tcgetattr(0).unwrap();
            let mut raw_termios = prev_termios.clone();
            termios::cfmakeraw(&mut raw_termios);
            termios::tcsetattr(0, termios::SetArg::TCSANOW, &raw_termios).unwrap();
            *stored_termios = Some(prev_termios);
        }
    }
}

pub fn canonical_mode() {
    let mut stored_termios = STORED_TERMIOS.lock().unwrap();
    match &*stored_termios {
        Some(config) => {
            termios::tcsetattr(0, termios::SetArg::TCSANOW, config).unwrap();
            *stored_termios = None;
        }
        None => {}
    }
}
