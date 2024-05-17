use compiler::{init, run_code};
use futures::executor;

#[cfg(debug_assertions)]
fn flush_if_debug() {
    use std::io::Write;
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

async fn read_async(prompt: String) -> String {
    println(prompt);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let len = input.len();
    input.truncate(len - 1);
    input
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

    let input = std::fs::read_to_string(&args[1]).expect("Failed to read file");
    executor::block_on(run_code(&input, read_async));
}
