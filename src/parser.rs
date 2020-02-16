use crate::ast::{
    Declaration, Expression, ExpressionKind, ExpressionValue, Statement, Type,
    TypeKind,
};
use crate::scanner::{Category, Token};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEndOfTokens {
        line: usize,
        column: usize,
    },
    UnexpectedToken {
        unexpected: Token,
        expected: Vec<Category>,
    },
    ExpectedButMissingToken {
        after: Token,
        expected: Vec<Category>,
    },
    UnknownTypeIdentifier {
        token: Token,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedEndOfTokens { line, column } => write!(
                f,
                "Unexpected end of tokens after token at line {} column {}.",
                line, column
            ),
            ParseError::UnexpectedToken {
                unexpected,
                expected,
            } => write!(
                f,
                "Unexpected token at line {} column {}. Found {}, expected {:?}",
                unexpected.line, unexpected.column, unexpected.category, expected
            ),
            ParseError::ExpectedButMissingToken { after, expected } => write!(
                f,
                "Expected one of {:?} after {} at line {} column {}",
                expected, after.category, after.line, after.column
            ),
            ParseError::UnknownTypeIdentifier { token } => write!(
                f,
                "Type identifier {} at line {} column {} is unknown",
                token.lexeme, token.line, token.column
            ),
        }
    }
}

impl Error for ParseError {}

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

    pub fn consume(&mut self, amount: usize) {
        self.current_index += amount;
    }

    pub fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.current_index)?;
        self.current_index += 1;
        Some(token.clone())
    }

    pub fn peek(&mut self, n_ahead: usize) -> Option<Token> {
        let token = self.tokens.get(self.current_index + n_ahead - 1)?;
        Some(token.clone())
    }

    pub fn peek_previous(&self) -> Option<Token> {
        let token = self.tokens.get(self.current_index - 1)?;
        Some(token.clone())
    }

    pub fn is_empty(&self) -> bool {
        self.current_index == self.tokens.len()
    }

    pub fn remain(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.tokens.len() - self.current_index
        }
    }
}

fn error_on_empty_stream(stream: &TokenStream) -> Result<(), ParseError> {
    if stream.remain() == 0 {
        let last = stream.peek_previous().unwrap();
        let (line, column) = (last.line, last.column);
        return Err(ParseError::UnexpectedEndOfTokens { line, column });
    }
    Ok(())
}

fn parse_token(
    category: Category,
    stream: &mut TokenStream,
) -> Result<(String, Category), ParseError> {
    error_on_empty_stream(stream)?;
    let token = stream.peek(1);
    let expected = [category].to_vec();
    if token.is_none() {
        let after = stream.peek_previous().unwrap();
        return Err(ParseError::ExpectedButMissingToken { after, expected });
    }
    let token = token.unwrap();
    match token.category {
        x if x == category => {
            stream.consume(1);
            return Ok((token.lexeme, token.category));
        }
        _ => Err(ParseError::UnexpectedToken {
            unexpected: token,
            expected,
        }),
    }
}

pub fn parse(tokens: &[Token]) -> Result<Declaration, ParseError> {
    if tokens.is_empty() {
        return Ok(Declaration {
                name: String::from("Empty file"),
                type_: Type{kind: TypeKind::Text, subtype: None, param_list: None},
                value: None,
                code: None,
                next: None,
        });
    }
    let mut token_stream = TokenStream::new(tokens);
    parse_program(&mut token_stream)
}

fn parse_program(stream: &mut TokenStream) -> Result<Declaration, ParseError> {
    let mut first_declaration = parse_declaration(stream)?;
    while !stream.is_empty() {
        match parse_declaration(stream) {
            Err(x) => {
                return Err(x);
            }
            Ok(declaration) => first_declaration.attach_most_next(declaration),
        }
    }
    Ok(first_declaration)
}

fn parse_declaration(stream: &mut TokenStream) -> Result<Declaration, ParseError> {
    error_on_empty_stream(stream)?;
    let name = parse_token(Category::Identifier, stream)?.0;
    parse_token(Category::Colon, stream)?;
    match stream.peek(1).unwrap().category {
        // parse variable or array declaration
        Category::Identifier => {
            let type_ = parse_full_type(stream)?;
            let value = parse_variable_assignment(stream)?;
            return Ok(Declaration::new_value(name, type_, value));
        }
        Category::ArrayKeyword => {
            let type_ = parse_full_type(stream)?;
            let value = parse_variable_assignment(stream)?;
            return Ok(Declaration::new_value(name, type_, value));
        }
        Category::FunctionKeyword => {
            let type_ = parse_full_type(stream)?;
            let code = parse_function_assignment(stream)?;
            return Ok(Declaration::new_function(name, type_, code));
        }
        _ => {
            let unexpected = stream.peek(1).unwrap();
            let expected = [
                Category::Identifier,
                Category::ArrayKeyword,
                Category::FunctionKeyword,
            ]
            .to_vec();
            return Err(ParseError::UnexpectedToken {
                unexpected,
                expected,
            });
        }
    }
}

