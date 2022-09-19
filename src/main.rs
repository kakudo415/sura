use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

fn send_escape_sequence_csi(code: &str) {
    print!("\x1B[{}", code);
}

struct Editor {}

impl Editor {
    fn new() -> Self {
        send_escape_sequence_csi("?1049h");
        Editor {}
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        send_escape_sequence_csi("?1049l");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("ERROR: Select the input file");
    }

    let file = File::open(&args[1]);
    let reader = BufReader::new(file.unwrap());

    let editor = Editor::new();

    for result in reader.lines() {
        let line = result.unwrap();
        println!("{}", line);
    }
    std::thread::sleep(std::time::Duration::from_secs(3));
}
