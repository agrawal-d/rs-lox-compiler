use compiler::{init, run_code};
use std::io::Write;

fn print(output: String) {
    print!("{}", output);
}

fn println(output: String) {
    println!("{}", output);
}

fn main() {
    init(print, println);
    println!("Write a line of code below:");
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        run_code(&input);
        println!("========================================")
    }
}
