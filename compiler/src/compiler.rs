use crate::{
    chunk::Chunk,
    common::{identifiers_equal, Opcode},
    dbgln,
    interner::Interner,
    scanner::{Scanner, Token, TokenType},
    value::Value,
    xprint, xprintln,
};
use anyhow::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{collections::HashMap, rc::Rc};

#[repr(u8)]
#[derive(Eq, Clone, Copy, TryFromPrimitive, PartialEq, PartialOrd, IntoPrimitive, strum_macros::Display)]
// Low to High precedence
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

type Parsefn<'src> = fn(&mut Compiler<'src>, bool);

struct ParseRule<'src> {
    prefix: Option<Parsefn<'src>>,
    infix: Option<Parsefn<'src>>,
    precedence: Precedence,
}

struct Local {
    name: Token,
    depth: isize,
}

/// This is a table that, given a token type, lets us find
/// 1. the function to compile a prefix expression starting with a token of that type,
/// 2. the function to compile an infix expression whose left operand is followed by a token of that type, and
/// 3. the precedence of an infix expression that uses that token as an operator.
fn get_rules<'src>() -> HashMap<TokenType, ParseRule<'src>> {
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
    add_rule!(map, LeftBracket, None, None, Precedence::None);
    add_rule!(map, RightBracket, None, None, Precedence::None);
    add_rule!(map, Comma, None, None, Precedence::None);
    add_rule!(map, Dot, None, None, Precedence::None);
    add_rule!(map, Minus, Some(Compiler::unary), Some(Compiler::binary), Precedence::Term);
    add_rule!(map, Plus, None, Some(Compiler::binary), Precedence::Term);
    add_rule!(map, Semicolon, None, None, Precedence::None);
    add_rule!(map, Slash, None, Some(Compiler::binary), Precedence::Factor);
    add_rule!(map, Star, None, Some(Compiler::binary), Precedence::Factor);
    add_rule!(map, Modulo, None, Some(Compiler::binary), Precedence::Factor);
    add_rule!(map, Bang, Some(Compiler::unary), None, Precedence::None);
    add_rule!(map, BangEqual, None, Some(Compiler::binary), Precedence::Equality);
    add_rule!(map, Equal, None, None, Precedence::None);
    add_rule!(map, EqualEqual, None, Some(Compiler::binary), Precedence::Equality);
    add_rule!(map, Greater, None, Some(Compiler::binary), Precedence::Comparison);
    add_rule!(map, GreaterEqual, None, Some(Compiler::binary), Precedence::Comparison);
    add_rule!(map, Less, None, Some(Compiler::binary), Precedence::Comparison);
    add_rule!(map, LessEqual, None, None, Precedence::Comparison);
    add_rule!(map, Identifier, Some(Compiler::variable), None, Precedence::None);
    add_rule!(map, String, Some(Compiler::string), None, Precedence::None);
    add_rule!(map, Number, Some(Compiler::number), None, Precedence::None);
    add_rule!(map, And, None, Some(Compiler::and), Precedence::And);
    add_rule!(map, Class, None, None, Precedence::None);
    add_rule!(map, Else, None, None, Precedence::None);
    add_rule!(map, False, Some(Compiler::literal), None, Precedence::None);
    add_rule!(map, For, None, None, Precedence::None);
    add_rule!(map, Fun, None, None, Precedence::None);
    add_rule!(map, If, None, None, Precedence::None);
    add_rule!(map, Nil, Some(Compiler::literal), None, Precedence::None);
    add_rule!(map, Or, None, Some(Compiler::or), Precedence::Or);
    add_rule!(map, Print, None, None, Precedence::None);
    add_rule!(map, Return, None, None, Precedence::None);
    add_rule!(map, Super, None, None, Precedence::None);
    add_rule!(map, This, None, None, Precedence::None);
    add_rule!(map, True, Some(Compiler::literal), None, Precedence::None);
    add_rule!(map, Var, None, None, Precedence::None);
    add_rule!(map, While, None, None, Precedence::None);
    add_rule!(map, Error, None, None, Precedence::None);
    add_rule!(map, EOF, None, None, Precedence::None);

    map
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
        xprint!(" [line {}] Error", current.line);

        if current.typ == TokenType::EOF {
            xprint!(" at end");
        } else if current.typ == TokenType::Error {
            // Nothing.
        } else {
            eprint!(" at '{}'", current.source);
        }

        xprintln!(": {}", message);
        self.had_error = true;
    }

    fn consume(&mut self, typ: TokenType, message: &str) {
        if self.current.typ == typ {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn check_tt(&mut self, typ: TokenType) -> bool {
        self.current.typ == typ
    }

    fn match_tt(&mut self, typ: TokenType) -> bool {
        if !self.check_tt(typ) {
            return false;
        }

        self.advance();

        true
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        loop {
            self.current = self.scanner.scan_token();
            dbgln!("Current token: {}", self.current.typ);
            if self.current.typ != TokenType::Error {
                break;
            }

            let current_source: Rc<str> = self.current.source.clone();
            self.error_at_current(current_source.as_ref());
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.typ != TokenType::EOF {
            if self.previous.typ == TokenType::Semicolon {
                return;
            }

            match self.current.typ {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {}
            }

            self.advance()
        }
    }
}

pub struct Compiler<'src> {
    compiling_chunk: Chunk,
    parser: Parser,
    line: usize,
    interner: &'src mut Interner,
    rules: HashMap<TokenType, ParseRule<'src>>,
    locals: Vec<Local>,
    scope_depth: isize,
}

impl<'src> Compiler<'src> {
    pub fn compile(source: Rc<str>, interner: &mut Interner) -> Result<Chunk> {
        let line: usize = 0;
        let scanner: Scanner = Scanner::new(source);
        let parser = Parser::new(scanner);
        let rules = get_rules();
        let mut compiler = Compiler {
            compiling_chunk: Chunk::default(),
            line,
            parser,
            interner,
            rules,
            locals: Default::default(),
            scope_depth: 0,
        };

        dbgln!("== Parser (Scan on demand) ==");

        compiler.parser.advance();
        while !compiler.parser.match_tt(TokenType::EOF) {
            compiler.declaration();
        }

        compiler.end();
        Ok(compiler.compiling_chunk)
    }

    #[cfg(not(feature = "print_code"))]
    fn end(&mut self) {
        self.emit_return();
    }

    #[cfg(feature = "print_code")]
    fn end(&mut self) {
        self.emit_return();
        if !self.parser.had_error {
            self.compiling_chunk.disassemble("code", self.interner);
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        while !self.locals.is_empty() && self.locals.last().unwrap().depth > self.scope_depth {
            self.emit_byte(Opcode::Pop as u8);
            self.locals.pop();
        }
    }

    fn get_rule(&self, token_type: TokenType) -> &ParseRule<'src> {
        self.rules.get(&token_type).unwrap()
    }

    fn binary(&mut self, _can_assign: bool) {
        let operator_type = self.parser.previous.typ;
        let rule = self.get_rule(operator_type);
        self.parse_precedence(increment_prec(rule.precedence));

        match operator_type {
            TokenType::Plus => self.emit_byte(Opcode::Add as u8),
            TokenType::Minus => self.emit_byte(Opcode::Subtract as u8),
            TokenType::Star => self.emit_byte(Opcode::Multiply as u8),
            TokenType::Modulo => self.emit_byte(Opcode::Modulo as u8),
            TokenType::Slash => self.emit_byte(Opcode::Divide as u8),
            TokenType::BangEqual => self.emit_bytes(Opcode::Equal as u8, Opcode::Not as u8),
            TokenType::EqualEqual => self.emit_byte(Opcode::Equal as u8),
            TokenType::Greater => self.emit_byte(Opcode::Greater as u8),
            TokenType::GreaterEqual => self.emit_bytes(Opcode::Less as u8, Opcode::Not as u8),
            TokenType::Less => self.emit_byte(Opcode::Less as u8),
            TokenType::LessEqual => self.emit_bytes(Opcode::Greater as u8, Opcode::Not as u8),
            _ => (),
        }
    }

    fn literal(&mut self, _can_assign: bool) {
        match self.parser.previous.typ {
            TokenType::False => self.emit_byte(Opcode::False as u8),
            TokenType::Nil => self.emit_byte(Opcode::Nil as u8),
            TokenType::True => self.emit_byte(Opcode::True as u8),
            _ => (),
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn block(&mut self) {
        while !self.parser.check_tt(TokenType::RightBrace) && !self.parser.check_tt(TokenType::EOF) {
            self.declaration();
        }

        self.parser.consume(TokenType::RightBrace, "Expect '}' after block");
    }

    fn var_declaration(&mut self) {
        let (global_variable_idx, is_array) = self.parse_variable("Expect variable name");

        if is_array {
            self.expression();
            self.emit_byte(Opcode::DeclareArray as u8);
            self.parser.consume(TokenType::RightBracket, "Expect ']' after array size");
        } else if self.parser.match_tt(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(Opcode::Nil as u8);
        }

        self.parser.consume(TokenType::Semicolon, "Expect ';' after variable declaration");
        self.define_variable(global_variable_idx, is_array);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.parser.consume(TokenType::Semicolon, "Expect ';' after expression");
        self.emit_byte(Opcode::Pop as u8);
    }

    fn for_statement(&mut self) {
        self.begin_scope();
        self.parser.consume(TokenType::LeftParen, "Expect '(' after for");

        if self.parser.match_tt(TokenType::Semicolon) {
            // No initializer
        } else if self.parser.match_tt(TokenType::Var) {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.compiling_chunk.code.len();

        let mut exit_jump = usize::MAX;
        if !self.parser.match_tt(TokenType::Semicolon) {
            self.expression();
            self.parser.consume(TokenType::Semicolon, "Expect ';' after loop condition");

            exit_jump = self.emit_jump(Opcode::JumpIfFalse as u8);
            self.emit_byte(Opcode::Pop as u8);
        }

        if !self.parser.match_tt(TokenType::RightParen) {
            let body_jump = self.emit_jump(Opcode::Jump as u8);
            let increment_start = self.compiling_chunk.code.len();
            self.expression();
            self.emit_byte(Opcode::Pop as u8);
            self.parser.consume(TokenType::RightParen, "Expect ')' after for clauses");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if exit_jump != usize::MAX {
            self.patch_jump(exit_jump);
            self.emit_byte(Opcode::Pop as u8);
        }

        self.end_scope();
    }

    fn print_statement(&mut self) {
        self.expression();
        self.parser.consume(TokenType::Semicolon, "Expect ';' after expression");
        self.emit_byte(Opcode::Print as u8);
    }

    fn if_statement(&mut self) {
        self.parser.consume(TokenType::LeftParen, "Expect '(' after 'if'");
        self.expression();
        self.parser.consume(TokenType::RightParen, "Expect ')' after condition");

        let then_jump = self.emit_jump(Opcode::JumpIfFalse as u8);
        self.emit_byte(Opcode::Pop as u8);
        self.statement();

        let else_jump = self.emit_jump(Opcode::Jump as u8);
        self.patch_jump(then_jump);
        self.emit_byte(Opcode::Pop as u8);

        if self.parser.match_tt(TokenType::Else) {
            self.statement();
        }

        self.patch_jump(else_jump);
    }

    fn while_statement(&mut self) {
        let loop_start = self.compiling_chunk.code.len();
        self.parser.consume(TokenType::LeftParen, "Expect '(' after 'while'");
        self.expression();
        self.parser.consume(TokenType::RightParen, "Expect ')' after condition");

        let exit_jump = self.emit_jump(Opcode::JumpIfFalse as u8);
        self.emit_byte(Opcode::Pop as u8);
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(Opcode::Pop as u8);
    }

    fn declaration(&mut self) {
        if self.parser.match_tt(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.parser.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.parser.match_tt(TokenType::Print) {
            self.print_statement();
        } else if self.parser.match_tt(TokenType::If) {
            self.if_statement();
        } else if self.parser.match_tt(TokenType::While) {
            self.while_statement();
        } else if self.parser.match_tt(TokenType::For) {
            self.for_statement();
        } else if self.parser.match_tt(TokenType::LeftBrace) {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn grouping(&mut self, _can_assign: bool) {
        self.expression();
        self.parser.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn number(&mut self, _can_assign: bool) {
        let num = self.parser.previous.source.parse::<f64>().unwrap();
        self.emit_constant(Value::Number(num));
    }

    fn string(&mut self, _can_assign: bool) {
        let data = self.parser.previous.source.clone();
        let data = &data[1..data.len() - 1];
        let id = self.interner.intern(data);
        self.emit_constant(Value::Str(id));
    }

    fn named_variable(&mut self, token: &Token, can_assign: bool) {
        let get_op: Opcode;
        let set_op: Opcode;
        let mut arg: isize = self.resolve_local(token);

        if arg != -1 {
            set_op = Opcode::SetLocal;
            get_op = Opcode::GetLocal;
        } else {
            arg = self.identifier_constant(token) as isize;
            set_op = Opcode::SetGlobal;
            get_op = Opcode::GetGlobal;
        }

        if self.array_access_index() {
            dbgln!("{} is array access", token.source);
        }

        if can_assign && self.parser.match_tt(TokenType::Equal) {
            self.expression();
            self.emit_bytes(set_op as u8, arg as u8);
        } else {
            self.emit_bytes(get_op as u8, arg as u8);
        }
    }

    // Array index (or max value if not an array index)
    fn array_access_index(&mut self) -> bool {
        // Array index (or max value if not an array index)
        if self.parser.match_tt(TokenType::LeftBracket) {
            self.expression();
            self.parser.consume(TokenType::RightBracket, "Expect ']' after array index");
            true
        } else {
            self.emit_constant(Value::Nil);
            false
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(&self.parser.previous.clone(), can_assign);
    }

    fn unary(&mut self, can_assign: bool) {
        let _ = can_assign;
        let operator_type = self.parser.previous.typ;
        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(Opcode::Negate as u8),
            TokenType::Bang => self.emit_byte(Opcode::Not as u8),
            _ => (),
        }
    }

    /// Parse a variable.
    /// If its a global, the return value is the index in the constant pool.
    /// If its an array, the boolean value is true.
    fn parse_variable(&mut self, error_message: &str) -> (usize, bool) {
        self.parser.consume(TokenType::Identifier, error_message);
        let array_name: Token = self.parser.previous.clone();
        let is_array = self.parser.match_tt(TokenType::LeftBracket);

        if is_array {
            self.declare_local_variable(Some(array_name));
        } else {
            self.declare_local_variable(None);
        }

        if self.scope_depth > 0 || is_array {
            return (0, is_array);
        }

        let previous = &self.parser.previous.clone();
        (self.identifier_constant(previous), false)
    }

    /// Parse expressions with equal or higher precedence
    fn parse_precedence(&mut self, precedence: Precedence) {
        self.parser.advance();
        let prefix_rule = self.get_rule(self.parser.previous.typ).prefix;
        let can_assign = precedence <= Precedence::Assignment;

        match prefix_rule {
            Some(rule) => rule(self, can_assign),
            None => {
                self.parser.error_at_previous("Expect expression");
                return;
            }
        }

        while precedence <= self.get_rule(self.parser.current.typ).precedence {
            self.parser.advance();
            let infix_rule = self.get_rule(self.parser.previous.typ).infix;

            match infix_rule {
                Some(rule) => rule(self, can_assign),
                None => {
                    self.parser.error_at_previous("Expect expression");
                    return;
                }
            }
        }

        if can_assign && self.parser.match_tt(TokenType::Equal) {
            self.parser.error_at_current("Invalid assignment target");
        }
    }

    fn identifier_constant(&mut self, name: &Token) -> usize {
        let identifier = self.interner.intern(name.source.as_ref());
        self.make_constant(Value::Identifier(identifier))
    }

    fn mark_initialized(&mut self) {
        self.locals.last_mut().unwrap().depth = self.scope_depth;
    }

    fn resolve_local(&mut self, name: &Token) -> isize {
        dbgln!("Resolving local: {}", name.source);
        for (i, local) in self.locals.iter().enumerate().rev() {
            if identifiers_equal(&local.name, name) {
                if local.depth == -1 {
                    self.parser.error_at_current("Can't read local variable in its own initializer")
                }

                return i as isize;
            }
        }

        -1
    }

    fn add_local(&mut self, name: Token) {
        let local = Local {
            name: name.clone(),
            depth: -1,
        };

        dbgln!("Adding local: {}", name.source);

        self.locals.push(local);
    }

    fn declare_local_variable(&mut self, array: Option<Token>) {
        if self.scope_depth == 0 && array.is_none() {
            return;
        }

        let name = match array {
            Some(token) => token,
            None => self.parser.previous.clone(),
        };

        for local in self.locals.iter().rev() {
            if local.depth != -1 && local.depth < self.scope_depth {
                break;
            }

            if identifiers_equal(&name, &local.name) {
                self.parser.error_at_current("Already a variable with this name in this scope");
            }
        }

        self.add_local(name);
    }

    fn define_variable(&mut self, global: usize, is_array: bool) {
        if self.scope_depth > 0 || is_array {
            self.mark_initialized();
            return;
        }

        self.emit_bytes(Opcode::DefineGlobal as u8, global as u8);
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(Opcode::JumpIfFalse as u8);

        self.emit_byte(Opcode::Pop as u8);
        self.parse_precedence(Precedence::And);

        self.patch_jump(end_jump);
    }

    fn or(&mut self, _can_assign: bool) {
        let else_jump = self.emit_jump(Opcode::JumpIfFalse as u8);
        let end_jump = self.emit_jump(Opcode::Jump as u8);

        self.patch_jump(else_jump);
        self.emit_byte(Opcode::Pop as u8);

        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn make_constant(&mut self, value: Value) -> usize {
        self.compiling_chunk.add_constant(value)
    }

    fn emit_constant(&mut self, value: Value) {
        let index = self.compiling_chunk.add_constant(value);
        self.emit_bytes(Opcode::Constant as u8, index as u8);
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.compiling_chunk.code.len() - offset - 2;

        if jump > u16::MAX as usize {
            self.parser.error_at_current("Too much code to jump over");
        }

        let jump = jump as u16;
        self.compiling_chunk.code[offset] = ((jump >> 8) & 0xff) as u8;
        self.compiling_chunk.code[offset + 1] = (jump & 0xff) as u8;
    }

    fn emit_return(&mut self) {
        self.emit_byte(Opcode::Return as u8);
    }

    fn emit_byte(&mut self, byte: u8) {
        self.compiling_chunk.write_byte(byte, self.line);
    }

    fn emit_jump(&mut self, instr: u8) -> usize {
        self.emit_byte(instr);
        self.emit_bytes(0xff, 0xff);
        self.compiling_chunk.code.len() - 2
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_byte(Opcode::Loop as u8);

        let offset = self.compiling_chunk.code.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            self.parser.error_at_current("Loop body too large");
        }

        let offset = offset as u16;
        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }
}
