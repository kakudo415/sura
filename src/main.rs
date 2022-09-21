mod editor;
mod terminal;

use std::env;
use std::thread;
use std::time;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("ERROR: Select the input file");
    }

    let mut editor = editor::Editor::new(args[1].to_string());

    loop {
        if editor.is_closing {
            break;
        }
        editor.routine();
        thread::sleep(time::Duration::from_millis(1000 / 60));
    }

    editor.terminal.close();
}
