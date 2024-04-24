use compiler::{init, run_code};
use std::io::Write;

#[cfg(debug_assertions)]
fn flush_if_debug() {
    std::io::stdout().flush().unwrap();
}

#[cfg(not(debug_assertions))]
fn flush_if_debug() {}

fn print(output: String) {
    print!("{}", output);
    flush_if_debug();
}

fn println(output: String) {
    println!("{}", output);
    flush_if_debug();
}

fn help(args: &Vec<String>) {
    println(format!("Usage: {} <FILE> \nInterpret the program in FILE", args[0]));
}

fn main() {
    init(print, println);

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        help(&args);
        std::process::exit(1);
    }

    if args[1] == "-h" || args[1] == "--help" {
        help(&args);
        std::process::exit(0);
    }

    let path = &args[1];
    let input = std::fs::read_to_string(path).expect("Failed to read file");

    run_code(&input);
}
