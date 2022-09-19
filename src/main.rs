mod terminal;

use libc;
use std::env;
use std::io::Write;
use std::thread;
use std::time;

struct Editor {
    terminal: terminal::Terminal,
    is_closing: bool,
}

impl Editor {
    fn new() -> Self {
        Editor {
            terminal: terminal::Terminal::open(),
            is_closing: false,
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

    editor.terminal.close();
}
