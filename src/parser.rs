use std::error::Error;
use std::fmt;
use std::iter::Iterator;
use crate::scanner::{Category, Token};

#[derive(Debug, Clone, Eq, PartialEq)]
struct TokenStream {
    tokens: Vec<Token>,
    current_index: usize,
}

impl TokenStream {
    pub fn new(tokens: &[Token]) -> Self {
        TokenStream {
            tokens: tokens.to_vec(),
            current_index: 0,
        }
    }
    pub fn take(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.current_index)?;
        self.current_index += 1;
        println!("{:?}", self);
        Some(token.clone())
    }

    pub fn put_back(&mut self) {
        self.current_index -= 1;
    }

    pub fn is_empty(&self) -> bool {
        self.current_index == self.tokens.len()
    }

    pub fn how_many_are_remaining(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.tokens.len() - (self.current_index + 1)
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken{unexpected: Token, expected: &'static[Category]},
    ExpectedButMissingToken{after: Token, expected: &'static[Category]},
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken{unexpected, expected} => write!(
                f,
                "Unexpected token at line {} column {}. Found {}, expected {:?}",
                unexpected.line, unexpected.column, unexpected.category, expected
            ),
            ParseError::ExpectedButMissingToken{after, expected} => write!(
                f,
                "Expected one of {:?} after {} at line {} column {}",
                expected, after.category, after.line, after.column
            ),
        }
    }
}

impl Error for ParseError {}

