use std::{borrow::Cow, collections::HashMap};

use serde::Deserialize;
use thiserror::Error;
use tracing::warn;

/// A Burn install condition. If the condition couldn't be parsed, it is stored as `None` and
/// always evaluates to `true`, matching how unknown variables are treated.
// https://docs.firegiant.com/wix3/tutorial/com-expression-syntax-miscellanea/expression-syntax/
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "Cow<str>")]
pub struct InstallCondition(Option<Expr>);

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("Expected {expected:?}, got {found:?}")]
    Expected {
        expected: Token,
        found: Option<Token>,
    },
    #[error("Expected a literal after {op:?} and name {name}, got {found:?}")]
    ExpectedLiteral {
        op: Token,
        name: String,
        found: Option<Token>,
    },
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Option<Token>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value<'manifest> {
    Bool(bool),
    Int(u32),
    Str(Cow<'manifest, str>),
}

impl InstallCondition {
    pub fn new<S: AsRef<str>>(input: S) -> Self {
        let input = input.as_ref();
        Self(
            Self::try_parse(input)
                .inspect_err(|error| {
                    warn!("Failed to parse Burn install condition `{input}`: {error}");
                })
                .ok(),
        )
    }

    pub fn try_parse(input: &str) -> Result<Expr, ParseError> {
        Parser::new(tokenize(input)).parse_expr()
    }

    pub fn evaluate(&self, variables: &HashMap<&str, Value>) -> bool {
        self.inner().is_none_or(|expr| expr.eval(variables))
    }

