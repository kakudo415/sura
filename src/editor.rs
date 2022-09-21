mod keys;

use std::cmp;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;

use super::terminal;

pub struct Editor {
    path: String,
    lines: Vec<String>,
    pub terminal: terminal::Terminal,
    cursor: (usize, usize),  // (line, column)
    looking: (usize, usize), // Top left (line, column)
    pub is_closing: bool,
}

impl Editor {
    pub fn new(file_path: String) -> Self {
        let mut editor = Editor {
            path: file_path,
            lines: Vec::new(),
            terminal: terminal::Terminal::open(),
            cursor: (0, 0),
            looking: (0, 0),
            is_closing: false,
        };
        for line in BufReader::new(fs::File::open(&editor.path).unwrap()).lines() {
            editor.lines.push(line.unwrap());
        }
        editor.refresh();
        editor
    }

    pub fn routine(&mut self) {
        match keys::Input::new().event() {
            keys::Events::Character(ch) => self.insert(ch),
            keys::Events::CarriageReturn => self.next_line(),
            keys::Events::StartOfText => self.prev_page(),
            keys::Events::ShiftOut => self.next_page(),
            keys::Events::Delete => self.backspace(),
            keys::Events::DeviceControl1 => self.is_closing = true,
            keys::Events::DeviceControl3 => self.save(),
            keys::Events::CursorUp => {
                if self.cursor.0 > 0 {
                    self.cursor.0 -= 1;
                }
                self.cursor.1 = cmp::min(self.cursor.1, self.lines[self.cursor.0].len());
            }
            keys::Events::CursorDown => {
                if self.cursor.0 + 1 < self.lines.len() {
                    self.cursor.0 += 1;
                }
                self.cursor.1 = cmp::min(self.cursor.1, self.lines[self.cursor.0].len());
            }
            keys::Events::CursorForward => {
                if self.cursor.1 < self.lines[self.cursor.0].len() {
                    self.cursor.1 += 1;
                }
            }
            keys::Events::CursorBack => {
                if self.cursor.1 > 0 {
                    self.cursor.1 -= 1;
                }
            }
            _ => {
                panic!("EVENT ERROR!");
            }
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

    fn next_line(&mut self) {
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

    fn prev_page(&mut self) {
        let window_size = self.terminal.size();
        if self.cursor.0 > window_size.0 {
            self.cursor.0 -= window_size.0;
        } else {
            self.cursor.0 = 0;
        }
    }

    fn next_page(&mut self) {
        let window_size = self.terminal.size();
        self.cursor.0 = cmp::min(self.cursor.0 + window_size.0, self.lines.len());
    }

    fn refresh(&mut self) {
        terminal::clear();
        let window_size = self.terminal.size();
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
        let mut writer = BufWriter::new(fs::File::create(&self.path).unwrap());
        for line in &self.lines {
            writer.write(line.as_bytes()).unwrap();
            writer.write("\n".as_bytes()).unwrap();
        }
    }
}
