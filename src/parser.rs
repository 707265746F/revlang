use crate::lexer::Token;

#[derive(Debug)]
pub struct VarDecl {
    pub name:  String,
    pub ty:    String,
    pub value: u64,
}

pub fn parse_var_decl(tokens: &[Token]) -> Result<VarDecl, String> {
    let mut pos = 0;

    // expect 'let'
    match tokens.get(pos) {
        Some(Token::Let) => pos += 1,
        _ => return Err("Expected 'let'".to_string()),
    }

    // expect an identifier — the variable name
    let name = match tokens.get(pos) {
        Some(Token::Ident(n)) => { pos += 1; n.clone() }
        _ => return Err("Expected variable name".to_string()),
    };

    // expect ':'
    match tokens.get(pos) {
        Some(Token::Colon) => pos += 1,
        _ => return Err("Expected ':'".to_string()),
    }

    // expect a type
    let ty = match tokens.get(pos) {
        Some(Token::U8)  => { pos += 1; "u8".to_string()  }
        Some(Token::U16) => { pos += 1; "u16".to_string() }
        Some(Token::U32) => { pos += 1; "u32".to_string() }
        Some(Token::U64) => { pos += 1; "u64".to_string() }
        _ => return Err("Expected a type (u8, u16, u32, u64)".to_string()),
    };

    // expect '='
    match tokens.get(pos) {
        Some(Token::Equals) => pos += 1,
        _ => return Err("Expected '='".to_string()),
    }

    // expect a value — integer or hex literal
    let value = match tokens.get(pos) {
        Some(Token::IntLit(n)) => *n,
        Some(Token::HexLit(n)) => *n,
        _ => return Err("Expected a number value".to_string()),
    };

    Ok(VarDecl { name, ty, value })
}