mod lexer;
mod parser;
mod memory;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: revlang <file.rev> <pid>");
        return;
    }

    let input_path = &args[1];
    let pid: u32 = match args[2].parse() {
        Ok(p)  => p,
        Err(_) => { println!("Error: PID must be a number"); return; }
    };

    // read the .rev file
    let source = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Error reading file '{}': {}", input_path, e);
            return;
        }
    };

    // lexer
    let tokens = match lexer::tokenize(&source) {
        Ok(t)  => t,
        Err(e) => { println!("Lex error: {}", e); return; }
    };

    // parser
    let decl = match parser::parse_struct(&tokens) {
        Ok(d)  => d,
        Err(e) => { println!("Parse error: {}", e); return; }
    };

    println!("Struct: {}", decl.name);
    for field in &decl.fields {
        println!("  {} : {} @ 0x{:X}", field.name, field.ty, field.offset);
    }

    // attach to process
    println!("\nAttaching to PID {}...", pid);
    match memory::Process::attach(pid) {
        Ok(process) => {
            println!("Attached successfully!");

            // read each field
            println!("\nReading fields:");
            for field in &decl.fields {
                let address = field.offset;
                match field.ty.as_str() {
                    "u32" => {
                        match process.read_u32(address) {
                            Ok(val) => println!("  {} = {}", field.name, val),
                            Err(e)  => println!("  {} = ERROR: {}", field.name, e),
                        }
                    }
                    "u64" => {
                        match process.read_u64(address) {
                            Ok(val) => println!("  {} = {}", field.name, val),
                            Err(e)  => println!("  {} = ERROR: {}", field.name, e),
                        }
                    }
                    _ => println!("  {} = (type not supported yet)", field.name),
                }
            }
        }
        Err(e) => println!("Failed to attach: {}", e),
    }
}