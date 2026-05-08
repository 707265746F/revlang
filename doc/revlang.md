# RevLang

> A small scripting language for reverse engineers.  
> Built with Rust, designed for reading memory, defining game structures, parsing binary formats, and automating game bots.

---

## What is RevLang?

RevLang is a custom language created from scratch. The goal is to write scripts like this:

```revlang
struct Player {
    health: u32 @ 0x1A4,
    mana:   u32 @ 0x1A8,
    name:   str @ 0x1B0,
}

let p = attach("game.exe")
let player = p.read<Player>(base + 0x5C3F20)

while true {
    if player.health < 20 {
        use_potion()
    }
    attack_nearest_enemy()
    wait(500)
}
```

---

## Tools and language

- **Language:** Rust
- **Key crates (libraries):**
  - `logos` — lexer (tokenizer)
  - `chumsky` — parser
  - `inkwell` — code generation via LLVM (later phase)

---

## Compiler phases

Every compiler follows the same pipeline:

```
Source code (.rev file)
        ↓
    1. Lexer        → reads characters, produces tokens
        ↓
    2. Parser       → checks grammar, builds structure
        ↓
    3. AST          → Abstract Syntax Tree (the program as a tree)
        ↓
    4. Code gen     → outputs machine code / bytecode / scripts
```

---

## RevLang roadmap

### Phase 1 — Lexer + expressions `[current]`

Learn to tokenize the raw text of a `.rev` file.

**Goal:** read `1 + 2` or `0xFF & 0x0F` and produce a list of tokens.

**Tokens to support:**
- Integer literals: `42`, `0xFF`, `0xDEAD`
- Operators: `+`, `-`, `*`, `/`, `&`, `|`, `^`
- Parentheses: `(`, `)`

**Rust concepts needed:**
- `enum` — to define token types
- `match` — to handle each token
- Basic string slicing

**Example output:**
```
Input:  "0xFF + 10"
Tokens: [HexLit(0xFF), Plus, IntLit(10)]
```

---

### Phase 2 — Variables + basic types

Add variables and RevLang's type system.

**Goal:** parse and store `let base: u64 = 0x7FFF0000`

**Types to support:** `u8`, `u16`, `u32`, `u64`, `bool`, `str`

**Rust concepts needed:**
- `struct` — to represent a variable in memory
- `HashMap` — to store variables by name
- `Box<T>` — for tree nodes that point to other nodes

---

### Phase 3 — Structs with memory offsets

The most unique feature of RevLang — game structures with offsets.

**Goal:** parse structs like this:

```revlang
struct Player {
    health: u32 @ 0x1A4,
    mana:   u32 @ 0x1A8,
    name:   str @ 0x1B0,
}
```

**Rust concepts needed:**
- Recursive `enum` for AST nodes
- `Vec<Field>` to store struct fields
- Parsing the `@` offset syntax

---

### Phase 4a — Process + memory reading (PC / Windows) `[first target]`

Connect RevLang to real Windows processes.

**Goal:** attach to a PC game and read memory directly.

```revlang
let p = attach("game.exe")
let player = p.read<Player>(base + 0x5C3F20)
print(player.health)
```

**How it works on Windows:**
- Use the `winapi` Rust crate
- Call `OpenProcess` → get a handle to the game process
- Call `ReadProcessMemory` → read bytes at a given address
- No root needed, no sandbox — very direct

**RevLang output:** a native Rust binary, or a Frida `.js` script for Windows injection.

---

### Phase 4b — Android support `[future target]`

After PC works, extend RevLang to Android games.

**How it differs from PC:**
- Android uses a Linux security layer (SELinux) — no direct memory access
- Need root or Frida server injected into the device
- Read memory via `/proc/{pid}/mem` or Frida's Memory API

**RevLang output:** a Frida `.js` script injected via `frida -U -f com.game.package`

> The great news: both PC and Android can share the same Frida script output.  
> RevLang writes the script once — it runs on both platforms.

---

### Phase 5 — Bot scripts + control flow

Add `if`, `while`, `fn` — make RevLang a real scripting language.

**Goal:** write a farming bot in RevLang.

```revlang
fn farm_loop(p: Process) {
    let player = p.read<Player>(base + 0x5C3F20)
    while true {
        if player.health < 20 {
            use_potion()
        }
        attack_nearest_enemy()
        wait(500)
    }
}
```

---

## Rust basics needed for this project

Since we are learning Rust alongside RevLang, here are the key concepts in the order we need them:

| Concept | Why we need it | Phase |
|---|---|---|
| `enum` | Define token types and AST nodes | 1 |
| `match` | Handle every token case | 1 |
| `struct` | Represent fields, variables | 2 |
| `Vec<T>` | Lists of tokens, fields | 1 |
| `String` / `&str` | Read source code text | 1 |
| `HashMap` | Variable storage | 2 |
| `Box<T>` | Recursive tree nodes | 2 |
| `impl` | Add methods to types | 2 |
| `Result<T, E>` | Handle parse errors | 2 |
| Traits (`Display`) | Print tokens and AST | 3 |

---

## Project structure (Rust)

```
revlang/
├── Cargo.toml
└── src/
    ├── main.rs        ← entry point, reads the .rev file
    ├── lexer.rs       ← Phase 1: tokenizer
    ├── parser.rs      ← Phase 2–3: grammar + AST
    ├── ast.rs         ← the AST node definitions
    └── codegen.rs     ← Phase 4–5: output code
```

---

## Notes

- RevLang files use the `.rev` extension
- Hex literals are first-class: `0xFF` is a valid number
- The `@` symbol means "at memory offset" — unique to RevLang
- First target platform: **PC / Windows** — using `ReadProcessMemory` via the `winapi` crate
- Future target: **Android** — via Frida server injection
- Output targets: native Rust binary (PC), Frida `.js` scripts (PC + Android)

---

*Document created alongside the RevLang compiler project.*  
*Built step by step, phase by phase.*
