use crate::lexer::Token;

#[derive(Debug)]
pub struct VarDecl {
    pub name:  String,
    pub ty:    String,
    pub value: u64,
}

#[derive(Debug)]
pub struct Field {
    pub name:   String,
    pub ty:     String,
    pub offset: u64,
}

#[derive(Debug)]
pub struct StructDecl {
    pub name:   String,
    pub fields: Vec<Field>,
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

pub fn parse_struct(tokens: &[Token]) -> Result<StructDecl, String> {
    let mut pos = 0;

    // expect 'struct'
    match tokens.get(pos) {
        Some(Token::Struct) => pos += 1,
        _ => return Err("Expected 'struct'".to_string()),
    }

    // expect the struct name — e.g. 'Player'
    let name = match tokens.get(pos) {
        Some(Token::Ident(n)) => { pos += 1; n.clone() }
        _ => return Err("Expected struct name".to_string()),
    };

    // expect '{'
    match tokens.get(pos) {
        Some(Token::LBrace) => pos += 1,
        _ => return Err("Expected '{'".to_string()),
    }

    // parse fields one by one until we find '}'
    let mut fields = Vec::new();

    while let Some(token) = tokens.get(pos) {
        // if we find '}' the struct is done
        if *token == Token::RBrace {
            pos += 1;
            break;
        }

        // expect field name — e.g. 'health'
        let field_name = match tokens.get(pos) {
            Some(Token::Ident(n)) => { pos += 1; n.clone() }
            _ => return Err("Expected field name".to_string()),
        };

        // expect ':'
        match tokens.get(pos) {
            Some(Token::Colon) => pos += 1,
            _ => return Err("Expected ':'".to_string()),
        }

        // expect a type
        let field_ty = match tokens.get(pos) {
            Some(Token::U8)  => { pos += 1; "u8".to_string()  }
            Some(Token::U16) => { pos += 1; "u16".to_string() }
            Some(Token::U32) => { pos += 1; "u32".to_string() }
            Some(Token::U64) => { pos += 1; "u64".to_string() }
            Some(Token::Str) => { pos += 1; "str".to_string() }
            _ => return Err("Expected a type".to_string()),
        };

        // expect '@'
        match tokens.get(pos) {
            Some(Token::At) => pos += 1,
            _ => return Err("Expected '@'".to_string()),
        }

        // expect the offset — e.g. 0x1A4
        let offset = match tokens.get(pos) {
            Some(Token::HexLit(n)) => { pos += 1; *n }
            Some(Token::IntLit(n)) => { pos += 1; *n }
            _ => return Err("Expected offset value".to_string()),
        };

        // the comma after each field is optional — skip it if present
        if let Some(Token::Comma) = tokens.get(pos) {
            pos += 1;
        }

        fields.push(Field { name: field_name, ty: field_ty, offset });
    }

    Ok(StructDecl { name, fields })
}