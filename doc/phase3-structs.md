# Phase 3 — Structs with memory offsets

> The most unique feature of RevLang.  
> Define game structures with memory offsets — exactly like a reverser thinks.

---

## The big picture

Before Phase 3, your reverse engineering notes looked like this:

```
Player struct:
  health  → 0x1A4
  mana    → 0x1A8
  name    → 0x1B0
```

Just a text file — no structure, no validation, nothing automatic.

After Phase 3, you write this instead:

```revlang
struct Player {
    health: u32 @ 0x1A4,
    mana:   u32 @ 0x1A8,
    name:   str @ 0x1B0,
}
```

RevLang reads it, validates it, and later will use it to read real memory automatically!

---

## New tokens added

Three new tokens added to `enum Token` in `src/lexer.rs`:

```rust
LBrace,  // {
RBrace,  // }
At,      // @
Comma,   // ,
```

And added to the `match` in `tokenize`:

```rust
'{' => { tokens.push(Token::LBrace); chars.next(); }
'}' => { tokens.push(Token::RBrace); chars.next(); }
'@' => { tokens.push(Token::At);     chars.next(); }
',' => { tokens.push(Token::Comma);  chars.next(); }
```

Note: `@` was previously an unknown character — it now means "at memory offset"!

---

## New structs in the parser

In Java terms:

```java
// Java
class Field {
    public String name;
    public String ty;
    public long offset;
}

class StructDecl {
    public String name;
    public ArrayList<Field> fields;
}
```

In Rust:

```rust
// Rust — same idea, different syntax
#[derive(Debug)]
pub struct Field {
    pub name:   String,
    pub ty:     String,
    pub offset: u64,
}

#[derive(Debug)]
pub struct StructDecl {
    pub name:   String,
    pub fields: Vec<Field>,  // Vec = ArrayList in Java
}
```

`Vec<Field>` is a list that grows as we parse each field one by one.

---

## How the struct parser works

The parser reads tokens in this exact order for each field:

```
Checkpoint 1 → 'struct'         keyword
Checkpoint 2 → 'Player'         struct name (identifier)
Checkpoint 3 → '{'              open brace
Checkpoint 4 → loop until '}':
    → field name   e.g. 'health'
    → ':'
    → type         e.g. 'u32'
    → '@'
    → offset       e.g. 0x1A4
    → ',' (optional)
Checkpoint 5 → '}'              close brace
```

### The field loop

This is the key new concept in Phase 3 — a `while` loop that keeps parsing fields until it finds `}`:

```rust
let mut fields = Vec::new();

while let Some(token) = tokens.get(pos) {
    // if we find '}' the struct is done
    if *token == Token::RBrace {
        pos += 1;
        break;
    }

    // parse one field...
    fields.push(Field { name: field_name, ty: field_ty, offset });
}
```

This is different from `parse_var_decl` — that function had a fixed number of steps.  
Here we don't know how many fields the struct has — the loop handles any number!

---

## Full parse function

```rust
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
        if *token == Token::RBrace {
            pos += 1;
            break;
        }

        let field_name = match tokens.get(pos) {
            Some(Token::Ident(n)) => { pos += 1; n.clone() }
            _ => return Err("Expected field name".to_string()),
        };

        match tokens.get(pos) {
            Some(Token::Colon) => pos += 1,
            _ => return Err("Expected ':'".to_string()),
        }

        let field_ty = match tokens.get(pos) {
            Some(Token::U8)  => { pos += 1; "u8".to_string()  }
            Some(Token::U16) => { pos += 1; "u16".to_string() }
            Some(Token::U32) => { pos += 1; "u32".to_string() }
            Some(Token::U64) => { pos += 1; "u64".to_string() }
            Some(Token::Str) => { pos += 1; "str".to_string() }
            _ => return Err("Expected a type".to_string()),
        };

        match tokens.get(pos) {
            Some(Token::At) => pos += 1,
            _ => return Err("Expected '@'".to_string()),
        }

        let offset = match tokens.get(pos) {
            Some(Token::HexLit(n)) => { pos += 1; *n }
            Some(Token::IntLit(n)) => { pos += 1; *n }
            _ => return Err("Expected offset value".to_string()),
        };

        // comma after each field is optional
        if let Some(Token::Comma) = tokens.get(pos) {
            pos += 1;
        }

        fields.push(Field { name: field_name, ty: field_ty, offset });
    }

    Ok(StructDecl { name, fields })
}
```

---

## Key Rust concepts learned

### `Vec<T>` growing dynamically

```rust
let mut fields = Vec::new();   // empty list — like new ArrayList<>() in Java
fields.push(field);            // add item — like ArrayList.add() in Java
```

### `while let` loop

Keeps looping as long as `tokens.get(pos)` returns `Some(token)`:

```rust
while let Some(token) = tokens.get(pos) {
    // runs for each token until the list is empty
}
```

### `0x{:X}` formatting

Prints a number back as hex — very useful for memory addresses:

```rust
println!("offset: 0x{:X}", 0x1A4);  // prints: offset: 0x1A4
println!("offset: {}",     0x1A4);  // prints: offset: 420
```

Always use `0x{:X}` for memory offsets in RevLang output!

### Optional token — `if let`

The comma after each field is optional — we use `if let` to skip it if present:

```rust
if let Some(Token::Comma) = tokens.get(pos) {
    pos += 1;  // skip the comma if it exists
}
```

This is like checking `if (token != null)` in Java but more elegant.

---

## Verified output

```
Input:
  struct Player { health: u32 @ 0x1A4, mana: u32 @ 0x1A8, name: str @ 0x1B0 }

Struct: Player
  health : u32 @ 0x1A4
  mana : u32 @ 0x1A8
  name : str @ 0x1B0
```

RevLang reads the struct, validates every field, and prints it back with hex offsets!

---

## What comes next — Phase 4a

In Phase 4a we connect RevLang to real Windows processes.

```revlang
let p = attach("game.exe")
let player = p.read<Player>(base + 0x5C3F20)
print(player.health)
```

**How it works on Windows:**
- Use the `winapi` Rust crate
- Call `OpenProcess` → get a handle to the game process
- Call `ReadProcessMemory` → read bytes at a given address
- Use the `StructDecl` we built in Phase 3 to know the field offsets!

---

## Current doc folder

```
doc/
├── revlang.md         ✅ general overview and roadmap
├── phase1-lexer.md    ✅ lexer — character by character analysis
├── phase2-parser.md   ✅ parser — variables and structure validation
└── phase3-structs.md  ✅ structs — memory offset definitions
```

---

*Phase 3 completed and verified.*  
*Next: Phase 4a — Process and memory reading on PC / Windows.*