fn parse_variable_assignment(stream: &mut TokenStream) -> Result<Option<Expression>, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        // parse variable or array declaration
        Category::Equal => {
            parse_token(Category::Equal, stream)?;
            let expression = parse_expression(stream)?;
            parse_token(Category::Semicolon, stream)?;
            return Ok(Some(expression));
        }
        Category::Semicolon => {
            parse_token(Category::Semicolon, stream)?;
            return Ok(None);
        }
        _ => {
            let unexpected = stream.peek(1).unwrap();
            let expected = [Category::Equal, Category::Semicolon].to_vec();
            return Err(ParseError::UnexpectedToken {
                unexpected,
                expected,
            });
        }
    }
}

fn parse_function_assignment(stream: &mut TokenStream) -> Result<Option<Statement>, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        // parse variable or array declaration
        Category::Equal => {
            parse_token(Category::Equal, stream)?;
            let block = parse_block_statement(stream)?;
            return Ok(Some(block));
        }
        Category::Semicolon => {
            parse_token(Category::Semicolon, stream)?;
            return Ok(None);
        }
        _ => {
            let unexpected = stream.peek(1).unwrap();
            let expected = [Category::Equal, Category::Semicolon].to_vec();
            return Err(ParseError::UnexpectedToken {
                unexpected,
                expected,
            });
        }
    }
}

fn parse_statement(stream: &mut TokenStream) -> Result<Statement, ParseError> {
    error_on_empty_stream(stream)?;
    let one_ahead = stream.peek(1).unwrap();
    match one_ahead.category {
        Category::Identifier => {
            // if starts with identifier might be declaration/expression
            let two_ahead = stream.peek(2).unwrap();
            match two_ahead.category {
                //Category::Equal => {
                //// assignment
                //parse_assignment_statement(stream)
                //}
                Category::Colon => {
                    // declaration of variable
                    Ok(Statement::new_declaration(parse_declaration(stream)?))
                }
                _ => {
                    // it is probably an expression
                    let expression = parse_expression(stream)?;
                    parse_token(Category::Semicolon, stream)?;
                    Ok(Statement::new_expression(expression))
                }
            }
        }
        Category::IfKeyword => parse_if_else_statement(stream),
        Category::ForKeyword => parse_for_statement(stream),
        Category::PrintKeyword => parse_print_statement(stream),
        Category::ReturnKeyword => parse_return_statement(stream),
        Category::OpenBrace => parse_block_statement(stream),
        _ => {
            let unexpected = stream.peek(1).unwrap();
            let expected = [Category::Identifier, Category::Semicolon].to_vec();
            Err(ParseError::UnexpectedToken {
                unexpected,
                expected,
            })
        }
    }
}

fn parse_if_else_statement(stream: &mut TokenStream) -> Result<Statement, ParseError> {
    let (condition, body) = parse_if(stream)?;
    let else_body = parse_else(stream)?;
    Ok(Statement::new_if_else(condition, body, else_body))
}

fn parse_if(stream: &mut TokenStream) -> Result<(Expression, Statement), ParseError> {
    parse_token(Category::IfKeyword, stream)?;
    parse_token(Category::OpenParen, stream)?;
    let condition = parse_expression(stream)?;
    parse_token(Category::CloseParen, stream)?;
    let body = parse_statement(stream)?;
    Ok((condition, body))
}

fn parse_else(stream: &mut TokenStream) -> Result<Option<Statement>, ParseError> {
    if stream.peek(1).unwrap().category == Category::ElseKeyword {
        parse_token(Category::ElseKeyword, stream)?;
        return Ok(Some(parse_statement(stream)?));
    }
    Ok(None)
}

