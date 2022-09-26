mod input;

use std::cmp;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;

use super::terminal;
use super::Context;

pub struct Editor {
    path: String,
    lines: Vec<String>,
    cursor: (usize, usize), // (line, column)
    preserved_column: usize,
    looking: (usize, usize), // Top left (line, column)
    pub terminal: terminal::Terminal,
}

impl Editor {
    pub fn new(file_path: String) -> Self {
        let mut editor = Editor {
            path: file_path,
            lines: Vec::new(),
            cursor: (0, 0),
            preserved_column: 0,
            looking: (0, 0),
            terminal: terminal::Terminal::open(),
        };
        for line in BufReader::new(fs::File::open(&editor.path).unwrap()).lines() {
            editor.lines.push(line.unwrap());
        }
        editor.refresh();
        editor
    }

    pub fn routine(&mut self) -> Context {
        let mut context = Context {
            is_quit: false,
            is_modified: true,
            is_error: false,
        };

        match input::Input::new().event() {
            Ok(input::Event::Character(ch)) => {
                context.is_modified = true;
                self.insert(ch);
            }
            Ok(input::Event::CarriageReturn) => {
                context.is_modified = true;
                self.next_line();
            }
            Ok(input::Event::Delete) => {
                context.is_modified = true;
                self.backspace();
            }
            Ok(input::Event::Ctrl('S')) => {
                context.is_modified = false;
                self.save();
            }
            Ok(input::Event::Ctrl('Q')) => {
                // TODO: Check saved or not
                context.is_quit = true;
            }
            Ok(input::Event::CursorUp) => self.cursor_up(),
            Ok(input::Event::CursorDown) => self.cursor_down(),
            Ok(input::Event::CursorForward) => self.cursor_forward(),
            Ok(input::Event::CursorBack) => self.cursor_back(),
            Ok(input::Event::Ctrl('B')) => self.prev_page(),
            Ok(input::Event::Ctrl('N')) => self.next_page(),
            Err(msg) => {
                context.is_quit = true;
                context.is_error = true;
                eprintln!("INPUT EVENT ERROR: {}", msg);
            }
            _ => {
                context.is_quit = true;
                context.is_error = true;
            }
        }

        self.refresh();
        return context;
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

    fn next_page(&mut self) {
        let window_size = self.terminal.size();
        self.cursor.0 = cmp::min(self.cursor.0 + window_size.0, self.lines.len() - 1);
        self.looking.0 = cmp::min(
            self.looking.0 + window_size.0,
            self.lines.len() - window_size.0,
        );
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

    fn refresh(&mut self) {
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
        let mut writer = BufWriter::new(fs::File::create(&self.path).unwrap());
        for line in &self.lines {
            writer.write(line.as_bytes()).unwrap();
            writer.write("\n".as_bytes()).unwrap();
        }
    }
}
