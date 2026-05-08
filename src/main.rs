mod lexer;

fn main() {
    let inputs = vec![
        "0xFF + 10 - (0xDEAD & 42)",  // valid
        "0xFF + @@@",                  // invalid — @ is not valid in RevLang yet
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
            Err(e) => {
                println!("Error:  {}", e);
            }
        }
        println!();
    }
}