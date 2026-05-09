#[derive(Debug, PartialEq)]
pub enum Token {
    // literals
    IntLit(u64),
    HexLit(u64),

    // operators
    Plus,
    Minus,
    Star,
    Slash,
    Ampersand,
    Pipe,
    Caret,

    // punctuation
    LParen,   // (
    RParen,   // )
    Colon,    // :
    Equals,   // =
    LBrace,  // {
    RBrace,  // }
    At,      // @
    Comma,   // ,

    // keywords
    Let,
    Struct,
    Fn,

    // types
    U8,
    U16,
    U32,
    U64,
    Bool,
    Str,

    // identifier — a name invented by the developer
    Ident(String),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            // skip spaces
            ' ' | '\t' | '\n' => { chars.next(); }

            // operators
            '+' => { tokens.push(Token::Plus);      chars.next(); }
            '-' => { tokens.push(Token::Minus);     chars.next(); }
            '*' => { tokens.push(Token::Star);      chars.next(); }
            '/' => { tokens.push(Token::Slash);     chars.next(); }
            '&' => { tokens.push(Token::Ampersand); chars.next(); }
            '|' => { tokens.push(Token::Pipe);      chars.next(); }
            '^' => { tokens.push(Token::Caret);     chars.next(); }
            '(' => { tokens.push(Token::LParen);    chars.next(); }
            ')' => { tokens.push(Token::RParen);    chars.next(); }
            ':' => { tokens.push(Token::Colon);     chars.next(); }
            '=' => { tokens.push(Token::Equals);    chars.next(); }
            '{' => { tokens.push(Token::LBrace);    chars.next(); }
            '}' => { tokens.push(Token::RBrace);    chars.next(); }
            '@' => { tokens.push(Token::At);        chars.next(); }
            ',' => { tokens.push(Token::Comma);     chars.next(); }

            // keywords, types, and identifiers
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut word = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        word.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let token = match word.as_str() {
                    // keywords
                    "let"    => Token::Let,
                    "struct" => Token::Struct,
                    "fn"     => Token::Fn,
                    // types
                    "u8"     => Token::U8,
                    "u16"    => Token::U16,
                    "u32"    => Token::U32,
                    "u64"    => Token::U64,
                    "bool"   => Token::Bool,
                    "str"    => Token::Str,
                    // anything else is an identifier
                    _        => Token::Ident(word),
                };
                tokens.push(token);
            }

            // hex literals: 0xFF, 0xDEAD
            '0' => {
                chars.next();
                if chars.peek() == Some(&'x') || chars.peek() == Some(&'X') {
                    chars.next();
                    let mut hex = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_hexdigit() { hex.push(c); chars.next(); }
                        else { break; }
                    }
                    let value = u64::from_str_radix(&hex, 16).unwrap();
                    tokens.push(Token::HexLit(value));
                } else {
                    let mut num = String::from("0");
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() { num.push(c); chars.next(); }
                        else { break; }
                    }
                    tokens.push(Token::IntLit(num.parse().unwrap()));
                }
            }

            // decimal integers: 42, 100
            '1'..='9' => {
                let mut num = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() { num.push(c); chars.next(); }
                    else { break; }
                }
                tokens.push(Token::IntLit(num.parse().unwrap()));
            }

            // unknown character — error!
            _ => {
                return Err(format!("Unknown character '{}' in RevLang source", ch));
            }
        }
    }

    Ok(tokens)
}