    #[inline]
    const fn inner(&self) -> Option<&Expr> {
        self.0.as_ref()
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

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    const fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.peek() == Some(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::Expected {
                expected,
                found: self.peek().cloned(),
            })
        }
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_and()?;
        while matches!(self.peek(), Some(Token::Or)) {
            self.advance();
            let rhs = self.parse_and()?;
            expr = Expr::Or(Box::new(expr), Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_not()?;
        while matches!(self.peek(), Some(Token::And)) {
            self.advance();
            let rhs = self.parse_not()?;
            expr = Expr::And(Box::new(expr), Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Some(Token::Not)) {
            self.advance();
            let expr = self.parse_primary()?;
            Ok(Expr::Not(Box::new(expr)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                if matches!(
                    self.peek(),
                    Some(
                        Token::Eq
                            | Token::NotEq
                            | Token::EqAnyCase
                            | Token::Gt
                            | Token::Ge
                            | Token::Lt
                            | Token::Le
                    )
                ) && let Some(op) = self.peek().cloned()
                {
                    self.advance();

                    let lit = match self.peek() {
                        Some(&Token::Number(num)) => {
                            self.advance();
                            Literal::Int(num)
                        }
                        Some(Token::String(s)) => {
                            let s = s.clone();
                            self.advance();
                            Literal::Str(s)
                        }
                        Some(Token::Ident(s)) => {
                            let s = s.clone();
                            self.advance();
                            Literal::Str(s)
                        }
                        other => {
                            return Err(ParseError::ExpectedLiteral {
                                op,
                                name,
                                found: other.cloned(),
                            });
                        }
                    };

                    Ok(match op {
                        Token::Eq => Expr::Eq(name, lit),
                        Token::NotEq => Expr::NotEq(name, lit),
                        Token::EqAnyCase => Expr::EqAnyCase(name, lit),
                        Token::Gt => Expr::Gt(name, lit),
                        Token::Ge => Expr::Ge(name, lit),
                        Token::Lt => Expr::Lt(name, lit),
                        Token::Le => Expr::Le(name, lit),
                        _ => unreachable!("op is one of the comparison tokens matched above"),
                    })
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Some(&Token::Number(num)) => {
                self.advance();
                Ok(Expr::Literal(Literal::Int(num)))
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::Str(s)))
            }
            other => Err(ParseError::UnexpectedToken(other.cloned())),
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
                } else if chars.peek() == Some(&'>') {
                    chars.next();
                    tokens.push(Token::NotEq);
                } else {
                    tokens.push(Token::Lt);
                }
            }
            '~' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::EqAnyCase);
                }
            }
            '0'..='9' => {
                let mut value = 0;
                while let Some(digit) = chars.peek().and_then(|char| char.to_digit(10)) {
                    value = value * 10 + digit;
                    chars.next();
                }
                tokens.push(Token::Number(value));
            }
            '"' => {
                chars.next();

                let mut value = String::new();

                while let Some(&char) = chars.peek() {
                    chars.next();

                    if char == '"' {
                        break;
                    }

                    value.push(char);
                }

                tokens.push(Token::String(value));
            }
            _ => {
                let mut ident = String::new();
                while let Some(&char) = chars.peek().filter(|char| {
                    !char.is_whitespace() && !['(', ')', '=', '<', '>'].contains(char)
                }) {
                    ident.push(char);
                    chars.next();
                }
                tokens.push(match ident.to_uppercase().as_str() {
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
    Literal(Literal),
    Var(String),
    Eq(String, Literal),
    NotEq(String, Literal),
    EqAnyCase(String, Literal),
    Gt(String, Literal),
    Ge(String, Literal),
    Lt(String, Literal),
    Le(String, Literal),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
}

impl Expr {
    pub fn eval(&self, variables: &HashMap<&str, Value>) -> bool {
        match self {
            Self::Literal(Literal::Int(int)) => *int != 0,
            Self::Literal(Literal::Str(str)) => !str.is_empty(),
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
            Self::NotEq(name, literal) => !Self::Eq(name.clone(), literal.clone()).eval(variables),
            Self::EqAnyCase(name, literal) => match (variables.get(name.as_str()), literal) {
                (Some(Value::Str(val)), Literal::Str(lit_str)) => val.eq_ignore_ascii_case(lit_str),
                _ => Self::Eq(name.clone(), literal.clone()).eval(variables),
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
    LParen,    // (
    RParen,    // )
    And,       // AND
    Or,        // OR
    Not,       // NOT
    Eq,        // =
    NotEq,     // <>
    EqAnyCase, // ~=
    Gt,        // >
    Lt,        // <
    Ge,        // >=
    Le,        // <=
    Ident(String),
    String(String),
    Number(u32),
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::rstest;

    use super::{InstallCondition, Value};

    fn variables() -> HashMap<&'static str, Value<'static>> {
        HashMap::from([
            ("VersionNT64", Value::Bool(true)),
            ("NativeMachine", Value::Int(0x8664)),
            ("NeedsMongooseVcRedist", Value::Int(1)),
        ])
    }

    #[rstest]
    #[case("1", true)]
    #[case("0", false)]
    #[case(r#""""#, false)]
    #[case(r#""value""#, true)]
    #[case("NOT VersionNT64", false)]
    #[case("VersionNT64", true)]
    #[case("NativeMachine = 43620", false)]
    #[case("VersionNT64 AND NativeMachine <> 43620", true)]
    #[case("NeedsMongooseVcRedist = 1", true)]
    #[case("NeedsMongooseVcRedist = 1 AND VersionNT64>v6.0 AND VersionNT64", true)]
    #[case(
        "( ((VersionNT = v6.1) AND (ServicePackLevel = 1)) OR (VersionNT >= v6.2) )",
        true
    )]
    fn evaluate_condition(#[case] condition: &str, #[case] expected: bool) {
        assert_eq!(
            InstallCondition::new(condition).evaluate(&variables()),
            expected
        );
    }

    #[rstest]
    #[case::unclosed_paren("(VersionNT64")]
    #[case::missing_literal("NativeMachine =")]
    #[case::operator_without_name("= 1")]
    #[case::dangling_and("VersionNT64 AND")]
    #[case::empty("")]
    fn malformed_condition(#[case] condition: &str) {
        assert!(InstallCondition::try_parse(condition).is_err());
        // An unparseable condition is treated like an unknown variable and evaluates to true
        assert!(InstallCondition::new(condition).evaluate(&variables()));
    }
}