fn parse_for_statement(stream: &mut TokenStream) -> Result<Statement, ParseError> {
    parse_token(Category::ForKeyword, stream)?;
    parse_token(Category::OpenParen, stream)?;
    let one_ahead = stream.peek(1).unwrap().category;
    let two_ahead = stream.peek(2).unwrap().category;
    let three_ahead = stream.peek(3).unwrap().category;
    match [one_ahead, two_ahead, three_ahead] {
        [Category::Semicolon, Category::Semicolon, Category::CloseParen] => {
            stream.consume(3);
            return Ok(Statement::new_for(None, None, None, parse_statement(stream)?));
        }
        _ => {}
    }
    let init_expr = parse_expression(stream)?;
    parse_token(Category::Semicolon, stream)?;
    let condition_expr = parse_expression(stream)?;
    parse_token(Category::Semicolon, stream)?;
    let next_expression = parse_expression(stream)?;
    parse_token(Category::CloseParen, stream)?;
    let for_statement = parse_statement(stream)?;
    Ok(Statement::new_for(Some(init_expr), Some(condition_expr), Some(next_expression), for_statement))
}

fn parse_print_statement(stream: &mut TokenStream) -> Result<Statement, ParseError> {
    parse_token(Category::PrintKeyword, stream)?;
    let mut print_statement = Statement::new_print(None);
    if stream.peek(1).unwrap().category == Category::Semicolon {
        parse_token(Category::Semicolon, stream)?;
        return Ok(print_statement)
    } else {
        print_statement.attach_most_next(Statement::new_expression(parse_expression(stream)?));
    }
    loop {
        if stream.peek(1).unwrap().category == Category::Semicolon {
            parse_token(Category::Semicolon, stream)?;
            return Ok(print_statement)
        } else {
            parse_token(Category::Comma, stream)?;
            print_statement.attach_most_next(Statement::new_expression(parse_expression(stream)?));
        }
    }
}

fn parse_return_statement(stream: &mut TokenStream) -> Result<Statement, ParseError> {
    parse_token(Category::ReturnKeyword, stream)?;
    let return_statment = Statement::new_return(parse_expression(stream)?);
    parse_token(Category::Semicolon, stream)?;
    Ok(return_statment)
}

fn parse_block_statement(stream: &mut TokenStream) -> Result<Statement, ParseError> {
    parse_token(Category::OpenBrace, stream)?;
    let mut block = Statement::new_block(None);
    loop {
        error_on_empty_stream(stream)?;
        if stream.peek(1).unwrap().category == Category::CloseBrace {
            parse_token(Category::CloseBrace, stream)?;
            return Ok(block);
        }
        block.attach_most_next(parse_statement(stream)?);
    }
}

fn parse_expression(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    parse_assignment(stream)
}

fn parse_assignment(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    let logical_expr = parse_logical(stream)?; 
    let assignment_expr = parse_assignment_a(stream)?;
    if assignment_expr.is_some() {
        let mut assignment_expr = assignment_expr.unwrap();
        assignment_expr.attach_leftmost(logical_expr);
        return Ok(assignment_expr);
    }
    Ok(logical_expr)
}

fn parse_assignment_a(stream: &mut TokenStream) -> Result<Option<Expression>, ParseError> {
    let one_ahead = stream.peek(1).unwrap().category;
    match one_ahead {
        Category::Equal => {
            stream.consume(1);
            let logical_expr = parse_logical(stream)?;
            let assignment_expr = parse_assignment_a(stream)?;
            let result = make_recurrent_binary_expression(ExpressionKind::Assignment, logical_expr, assignment_expr);
            Ok(Some(result))
        }
        _ => Ok(None)
    }
}

fn parse_logical(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    let comparison_expr = parse_comparison(stream)?;
    let logical_expr = parse_logical_a(stream)?;
    if logical_expr.is_some() {
        let mut logical_expr = logical_expr.unwrap();
        logical_expr.attach_leftmost(comparison_expr);
        return Ok(logical_expr);
    }
    Ok(comparison_expr)
}

fn parse_logical_a(stream: &mut TokenStream) -> Result<Option<Expression>, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        Category::Pipe => {
            parse_token(Category::Pipe, stream)?;
            parse_token(Category::Pipe, stream)?;
            let comparison_expr = parse_comparison(stream)?;
            let logical_expr = parse_logical_a(stream)?;
            let result =
                make_recurrent_binary_expression(ExpressionKind::Or, comparison_expr, logical_expr);
            Ok(Some(result))
        }
        Category::Ampersand => {
            parse_token(Category::Ampersand, stream)?;
            parse_token(Category::Ampersand, stream)?;
            let comparison_expr = parse_comparison(stream)?;
            let logical_expr = parse_logical_a(stream)?;
            let result =
                make_recurrent_binary_expression(ExpressionKind::Or, comparison_expr, logical_expr);
            Ok(Some(result))
        }
        _ => Ok(None),
    }
}

