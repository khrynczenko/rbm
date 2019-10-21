use regex::Regex;

#[derive(Debug)]
pub enum ScanError {
    CannotScanToken { line: usize, column: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub lexeme: String,
    pub category: Category,
    pub line: usize,
    pub column: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Category {
    Identifier,
    Float,
    Integer,
    Character,
    Text,
    Equal,
    Plus,
    Minus,
    Slash,
    Star,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Less,
    More,
    Ampersand,
    Pipe,
    Percent,
    Semicolon,
}

struct CharacterStream {
    current_line: usize,
    current_column: usize,
    stream: String,
    current_index: usize,
}

impl CharacterStream {
    pub fn new(stream: &str) -> Self {
        CharacterStream {
            current_line: 1,
            current_column: 1,
            stream: String::from(stream),
            current_index: 0,
        }
    }
    pub fn consume(&mut self, amount: usize) {
        let mut current_line = self.current_line;
        let mut current_column = self.current_column;
        for c in self.get_remaining()[..amount].chars() {
            match c {
                '\n' => {
                    current_line += 1;
                    current_column = 1;
                }
                _ => {
                    current_column += 1;
                }
            }
        }
        self.current_line = current_line;
        self.current_column = current_column;
        self.current_index += amount;
    }
    pub fn get_remaining<'a>(&'a self) -> &'a str {
        &self.stream.as_str()[self.current_index..]
    }
}

pub fn tokenize(stream: &str) -> Result<Vec<Token>, ScanError> {
    let mut stream = CharacterStream::new(stream);
    let mut tokens = Vec::new();
    let token_scanners: [fn(&mut CharacterStream) -> Option<Token>; 20] = [
        try_identifier,
        try_float,
        try_integer,
        try_character,
        try_text,
        try_equal,
        try_plus,
        try_minus,
        try_slash,
        try_star,
        try_open_paren,
        try_close_paren,
        try_open_bracket,
        try_close_bracket,
        try_less,
        try_more,
        try_ampersand,
        try_pipe,
        try_percent,
        try_semicolon,
    ];
    loop {
        if stream.get_remaining().starts_with(' ') || stream.get_remaining().starts_with('\n') {
            stream.consume(1);
            continue;
        }
        let mut scanned = false;
        for scanner in &token_scanners {
            match scanner(&mut stream) {
                Some(token) => {
                    tokens.push(token);
                    scanned = true;
                    break;
                }
                None => (),
            }
        }
        if scanned {
            continue;
        };
        if !stream.get_remaining().is_empty() {
            let line = stream.current_line;
            let column = stream.current_column;
            return Err(ScanError::CannotScanToken { line, column });
        }
        break;
    }
    Ok(tokens)
}

fn make_token_scanner(
    pattern: &'static str,
    resulting_category: Category,
) -> impl Fn(&mut CharacterStream) -> Option<Token> {
    move |stream: &mut CharacterStream| {
        let re = Regex::new(pattern).unwrap();
        let remaining = stream.get_remaining();
        let re_match = re.find(remaining)?;
        let lexeme = String::from(re_match.as_str());
        let line = stream.current_line;
        let column = stream.current_column;
        let category = resulting_category;
        stream.consume(lexeme.len());
        Some(Token {
            lexeme,
            line,
            column,
            category,
        })
    }
}

fn try_identifier(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^[a-zA-Z_][a-zA-Z0-9_]*", Category::Identifier);
    scan(stream)
}

fn try_float(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^[0-9]+\.[0-9]*", Category::Float);
    scan(stream)
}

fn try_integer(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^[0-9]+", Category::Integer);
    scan(stream)
}

fn try_character(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r#"^'.'"#, Category::Character);
    scan(stream)
}

fn try_text(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r#"^".*""#, Category::Text);
    scan(stream)
}

fn try_equal(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^=", Category::Equal);
    scan(stream)
}

fn try_plus(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^\+", Category::Plus);
    scan(stream)
}

fn try_minus(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^\-", Category::Minus);
    scan(stream)
}

fn try_slash(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^/", Category::Slash);
    scan(stream)
}

fn try_star(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^\*", Category::Star);
    scan(stream)
}

fn try_open_paren(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^\(", Category::OpenParen);
    scan(stream)
}

fn try_close_paren(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^\)", Category::CloseParen);
    scan(stream)
}

