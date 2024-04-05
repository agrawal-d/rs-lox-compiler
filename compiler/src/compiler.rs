use crate::{
    chunk::Chunk,
    common::Opcode,
    scanner::{Scanner, Token, TokenType},
    value::Value,
    xprint, xprintln,
};
use anyhow::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{collections::HashMap, rc::Rc, sync::OnceLock};

#[repr(u8)]
#[derive(Eq, Clone, Copy, TryFromPrimitive, PartialEq, PartialOrd, IntoPrimitive, strum_macros::Display)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

type Parsefn = fn(&mut Compiler);

struct ParseRule {
    prefix: Option<Parsefn>,
    infix: Option<Parsefn>,
    precedence: Precedence,
}

/// This is a table that, given a token type, lets us find
/// 1. the function to compile a prefix expression starting with a token of that type,
/// 2. the function to compile an infix expression whose left operand is followed by a token of that type, and
/// 3. the precedence of an infix expression that uses that token as an operator.
fn get_rules() -> &'static HashMap<TokenType, ParseRule> {
    static HASHMAP: OnceLock<HashMap<TokenType, ParseRule>> = OnceLock::new();

    HASHMAP.get_or_init(|| {
        let mut map = HashMap::new();
        use TokenType::*;

        macro_rules! add_rule {
            ($map: expr, $tokentype: expr, $prefix: expr, $infix: expr, $precedence: expr) => {
                $map.insert(
                    $tokentype,
                    ParseRule {
                        prefix: $prefix,
                        infix: $infix,
                        precedence: $precedence,
                    },
                );
            };
        }

        add_rule!(map, LeftParen, Some(Compiler::grouping), None, Precedence::None);
        add_rule!(map, RightParen, None, None, Precedence::None);
        add_rule!(map, LeftBrace, None, None, Precedence::None);
        add_rule!(map, RightBrace, None, None, Precedence::None);
        add_rule!(map, Comma, None, None, Precedence::None);
        add_rule!(map, Dot, None, None, Precedence::None);
        add_rule!(map, Minus, Some(Compiler::unary), Some(Compiler::binary), Precedence::Term);
        add_rule!(map, Plus, None, Some(Compiler::binary), Precedence::Term);
        add_rule!(map, Semicolon, None, None, Precedence::None);
        add_rule!(map, Slash, None, Some(Compiler::binary), Precedence::Factor);
        add_rule!(map, Star, None, Some(Compiler::binary), Precedence::Factor);
        add_rule!(map, Bang, None, None, Precedence::None);
        add_rule!(map, BangEqual, None, None, Precedence::None);
        add_rule!(map, Equal, None, None, Precedence::None);
        add_rule!(map, EqualEqual, None, None, Precedence::None);
        add_rule!(map, Greater, None, None, Precedence::None);
        add_rule!(map, GreaterEqual, None, None, Precedence::None);
        add_rule!(map, Less, None, None, Precedence::None);
        add_rule!(map, LessEqual, None, None, Precedence::None);
        add_rule!(map, Identifier, None, None, Precedence::None);
        add_rule!(map, String, None, None, Precedence::None);
        add_rule!(map, Number, Some(Compiler::number), None, Precedence::None);
        add_rule!(map, And, None, None, Precedence::None);
        add_rule!(map, Class, None, None, Precedence::None);
        add_rule!(map, Else, None, None, Precedence::None);
        add_rule!(map, False, None, None, Precedence::None);
        add_rule!(map, For, None, None, Precedence::None);
        add_rule!(map, Fun, None, None, Precedence::None);
        add_rule!(map, If, None, None, Precedence::None);
        add_rule!(map, Nil, None, None, Precedence::None);
        add_rule!(map, Or, None, None, Precedence::None);
        add_rule!(map, Print, None, None, Precedence::None);
        add_rule!(map, Return, None, None, Precedence::None);
        add_rule!(map, Super, None, None, Precedence::None);
        add_rule!(map, This, None, None, Precedence::None);
        add_rule!(map, True, None, None, Precedence::None);
        add_rule!(map, Var, None, None, Precedence::None);
        add_rule!(map, While, None, None, Precedence::None);
        add_rule!(map, Error, None, None, Precedence::None);
        add_rule!(map, EOF, None, None, Precedence::None);

        return map;
    })
}

fn get_rule(token_type: TokenType) -> &'static ParseRule {
    get_rules().get(&token_type).unwrap()
}

fn increment_prec(prec: Precedence) -> Precedence {
    (prec as u8 + 1).try_into().unwrap()
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
        let current = if current { &self.current } else { &self.previous };

        if self.panic_mode {
            return;
        }

        self.panic_mode = true;
        xprint!("[line {}] Error", current.line);

        if current.typ == TokenType::EOF {
            xprint!(" at end");
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
        compiler.parser.consume(TokenType::EOF, "Expect end of expression.");
        compiler.end();
        return Ok(compiler.compiling_chunk);
    }

    #[cfg(not(feature = "print_code"))]
    fn end(&mut self) {
        self.emit_return();
    }

    #[cfg(feature = "print_code")]
    fn end(&mut self) {
        self.emit_return();
        if !self.parser.had_error {
            self.compiling_chunk.disassemble("code");
        }
    }

    fn binary(&mut self) {
        let operator_type = self.parser.previous.typ;
        let rule = get_rule(operator_type);
        self.parse_precedence(increment_prec(rule.precedence));

        match operator_type {
            TokenType::Plus => self.emit_byte(Opcode::Add as u8),
            TokenType::Minus => self.emit_byte(Opcode::Subtract as u8),
            TokenType::Star => self.emit_byte(Opcode::Multiply as u8),
            TokenType::Slash => self.emit_byte(Opcode::Divide as u8),
            _ => return,
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn grouping(&mut self) {
        self.expression();
        self.parser.consume(TokenType::RightParen, "Expect ')' after expression.");
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
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.parser.advance();
        let prefix_rule = get_rule(self.parser.previous.typ).prefix;

        match prefix_rule {
            Some(rule) => rule(self),
            None => {
                self.parser.error_at_previous("Expect expression");
                return;
            }
        }

        while precedence <= get_rule(self.parser.current.typ).precedence {
            self.parser.advance();
            let infix_rule = get_rule(self.parser.previous.typ).infix;

            match infix_rule {
                Some(rule) => rule(self),
                None => {
                    self.parser.error_at_previous("Expect expression");
                    return;
                }
            }
        }
    }

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