fn parse_comparison(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    let arithmetical_add_sub_expr = parse_arithmetical_add_sub(stream)?;
    let comparison_expr = parse_comparison_a(stream)?;
    if comparison_expr.is_some() {
        let mut comparison_expr = comparison_expr.unwrap();
        comparison_expr.attach_leftmost(arithmetical_add_sub_expr);
        return Ok(comparison_expr);
    }
    Ok(arithmetical_add_sub_expr)
}

fn parse_comparison_a(stream: &mut TokenStream) -> Result<Option<Expression>, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        Category::Less => {
            parse_token(Category::Less, stream)?;
            Ok(parse_comparison_a_open_angle(stream)?)
        }
        Category::More => {
            parse_token(Category::More, stream)?;
            Ok(parse_comparison_a_close_angle(stream)?)
        }
        Category::Equal => {
            if stream.peek(2).unwrap().category == Category::Equal {
                parse_token(Category::Equal, stream)?;
                parse_token(Category::Equal, stream)?;
                let expr = parse_arithmetical_add_sub(stream)?;
                let recurrent_expr = parse_comparison_a(stream)?;
                let result =
                    make_recurrent_binary_expression(ExpressionKind::Equal, expr, recurrent_expr);
                Ok(Some(result))
            } else {
                Ok(None)
            }
        }
        Category::Exclamation => {
            parse_token(Category::Exclamation, stream)?;
            parse_token(Category::Equal, stream)?;
            let expr = parse_arithmetical_add_sub(stream)?;
            let recurrent_expr = parse_comparison_a(stream)?;
            let result =
                make_recurrent_binary_expression(ExpressionKind::NotEqual, expr, recurrent_expr);
            Ok(Some(result))
        }
        _ => Ok(None),
    }
}

fn parse_comparison_a_open_angle(
    stream: &mut TokenStream,
) -> Result<Option<Expression>, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        Category::Equal => {
            parse_token(Category::Equal, stream)?;
            let expr = parse_arithmetical_add_sub(stream)?;
            let recurrent_expr = parse_comparison_a(stream)?;
            let result =
                make_recurrent_binary_expression(ExpressionKind::LessEqual, expr, recurrent_expr);
            Ok(Some(result))
        }
        _ => {
            let expr = parse_arithmetical_add_sub(stream)?;
            let recurrent_expr = parse_comparison_a(stream)?;
            let result =
                make_recurrent_binary_expression(ExpressionKind::Less, expr, recurrent_expr);
            Ok(Some(result))
        }
    }
}

fn parse_comparison_a_close_angle(
    stream: &mut TokenStream,
) -> Result<Option<Expression>, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        Category::Equal => {
            parse_token(Category::Equal, stream)?;
            let expr = parse_arithmetical_add_sub(stream)?;
            let recurrent_expr = parse_comparison_a(stream)?;
            let result =
                make_recurrent_binary_expression(ExpressionKind::MoreEqual, expr, recurrent_expr);
            Ok(Some(result))
        }
        _ => {
            let expr = parse_arithmetical_add_sub(stream)?;
            let recurrent_expr = parse_comparison_a(stream)?;
            let result =
                make_recurrent_binary_expression(ExpressionKind::More, expr, recurrent_expr);
            Ok(Some(result))
        }
    }
}

fn parse_arithmetical_add_sub(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    let arithmetical_mul_div_expr = parse_arithmetical_mul_div_mod(stream)?;
    let arithmetical_add_sub_expr = parse_arithmetical_add_sub_a(stream)?;
    if arithmetical_add_sub_expr.is_some() {
        let mut arithmetical_add_sub_expr = arithmetical_add_sub_expr.unwrap();
        arithmetical_add_sub_expr.attach_leftmost(arithmetical_mul_div_expr);
        return Ok(arithmetical_add_sub_expr);
    }
    Ok(arithmetical_mul_div_expr)
}

fn parse_arithmetical_add_sub_a(
    stream: &mut TokenStream,
) -> Result<Option<Expression>, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        Category::Plus => {
            parse_token(Category::Plus, stream)?;
            let expr = parse_arithmetical_mul_div_mod(stream)?;
            let recurrent_expr = parse_arithmetical_add_sub_a(stream)?;
            Ok(Some(make_recurrent_binary_expression(
                ExpressionKind::Addition,
                expr,
                recurrent_expr,
            )))
        }
        Category::Minus => {
            parse_token(Category::Minus, stream)?;
            let expr = parse_arithmetical_mul_div_mod(stream)?;
            let recurrent_expr = parse_arithmetical_add_sub_a(stream)?;
            Ok(Some(make_recurrent_binary_expression(
                ExpressionKind::Subtraction,
                expr,
                recurrent_expr,
            )))
        }
        _ => Ok(None),
    }
}

