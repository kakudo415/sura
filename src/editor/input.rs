pub enum Event {
    BackSpace,
    CarriageReturn,
    LineFeed,
    Delete,

    CursorUp,
    CursorDown,
    CursorForward,
    CursorBack,

    Ctrl(char),
    Character(char),
}

pub struct Input {
    sequence: u32,
}

impl Input {
    pub fn new() -> Self {
        Input { sequence: 0 }
    }

    pub fn event(&mut self) -> Result<Event, &str> {
        self.sequence = self.next().unwrap() as u32;
        match self.sequence {
            0x1B => match self.next().unwrap() {
                0x5B => match self.next().unwrap() {
                    0x41 => Ok(Event::CursorUp),
                    0x42 => Ok(Event::CursorDown),
                    0x43 => Ok(Event::CursorForward),
                    0x44 => Ok(Event::CursorBack),
                    _ => Err("UNKNOWN ESCAPE SEQUENCE (CURSOR MOVING?)"),
                },
                _ => Err("UNKNOWN ESCAPE SEQUENCE"),
            },
            // Control Characters
            0x08 => Ok(Event::BackSpace),
            0x0A => Ok(Event::LineFeed),
            0x0D => Ok(Event::CarriageReturn),
            0x7F => Ok(Event::Delete),
            // Ctrl + ?
            0x01..=0x1A => Ok(Event::Ctrl(char::from_u32(self.sequence + 0x40).unwrap())),
            // UTF-8 Characters
            0x20..=0x7E => {
                // ASCII Characters
                Ok(Event::Character(char_from_utf8_u32(self.sequence << 24)))
            }
            0xC2..=0xDF => {
                self.sequence <<= 8;
                self.sequence += self.next().unwrap() as u32;
                Ok(Event::Character(char_from_utf8_u32(self.sequence << 16)))
            }
            0xE0..=0xEF => {
                self.sequence <<= 8;
                self.sequence += self.next().unwrap() as u32;
                self.sequence <<= 8;
                self.sequence += self.next().unwrap() as u32;
                Ok(Event::Character(char_from_utf8_u32(self.sequence << 8)))
            }
            0xF0..=0xF4 => {
                self.sequence <<= 8;
                self.sequence += self.next().unwrap() as u32;
                self.sequence <<= 8;
                self.sequence += self.next().unwrap() as u32;
                self.sequence <<= 8;
                self.sequence += self.next().unwrap() as u32;
                Ok(Event::Character(char_from_utf8_u32(self.sequence)))
            }
            _ => Err("UNKNOWN INPUT"),
        }
    }
}

impl Iterator for Input {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0; 1];
        loop {
            if unsafe { libc::read(0, (&mut buffer).as_ptr() as *mut libc::c_void, 1) } > 0 {
                return Some(buffer[0]);
            }
        }
    }
}

fn char_from_utf8_u32(src: u32) -> char {
    const CONTINUATION_BYTE_MASK: u32 = 0b00111111;
    match src >> 24 {
        0x20..=0x7E => {
            let src = src >> 24;
            char::from_u32(src).unwrap()
        }
        0xC2..=0xDF => {
            // 110x xxxx 10xx xxxx
            let src = src >> 16;
            let b1 = (src >> 8) & 0b00011111;
            let b2 = src & CONTINUATION_BYTE_MASK;
            char::from_u32((b1 << 6) + b2).unwrap()
        }
        0xE0..=0xEF => {
            // 1110 xxxx 10xx xxxx 10xx xxxx
            let src = src >> 8;
            let b1 = (src >> 16) & 0b00001111;
            let b2 = (src >> 8) & CONTINUATION_BYTE_MASK;
            let b3 = src & CONTINUATION_BYTE_MASK;
            char::from_u32((b1 << 12) + (b2 << 6) + b3).unwrap()
        }
        0xF0..=0xF4 => {
            // 1111 0xxx 10xx xxxx 10xx xxxx 10xx xxxx
            let b1 = (src >> 24) & 0b00000111;
            let b2 = (src >> 16) & CONTINUATION_BYTE_MASK;
            let b3 = (src >> 8) & CONTINUATION_BYTE_MASK;
            let b4 = src & CONTINUATION_BYTE_MASK;
            char::from_u32((b1 << 18) + (b2 << 12) + (b3 << 6) + b4).unwrap()
        }
        _ => {
            panic!("INVALID UTF-8 CHARACTER")
        }
    }
}
