use std::cmp;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::process;

use super::terminal;
use crate::Event;
use crate::KeyPress;

pub struct Editor {
    filepath: String,
    lines: Vec<String>,
    mode: Mode,
    cursor: (usize, usize), // (line, column)
    preserved_column: usize,
    looking: (usize, usize), // Top left (line, column)
}

enum Mode {
    Normal,
    Command,
}

impl Editor {
    pub fn new(filepath: String) -> Self {
        let mut editor = Editor {
            filepath,
            lines: Vec::new(),
            mode: Mode::Normal,
            cursor: (0, 0),
            preserved_column: 0,
            looking: (0, 0),
        };
        terminal::open();
        for line in BufReader::new(fs::File::open(&editor.filepath).unwrap()).lines() {
            editor.lines.push(line.unwrap());
        }
        editor.refresh();
        editor
    }

    pub async fn keypress_handler(&mut self, event: Event) {
        match event {
            Event::KeyPress(keypress) => match keypress {
                KeyPress::Character(character) => {
                    match self.mode {
                        Mode::Normal => {
                            self.insert(character);
                        }
                        Mode::Command => {
                            match character {
                                'h' => self.cursor_back(),
                                'j' => self.cursor_down(),
                                'k' => self.cursor_up(),
                                'l' => self.cursor_forward(),
                                'p' => self.page_forward(),
                                'P' => self.page_back(),
                                'w' => self.word_forward(),
                                'W' => self.word_back(),
                                _ => (),
                            };
                        }
                    };
                }
                KeyPress::CarriageReturn => {
                    self.nextline();
                }
                KeyPress::Delete => {
                    self.backspace();
                }

                KeyPress::Control('F') => {
                    match self.mode {
                        Mode::Normal => self.mode = Mode::Command,
                        Mode::Command => self.mode = Mode::Normal,
                    };
                }
                KeyPress::Control('S') => {
                    self.save();
                }
                KeyPress::Control('Q') => {
                    // TODO: Check saved or not
                    terminal::close();
                    process::exit(0);
                }
                KeyPress::CursorUp => self.cursor_up(),
                KeyPress::CursorDown => self.cursor_down(),
                KeyPress::CursorForward => self.cursor_forward(),
                KeyPress::CursorBack => self.cursor_back(),
                _ => {
                    panic!("UNSUPPORTED KEY EVENT");
                }
            },
        }

        self.refresh();
    }

    fn insert(&mut self, ch: char) {
        if self.lines.len() <= self.cursor.0 {
            self.lines.resize(self.cursor.0 + 1, Default::default());
        }
        self.lines[self.cursor.0].insert(self.cursor.1, ch);
        self.cursor.1 += 1;
    }

    fn backspace(&mut self) {
        if self.cursor.1 == 0 {
            if self.cursor.0 == 0 {
                return;
            }
            let prev_line_len = self.lines[self.cursor.0 - 1].len();
            self.lines[self.cursor.0 - 1] = format!(
                "{}{}",
                self.lines[self.cursor.0 - 1],
                self.lines[self.cursor.0]
            );
            for i in self.cursor.0..(self.lines.len() - 1) {
                self.lines[i] = self.lines[i + 1].clone();
            }
            self.lines.pop();
            self.cursor.0 -= 1;
            self.cursor.1 = prev_line_len;
            return;
        }
        self.lines[self.cursor.0].remove(self.cursor.1 - 1);
        self.cursor.1 -= 1;
    }

    fn nextline(&mut self) {
        self.lines.push(String::new());
        for i in ((self.cursor.0 + 1)..self.lines.len()).rev() {
            self.lines[i] = self.lines[i - 1].clone();
        }
        match self.lines[self.cursor.0].char_indices().nth(self.cursor.1) {
            Some((pos, _)) => {
                self.lines[self.cursor.0 + 1] = self.lines[self.cursor.0][pos..].to_string();
                self.lines[self.cursor.0] = self.lines[self.cursor.0][..pos].to_string();
            }
            None => {
                self.lines[self.cursor.0 + 1] = String::new();
            }
        };
        self.cursor.0 += 1;
        self.cursor.1 = 0;
    }

    fn cursor_up(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
        self.cursor.1 = cmp::min(self.preserved_column, self.lines[self.cursor.0].len());
    }

    fn cursor_down(&mut self) {
        if self.cursor.0 + 1 < self.lines.len() {
            self.cursor.0 += 1;
        }
        self.cursor.1 = cmp::min(self.preserved_column, self.lines[self.cursor.0].len());
    }

    fn cursor_forward(&mut self) {
        if self.cursor.1 < self.lines[self.cursor.0].len() {
            self.cursor.1 += 1;
            self.preserved_column = self.cursor.1;
        }
    }

    fn cursor_back(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
            self.preserved_column = self.cursor.1;
        }
    }

    fn page_forward(&mut self) {
        let window_size = terminal::size();
        self.cursor.0 = cmp::min(self.cursor.0 + window_size.0, self.lines.len() - 1);
        self.looking.0 = cmp::min(
            self.looking.0 + window_size.0,
            self.lines.len() - window_size.0,
        );
    }

    fn page_back(&mut self) {
        let window_size = terminal::size();
        if self.cursor.0 > window_size.0 {
            self.cursor.0 -= window_size.0;
            if self.looking.0 > window_size.0 {
                self.looking.0 -= window_size.0;
            } else {
                self.looking.0 = 0;
            }
        } else {
            self.cursor.0 = 0;
            self.looking.0 = 0;
        }
    }

    fn word_forward(&mut self) {
        todo!()
    }

    fn word_back(&mut self) {
        todo!()
    }

    fn refresh(&mut self) {
        let window_size = terminal::size();
        if self.cursor.0 < self.looking.0 {
            self.looking.0 = self.cursor.0;
        }
        if self.cursor.0 >= self.looking.0 + window_size.0 {
            self.looking.0 = self.cursor.0 - window_size.0 + 1;
        }
        for row in 0..window_size.0 {
            terminal::move_cursor(row + 1, 1);
            if self.looking.0 + row >= self.lines.len() {
                break;
            }
            terminal::clear_line();
            print!("{}", self.lines[self.looking.0 + row]);
        }
        terminal::move_cursor(self.cursor.0 - self.looking.0 + 1, self.cursor.1 + 1);
        loop {
            match std::io::stdout().flush() {
                Ok(_) => {
                    break;
                }
                Err(_) => {}
            }
        }
    }

    fn save(&mut self) {
        let mut writer = BufWriter::new(fs::File::create(&self.filepath).unwrap());
        for line in &self.lines {
            writer.write(line.as_bytes()).unwrap();
            writer.write("\n".as_bytes()).unwrap();
        }
    }
}