fn parse_arithmetical_mul_div_mod(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    let exponentiation_expr = parse_exponentiation(stream)?;
    let arithmetical_mul_div_expr = parse_arithmetical_mul_div_mod_a(stream)?;
    if arithmetical_mul_div_expr.is_some() {
        let mut arithmetical_mul_div_expr = arithmetical_mul_div_expr.unwrap();
        arithmetical_mul_div_expr.attach_leftmost(exponentiation_expr);
        return Ok(arithmetical_mul_div_expr);
    }
    Ok(exponentiation_expr)
}

fn parse_arithmetical_mul_div_mod_a(
    stream: &mut TokenStream,
) -> Result<Option<Expression>, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        Category::Star => {
            parse_token(Category::Star, stream)?;
            let expr = parse_exponentiation(stream)?;
            let recurrent_expr = parse_arithmetical_mul_div_mod_a(stream)?;
            Ok(Some(make_recurrent_binary_expression(
                ExpressionKind::Multiplication,
                expr,
                recurrent_expr,
            )))
        }
        Category::Slash => {
            parse_token(Category::Slash, stream)?;
            let expr = parse_exponentiation(stream)?;
            let recurrent_expr = parse_arithmetical_mul_div_mod_a(stream)?;
            Ok(Some(make_recurrent_binary_expression(
                ExpressionKind::Division,
                expr,
                recurrent_expr,
            )))
        }
        Category::Percent => {
            parse_token(Category::Percent, stream)?;
            let expr = parse_exponentiation(stream)?;
            let recurrent_expr = parse_arithmetical_mul_div_mod_a(stream)?;
            Ok(Some(make_recurrent_binary_expression(
                ExpressionKind::Modulo,
                expr,
                recurrent_expr,
            )))
        }
        _ => Ok(None),
    }
}

fn parse_exponentiation(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    let unary_expr = parse_unary(stream)?;
    let exponentiation_expr = parse_exponentiation_a(stream)?;
    if exponentiation_expr.is_some() {
        let mut exponentiation_expr = exponentiation_expr.unwrap();
        exponentiation_expr.attach_leftmost(unary_expr);
        return Ok(exponentiation_expr);
    }
    Ok(unary_expr)
}

fn parse_exponentiation_a(stream: &mut TokenStream) -> Result<Option<Expression>, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        Category::Dash => {
            parse_token(Category::Dash, stream)?;
            let expr = parse_unary(stream)?;
            let recurrent_expr = parse_exponentiation_a(stream)?;
            Ok(Some(make_recurrent_binary_expression(
                ExpressionKind::Power,
                expr,
                recurrent_expr,
            )))
        }
        _ => Ok(None),
    }
}

fn parse_unary(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    let unary_expr = parse_unary_a(stream)?;
    let postfix_expr = parse_postfix(stream)?;
    if unary_expr.is_some() {
        let mut unary_expr = unary_expr.unwrap();
        unary_expr.attach_leftmost(postfix_expr);
        return Ok(unary_expr);
    }
    Ok(postfix_expr)
}

fn parse_unary_a(stream: &mut TokenStream) -> Result<Option<Expression>, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        Category::Minus => {
            parse_token(Category::Minus, stream)?;
            let minus = Expression {
                kind: ExpressionKind::Minus,
                left: parse_unary_a(stream)?.map(|x| Box::new(x)),
                right: None,
                value: None,
            };
            Ok(Some(minus))
        }
        Category::Exclamation => {
            parse_token(Category::Exclamation, stream)?;
            let exclemation = Expression {
                kind: ExpressionKind::Negation,
                left: parse_unary_a(stream)?.map(|x| Box::new(x)),
                right: None,
                value: None,
            };
            Ok(Some(exclemation))
        }
        _ => Ok(None),
    }
}

fn parse_postfix(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    let subscript_expr = parse_subscript_call(stream)?;
    let postfix_expr = parse_postfix_a(stream)?;
    if postfix_expr.is_some() {
        let mut postfix_expr = postfix_expr.unwrap();
        postfix_expr.attach_leftmost(subscript_expr);
        return Ok(postfix_expr);
    }
    Ok(subscript_expr)
}

