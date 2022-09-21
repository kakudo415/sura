mod terminal;

use libc;
use std::cmp;
use std::env;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::thread;
use std::time;

const ASCII_ESC: u32 = 27;
const ASCII_BS: u32 = 8;
const ASCII_CR: u32 = 13;
const ASCII_LF: u32 = 10;
const ASCII_DC1: u32 = 17; // ^Q
const ASCII_DC3: u32 = 19; // ^S
const ASCII_DEL: u32 = 127;

struct Editor {
    path: String,
    lines: Vec<String>,
    terminal: terminal::Terminal,
    cursor: (usize, usize),  // (line, column)
    looking: (usize, usize), // Top left (line, column)
    is_closing: bool,
}

impl Editor {
    fn new(file_path: String) -> Self {
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

    fn routine(&mut self) {
        let mut buf = [0; 1];
        let ptr = &mut buf;

        if unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 1) } <= 0 {
            return;
        }

        match (*ptr)[0] {
            ASCII_ESC => {
                if unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 1) } <= 0 {
                    return;
                }
                match (*ptr)[0] {
                    // ARROW
                    91 => {
                        if unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 1) } <= 0 {
                            return;
                        }
                        match (*ptr)[0] {
                            // UP
                            65 => {
                                if self.cursor.0 > 0 {
                                    self.cursor.0 -= 1;
                                }
                                self.cursor.1 =
                                    cmp::min(self.cursor.1, self.lines[self.cursor.0].len());
                            }
                            // DOWN
                            66 => {
                                if self.cursor.0 + 1 < self.lines.len() {
                                    self.cursor.0 += 1;
                                }
                                self.cursor.1 =
                                    cmp::min(self.cursor.1, self.lines[self.cursor.0].len());
                            }
                            // RIGHT
                            67 => {
                                if self.cursor.1 < self.lines[self.cursor.0].len() {
                                    self.cursor.1 += 1;
                                }
                            }
                            // LEFT
                            68 => {
                                if self.cursor.1 > 0 {
                                    self.cursor.1 -= 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            ASCII_DC1 => self.is_closing = true,
            ASCII_DC3 => self.save(),
            ASCII_DEL => self.backspace(),
            ASCII_CR => self.next_line(),
            _ => {
                let ch = char::from_u32((*ptr)[0]).unwrap();
                eprintln!("{}", (*ptr)[0]);
                self.insert(ch);
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

    fn refresh(&self) {
        terminal::clear();
        let window_size = self.terminal.size();
        for row in 0..(window_size.0 - 1) {
            terminal::move_cursor(row + 1, 1);
            if self.looking.0 + row >= self.lines.len() {
                break;
            }
            print!("{}", self.lines[self.looking.0 + row]);
        }
        terminal::move_cursor(self.cursor.0 + 1, self.cursor.1 + 1);
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
            writer.write(line.as_bytes());
            writer.write("\n".as_bytes());
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("ERROR: Select the input file");
    }

    let mut editor = Editor::new(args[1].to_string());

    loop {
        if editor.is_closing {
            break;
        }
        editor.routine();
        thread::sleep(time::Duration::from_millis(1000 / 60));
    }

    editor.terminal.close();
}
