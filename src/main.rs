mod lexer;

fn main() {
    let inputs = vec![
        "0xFF + 10 - (0xDEAD & 42)",   // phase 1 — still works
        "let base: u64 = 0x7FFF0000",  // phase 2 — new!
        "let health: u32 = 100",       // phase 2 — new!
        "let player: unknown = 0xFF",  // identifier test
    ];

    for input in inputs {
        println!("Input:  \"{}\"", input);
        match lexer::tokenize(input) {
            Ok(tokens) => {
                println!("Tokens:");
                for token in tokens {
                    println!("  {:?}", token);
                }
            }
            Err(e) => println!("Error:  {}", e),
        }
        println!();
    }
}