fn parse_postfix_a(stream: &mut TokenStream) -> Result<Option<Expression>, ParseError> {
    error_on_empty_stream(stream)?;
    if stream.remain() < 2 {
        return Ok(None);
    }
    let one_ahead = stream.peek(1).unwrap().category;
    let two_ahead = stream.peek(2).unwrap().category;
    match [one_ahead, two_ahead] {
        [Category::Plus, Category::Plus] => {
            parse_token(Category::Plus, stream)?;
            parse_token(Category::Plus, stream)?;
            Ok(Some(Expression {
                kind: ExpressionKind::Incrementation,
                left: parse_postfix_a(stream)?.map(|x| Box::new(x)),
                right: None,
                value: None,
            }))
        }
        [Category::Minus, Category::Minus] => {
            parse_token(Category::Minus, stream)?;
            parse_token(Category::Minus, stream)?;
            Ok(Some(Expression {
                kind: ExpressionKind::Decrementation,
                left: parse_postfix_a(stream)?.map(|x| Box::new(x)),
                right: None,
                value: None,
            }))
        }
        _ => Ok(None),
    }
}

fn parse_subscript_call(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    let value_expr = parse_value(stream)?;
    let mut subscripts_and_calls = parse_subscript_call_a(stream)?;
    if !subscripts_and_calls.is_empty() {
        subscripts_and_calls.reverse();
        let mut root = subscripts_and_calls.first().unwrap().clone();
        for subscript_or_call in subscripts_and_calls.into_iter().skip(1) {
            root.attach_leftmost(subscript_or_call.clone());
        }
        root.attach_leftmost(value_expr);
        return Ok(root);
    }
    Ok(value_expr)
}

fn parse_call(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    parse_token(Category::OpenParen, stream)?;
    let args_expr = parse_function_arguments(stream)?;
    parse_token(Category::CloseParen, stream)?;
    Ok(Expression {
        kind: ExpressionKind::FunctionCall,
        left: None,
        right: args_expr.map(|x| Box::new(x)),
        value: None,
    })
}

fn parse_subscript(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    parse_token(Category::OpenBracket, stream)?;
    let expr = parse_expression(stream)?;
    parse_token(Category::CloseBracket, stream)?;
    Ok(Expression {
        kind: ExpressionKind::Subscript,
        left: None,
        right: Some(Box::new(expr)),
        value: None,
    })
}

fn parse_subscript_call_a(stream: &mut TokenStream) -> Result<Vec<Expression>, ParseError> {
    error_on_empty_stream(stream)?;
    let mut subscripts_and_calls = Vec::new();
    loop {
        match stream.peek(1).unwrap().category {
            // function call
            Category::OpenParen => {
                subscripts_and_calls.push(parse_call(stream)?);
            }
            // subscript
            Category::OpenBracket => {
                subscripts_and_calls.push(parse_subscript(stream)?);
            }
            _ => break,
        }
    }
    return Ok(subscripts_and_calls);
}

fn parse_value(stream: &mut TokenStream) -> Result<Expression, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        Category::OpenParen => {
            parse_token(Category::OpenParen, stream)?;
            let expression: Expression = parse_expression(stream)?;
            parse_token(Category::CloseParen, stream)?;
            Ok(expression)
        }
        Category::Identifier
        | Category::Integer
        | Category::Float
        | Category::Character
        | Category::Text => {
            let token = stream.next().unwrap();
            let value_expression = value_to_expression_value(token.lexeme, token.category);
            Ok(Expression::new_value(value_expression))
        }
        Category::OpenBrace => {
            parse_token(Category::OpenBrace, stream)?;
            let mut array_values: Vec<Expression> = Vec::new();
            let mut array_expr = Expression {
                kind: ExpressionKind::Array,
                left: None,
                right: None,
                value: None,
            };
            if stream.peek(1).unwrap().category == Category::CloseBrace {
                parse_token(Category::CloseBrace, stream)?;
                array_expr.value = Some(ExpressionValue::Array(array_values));
                return Ok(array_expr);
            }
            array_values.push(parse_expression(stream)?);
            loop {
                if stream.peek(1).unwrap().category == Category::CloseBrace {
                    parse_token(Category::CloseBrace, stream)?;
                    array_expr.value = Some(ExpressionValue::Array(array_values));
                    return Ok(array_expr);
                }
                parse_token(Category::Comma, stream)?;
                array_values.push(parse_expression(stream)?);
            }
        }
        _ => {
            let unexpected = stream.peek(1).unwrap();
            let expected = [Category::Identifier, Category::Integer].to_vec();
            Err(ParseError::UnexpectedToken {
                unexpected,
                expected,
            })
        }
    }
}

