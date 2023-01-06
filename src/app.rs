use serde_json::json;
use std::env;
use tokio::sync::oneshot;
use tokio::{self, sync::mpsc};

use crate::editor;
use crate::language;
use crate::language::message;
use crate::terminal;

pub struct App {
    editors: Vec<editor::Editor>, // TODO: Support multiple editors
}

#[derive(Debug)]
pub enum Event {
    KeyPress(KeyPress),
    LanguageServerNotification(message::Notification),
}

#[derive(Debug)]
pub enum KeyPress {
    BackSpace,
    CarriageReturn,
    LineFeed,
    Delete,

    CursorUp,
    CursorDown,
    CursorForward,
    CursorBack,

    Control(char),
    Character(char),
}

impl App {
    pub fn new() -> Self {
        App {
            editors: Vec::new(),
        }
    }

    pub async fn start(&mut self) {
        let args: Vec<String> = env::args().collect();
        if args.len() != 3 {
            panic!("ERROR: Select the input file and Choose language server");
        }

        let editor = editor::Editor::new(args[1].to_string());
        self.editors.push(editor);

        let (event_dispatcher, mut event_queue) = mpsc::channel(16);

        let (language_request_sender, request_queue) = mpsc::channel(16);
        tokio::spawn(language::start(
            args[2].clone(),
            event_dispatcher.clone(),
            request_queue,
        ));

        let (tx, rx) = oneshot::channel();
        let request =
            crate::language::message::Request::new("initialize", json!({"capabilities": {}}));
        language_request_sender
            .send((language::message::ClientMessage::Request(request), tx))
            .await
            .unwrap();
        let response = rx.await;
        let (tx, _) = oneshot::channel();
        match response {
            Ok(_) => {
                language_request_sender
                    .send((
                        language::message::ClientMessage::Notification(
                            language::message::Notification::new("initialized", json!({})),
                        ),
                        tx,
                    ))
                    .await
                    .unwrap();
            }
            Err(_) => panic!("LANGUAGE SERVER INITIALIZATION FAILED"),
        }

        tokio::spawn(terminal::listen(event_dispatcher.clone()));

        loop {
            let event = event_queue.recv().await.unwrap();
            match event {
                Event::KeyPress(keypress) => {
                    self.editors[0].keypress_handler(keypress).await;
                }
                Event::LanguageServerNotification(response) => {
                    eprintln!("{}", serde_json::to_string(&response).unwrap());
                }
            }
        }
    }
}
