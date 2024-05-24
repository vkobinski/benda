use core::panic;
use std::{fmt::format, fs::File, io::Write, vec};

use super::scanner::{Token, TokenType};

trait Stmt {
    fn resolve(&self) -> String;
}

struct Module {
    body: Vec<Box<dyn Stmt>>,
}

impl Stmt for Module {
    fn resolve(&self) -> String {
        let mut lines: Vec<String> = vec!();

        let iter = self.body.iter();

        for func in iter {
            let function = func.resolve();
            lines.push(format!("{}\n", function));
            lines.push("def main:\n".to_string());
            let name = function.split("\n").nth(0).unwrap().split(" ").nth(1).unwrap().replace(":", "");
            lines.push(format!("  val = {}\n", name));
            lines.push(format!("  return val"));
        };


        lines.concat()
    }
}

struct FunctionDef {
    name: String,
    args: Option<Vec<String>>,
    body: Vec<Box<dyn Stmt>>,
}


impl Stmt for FunctionDef {
    fn resolve(&self) -> String {
        let mut lines:Vec<String> = vec!();
        lines.push(format!("def {}():\n", self.name));
        lines.push(format!("  {}", self.body.get(0).unwrap().resolve()));

        lines.concat()
    }
}

struct Return {
    value: Option<Box<dyn Stmt>>,
}

impl Stmt for Return {
    fn resolve(&self) -> String {
        return format!("return {}", self.value.as_ref().unwrap().resolve())
    }
}

struct Constant {
    value: Token,
}

impl Stmt for Constant {
    fn resolve(&self) -> String {
        return self.value.lexeme.to_string();
    }
}


pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current : 0,
        }
    }

    pub fn parse(&mut self) {
        let mut statements: Vec<Box<dyn Stmt>> = vec![];

        statements.push(self.module());


        let mut file = File::create("test.bend").unwrap();
        let bytes = file.write(statements.get(0).unwrap().resolve().as_bytes());

        let _ = file.flush();

    }

    fn module(&mut self) -> Box<dyn Stmt> {
        let mut body: Vec<Box<dyn Stmt>> = vec![];

        self.consume_until(TokenType::LeftBracket);

        body.push(self.function_def());

        self.consume_until(TokenType::RightParen);

        return Box::new(Module { body });

    }

    fn function_def(&mut self) -> Box<dyn Stmt> {
        self.consume(TokenType::FunctionDef);
        self.consume(TokenType::LeftParen);
        let name = self.argument(TokenType::Name);
        // Ignore args
        self.consume_until(TokenType::Body);
        self.consume_until(TokenType::LeftBracket);
        let mut body: Vec<Box<dyn Stmt>> = vec![];

        body.push(self.return_stmt());

        self.consume_until(TokenType::RightParen);

        // Add args parsing
        return Box::new(FunctionDef { name: name.literal.unwrap().clone(), args: None, body });
    }

    fn return_stmt(&mut self) -> Box<dyn Stmt> {
        match self.consume(TokenType::Return) {
            Some(_) => {
                self.consume(TokenType::LeftParen);
                let val = self.argument(TokenType::Value);
                let constant = Constant {value: val.clone()};
                self.consume_until(TokenType::RightParen);
                return Box::new(Return {value: Some(Box::new(constant))});
            },
            None => {
                panic!("Missing return!");
            }
        }

    }

    fn argument(&mut self, token: TokenType) -> Token {
        self.consume(token);
        match self.consume(TokenType::Equal) {
            Some(_) => {
                let next = self.peek();
                let token = match next.tType {
                    TokenType::Constant => {
                        self.consume_until(TokenType::Equal);
                        self.consume(TokenType::Number).unwrap().clone()
                    },
                    TokenType::String => {
                        self.consume(TokenType::String).unwrap().clone()
                    },
                    _ => panic!("Can't parse this Token"),
                };
                if self.peek().tType == TokenType::Comma {
                    self.advance();
                };
                token
            },
            _ => {
                panic!("Implement missing arg");
            }
        }

        
    }

    fn arguments(&mut self) {
        self.consume_until(TokenType::RightBracket);
    }

    fn consume_until(&mut self, tType: TokenType) {
        while self.advance().tType != tType {
        }
    }

    fn is_at_end(&self) -> bool {
        return match self.peek().tType  {
            TokenType::EOF => true,
            _ => false,
        };
    }

    fn consume(&mut self, token: TokenType) -> Option<&Token> {
        if self.check(token) {
            return Some(self.advance());
        }

        return None;
    }

    fn check(&self, tType: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        let cur = &self.peek().tType;
        return *cur == tType;

    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        return self.previous();
    }

    fn peek(&self) -> &Token {
        return self.tokens.get(self.current).unwrap();

    }

    fn previous(&self) -> &Token {
        return self.tokens.get(self.current-1).unwrap();
    }

}