use crate::editor::Editor;

use tokio::io::{self, AsyncReadExt};

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

pub async fn keypress_listener(mut editor: Editor) {
    loop {
        let mut buffer: [u8; 4] = [0, 0, 0, 0];
        io::stdin().read_exact(&mut buffer[0..1]).await.unwrap();
        match buffer[0] {
            0x00..=0x7F => match buffer[0] {
                0x1B => {
                    io::stdin().read_exact(&mut buffer[1..2]).await.unwrap();
                    match buffer[1] {
                        0x5B => {
                            io::stdin().read_exact(&mut buffer[2..3]).await.unwrap();
                            match buffer[2] {
                                0x41 => {
                                    editor
                                        .keypress_handler(Event::KeyPress(KeyPress::CursorUp))
                                        .await;
                                }
                                0x42 => {
                                    editor
                                        .keypress_handler(Event::KeyPress(KeyPress::CursorDown))
                                        .await;
                                }
                                0x43 => {
                                    editor
                                        .keypress_handler(Event::KeyPress(KeyPress::CursorForward))
                                        .await;
                                }
                                0x44 => {
                                    editor
                                        .keypress_handler(Event::KeyPress(KeyPress::CursorBack))
                                        .await;
                                }
                                _ => panic!("UNKNOWN ESCAPE SEQUENCE"),
                            }
                        }
                        _ => panic!("UNKNOWN ESCAPE SEQUENCE"),
                    }
                }
                0x20..=0x7E => {
                    let character = char::from_u32(buffer[0] as u32).unwrap();
                    editor
                        .keypress_handler(Event::KeyPress(KeyPress::Character(character)))
                        .await;
                }
                0x08 => {
                    editor
                        .keypress_handler(Event::KeyPress(KeyPress::BackSpace))
                        .await;
                }
                0x0A => {
                    editor
                        .keypress_handler(Event::KeyPress(KeyPress::LineFeed))
                        .await;
                }
                0x0D => {
                    editor
                        .keypress_handler(Event::KeyPress(KeyPress::CarriageReturn))
                        .await;
                }
                0x7F => {
                    editor
                        .keypress_handler(Event::KeyPress(KeyPress::Delete))
                        .await;
                }
                0x01..=0x1A => {
                    let character = char::from_u32(buffer[0] as u32 + 0x40).unwrap();
                    editor
                        .keypress_handler(Event::KeyPress(KeyPress::Control(character)))
                        .await;
                }
                _ => {
                    panic!("UNKNOWN ASCII CHARACTER");
                }
            },
            // FIXME: UTF-8
            0xC2..=0xDF => {
                io::stdin().read_exact(&mut buffer[1..2]).await.unwrap();
                todo!();
            }
            0xE0..=0xEF => {
                io::stdin().read_exact(&mut buffer[1..3]).await.unwrap();
                todo!();
            }
            0xF0..=0xF4 => {
                io::stdin().read_exact(&mut buffer[1..4]).await.unwrap();
                todo!();
            }
            _ => {
                panic!("UNKNOWN CHARACTER");
            }
        }
    }
}
