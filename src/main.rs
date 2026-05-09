mod lexer;
mod parser;

fn main() {
    let inputs = vec![
        "let base: u64 = 0x7FFF0000",
        "let health: u32 = 100",
        "let broken: u32",          // missing = and value
    ];

    for input in inputs {
        println!("Input: \"{}\"", input);
        match lexer::tokenize(input) {
            Ok(tokens) => {
                match parser::parse_var_decl(&tokens) {
                    Ok(decl) => println!("Parsed: {:?}", decl),
                    Err(e)   => println!("Parse error: {}", e),
                }
            }
            Err(e) => println!("Lex error: {}", e),
        }
        println!();
    }
}