fn try_open_bracket(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^\[", Category::OpenBracket);
    scan(stream)
}

fn try_close_bracket(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^\]", Category::CloseBracket);
    scan(stream)
}

fn try_less(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^<", Category::Less);
    scan(stream)
}

fn try_more(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^>", Category::More);
    scan(stream)
}

fn try_ampersand(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^&", Category::Ampersand);
    scan(stream)
}

fn try_pipe(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^\|", Category::Pipe);
    scan(stream)
}

fn try_percent(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^%", Category::Percent);
    scan(stream)
}

fn try_semicolon(stream: &mut CharacterStream) -> Option<Token> {
    let scan = make_token_scanner(r"^;", Category::Semicolon);
    scan(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_all_possible_tokens() {
        let stream = r#"_identifier1 1.25 19 'c' "string" = + - / * ( ) [ ] < > & | % ;"#;
        let identifier = Token {
            lexeme: String::from("_identifier1"),
            category: Category::Identifier,
            line: 1,
            column: 1,
        };
        let float = Token {
            lexeme: String::from("1.25"),
            category: Category::Float,
            line: 1,
            column: 14,
        };
        let integer = Token {
            lexeme: String::from("19"),
            category: Category::Integer,
            line: 1,
            column: 19,
        };
        let character = Token {
            lexeme: String::from("\'c\'"),
            category: Category::Character,
            line: 1,
            column: 22,
        };
        let string = Token {
            lexeme: String::from("\"string\""),
            category: Category::Text,
            line: 1,
            column: 26,
        };
        let equal = Token {
            lexeme: String::from("="),
            category: Category::Equal,
            line: 1,
            column: 35,
        };
        let plus = Token {
            lexeme: String::from("+"),
            category: Category::Plus,
            line: 1,
            column: 37,
        };

        let minus = Token {
            lexeme: String::from("-"),
            category: Category::Minus,
            line: 1,
            column: 39,
        };

        let slash = Token {
            lexeme: String::from("/"),
            category: Category::Slash,
            line: 1,
            column: 41,
        };

        let star = Token {
            lexeme: String::from("*"),
            category: Category::Star,
            line: 1,
            column: 43,
        };

        let open_paren = Token {
            lexeme: String::from("("),
            category: Category::OpenParen,
            line: 1,
            column: 45,
        };

        let close_paren = Token {
            lexeme: String::from(")"),
            category: Category::CloseParen,
            line: 1,
            column: 47,
        };

        let open_bracket = Token {
            lexeme: String::from("["),
            category: Category::OpenBracket,
            line: 1,
            column: 49,
        };

        let close_bracket = Token {
            lexeme: String::from("]"),
            category: Category::CloseBracket,
            line: 1,
            column: 51,
        };

        let less = Token {
            lexeme: String::from("<"),
            category: Category::Less,
            line: 1,
            column: 53,
        };

        let more = Token {
            lexeme: String::from(">"),
            category: Category::More,
            line: 1,
            column: 55,
        };

        let ampersand = Token {
            lexeme: String::from("&"),
            category: Category::Ampersand,
            line: 1,
            column: 57,
        };

        let pipe = Token {
            lexeme: String::from("|"),
            category: Category::Pipe,
            line: 1,
            column: 59,
        };

        let percent = Token {
            lexeme: String::from("%"),
            category: Category::Percent,
            line: 1,
            column: 61,
        };

        let semicolon = Token {
            lexeme: String::from(";"),
            category: Category::Semicolon,
            line: 1,
            column: 63,
        };

        let tokens = tokenize(stream).unwrap();
        assert_eq!(tokens[0], identifier);
        assert_eq!(tokens[1], float);
        assert_eq!(tokens[2], integer);
        assert_eq!(tokens[3], character);
        assert_eq!(tokens[4], string);
        assert_eq!(tokens[5], equal);
        assert_eq!(tokens[6], plus);
        assert_eq!(tokens[7], minus);
        assert_eq!(tokens[8], slash);
        assert_eq!(tokens[9], star);
        assert_eq!(tokens[10], open_paren);
        assert_eq!(tokens[11], close_paren);
        assert_eq!(tokens[12], open_bracket);
        assert_eq!(tokens[13], close_bracket);
        assert_eq!(tokens[14], less);
        assert_eq!(tokens[15], more);
        assert_eq!(tokens[16], ampersand);
        assert_eq!(tokens[17], pipe);
        assert_eq!(tokens[18], percent);
        assert_eq!(tokens[19], semicolon);
    }
}
