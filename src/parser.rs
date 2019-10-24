use std::error::Error;
use std::fmt;
use crate::scanner::{Category, Token};

#[derive(Debug, Clone, Eq, PartialEq)]
struct TokenStream {
    tokens: Vec<Token>,
    current_index: usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        TokenStream {
            tokens,
            current_index: 0,
        }
    }
    pub fn take(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.current_index)?;
        self.current_index += 1;
        Some(token.clone())

    }
    pub fn put_back(&mut self) {
        self.current_index -= 1;
    }

    pub fn consume(&mut self, category: Category) -> Option<Token> {
        let token = &self.tokens[self.current_index];
        if token.category == category {
            self.current_index += 1;
            return Some(token.clone());
        }
        None
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken(token) => write!(
                f,
                "Unexpeted token at line {} column {}",
                token.line, token.column
            ),
        }
    }
}

impl Error for ParseError {}

fn parse_expression(stream: &[Token]) -> Result<(), ParseError> {
    Ok(())
}

fn parse_literal(stream: &[Token])-> Result<(), ParseError> {
    if stream.is_empty() {
        return Err(ParseError::UnexpectedToken(Token{column: 1, line: 1, category: Category::Integer, lexeme: String::from("1")}));
    }
    let token = stream.first().unwrap();
    match token.category {
        Category::Float => token,
        Category::Integer => token,
        Category::Character => token,
        Category::Text => token,
        _ => {return Err(ParseError::UnexpectedToken(token.clone()));},
    };
    Ok(())

}

fn parse_identifier(stream: &[Token])-> Result<(), ParseError> {
    if stream.is_empty() {
        return Err(ParseError::UnexpectedToken(Token{column: 1, line: 1, category: Category::Integer, lexeme: String::from("1")}));
    }
    let token = stream.first().unwrap();
    match token.category {
        Category::Identifier => token,
        _ => {return Err(ParseError::UnexpectedToken(token.clone()));},
    };
    Ok(())

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_correct_program() {
    }
}