fn parse_token(categories: &'static[Category], stream: &mut TokenStream) -> Result<Token, ParseError>
{
    let next = stream.take();
    if next.is_none() {
        stream.put_back();
        stream.put_back();
        let last_token = stream.take().unwrap();
        let line = last_token.line;
        let column = last_token.column + last_token.lexeme.len();
        return Err(ParseError::ExpectedButMissingToken{after: last_token, expected: categories});
    }
    let token = next.unwrap();
    let is_of_category = categories.iter().any(|category| *category == token.category);
    if !is_of_category {
        stream.put_back();
        return Err(ParseError::UnexpectedToken{unexpected: token, expected: categories});
    }
    Ok(token)
}

fn parse(tokens: &[Token]) -> Result<(), ParseError> {
    let mut token_stream = TokenStream::new(tokens);
    parse_program(&mut token_stream)
}

fn parse_program(stream: &mut TokenStream) -> Result<(), ParseError> {
    let first_declaration = parse_declaration(stream);
    if first_declaration.is_err() {
        return first_declaration;
    }
    while !stream.is_empty() {
        match parse_declaration(stream) 
        {
            Err(x) => return Err(x),
            _ => continue,
        }
    }
    Ok(())
}


fn parse_declaration(stream: &mut TokenStream) -> Result<(), ParseError> {
    let name_identifier = parse_token(&[Category::Identifier], stream)?;
    println!("PARSED IDENTIFIER");
    let colon = parse_token(&[Category::Colon], stream)?;
    println!("PARSED COLOn");
    let type_identifier = parse_token(&[Category::Identifier], stream)?;
    println!("PARSED IDENTIFIER");
    let equal = parse_token(&[Category::Equal], stream)?;
    println!("PARSED EQUAL");
    let expression = parse_expression(stream)?;
    println!("PARSED EXPRESSION");
    let semicolon = parse_token(&[Category::Semicolon], stream)?;
    println!("PARSED SEMICOLON");
    Ok(())
}


fn parse_expression(stream: &mut TokenStream ) -> Result<(), ParseError> {
    //parse_token(Category::Identifier, stream).map( |x| {Ok(())})?
    parse_logical(stream)
}


fn parse_logical(stream: &mut TokenStream ) -> Result<(), ParseError> {
    //parse_token(Category::Identifier, stream).map( |x| {Ok(())})?
    parse_comparison(stream)?;
    parse_logical_a(stream)
}

fn parse_logical_a(stream: &mut TokenStream ) -> Result<(), ParseError> {
    //parse_token(Category::Identifier, stream).map( |x| {Ok(())})?
    if parse_token(&[Category::Pipe], stream).is_ok(){
        parse_token(&[Category::Pipe], stream)?;
        parse_comparison(stream)?;
        parse_logical_a(stream)?;
    } else if parse_token(&[Category::Ampersand], stream).is_ok() {
        parse_token(&[Category::Ampersand], stream)?;
        parse_comparison(stream)?;
        parse_logical_a(stream)?;
    } else {
        // match empty string
    }
    Ok(())
}

fn parse_comparison(stream: &mut TokenStream ) -> Result<(), ParseError> {
    parse_arithmetical_add_sub(stream)?;
    parse_comparison_a(stream)
}

fn parse_comparison_a(stream: &mut TokenStream ) -> Result<(), ParseError> {
    if parse_token(&[Category::Less], stream).is_ok(){
        parse_comparison_a_open_angle(stream)?;
    } else if parse_token(&[Category::More], stream).is_ok() {
        parse_comparison_a_close_angle(stream)?;
    } else if parse_token(&[Category::Equal], stream).is_ok() {
        parse_token(&[Category::Equal], stream)?;
        parse_arithmetical_add_sub(stream)?;
        parse_comparison_a(stream)?;

    } else if parse_token(&[Category::Exclamation], stream).is_ok() {
        parse_token(&[Category::Equal], stream)?;
        parse_arithmetical_add_sub(stream)?;
        parse_comparison_a(stream)?;
    } else {
        // match empty string
    }
    Ok(())
}

fn parse_comparison_a_open_angle(stream: &mut TokenStream ) -> Result<(), ParseError> {
    if parse_token(&[Category::Equal], stream).is_ok() {
        // now its less_or_equal comparison, change ast accordingly
    }
    parse_arithmetical_add_sub(stream)?;
    parse_comparison_a(stream)
}

fn parse_comparison_a_close_angle(stream: &mut TokenStream ) -> Result<(), ParseError> {
    if parse_token(&[Category::Equal], stream).is_ok() {
        // now its more_or_equal comparison, change ast accordingly
    }
    parse_arithmetical_add_sub(stream)?;
    parse_comparison_a(stream)
    
}

fn parse_arithmetical_add_sub(stream: &mut TokenStream ) -> Result<(), ParseError> {
    parse_arithmetical_mul_div_mod(stream)?;
    parse_arithmetical_add_sub_a(stream)?;
    Ok(())
}

fn parse_arithmetical_add_sub_a(stream: &mut TokenStream ) -> Result<(), ParseError> {
    if parse_token(&[Category::Plus], stream).is_ok(){
        parse_arithmetical_mul_div_mod(stream)?;
        parse_arithmetical_add_sub_a(stream)?;

    } else if parse_token(&[Category::Minus], stream).is_ok() {
        parse_arithmetical_mul_div_mod(stream)?;
        parse_arithmetical_add_sub_a(stream)?;
    } else {
        // matched empty string
    }
    Ok(())
}

fn parse_arithmetical_mul_div_mod(stream: &mut TokenStream ) -> Result<(), ParseError> {
    parse_exponentiation(stream)?;
    parse_arithmetical_mul_div_mod_a(stream)?;
    Ok(())
}

fn parse_arithmetical_mul_div_mod_a(stream: &mut TokenStream ) -> Result<(), ParseError> {
    if parse_token(&[Category::Star], stream).is_ok(){
        parse_exponentiation(stream)?;
        parse_arithmetical_mul_div_mod_a(stream)?;
    } else if parse_token(&[Category::Slash], stream).is_ok() {
        parse_exponentiation(stream)?;
        parse_arithmetical_mul_div_mod_a(stream)?;
    } else if parse_token(&[Category::Percent], stream).is_ok() {
        parse_exponentiation(stream)?;
        parse_arithmetical_mul_div_mod_a(stream)?;
    } else {
        // matched empty string
    }
    Ok(())
}

fn parse_exponentiation(stream: &mut TokenStream ) -> Result<(), ParseError> {
    parse_unary(stream)?;
    parse_exponentiation_a(stream)?;
    Ok(())
}

fn parse_exponentiation_a(stream: &mut TokenStream ) -> Result<(), ParseError> {
    if parse_token(&[Category::Dash], stream).is_ok(){
        parse_unary(stream)?;
        parse_exponentiation_a(stream)?;
    } else  {
        // empty string matched
    }
    Ok(())
}

fn parse_unary(stream: &mut TokenStream ) -> Result<(), ParseError> {
    parse_postfix(stream)?;
    parse_unary_a(stream)?;
    Ok(())
}

fn parse_unary_a(stream: &mut TokenStream ) -> Result<(), ParseError> {
    if parse_token(&[Category::Minus], stream).is_ok(){
        parse_postfix(stream)?;
        parse_unary_a(stream)?;
    } else if parse_token(&[Category::Exclamation], stream).is_ok() {
        parse_postfix(stream)?;
        parse_unary_a(stream)?;
    } else {
        // matched empty string
    }
    Ok(())
}

fn parse_postfix(stream: &mut TokenStream ) -> Result<(), ParseError> {
    if parse_subscript(stream).is_ok(){
        parse_postfix_a(stream)?;
    } else if parse_function_call(stream).is_ok() {
        parse_postfix_a(stream)?;
    } else if parse_literal(stream).is_ok() {
        parse_postfix_a(stream)?;
    } else if parse_token(&[Category::Identifier], stream).is_ok() {
        parse_postfix_a(stream)?;
    }
    Ok(())
}

fn parse_postfix_a(stream: &mut TokenStream ) -> Result<(), ParseError> {
    if parse_token(&[Category::Plus], stream).is_ok() {
        parse_token(&[Category::Plus], stream)?;
    } else if parse_token(&[Category::Minus], stream).is_ok() {
        parse_token(&[Category::Minus], stream)?;
    } 
    Ok(())
}

fn parse_subscript(stream: &mut TokenStream ) -> Result<(), ParseError> {
    parse_token(&[Category::Identifier], stream)?;
    parse_token(&[Category::OpenBracket], stream)?;
    parse_expression(stream)?;
    parse_token(&[Category::CloseBracket], stream)?;
    Ok(())
}

fn parse_function_call(stream: &mut TokenStream ) -> Result<(), ParseError> {
    parse_token(&[Category::Identifier], stream)?;
    parse_token(&[Category::OpenParen], stream)?;
    parse_function_parameters(stream)?;
    parse_token(&[Category::CloseParen], stream)?;
    Ok(())
}

fn parse_function_parameters(stream: &mut TokenStream ) -> Result<(), ParseError> {
    loop {
        if parse_expression(stream).is_err() {
            break;
        } else {
            parse_token(&[Category::Comma], stream)?;
        }
    }
    Ok(())
}

fn parse_literal(stream: &mut TokenStream)-> Result<(), ParseError> {
    let token = parse_token(&[Category::Float, Category::Integer, Category::Character, Category::Text], stream)?;
    Ok(())

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_correct_program() {
        let mut declaration_tokens: Vec<Token> = Vec::new();
        declaration_tokens.push(
            Token{
                category: Category::Identifier,
                lexeme: String::from("x"),
                line: 1,
                column: 1});
        declaration_tokens.push(
            Token{
                category: Category::Colon,
                lexeme: String::from(":"),
                line: 1,
                column: 1});
        declaration_tokens.push(
            Token{
                category: Category::Identifier,
                lexeme: String::from("string"),
                line: 1,
                column: 1});
        declaration_tokens.push(
            Token{
                category: Category::Equal,
                lexeme: String::from("="),
                line: 1,
                column: 1});
        declaration_tokens.push(
            Token{
                category: Category::Identifier,
                lexeme: String::from("something"),
                line: 1,
                column: 1});
        declaration_tokens.push(
            Token{
                category: Category::Semicolon,
                lexeme: String::from(";"),
                line: 1,
                column: 1});
        assert!(parse(&declaration_tokens).is_ok());
    }
}
