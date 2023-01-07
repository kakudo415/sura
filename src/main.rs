mod editor;
mod language;
mod message;
mod terminal;

use std::env;
use tokio::sync::mpsc;

use message::Event;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("ERROR: Select the input file and Choose language server");
    }

    let mut editor = editor::Editor::new(args[1].to_string());

    let (event_sender, mut event_queue) = mpsc::unbounded_channel();
    tokio::spawn(terminal::listen(event_sender.clone()));

    let mut client = language::initialize(args[2].clone(), event_sender.clone()).await;

    loop {
        let event = event_queue.recv().await.unwrap();
        match event {
            Event::KeyPress(keypress) => {
                editor.keypress_handler(keypress).await;
            }
            Event::LanguageNotification(response) => {
                eprintln!(
                    "NOTIFICATION\n{}",
                    serde_json::to_string(&response).unwrap()
                );
            }
        }
    }
}
