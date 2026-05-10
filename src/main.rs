mod lexer;
mod parser;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: revlang <file.rev>");
        return;
    }

    let input_path = &args[1];

    let source = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Error reading file '{}': {}", input_path, e);
            return;
        }
    };

    println!("RevLang compiling: {}", input_path);

    let tokens = match lexer::tokenize(&source) {
        Ok(t)  => t,
        Err(e) => { println!("Lex error: {}", e); return; }
    };

    let decl = match parser::parse_struct(&tokens) {
        Ok(d)  => d,
        Err(e) => { println!("Parse error: {}", e); return; }
    };

    println!("Struct: {}", decl.name);
    for field in &decl.fields {
        println!("  {} : {} @ 0x{:X}", field.name, field.ty, field.offset);
    }
}