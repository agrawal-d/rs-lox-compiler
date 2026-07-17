use compiler::compiler::Compiler;
use compiler::fun::FunType;
use compiler::vm::Vm;
use compiler::{init, run_file};
use futures::executor;
use futures::FutureExt;
use std::io::{self, Write};
use std::panic::AssertUnwindSafe;
use std::rc::Rc;

use std::cell::Cell;

thread_local! {
    static SUPPRESS_OUTPUT: Cell<bool> = Cell::new(false);
}

#[cfg(debug_assertions)]
fn flush_if_debug() {
    use std::io::Write;
    std::io::stdout().flush().unwrap();
}

#[cfg(not(debug_assertions))]
fn flush_if_debug() {}

fn print(output: String) {
    if !SUPPRESS_OUTPUT.with(|s| s.get()) {
        print!("{}", output);
        flush_if_debug();
    }
}

fn println(output: String) {
    if !SUPPRESS_OUTPUT.with(|s| s.get()) {
        println!("{}", output);
        flush_if_debug();
    }
}

fn help(args: &[String]) {
    println(format!("Usage: {} <FILE> \nInterpret the program in FILE", args[0]));
}

async fn read_async(prompt: String) -> String {
    println(prompt);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    if input.ends_with('\n') {
        input.pop();
    }
    if input.ends_with('\r') {
        input.pop();
    }
    input
}

struct InputChecker {
    in_string: bool,
    brace_depth: i32,
    paren_depth: i32,
}

impl InputChecker {
    fn new() -> Self {
        Self {
            in_string: false,
            brace_depth: 0,
            paren_depth: 0,
        }
    }

    fn feed(&mut self, line: &str) {
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if self.in_string {
                if c == '"' {
                    self.in_string = false;
                } else if c == '\\' {
                    // Consume next character if escaped
                    let _ = chars.next();
                }
            } else {
                match c {
                    '"' => self.in_string = true,
                    '/' => {
                        if chars.peek() == Some(&'/') {
                            // Single line comment, rest of the line is ignored
                            break;
                        }
                    }
                    '{' => self.brace_depth += 1,
                    '}' => self.brace_depth -= 1,
                    '(' => self.paren_depth += 1,
                    ')' => self.paren_depth -= 1,
                    _ => {}
                }
            }
        }
    }

    fn is_balanced(&self) -> bool {
        !self.in_string && self.brace_depth <= 0 && self.paren_depth <= 0
    }
}

fn run_repl() {
    println!("Lox REPL. Press Ctrl+C or Ctrl+D to exit.");

    let mut interner = compiler::interner::Interner::with_capacity(1024);
    let mut vm = Vm::new_repl(&mut interner, read_async);

    let stdin = io::stdin();
    let mut accumulated_input = String::new();
    let mut checker = InputChecker::new();

    loop {
        if accumulated_input.is_empty() {
            print!("> ");
        } else {
            print!(".. ");
        }
        io::stdout().flush().unwrap();

        let mut line = String::new();
        let bytes_read = stdin.read_line(&mut line).unwrap();
        if bytes_read == 0 {
            // EOF (Ctrl+D)
            println!("");
            break;
        }

        checker.feed(&line);
        accumulated_input.push_str(&line);

        if checker.is_balanced() {
            let code = accumulated_input.trim();
            if !code.is_empty() {
                let source: Rc<str> = Rc::from(code);
                let mut compile_result = None;
                let is_potential_expression = !code.ends_with(';') && !code.ends_with('}');

                if is_potential_expression {
                    // Try compiling as a REPL expression first
                    SUPPRESS_OUTPUT.with(|s| s.set(true));
                    let spec_res = Compiler::compile(source.clone(), None, vm.interner, &mut vm.functions, FunType::ReplExpression);
                    SUPPRESS_OUTPUT.with(|s| s.set(false));

                    if let Ok((fun, false)) = spec_res {
                        compile_result = Some((fun, false));
                    }
                }

                let compile_res = match compile_result {
                    Some(res) => Ok(res),
                    None => Compiler::compile(source, None, vm.interner, &mut vm.functions, FunType::Script),
                };

                match compile_res {
                    Ok((fun, had_error)) => {
                        if !had_error {
                            let result = executor::block_on(async { AssertUnwindSafe(vm.run_repl_chunk(fun)).catch_unwind().await });
                            match result {
                                Ok(Ok(())) => {}
                                Ok(Err(e)) => {
                                    println!("Runtime error: {}", e);
                                }
                                Err(_) => {}
                            }
                        }
                    }
                    Err(e) => {
                        println!("Compilation error: {}", e);
                    }
                }
            }
            // Reset for the next input
            accumulated_input.clear();
            checker = InputChecker::new();
        }
    }
}

fn main() {
    init(print, println);

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        run_repl();
        return;
    }

    if args.len() != 2 {
        help(&args);
        std::process::exit(1);
    }

    if args[1] == "-h" || args[1] == "--help" {
        help(&args);
        std::process::exit(0);
    }

    let file_path = &args[1];
    executor::block_on(run_file(file_path, read_async));
}
