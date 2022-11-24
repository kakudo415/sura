use std::env;
use tokio::{self, io::AsyncReadExt};

use crate::editor;

pub struct App {
    editors: Vec<editor::Editor>, // TODO: Support multiple editors
}

pub enum Event {
    KeyPress(KeyPress),
}

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
        if args.len() != 2 {
            panic!("ERROR: Select the input file");
        }

        let editor = editor::Editor::new(args[1].to_string());
        self.editors.push(editor);

        self.listen_and_deliver_keys().await;
    }

    async fn listen_and_deliver_keys(&mut self) {
        let mut buf = [0];
        loop {
            tokio::io::stdin().read(&mut buf).await.unwrap();
            if buf[0] > 0x7F {
                todo!(); // UTF-8
            }
            match buf[0] {
                0x08 => {
                    self.editors[0]
                        .keypress_handler(Event::KeyPress(KeyPress::BackSpace))
                        .await;
                }
                0x0A => {
                    self.editors[0]
                        .keypress_handler(Event::KeyPress(KeyPress::LineFeed))
                        .await;
                }
                0x0D => {
                    self.editors[0]
                        .keypress_handler(Event::KeyPress(KeyPress::CarriageReturn))
                        .await;
                }
                0x7F => {
                    self.editors[0]
                        .keypress_handler(Event::KeyPress(KeyPress::Delete))
                        .await;
                }
                0x1B => {
                    self.editors[0]
                        .keypress_handler(Event::KeyPress(read_escape_sequence().await))
                        .await;
                }
                0x01..=0x1A => {
                    self.editors[0]
                        .keypress_handler(Event::KeyPress(KeyPress::Control(
                            (buf[0] + 0x40) as char,
                        )))
                        .await;
                }
                0x20..=0x7E => {
                    self.editors[0]
                        .keypress_handler(Event::KeyPress(KeyPress::Character(buf[0] as char)))
                        .await;
                }
                _ => {
                    panic!("UNKNOWN CHARACTER");
                }
            }
        }
    }
}

async fn read_escape_sequence() -> KeyPress {
    let mut buf = [0];
    tokio::io::stdin().read(&mut buf).await.unwrap();
    match buf[0] {
        0x5B => {
            tokio::io::stdin().read(&mut buf).await.unwrap();
            match buf[0] {
                0x41 => KeyPress::CursorUp,
                0x42 => KeyPress::CursorDown,
                0x43 => KeyPress::CursorForward,
                0x44 => KeyPress::CursorBack,
                _ => {
                    panic!("UNKNOWN ESCAPE SEQUENCE (CURSOR MOVING?)")
                }
            }
        }
        _ => {
            panic!("UNKNOWN ESCAPE SEQUENCE");
        }
    }
}
