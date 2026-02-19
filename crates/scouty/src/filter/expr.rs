//! Filter expression AST and parser.
//!
//! Grammar:
//!   expr     = or_expr
//!   or_expr  = and_expr ("OR" and_expr)*
//!   and_expr = not_expr ("AND" not_expr)*
//!   not_expr = "NOT" not_expr | primary
//!   primary  = "(" expr ")" | comparison
//!   comparison = field op value
//!   field    = identifier (dotted allowed: "metadata.key")
//!   op       = "=" | "!=" | ">" | ">=" | "<" | "<=" | "contains" | "starts_with" | "ends_with" | "regex"
//!   value    = quoted_string | unquoted_token

use std::fmt;

/// Comparison operators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Eq => write!(f, "="),
            Op::Ne => write!(f, "!="),
            Op::Gt => write!(f, ">"),
            Op::Ge => write!(f, ">="),
            Op::Lt => write!(f, "<"),
            Op::Le => write!(f, "<="),
            Op::Contains => write!(f, "contains"),
            Op::StartsWith => write!(f, "starts_with"),
            Op::EndsWith => write!(f, "ends_with"),
            Op::Regex => write!(f, "regex"),
        }
    }
}

/// Filter expression AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// field op value
    Comparison {
        field: String,
        op: Op,
        value: String,
    },
    /// expr AND expr
    And(Box<Expr>, Box<Expr>),
    /// expr OR expr
    Or(Box<Expr>, Box<Expr>),
    /// NOT expr
    Not(Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Comparison { field, op, value } => write!(f, "{} {} \"{}\"", field, op, value),
            Expr::And(l, r) => write!(f, "({} AND {})", l, r),
            Expr::Or(l, r) => write!(f, "({} OR {})", l, r),
            Expr::Not(e) => write!(f, "NOT {}", e),
        }
    }
}

/// Tokens for the expression lexer.
#[derive(Debug, Clone, PartialEq)]
enum Token {
    LParen,
    RParen,
    And,
    Or,
    Not,
    Op(Op),
    Str(String),
}

/// Tokenize an expression string.
fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Skip whitespace
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        // Parentheses
        if chars[i] == '(' {
            tokens.push(Token::LParen);
            i += 1;
            continue;
        }
        if chars[i] == ')' {
            tokens.push(Token::RParen);
            i += 1;
            continue;
        }

        // Multi-char operators
        if chars[i] == '!' && i + 1 < len && chars[i + 1] == '=' {
            tokens.push(Token::Op(Op::Ne));
            i += 2;
            continue;
        }
        if chars[i] == '>' && i + 1 < len && chars[i + 1] == '=' {
            tokens.push(Token::Op(Op::Ge));
            i += 2;
            continue;
        }
        if chars[i] == '<' && i + 1 < len && chars[i + 1] == '=' {
            tokens.push(Token::Op(Op::Le));
            i += 2;
            continue;
        }

        // Single-char operators
        if chars[i] == '=' {
            tokens.push(Token::Op(Op::Eq));
            i += 1;
            continue;
        }
        if chars[i] == '>' {
            tokens.push(Token::Op(Op::Gt));
            i += 1;
            continue;
        }
        if chars[i] == '<' {
            tokens.push(Token::Op(Op::Lt));
            i += 1;
            continue;
        }

        // Quoted string
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            i += 1;
            let mut s = String::new();
            while i < len && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < len && (chars[i + 1] == '\\' || chars[i + 1] == quote) {
                    i += 1;
                    s.push(chars[i]);
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            if i >= len {
                return Err(format!("Unterminated string starting with {}", quote));
            }
            i += 1; // skip closing quote
            tokens.push(Token::Str(s));
            continue;
        }

        // Identifier / keyword
        if chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '.' {
            let mut word = String::new();
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '.') {
                word.push(chars[i]);
                i += 1;
            }
            let token = match word.to_uppercase().as_str() {
                "AND" => Token::And,
                "OR" => Token::Or,
                "NOT" => Token::Not,
                "CONTAINS" => Token::Op(Op::Contains),
                "STARTS_WITH" => Token::Op(Op::StartsWith),
                "ENDS_WITH" => Token::Op(Op::EndsWith),
                "REGEX" => Token::Op(Op::Regex),
                _ => Token::Str(word),
            };
            tokens.push(token);
            continue;
        }

        return Err(format!("Unexpected character '{}' at position {}", chars[i], i));
    }

    Ok(tokens)
}

/// Recursive descent parser.
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    fn expect_str(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token::Str(s)) => Ok(s),
            other => Err(format!("Expected identifier or string, got {:?}", other)),
        }
    }

    /// expr = or_expr
    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    /// or_expr = and_expr ("OR" and_expr)*
    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.peek() == Some(&Token::Or) {
            self.next();
            let right = self.parse_and()?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// and_expr = not_expr ("AND" not_expr)*
    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_not()?;
        while self.peek() == Some(&Token::And) {
            self.next();
            let right = self.parse_not()?;
            left = Expr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// not_expr = "NOT" not_expr | primary
    fn parse_not(&mut self) -> Result<Expr, String> {
        if self.peek() == Some(&Token::Not) {
            self.next();
            let inner = self.parse_not()?;
            Ok(Expr::Not(Box::new(inner)))
        } else {
            self.parse_primary()
        }
    }

    /// primary = "(" expr ")" | comparison
    fn parse_primary(&mut self) -> Result<Expr, String> {
        if self.peek() == Some(&Token::LParen) {
            self.next();
            let expr = self.parse_expr()?;
            match self.next() {
                Some(Token::RParen) => Ok(expr),
                other => Err(format!("Expected ')', got {:?}", other)),
            }
        } else {
            self.parse_comparison()
        }
    }

    /// comparison = field op value
    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let field = self.expect_str()?;
        let op = match self.next() {
            Some(Token::Op(op)) => op,
            other => return Err(format!("Expected operator after '{}', got {:?}", field, other)),
        };
        let value = self.expect_str()?;
        Ok(Expr::Comparison { field, op, value })
    }
}

/// Parse a filter expression string into an AST.
pub fn parse(input: &str) -> Result<Expr, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;
    if parser.pos < parser.tokens.len() {
        return Err(format!(
            "Unexpected token at position {}: {:?}",
            parser.pos,
            parser.tokens[parser.pos]
        ));
    }
    Ok(expr)
}

/// Validate that any `regex` operator values are valid regexes.
pub fn validate(expr: &Expr) -> Result<(), String> {
    match expr {
        Expr::Comparison {
            op: Op::Regex,
            value,
            ..
        } => {
            regex::Regex::new(value)
                .map_err(|e| format!("Invalid regex '{}': {}", value, e))?;
            Ok(())
        }
        Expr::Comparison { .. } => Ok(()),
        Expr::And(l, r) | Expr::Or(l, r) => {
            validate(l)?;
            validate(r)
        }
        Expr::Not(inner) => validate(inner),
    }
}

#[cfg(test)]
#[path = "expr_tests.rs"]
mod expr_tests;
