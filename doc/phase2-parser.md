# Phase 2 — Variables and Parser

> The parser is the second phase of the RevLang compiler.  
> It receives a list of tokens from the lexer and validates their structure and order.

---

## The big picture

After Phase 1 we had this:

```
"let base: u64 = 0x7FFF0000"
        ↓  lexer
[Let, Ident("base"), Colon, U64, Equals, HexLit(2147418112)]
```

In Phase 2 we added two things:

1. Extended the lexer with keywords, types and identifiers
2. Built the parser that understands the structure of a variable declaration

---

## Lexer extensions — new tokens

New tokens added to `enum Token` in `src/lexer.rs`:

```rust
// punctuation
Colon,    // :
Equals,   // =

// keywords
Let,
Struct,
Fn,

// types
U8,
U16,
U32,
U64,
Bool,
Str,

// identifier — a name invented by the developer
Ident(String),
```

### How keyword detection works

The lexer reads a full word character by character, then decides what it is:

```
reads "let"    → is it a keyword? YES → Token::Let
reads "base"   → is it a keyword? NO  → Token::Ident("base")
reads "u64"    → is it a type?    YES → Token::U64
reads "player" → is it a keyword? NO  → Token::Ident("player")
```

This is called **keyword detection** — every real language does it this way.

### The word reading loop

```rust
'a'..='z' | 'A'..='Z' | '_' => {
    let mut word = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
            chars.next();
        } else {
            break;
        }
    }
    let token = match word.as_str() {
        "let"    => Token::Let,
        "struct" => Token::Struct,
        "fn"     => Token::Fn,
        "u8"     => Token::U8,
        "u16"    => Token::U16,
        "u32"    => Token::U32,
        "u64"    => Token::U64,
        "bool"   => Token::Bool,
        "str"    => Token::Str,
        _        => Token::Ident(word),
    };
    tokens.push(token);
}
```

---

## Lexer vs Parser — the key concept

This is the most important concept of the compiler:

```
lexer.rs  → WHAT are the valid pieces?
parser.rs → ARE the pieces in the correct ORDER?
```

A real example:

```
"let base: u64 = 0x7FFF0000"  ✅ lexer  ✅ parser
"let !!!: u64 = 0x7FFF0000"   ❌ lexer  (!!!) is not valid)
"base let u64: = 0x7FFF0000"  ✅ lexer  ❌ parser (wrong order)
```

It is exactly like English grammar:

```
"The cat ate the fish"  ✅ valid words  ✅ correct order
"Zxqk @@@ blurp"       ❌ invalid words
"Fish the cat the ate"  ✅ valid words  ❌ wrong order
```

The lexer is the dictionary — it checks if words exist.  
The parser is the grammar teacher — it checks if the sentence makes sense!

---

## The parser — `src/parser.rs`

### VarDecl struct

The parser produces a `VarDecl` — a structured representation of a variable declaration:

```rust
#[derive(Debug)]
pub struct VarDecl {
    pub name:  String,   // the variable name
    pub ty:    String,   // the type as a string
    pub value: u64,      // the value
}
```

Note: we use `ty` instead of `type` because `type` is a reserved keyword in Rust —
just like `class` is reserved in Kotlin!

### How `pos` works

The parser uses a position counter `pos` — like a finger pointing at the current token:

```
pos→  0        1           2       3    4       5
    [ Let,  Ident("base"), Colon,  U64, Equals, HexLit(2147418112) ]
```

Each time we check a token and it is correct, we move the finger forward with `pos += 1`.

### The parser as a checkpoint system

The parser is like a strict security checkpoint at an airport:

```
Checkpoint 1 → show your passport     (must be 'let')
Checkpoint 2 → show your ticket       (must be a name)
Checkpoint 3 → show your boarding     (must be ':')
Checkpoint 4 → show your destination  (must be a type)
Checkpoint 5 → show your seat         (must be '=')
Checkpoint 6 → show your luggage      (must be a number)
```

If you fail any checkpoint → error!

### Full parse function

```rust
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
```

---

## Key Rust concepts learned

### `struct`

A `struct` groups related data together — like `data class` in Kotlin:

```kotlin
// Kotlin
data class VarDecl(val name: String, val ty: String, val value: Long)
```

```rust
// Rust
struct VarDecl {
    name:  String,
    ty:    String,
    value: u64,
}
```

### `tokens.get(pos)`

Instead of accessing a list directly with `tokens[pos]` (which can crash if out of bounds),
`tokens.get(pos)` returns an `Option` — either `Some(token)` or `None`. Safe by default!

```rust
// risky in other languages — can crash!
tokens[pos]

// safe in Rust — returns Some or None
tokens.get(pos)
```

### Nested `match`

The pipeline uses two `match` — one inside the other:

```rust
match lexer::tokenize(input) {        // first check the lexer
    Ok(tokens) => {
        match parser::parse_var_decl(&tokens) {  // then check the parser
            Ok(decl) => println!("Parsed: {:?}", decl),
            Err(e)   => println!("Parse error: {}", e),
        }
    }
    Err(e) => println!("Lex error: {}", e),
}
```

This is the pipeline working — lexer feeds the parser!

---

## Verified output

```
Input: "let base: u64 = 0x7FFF0000"
Parsed: VarDecl { name: "base", ty: "u64", value: 2147418112 }

Input: "let health: u32 = 100"
Parsed: VarDecl { name: "health", ty: "u32", value: 100 }

Input: "let broken: u32"
Parse error: Expected '='
```

---

## What comes next — Phase 3

In Phase 3 we will add struct definitions with memory offsets:

```revlang
struct Player {
    health: u32 @ 0x1A4,
    mana:   u32 @ 0x1A8,
    name:   str @ 0x1B0,
}
```

New tokens needed: `{`, `}`, `@`  
New Rust concepts: recursive `enum` for AST nodes, `Vec<Field>`

---

## Full files

### `src/parser.rs`

```rust
use crate::lexer::Token;

#[derive(Debug)]
pub struct VarDecl {
    pub name:  String,
    pub ty:    String,
    pub value: u64,
}

pub fn parse_var_decl(tokens: &[Token]) -> Result<VarDecl, String> {
    let mut pos = 0;

    match tokens.get(pos) {
        Some(Token::Let) => pos += 1,
        _ => return Err("Expected 'let'".to_string()),
    }

    let name = match tokens.get(pos) {
        Some(Token::Ident(n)) => { pos += 1; n.clone() }
        _ => return Err("Expected variable name".to_string()),
    };

    match tokens.get(pos) {
        Some(Token::Colon) => pos += 1,
        _ => return Err("Expected ':'".to_string()),
    }

    let ty = match tokens.get(pos) {
        Some(Token::U8)  => { pos += 1; "u8".to_string()  }
        Some(Token::U16) => { pos += 1; "u16".to_string() }
        Some(Token::U32) => { pos += 1; "u32".to_string() }
        Some(Token::U64) => { pos += 1; "u64".to_string() }
        _ => return Err("Expected a type (u8, u16, u32, u64)".to_string()),
    };

    match tokens.get(pos) {
        Some(Token::Equals) => pos += 1,
        _ => return Err("Expected '='".to_string()),
    }

    let value = match tokens.get(pos) {
        Some(Token::IntLit(n)) => *n,
        Some(Token::HexLit(n)) => *n,
        _ => return Err("Expected a number value".to_string()),
    };

    Ok(VarDecl { name, ty, value })
}
```

---

*Phase 2 completed and verified.*  
*Next: Phase 3 — Structs with memory offsets.*
