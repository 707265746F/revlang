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
    LParen,
    RParen,
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