fn parse_function_arguments(stream: &mut TokenStream) -> Result<Option<Expression>, ParseError> {
    error_on_empty_stream(stream)?;
    let mut first_arg_expr = Expression {
        kind: ExpressionKind::FunctionArg,
        left: None,
        right: None,
        value: None,
    };
    match stream.peek(1).unwrap().category {
        Category::CloseParen => return Ok(None),
        Category::Identifier
        | Category::Integer
        | Category::Float
        | Category::Character
        | Category::Text => {
            first_arg_expr.attach_rightmost(parse_expression(stream)?);
        }
        _ => {
            let unexpected = stream.peek(1).unwrap();
            let expected = [Category::Identifier].to_vec();
            return Err(ParseError::UnexpectedToken {
                unexpected,
                expected,
            });
        }
    }
    loop {
        match stream.peek(1).unwrap().category {
            Category::Comma => {
                parse_token(Category::Comma, stream)?;
                first_arg_expr.attach_rightmost(parse_expression(stream)?);
            }
            Category::CloseParen => return Ok(Some(first_arg_expr)),
            _ => {
                let unexpected = stream.peek(1).unwrap();
                let expected = [Category::Identifier].to_vec();
                return Err(ParseError::UnexpectedToken {
                    unexpected,
                    expected,
                });
            }
        }
    }
}

fn parse_id_parameter(stream: &mut TokenStream) -> Result<(), ParseError> {
    error_on_empty_stream(stream)?;
    parse_token(Category::Identifier, stream)?;
    parse_token(Category::Colon, stream)?;
    parse_empty_type(stream)?;
    Ok(())
}

fn parse_non_id_parameter(stream: &mut TokenStream) -> Result<(), ParseError> {
    error_on_empty_stream(stream)?;
    parse_empty_type(stream)?;
    Ok(())
}

fn parse_id_parameters(stream: &mut TokenStream) -> Result<(), ParseError> {
    error_on_empty_stream(stream)?;
    parse_token(Category::OpenParen, stream)?;
    match stream.peek(1).unwrap().category {
        Category::CloseParen => {
            parse_token(Category::CloseParen, stream)?;
            return Ok(());
        }
        Category::Identifier => {
            parse_id_parameter(stream)?;
        }
        _ => {
            let unexpected = stream.peek(1).unwrap();
            let expected = [Category::Identifier].to_vec();
            return Err(ParseError::UnexpectedToken {
                unexpected,
                expected,
            });
        }
    }
    loop {
        match stream.peek(1).unwrap().category {
            Category::CloseParen => {
                parse_token(Category::CloseParen, stream)?;
                return Ok(());
            }
            Category::Comma => {
                parse_token(Category::Comma, stream)?;
                parse_id_parameter(stream)?;
            }
            _ => {
                let unexpected = stream.peek(1).unwrap();
                let expected = [Category::Identifier].to_vec();
                return Err(ParseError::UnexpectedToken {
                    unexpected,
                    expected,
                });
            }
        }
    }
}

fn parse_non_id_parameters(stream: &mut TokenStream) -> Result<(), ParseError> {
    error_on_empty_stream(stream)?;
    parse_token(Category::OpenParen, stream)?;
    match stream.peek(1).unwrap().category {
        Category::CloseParen => {
            parse_token(Category::CloseParen, stream)?;
            return Ok(());
        }
        Category::Identifier | Category::ArrayKeyword | Category::FunctionKeyword => {
            parse_empty_type(stream)?;
        }
        _ => {
            let unexpected = stream.peek(1).unwrap();
            let expected = [Category::Identifier].to_vec();
            return Err(ParseError::UnexpectedToken {
                unexpected,
                expected,
            });
        }
    }
    loop {
        match stream.peek(1).unwrap().category {
            Category::CloseParen => {
                parse_token(Category::CloseParen, stream)?;
                return Ok(());
            }
            Category::Comma => {
                parse_token(Category::Comma, stream)?;
                parse_empty_type(stream)?;
            }
            _ => {
                let unexpected = stream.peek(1).unwrap();
                let expected = [Category::Identifier].to_vec();
                return Err(ParseError::UnexpectedToken {
                    unexpected,
                    expected,
                });
            }
        }
    }
}

