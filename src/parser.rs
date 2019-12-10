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
        //println!("{:?}", self);
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

pub fn parse(tokens: &[Token]) -> Result<(), ParseError> {
    if tokens.is_empty() {
        return Ok(());
    }
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
    println!("PARSE_DECLARATION");
    parse_declaration_beginning(stream)?;
    parse_declaration_rest(stream)?;
    Ok(())
}

fn parse_declaration_beginning(stream: &mut TokenStream) -> Result<(), ParseError> {
    let name_identifier = parse_token(&[Category::Identifier], stream)?;
    let colon = parse_token(&[Category::Colon], stream)?;
    Ok(())
}

fn parse_declaration_rest(stream: &mut TokenStream) -> Result<(), ParseError> {
    if parse_token(&[Category::Identifier], stream).is_ok() {
        // decleration of an variable
        if parse_token(&[Category::Semicolon], stream).is_ok() {
            println!("PARSING IDENTFIER SEMICOLON");
        } else {
            println!("PARSING IDENTFIER TOKEN START");
            parse_token(&[Category::Equal], stream)?;
            parse_expression(stream)?;
            parse_token(&[Category::Semicolon], stream)?;
            println!("PARSING IDENTFIER TOKEN END");
        }
    } else if parse_token(&[Category::ArrayKeyword], stream).is_ok() {
        println!("PARSING ARRAY");
        // decleration of an array
        parse_token(&[Category::Identifier], stream)?;
        parse_token(&[Category::Identifier], stream)?;
        parse_token(&[Category::Identifier], stream)?;
    } else if parse_token(&[Category::FunctionKeyword], stream).is_ok() {
        // declaration of a function
        println!("PARSING FUNCTION");
        parse_token(&[Category::Identifier], stream)?;
        parse_token(&[Category::OpenParen], stream)?;
        parse_function_parameters(stream)?;
        parse_token(&[Category::CloseParen], stream)?;
        parse_token(&[Category::Equal], stream)?;
        parse_block_statement(stream)?;
    } else {
        let expected = &[Category::Identifier, Category::ArrayKeyword, Category::FunctionKeyword, Category::Semicolon];
        if stream.how_many_are_remaining() == 0 { 
            stream.put_back();
            let after = stream.take().unwrap();
            return Err(ParseError::ExpectedButMissingToken{after, expected});
        }
        let unexpected = stream.take().unwrap();
        return Err(ParseError::UnexpectedToken{expected, unexpected});
    }
    Ok(())
}


fn parse_statement(stream: &mut TokenStream ) -> Result<(), ParseError> {
    if parse_assignment_statement(stream).is_ok() {
        println!("s_assignmnent");
    } else if parse_declaration(stream).is_ok() {
        println!("s_declaration");
        let i = stream.take().unwrap().line;
        stream.put_back();
        println!("{}", i);
    } else if parse_if_else_statement(stream).is_ok() {
        println!("s_if_else");
    } else if parse_for_statement(stream).is_ok(){
        println!("s_for");
    } else if parse_return_statement(stream).is_ok(){
        println!("s_return");
    } else if parse_print_statement(stream).is_ok(){
        println!("s_print");
    } else if parse_expression(stream).is_ok() {
        println!("s_epression");
    } else {
        let expected = &[Category::IfKeyword, Category::ForKeyword, Category::Identifier, Category::PrintKeyword, Category::ReturnKeyword];
        let unexpected = stream.take().unwrap();
        return Err(ParseError::UnexpectedToken{expected, unexpected});
    }
    Ok(())
}

fn parse_assignment_statement(stream: &mut TokenStream ) -> Result<(), ParseError> {
        parse_token(&[Category::Identifier], stream)?;
        parse_token(&[Category::Equal], stream)?;
        parse_expression(stream)?;
        parse_token(&[Category::Semicolon], stream)?;
        Ok(())
}

