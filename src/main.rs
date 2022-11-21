mod editor;
mod event;
mod terminal;

pub use event::*;

use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("ERROR: Select the input file");
    }

    let editor = editor::Editor::new(args[1].to_string());

    tokio::join!(event::keypress_listener(editor));

    terminal::close();
}
