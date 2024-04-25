use std::{collections::HashMap, rc::Rc, sync::OnceLock};

pub struct Scanner {
    start: usize,
    current: usize,
    source: Rc<str>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub typ: TokenType,
    pub source: Rc<str>,
    pub line: usize,
}

impl Default for Token {
    fn default() -> Self {
        Self::new()
    }
}

impl Token {
    pub fn new() -> Token {
        Token {
            typ: TokenType::Error,
            source: Rc::from(""),
            line: 0,
        }
    }
}

impl Scanner {
    pub fn new(source: Rc<str>) -> Scanner {
        Scanner {
            start: 0,
            current: 0,
            source,
            line: 1,
        }
    }

    fn get_ident_tokentype_map() -> &'static HashMap<&'static str, TokenType> {
        static HASHMAP: OnceLock<HashMap<&'static str, TokenType>> = OnceLock::new();
        HASHMAP.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("and", TokenType::And);
            m.insert("class", TokenType::Class);
            m.insert("else", TokenType::Else);
            m.insert("if", TokenType::If);
            m.insert("nil", TokenType::Nil);
            m.insert("or", TokenType::Or);
            m.insert("print", TokenType::Print);
            m.insert("return", TokenType::Return);
            m.insert("super", TokenType::Super);
            m.insert("var", TokenType::Var);
            m.insert("while", TokenType::While);
            m.insert("for", TokenType::For);
            m.insert("true", TokenType::True);
            m.insert("false", TokenType::False);
            m
        })
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::EOF);
        }

        match self.advance() {
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            '[' => self.make_token(TokenType::LeftBracket),
            ']' => self.make_token(TokenType::RightBracket),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            ';' => self.make_token(TokenType::Semicolon),
            '*' => self.make_token(TokenType::Star),
            '!' => {
                if self.match_char('=') {
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                }
            }
            '/' => {
                if self.peek2() == '/' {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    self.scan_token()
                } else {
                    self.make_token(TokenType::Slash)
                }
            }
            '"' => self.string(),
            c => {
                if c.is_ascii_digit() {
                    self.number()
                } else if c.is_alphabetic() {
                    self.identifier()
                } else {
                    self.error_token(format!("Unexpected character '{}' at position {}", c, self.start))
                }
            }
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.current) != Some(expected) {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source
            .chars()
            .nth(self.current)
            .unwrap_or_else(|| panic!("Could not get {}th  character", self.current))
    }

    fn peek2(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        self.source
            .chars()
            .nth(self.current + 1)
            .unwrap_or_else(|| panic!("Could not get {}th  character", self.current + 1))
    }

    pub fn advance(&mut self) -> char {
        self.current += 1;
        return self
            .source
            .chars()
            .nth(self.current - 1)
            .unwrap_or_else(|| panic!("Could not get {}th  character", self.current - 1));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn skip_whitespace(&mut self) {
        while let ' ' | '\r' | '\t' | '\n' = self.peek() {
            self.advance();
        }
    }

    pub fn make_token(&self, typ: TokenType) -> Token {
        Token {
            typ,
            source: self.source[self.start..self.current].into(),
            line: self.line,
        }
    }

    fn error_token(&self, msg: String) -> Token {
        Token {
            typ: TokenType::Error,
            source: Rc::from(msg),
            line: self.line,
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token(String::from("Unterminated string"));
        }

        // The closing quote
        self.advance();

        self.make_token(TokenType::String)
    }

    fn number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek2().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token {
        while self.peek().is_alphanumeric() {
            self.advance();
        }

        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        let text: &str = &self.source[self.start..self.current];
        *Scanner::get_ident_tokentype_map().get(text).unwrap_or(&TokenType::Identifier)
    }
}

#[derive(strum_macros::Display, PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub enum TokenType {
    // Single char
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two chars
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Error,
    EOF,
}