fn parse_if_else_statement(stream: &mut TokenStream ) -> Result<(), ParseError> {
        parse_token(&[Category::IfKeyword], stream)?;
        parse_token(&[Category::OpenParen], stream)?;
        parse_expression(stream)?;
        parse_token(&[Category::CloseParen], stream)?;
        parse_block_statement(stream)?;
        parse_token(&[Category::ElseKeyword], stream)?;
        parse_block_statement(stream)?;
        Ok(())
}

fn parse_for_statement(stream: &mut TokenStream ) -> Result<(), ParseError> {
        parse_token(&[Category::ForKeyword], stream)?;
        parse_token(&[Category::OpenParen], stream)?;
        parse_expression(stream)?;
        parse_token(&[Category::Semicolon], stream)?;
        parse_expression(stream)?;
        parse_token(&[Category::Semicolon], stream)?;
        parse_expression(stream)?;
        parse_token(&[Category::CloseParen], stream)?;
        parse_block_statement(stream)?;
        Ok(())
}

fn parse_return_statement(stream: &mut TokenStream ) -> Result<(), ParseError> {
        parse_token(&[Category::ReturnKeyword], stream)?;
        parse_expression(stream)?;
        Ok(())
}

fn parse_print_statement(stream: &mut TokenStream ) -> Result<(), ParseError> {
    parse_token(&[Category::PrintKeyword], stream)?;
    parse_expression(stream)?;
    Ok(())
}

fn parse_block_statement(stream: &mut TokenStream ) -> Result<(), ParseError> {
    parse_token(&[Category::OpenBrace], stream)?;
    loop {
        if parse_token(&[Category::CloseBrace], stream).is_ok() {
            break;
        }
        let x = parse_statement(stream)?;
        let mut s = String::new();
        let stdi = std::io::stdin();
        stdi.read_line(&mut s);
    }
    Ok(())
}


fn parse_expression(stream: &mut TokenStream ) -> Result<(), ParseError> {
    //parse_token(Category::Identifier, stream).map( |x| {Ok(())})?
    parse_logical(stream)
}


fn parse_logical(stream: &mut TokenStream ) -> Result<(), ParseError> {
    parse_comparison(stream)?;
    parse_logical_a(stream)
}

fn parse_logical_a(stream: &mut TokenStream ) -> Result<(), ParseError> {
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
    if parse_token(&[Category::Identifier], stream).is_ok(){
        if parse_subscript(stream).is_ok() {
        } else if parse_function_call(stream).is_ok() {
        } else {
        }
        parse_postfix_a(stream)?;
    } else if parse_literal(stream).is_ok() {
        parse_postfix_a(stream)?;
    } else {
        let expected = &[Category::Identifier];
        if stream.how_many_are_remaining() == 0 {
            stream.put_back();
        }
        let unexpected = stream.take().unwrap();
        return Err(ParseError::UnexpectedToken{expected, unexpected});
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
    parse_token(&[Category::OpenBracket], stream)?;
    parse_expression(stream)?;
    parse_token(&[Category::CloseBracket], stream)?;
    Ok(())
}

fn parse_function_call(stream: &mut TokenStream ) -> Result<(), ParseError> {
    parse_token(&[Category::OpenParen], stream)?;
    parse_function_parameters(stream)?;
    parse_token(&[Category::CloseParen], stream)?;
    Ok(())
}

fn parse_function_parameters(stream: &mut TokenStream ) -> Result<(), ParseError> {
    loop {
        if stream.take().unwrap().category == Category::CloseParen {
            stream.put_back();
            break;
        } else {
            stream.put_back();
        }
        parse_token(&[Category::Identifier], stream)?;
        parse_token(&[Category::Colon], stream)?;
        parse_token(&[Category::Identifier], stream)?;
        if parse_token(&[Category::Comma], stream).is_ok() {
        } else {
            break;
        }
    }
    Ok(())
}

fn parse_literal(stream: &mut TokenStream)-> Result<(), ParseError> {
    parse_token(&[Category::Float, Category::Integer, Category::Character, Category::Text], stream)?;
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
