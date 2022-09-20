mod terminal;

use libc;
use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::thread;
use std::time;

const ASCII_ESC: u32 = 27;
const ASCII_BS: u32 = 8;
const ASCII_CR: u32 = 13;
const ASCII_LF: u32 = 10;
const ASCII_DC1: u32 = 17; // ^Q
const ASCII_DEL: u32 = 127;

struct Editor {
    lines: Vec<String>,
    terminal: terminal::Terminal,
    cursor: (usize, usize), // (line, column)
    is_closing: bool,
}

impl Editor {
    fn new() -> Self {
        Editor {
            lines: Vec::new(),
            terminal: terminal::Terminal::open(),
            cursor: (0, 0),
            is_closing: false,
        }
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
                            }
                            // DOWN
                            66 => {
                                if self.cursor.0 < self.lines.len() - 1 {
                                    self.cursor.0 += 1;
                                }
                            }
                            // RIGHT
                            67 => {
                                if self.cursor.1 < self.lines[self.cursor.1].len() - 1 {
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
            self.cursor.0 -= 1;
            self.cursor.1 = prev_line_len;
            return;
        }
        self.lines[self.cursor.0].remove(self.cursor.1 - 1);
        self.cursor.1 -= 1;
    }

    fn next_line(&mut self) {
        self.lines.resize(self.lines.len() + 1, Default::default());
        for i in ((self.cursor.0 + 1)..(self.lines.len() - 1)).rev() {
            self.lines[i] = self.lines[i - 1].clone();
        }
        let next_line_head = self.lines[self.cursor.0]
            .char_indices()
            .nth(self.cursor.1)
            .unwrap()
            .0;
        self.lines[self.cursor.0 + 1] = self.lines[self.cursor.0][next_line_head..].to_string();
        self.lines[self.cursor.0] = self.lines[self.cursor.0][..next_line_head].to_string();
        self.cursor.0 += 1;
        self.cursor.1 = 0;
    }

    fn refresh(&self) {
        terminal::clear();
        let mut row = 1;
        for line in &self.lines {
            terminal::move_cursor(row, 1);
            print!("{}", line);
            row += 1;
        }
        terminal::move_cursor(self.cursor.0 + 1, self.cursor.1 + 1);
        std::io::stdout().flush().unwrap();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("ERROR: Select the input file");
    }

    let mut editor = Editor::new();

    for line in BufReader::new(File::open(&args[1]).unwrap()).lines() {
        editor.lines.push(line.unwrap());
    }
    editor.refresh();
    terminal::move_cursor(1, 1);

    loop {
        if editor.is_closing {
            break;
        }
        editor.routine();
        thread::sleep(time::Duration::from_millis(1000 / 60));
    }

    editor.terminal.close();
}
