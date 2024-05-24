use std::char;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum TokenType {
    LeftParen, 
    RightParen,
    LeftBracket,
    RightBracket,
    Equal,
    String,
    Comma,
    Identifier,
    Argument,
    EOF,
    Body,
    Name,
    Id,
    Func,
    Attr,
    Value,
    Number,
    Left,
    Right,
    Op,
    Args,
    Module,
    FunctionDef,
    Call,
    Assign,
    BinOp,
    Constant,
    Return,
    Add,
    None
}


#[derive(Clone, Debug)]
pub struct Token {
    pub tType: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub literal: Option<String>,
}

pub struct Scanner {
    pub source: String,
    pub start: usize,
    pub current: usize,
    pub line: usize,
    pub tokens: Vec<Token>,
}


impl Scanner {

    pub fn new(source: String) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            line: 1,
            tokens: vec![],
        }
    }

    pub fn scan_tokens(&mut self) {

        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.add_token(TokenType::EOF, None);

    }

    fn scan_token(&mut self) {

        let c: char = self.advance();

        match c {
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '[' => self.add_token(TokenType::LeftBracket, None),
            ']' => self.add_token(TokenType::RightBracket, None),
            '=' => self.add_token(TokenType::Equal, None),
            ',' => self.add_token(TokenType::Comma, None),
            '\n' => self.line += 1,
            '\'' => self.string(),
            _ => {
                if c.is_alphabetic() {
                    self.identifier();
                } else if c.is_digit(10) {
                    self.number();
                } else {
                }

            },
        }

    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();

    }

    fn string(&mut self) {

        while self.peek()  != '\'' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        self.advance();

        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenType::String, Some(value));

    }

    fn identifier(&mut self) {

        while self.is_alphanurmeric(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let tType = match text {
            "Module" => TokenType::Module,
            "FunctionDef" => TokenType::FunctionDef,
            "Call" => TokenType::Call,
            "Assign" => TokenType::Assign,
            "BinOp" => TokenType::BinOp,
            "Constant" => TokenType::Constant,
            "Return" => TokenType::Return,
            "Add" => TokenType::Add,
            "Name" => TokenType::Name,
            _ => TokenType::None,
        };

        if tType == TokenType::None {
            let tType = match text {
                "body" => TokenType::Body,
                "name" => TokenType::Name,
                "id" => TokenType::Id,
                "func" => TokenType::Func,
                "attr" => TokenType::Attr,
                "value" => TokenType::Value,
                "left" => TokenType::Left,
                "right" => TokenType::Right,
                "op" => TokenType::Op,
                "args" => TokenType::Args,
                _ => TokenType::None,
            };
            self.add_token(tType, None);
            return;
        }

        self.add_token(tType,None);

    }
    
    fn number(&mut self) {
        while self.is_digit(self.peek())  {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        self.add_token(TokenType::Number, Some(self.source[self.start..self.current].to_string()));

    }

    fn is_alphanurmeric(&self, c: char) -> bool {
        return c.is_alphanumeric();
    }

    fn is_digit(&self, c: char) -> bool {
        return c.is_digit(10);
    }

    fn add_token(&mut self, t_type: TokenType, literal: Option<String>) {
        let text = self.source[self.start..self.current].to_string();
        let token = Token {
            tType: t_type,
            lexeme: text,
            literal,
            line: self.line,
        };
        self.tokens.push(token);
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        return self.source.chars().nth(self.current - 1).unwrap();
    }

    fn peek(&self) -> char {
        return self.source.chars().nth(self.current).unwrap();
    }

    fn peek_next(&self) -> char {
        return self.source.chars().nth(self.current + 1).unwrap();
    }
}
