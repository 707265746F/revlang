mod lexer;
mod parser;

fn main() {
    let input = "struct Player { health: u32 @ 0x1A4, mana: u32 @ 0x1A8, name: str @ 0x1B0 }";

    println!("Input:\n  {}\n", input);
    match lexer::tokenize(input) {
        Ok(tokens) => {
            match parser::parse_struct(&tokens) {
                Ok(s) => {
                    println!("Struct: {}", s.name);
                    for field in s.fields {
                        println!(
                            "  {} : {} @ 0x{:X}",
                            field.name, field.ty, field.offset
                        );
                    }
                }
                Err(e) => println!("Parse error: {}", e),
            }
        }
        Err(e) => println!("Lex error: {}", e),
    }
}