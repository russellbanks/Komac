use std::{borrow::Cow, collections::HashMap};

use serde::Deserialize;
use tracing::warn;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "Cow<str>")]
pub struct InstallCondition(Expr);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value<'manifest> {
    Bool(bool),
    Int(u32),
    Str(Cow<'manifest, str>),
}

impl InstallCondition {
    pub fn new<S: AsRef<str>>(input: S) -> Self {
        Self(Parser::new(tokenize(input.as_ref())).parse_expr())
    }

    pub fn evaluate(&self, variables: &HashMap<&str, Value>) -> bool {
        self.inner().eval(variables)
    }

    #[inline]
    const fn inner(&self) -> &Expr {
        &self.0
    }
}

impl From<Cow<'_, str>> for InstallCondition {
    fn from(s: Cow<str>) -> Self {
        Self::new(s)
    }
}

impl From<&str> for InstallCondition {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    #[inline]
    const fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_or()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    const fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: &Token) {
        if self.peek() == Some(expected) {
            self.advance();
        } else {
            panic!("Expected {expected:?}, got {:?}", self.peek());
        }
    }

    fn parse_or(&mut self) -> Expr {
        let mut expr = self.parse_and();
        while matches!(self.peek(), Some(Token::Or)) {
            self.advance();
            let rhs = self.parse_and();
            expr = Expr::Or(Box::new(expr), Box::new(rhs));
        }
        expr
    }

    fn parse_and(&mut self) -> Expr {
        let mut expr = self.parse_not();
        while matches!(self.peek(), Some(Token::And)) {
            self.advance();
            let rhs = self.parse_not();
            expr = Expr::And(Box::new(expr), Box::new(rhs));
        }
        expr
    }

    fn parse_not(&mut self) -> Expr {
        if matches!(self.peek(), Some(Token::Not)) {
            self.advance();
            let expr = self.parse_primary();
            Expr::Not(Box::new(expr))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Expr {
        match self.peek() {
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr();
                self.expect(&Token::RParen);
                expr
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                if matches!(
                    self.peek(),
                    Some(Token::Eq | Token::Gt | Token::Ge | Token::Lt | Token::Le)
                ) {
                    let op = self.peek().cloned().unwrap();
                    self.advance();

                    let lit = match self.peek() {
                        Some(&Token::Number(num)) => {
                            self.advance();
                            Literal::Int(num)
                        }
                        Some(Token::Ident(s)) => {
                            let s = s.clone();
                            self.advance();
                            Literal::Str(s)
                        }
                        other => panic!("Expected literal after operator, got {other:?}"),
                    };

                    match op {
                        Token::Eq => Expr::Eq(name, lit),
                        Token::Gt => Expr::Gt(name, lit),
                        Token::Ge => Expr::Ge(name, lit),
                        Token::Lt => Expr::Lt(name, lit),
                        Token::Le => Expr::Le(name, lit),
                        _ => unreachable!(),
                    }
                } else {
                    Expr::Var(name)
                }
            }
            other => panic!("Unexpected token: {other:?}"),
        }
    }
}

fn tokenize(input: &str) -> Vec<Token> {
    const AND: &str = "AND";
    const OR: &str = "OR";
    const NOT: &str = "NOT";

    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&char) = chars.peek() {
        match char {
            char if char.is_whitespace() => {
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '=' => {
                tokens.push(Token::Eq);
                chars.next();
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Ge);
                } else {
                    tokens.push(Token::Gt);
                }
            }
            '<' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Le);
                } else {
                    tokens.push(Token::Lt);
                }
            }
            '0'..='9' => {
                let mut value = 0;
                while let Some(digit) = chars.peek().and_then(|char| char.to_digit(10)) {
                    value *= 10 + digit;
                    chars.next();
                }
                tokens.push(Token::Number(value));
            }
            _ => {
                let mut ident = String::new();
                while let Some(&char) = chars.peek().filter(|char| {
                    !char.is_whitespace() && !['(', ')', '=', '<', '>'].contains(char)
                }) {
                    ident.push(char);
                    chars.next();
                }
                tokens.push(match ident.as_str() {
                    AND => Token::And,
                    OR => Token::Or,
                    NOT => Token::Not,
                    _ => Token::Ident(ident),
                });
            }
        }
    }

    tokens
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Literal {
    Int(u32),
    Str(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Var(String),
    Eq(String, Literal),
    Gt(String, Literal),
    Ge(String, Literal),
    Lt(String, Literal),
    Le(String, Literal),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

impl Expr {
    pub fn eval(&self, variables: &HashMap<&str, Value>) -> bool {
        match self {
            Self::Var(name) => match variables.get(name.as_str()) {
                Some(&Value::Bool(bool)) => bool,
                Some(&Value::Int(int)) => int != 0,
                Some(Value::Str(str)) => !str.is_empty(),
                None => {
                    warn!("Variable `{name}` not found in Burn variables");
                    true
                }
            },
            Self::Eq(name, literal) => match (variables.get(name.as_str()), literal) {
                (Some(Value::Int(int)), Literal::Int(lit_int)) => int == lit_int,
                (Some(Value::Str(val)), Literal::Str(lit_str)) => val == lit_str,
                (Some(Value::Bool(bool)), &Literal::Int(lit_int)) => (lit_int == 1) == *bool,
                (None, _) => {
                    warn!("Variable `{name}` not found in Burn variables");
                    true
                }
                _ => true,
            },
            Self::Gt(name, literal) => match (variables.get(name.as_str()), literal) {
                (Some(Value::Int(int)), Literal::Int(lit_int)) => int > lit_int,
                (None, _) => {
                    warn!("Variable `{name}` not found in Burn variables");
                    true
                }
                _ => true,
            },
            Self::Ge(name, literal) => match (variables.get(name.as_str()), literal) {
                (Some(Value::Int(int)), Literal::Int(lit_int)) => int >= lit_int,
                (None, _) => {
                    warn!("Variable `{name}` not found in Burn variables");
                    true
                }
                _ => true,
            },
            Self::Lt(name, literal) => match (variables.get(name.as_str()), literal) {
                (Some(Value::Int(int)), Literal::Int(lit_int)) => int < lit_int,
                (None, _) => {
                    warn!("Variable `{name}` not found in Burn variables");
                    true
                }
                _ => true,
            },
            Self::Le(name, literal) => match (variables.get(name.as_str()), literal) {
                (Some(Value::Int(int)), Literal::Int(lit_int)) => int <= lit_int,
                (None, _) => {
                    warn!("Variable `{name}` not found in Burn variables");
                    true
                }
                _ => true,
            },
            Self::Not(inner) => !inner.eval(variables),
            Self::And(lhs, rhs) => lhs.eval(variables) && rhs.eval(variables),
            Self::Or(lhs, rhs) => lhs.eval(variables) || rhs.eval(variables),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    LParen, // (
    RParen, // )
    And,    // AND
    Or,     // OR
    Not,    // NOT
    Eq,     // =
    Gt,     // >
    Lt,     // <
    Ge,     // >=
    Le,     // <=
    Ident(String),
    Number(u32),
}
