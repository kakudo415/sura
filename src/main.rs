mod config;
mod editor;
mod language;
mod message;
mod terminal;

use std::env;
use std::fs;
use tokio::sync::mpsc;

use message::*;

#[tokio::main]
async fn main() {
    let config = config::load().unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("ERROR: Select the input file");
    }
    let content = fs::read_to_string(&args[1]).unwrap();

    let mut editor = editor::Editor::new(args[1].to_string());

    let (event_sender, mut event_queue) = mpsc::unbounded_channel();
    tokio::spawn(terminal::listen(event_sender.clone()));

    let mut client = language::initialize(
        config.language_servers.get("rust").unwrap().clone(),
        event_sender.clone(),
    )
    .await
    .unwrap();

    let uri = "file:///".to_string() + &args[1];

    client.did_open("rust", &uri, &content).await.unwrap();

    loop {
        let event = event_queue.recv().await.unwrap();
        match event {
            Event::KeyPress(keypress) => {
                if let KeyPress::Control('Q') = keypress {
                    break;
                }
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

    client.did_close(&uri).await.unwrap();

    client.shutdown().await.unwrap();
    terminal::close();
    std::process::exit(0);
}
