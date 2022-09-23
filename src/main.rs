mod editor;
mod terminal;

use std::env;
use std::thread;
use std::time;

pub struct Context {
    is_quit: bool,
    is_modified: bool,
    is_error: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("ERROR: Select the input file");
    }

    let mut editor = editor::Editor::new(args[1].to_string());

    loop {
        let context = editor.routine();
        if context.is_quit {
            if context.is_error {
                eprintln!("FATAL ERROR. QUIT.");
            }
            break;
        }
        thread::sleep(time::Duration::from_millis(1000 / 60));
    }

    editor.terminal.close();
}
