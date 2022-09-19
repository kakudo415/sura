use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("ERROR: Select the input file");
    }

    let file = File::open(&args[1]);
    let reader = BufReader::new(file.unwrap());

    print!("\x1B[s");
    print!("\x1B[2J");
    print!("\x1B[1;1H");

    for result in reader.lines() {
        let line = result.unwrap();
        println!("{}", line);
    }

    print!("\x1B[u");
}
