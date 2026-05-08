# Phase 1 — Lexer

> The lexer is the first phase of the RevLang compiler.  
> It reads raw text character by character and produces a list of tokens.

---

## What is a Lexer?

A lexer (also called tokenizer) is like a security guard at the door.  
It reads every character of your `.rev` file and asks: **"do I know you?"**

- If yes → it produces a token and continues
- If no → it returns an error with the unknown character

**Simple example:**

```
Input text:   "0xFF + 10"
              ↓
Lexer reads:  '0', 'x', 'F', 'F', ' ', '+', ' ', '1', '0'
              ↓
Output:       [HexLit(255), Plus, IntLit(10)]
```

---

## Token types

A token is a small, meaningful piece of your source code.  
In RevLang, every token is defined as a Rust `enum`:

```rust
#[derive(Debug, PartialEq)]
pub enum Token {
    // literals — carry a value inside
    IntLit(u64),     // e.g. 42, 100
    HexLit(u64),     // e.g. 0xFF, 0xDEAD

    // operators
    Plus,            // +
    Minus,           // -
    Star,            // *
    Slash,           // /
    Ampersand,       // &
    Pipe,            // |
    Caret,           // ^

    // punctuation
    LParen,          // (
    RParen,          // )
}
```

### Why `enum`?

In Kotlin you know `enum class` — Rust `enum` is similar but much more powerful.  
Each variant can carry data inside:

```rust
// This variant carries a u64 number inside it
IntLit(u64)

// This variant carries nothing — it is just a symbol
Plus
```

This is perfect for tokens because `Plus` is just a symbol,  
but a number token needs to carry the actual value with it.

---

## The tokenize function

This is the main function of the lexer.  
It reads the input string and returns a list of tokens, or an error.

```rust
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
```

---

## Key Rust concepts learned

### `Result<T, E>`

Instead of crashing when something is wrong, Rust functions return a `Result`:

```rust
Ok(tokens)    // everything went fine — here are your tokens
Err(message)  // something was wrong — here is what happened
```

This is like `Result<T, E>` in Kotlin, or a success/error response in Retrofit.  
The caller decides what to do with the error — the lexer just reports it.

### `match`

`match` is like a `when` in Kotlin but more powerful.  
It must cover every possible case — Rust will not compile if you forget one:

```rust
match ch {
    '+' => { /* handle plus */ }
    '-' => { /* handle minus */ }
    _   => { /* handle everything else */ }  // the _ means "anything"
}
```

### `Vec<T>`

`Vec` is a dynamic list — like `ArrayList` in Java or `MutableList` in Kotlin:

```rust
let mut tokens = Vec::new();   // create empty list
tokens.push(Token::Plus);      // add an item
```

---

## The bug we found — `take_while` vs `peek`

### The problem

The first version of the number parsing used `take_while`:

```rust
// DANGEROUS version
let num: String = chars
    .take_while(|c| c.is_ascii_digit())
    .collect();
```

This caused a bug — the `)` at the end of `(0xDEAD & 42)` was being lost!

**Why?** `take_while` reads one character ahead to check the condition,  
and if the condition is false, it throws that character away.

```
Reading "42)" with take_while:
  reads '4' → is digit? yes → keep it
  reads '2' → is digit? yes → keep it
  reads ')' → is digit? no  → THROW IT AWAY! ← bug!
```

### The fix — use `peek`

`peek` looks at the next character WITHOUT consuming it.  
Only if the character is valid, we call `chars.next()` to actually advance:

```rust
// SAFE version
while let Some(&c) = chars.peek() {
    if c.is_ascii_digit() {
        num.push(c);
        chars.next(); // only move forward if we want this character
    } else {
        break; // stop — but ')' is still in the stream!
    }
}
```

```
Reading "42)" with peek:
  peek '4' → is digit? yes → push, advance
  peek '2' → is digit? yes → push, advance
  peek ')' → is digit? no  → break! ')' is still in the stream ✓
```

### The analogy

Think of it like reading memory in Cheat Engine:  
- `peek` = reading a memory address without changing the program counter  
- `take_while` = accidentally incrementing the program counter

---

## Final verified output

```
Input:  "0xFF + 10 - (0xDEAD & 42)"
Tokens:
  HexLit(255)
  Plus
  IntLit(10)
  Minus
  LParen
  HexLit(57005)
  Ampersand
  IntLit(42)
  RParen

Input:  "0xFF + @@@"
Error:  Unknown character '@' in RevLang source
```

Note: `0xFF` = 255 and `0xDEAD` = 57005 in decimal.  
The lexer converts hex to real numbers internally — that is correct behavior.

---

## What comes next — Phase 2

In Phase 2 we will add keywords and types to the lexer:

```
let    → Keyword(Let)
struct → Keyword(Struct)
u32    → Type(U32)
u64    → Type(U64)
bool   → Type(Bool)
```

Then we will build the parser that understands:

```revlang
let base: u64 = 0x7FFF0000
```

---

## Full file — `src/lexer.rs`

```rust
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
            ' ' | '\t' | '\n' => { chars.next(); }

            '+' => { tokens.push(Token::Plus);      chars.next(); }
            '-' => { tokens.push(Token::Minus);     chars.next(); }
            '*' => { tokens.push(Token::Star);      chars.next(); }
            '/' => { tokens.push(Token::Slash);     chars.next(); }
            '&' => { tokens.push(Token::Ampersand); chars.next(); }
            '|' => { tokens.push(Token::Pipe);      chars.next(); }
            '^' => { tokens.push(Token::Caret);     chars.next(); }
            '(' => { tokens.push(Token::LParen);    chars.next(); }
            ')' => { tokens.push(Token::RParen);    chars.next(); }

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

            '1'..='9' => {
                let mut num = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() { num.push(c); chars.next(); }
                    else { break; }
                }
                tokens.push(Token::IntLit(num.parse().unwrap()));
            }

            _ => {
                return Err(format!("Unknown character '{}' in RevLang source", ch));
            }
        }
    }

    Ok(tokens)
}
```

---

*Phase 1 completed and verified.*  
*Next: Phase 2 — Variables and types.*
