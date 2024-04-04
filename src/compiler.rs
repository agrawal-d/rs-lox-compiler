use crate::{
    chunk::Chunk,
    common::Opcode,
    jsprint, jsprintln,
    scanner::{Scanner, Token, TokenType},
    value::Value,
};
use anyhow::*;
use log::error;
use std::rc::Rc;

enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

struct Parser {
    pub scanner: Scanner,
    pub current: Token,
    pub previous: Token,
    pub had_error: bool,
    pub panic_mode: bool,
}

impl Parser {
    fn new(scanner: Scanner) -> Parser {
        Parser {
            scanner,
            current: Token::new(),
            previous: Token::new(),
            had_error: false,
            panic_mode: false,
        }
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(true, message);
    }

    fn error_at_previous(&mut self, message: &str) {
        self.error_at(false, message);
    }

    fn error_at(&mut self, current: bool, message: &str) {
        let current = if current {
            &self.current
        } else {
            &self.previous
        };

        if self.panic_mode {
            return;
        }

        self.panic_mode = true;
        jsprint!("[line {}] Error", current.line);

        if current.typ == TokenType::EOF {
            jsprint!(" at end");
        } else if current.typ == TokenType::Error {
            // Nothing.
        } else {
            eprint!(" at '{}'", current.source);
        }

        eprintln!(": {}", message);
        self.had_error = true;
    }

    fn consume(&mut self, typ: TokenType, message: &str) {
        if self.current.typ == typ {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        loop {
            self.current = self.scanner.scan_token();
            if self.current.typ != TokenType::Error {
                break;
            }

            let current_source: Rc<str> = self.current.source.clone();
            self.error_at_current(current_source.as_ref());
        }
    }
}

pub struct Compiler {
    compiling_chunk: Chunk,
    parser: Parser,
    line: usize,
}

impl Compiler {
    pub fn compile(source: Rc<str>) -> Result<Chunk> {
        let line: usize = 0;
        let scanner: Scanner = Scanner::new(source);
        let parser = Parser::new(scanner);
        let mut compiler = Compiler {
            compiling_chunk: Chunk::default(),
            line,
            parser,
        };

        compiler.parser.advance();
        compiler.expression();
        compiler
            .parser
            .consume(TokenType::EOF, "Expect end of expression.");
        compiler.end();
        panic!("Done")
    }

    fn end(&mut self) {
        self.emit_return();
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn grouping(&mut self) {
        self.expression();
        self.parser
            .consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn number(&mut self) {
        let value = self.parser.previous.source.parse::<f64>().unwrap();
        self.emit_constant(value);
    }

    fn unary(&mut self) {
        let operator_type = self.parser.previous.typ;
        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(Opcode::Negate as u8),
            _ => return,
        }
    }

    // Parse expressions with equal or higher precedence
    fn parse_precedence(&mut self, precedence: Precedence) {}

    fn make_constant(&mut self, value: Value) -> usize {
        self.compiling_chunk.add_constant(value)
    }

    fn emit_constant(&mut self, value: Value) {
        let index = self.compiling_chunk.add_constant(value);
        self.emit_bytes(Opcode::Constant as u8, index as u8);
    }

    fn emit_return(&mut self) {
        self.emit_byte(Opcode::Return as u8);
    }

    fn emit_byte(&mut self, byte: u8) {
        self.compiling_chunk.write_byte(byte, self.line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }
}