fn parse_full_type(stream: &mut TokenStream) -> Result<Type, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        Category::Identifier => {
            let type_name = parse_token(Category::Identifier, stream)?.0;
            let type_option = Type::from_name(type_name);
            if type_option.is_none() {
                return Err(ParseError::UnknownTypeIdentifier{token: stream.peek_previous().unwrap()});
            }
            return Ok(type_option.unwrap());
        }
        Category::ArrayKeyword => {
            let mut type_ = Type {kind: TypeKind::Array,
                              subtype: None,
                              param_list: None,
                            };
            parse_token(Category::ArrayKeyword, stream)?;
            parse_token(Category::OpenBracket, stream)?;
            parse_token(Category::Integer, stream)?;
            parse_token(Category::CloseBracket, stream)?;
            type_.subtype = Some(Box::new(parse_full_type(stream)?));
            return Ok(type_);
        }
        Category::FunctionKeyword => {
            let mut type_ = Type {kind: TypeKind::Function,
                                  subtype: None,
                                  param_list: None,
                                 };
            parse_token(Category::FunctionKeyword, stream)?;
            type_.subtype = Some(Box::new(parse_empty_type(stream)?));
            parse_id_parameters(stream)?;
            return Ok(type_);
        }
        _ => {
            let unexpected = stream.peek(1).unwrap();
            let expected = [
                Category::Identifier,
                Category::ArrayKeyword,
                Category::FunctionKeyword,
            ]
            .to_vec();
            return Err(ParseError::UnexpectedToken {
                unexpected,
                expected,
            });
        }
    }
}

fn parse_empty_type(stream: &mut TokenStream) -> Result<Type, ParseError> {
    error_on_empty_stream(stream)?;
    match stream.peek(1).unwrap().category {
        Category::Identifier => {
            let type_name = parse_token(Category::Identifier, stream)?.0;
            let type_ = Type::from_name(type_name).unwrap();
            return Ok(type_);
        }
        Category::ArrayKeyword => {
            let mut type_ = Type {kind: TypeKind::Array,
                                  subtype: None,
                                  param_list: None,
                                 };
            parse_token(Category::ArrayKeyword, stream)?;
            parse_token(Category::OpenBracket, stream)?;
            parse_token(Category::CloseBracket, stream)?;
            type_.subtype = Some(Box::new(parse_empty_type(stream)?));
            return Ok(type_);
        }
        Category::FunctionKeyword => {
            let mut type_ = Type {kind: TypeKind::Function,
                                  subtype: None,
                                  param_list: None,
                                 };
            parse_token(Category::FunctionKeyword, stream)?;
            type_.subtype = Some(Box::new(parse_empty_type(stream)?));
            parse_non_id_parameters(stream)?;
            return Ok(type_);
        }
        _ => {
            let unexpected = stream.peek(1).unwrap();
            let expected = [
                Category::Identifier,
                Category::ArrayKeyword,
                Category::FunctionKeyword,
            ]
            .to_vec();
            return Err(ParseError::UnexpectedToken {
                unexpected,
                expected,
            });
        }
    }
}

fn make_recurrent_binary_expression(
    kind: ExpressionKind,
    expr: Expression,
    recurrent_expr: Option<Expression>,
) -> Expression {
    let normal_expr = Expression {
        kind,
        left: None,
        right: Some(Box::new(expr)),
        value: None,
    };
    if recurrent_expr.is_some() {
        let mut recurrent_expr = recurrent_expr.unwrap();
        recurrent_expr.left = Some(Box::new(normal_expr));
        return recurrent_expr;
    }
    return normal_expr;
}

fn value_to_expression_value(lexeme: String, category: Category) -> ExpressionValue {
    match category {
        Category::Identifier => ExpressionValue::Name(lexeme),
        Category::Integer => ExpressionValue::Integer(lexeme.as_str().parse::<usize>().unwrap()),
        Category::Float => ExpressionValue::Float(lexeme.as_str().parse::<f64>().unwrap()),
        Category::Character => ExpressionValue::Character(lexeme.chars().next().unwrap()),
        Category::Text => ExpressionValue::Text(lexeme),
        _ => panic!("Read fake literal."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_correct_program() {
        let mut declaration_tokens: Vec<Token> = Vec::new();
        declaration_tokens.push(Token {
            category: Category::Identifier,
            lexeme: String::from("x"),
            line: 1,
            column: 1,
        });
        declaration_tokens.push(Token {
            category: Category::Colon,
            lexeme: String::from(":"),
            line: 1,
            column: 1,
        });
        declaration_tokens.push(Token {
            category: Category::Identifier,
            lexeme: String::from("string"),
            line: 1,
            column: 1,
        });
        declaration_tokens.push(Token {
            category: Category::Equal,
            lexeme: String::from("="),
            line: 1,
            column: 1,
        });
        declaration_tokens.push(Token {
            category: Category::Identifier,
            lexeme: String::from("something"),
            line: 1,
            column: 1,
        });
        declaration_tokens.push(Token {
            category: Category::Semicolon,
            lexeme: String::from(";"),
            line: 1,
            column: 1,
        });
        assert!(parse(&declaration_tokens).is_ok());
    }
}
