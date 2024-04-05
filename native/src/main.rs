use compiler::{run_code, init};

fn print(output: String) {
    print!("{}", output);
}

fn println(output: String) {
    println!("{}", output);
}

fn main() {
    init(print, println);
    run_code("1+2");
}
