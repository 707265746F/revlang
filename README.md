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

let p = attach(1234)
read<Player>(p, 0x5C3F20)

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
- **Memory reading:**
  - Linux / Android → `/proc/{pid}/mem` — direct memory access
  - Windows → `ReadProcessMemory` — WinAPI (future)
- **No external tools required** — RevLang reads memory directly!

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
    3. Memory       → attaches to process, reads real values
```

---

## RevLang roadmap

### Phase 1 — Lexer + expressions `[DONE]`

Tokenize the raw text of a `.rev` file.

**What we built:**
- `enum Token` — all valid token types in RevLang
- `fn tokenize(input: &str) -> Result<Vec<Token>, String>` — the lexer function
- Error reporting — unknown characters return a clear error message
- Safe character reading with `peek` instead of `take_while`

**Tokens supported:**
- Integer literals: `42`, `0xFF`, `0xDEAD`
- Operators: `+`, `-`, `*`, `/`, `&`, `|`, `^`
- Parentheses: `(`, `)`

**Key Rust concepts learned:**
- `enum` with data — `HexLit(u64)`, `IntLit(u64)`
- `match` — pattern matching every character
- `Result<T, E>` — returning `Ok(tokens)` or `Err(message)`
- `peek()` vs `take_while` — reading without consuming

**Verified output:**
```
Input:  "0xFF + 10 - (0xDEAD & 42)"
Tokens: HexLit(255), Plus, IntLit(10), Minus,
        LParen, HexLit(57005), Ampersand, IntLit(42), RParen

Input:  "0xFF + @@@"
Error:  Unknown character '@' in RevLang source
```

---

### Phase 2 — Variables + basic types `[DONE]`

Add variables and RevLang's type system.

**What we built:**
- Keywords: `let`, `struct`, `fn`
- Types: `u8`, `u16`, `u32`, `u64`, `bool`, `str`
- New symbols: `:`, `=`
- Identifiers: variable names invented by the developer
- `VarDecl` struct — represents a parsed variable declaration
- `parse_var_decl()` — validates `let name: type = value`

**Key Rust concepts learned:**
- `struct` — grouping related data like a Java class
- `tokens.get(pos)` — safe list access returning `Option`
- Nested `match` — lexer feeds the parser
- `pos` counter — a finger moving through the token list

**Verified output:**
```
Input: "let base: u64 = 0x7FFF0000"
Parsed: VarDecl { name: "base", ty: "u64", value: 2147418112 }

Input: "let broken: u32"
Parse error: Expected '='
```

---

### Phase 3 — Structs with memory offsets `[DONE]`

The most unique feature of RevLang — game structures with offsets.

**What we built:**
- New tokens: `{`, `}`, `@`, `,`
- `Field` struct — one field with name, type and offset
- `StructDecl` struct — a full struct with a list of fields
- `parse_struct()` — validates the full struct syntax

**Goal:** parse structs like this:

```revlang
struct Player {
    health: u32 @ 0x1A4,
    mana:   u32 @ 0x1A8,
    name:   str @ 0x1B0,
}
```

**Key Rust concepts learned:**
- `Vec<Field>` growing dynamically — like `ArrayList` in Java
- `while let` loop — parse any number of fields
- `0x{:X}` formatting — print numbers as hex
- `if let` — handle optional tokens like `,`

**Verified output:**
```
Struct: Player
  health : u32 @ 0x1A4
  mana : u32 @ 0x1A8
  name : str @ 0x1B0
```

---

### Phase 4a — Direct memory reading Linux `[DONE]`

Connect RevLang to real Linux processes via `/proc/{pid}/mem`.

**What we built:**
- `memory.rs` — the memory reading module
- `Process::attach(pid)` — attach to a running process
- `Process::read_u32(address)` — read 4 bytes as u32
- `Process::read_u64(address)` — read 8 bytes as u64
- `main.rs` — reads a real `.rev` file and a PID from terminal

**How it works:**
```
1 → open  /proc/{pid}/mem  as a file
2 → seek  to the memory address
3 → read  N bytes from that position
4 → convert bytes to u32/u64 (little endian)
```

**Usage:**
```bash
sudo ./revlang player.rev 1234
```

**Verified output:**
```
Attaching to PID 22245...
Attached successfully!
Reading fields:
  field1 = 0
  field2 = 0
  field3 = 33
```

**Key Rust concepts learned:**
- `impl Process` — adding methods to a struct
- `File::open` + `Seek` + `Read` — reading files at specific positions
- `from_le_bytes` — converting bytes to numbers (little endian)
- `std::env::args()` — reading terminal arguments
- `fs::read_to_string()` — reading a file from disk

---

### Phase 4b — Windows memory reading `[next]`

Add Windows support using `ReadProcessMemory`.

**How it differs from Linux:**
- Use the `winapi` Rust crate
- Call `OpenProcess` → get a handle to the game process
- Call `ReadProcessMemory` → read bytes at a given address
- No `/proc` filesystem — Windows uses API calls instead

**Same RevLang syntax — only `memory.rs` changes internally!**

---

### Phase 4c — Android memory reading `[future]`

Android uses the same Linux kernel — very similar to Phase 4a!

**How it works:**
- Same `/proc/{pid}/mem` approach as Linux
- Needs root access or ptrace permissions
- ADB shell to find the game PID: `adb shell pidof com.game.package`

**Almost free after Phase 4a — same code, same approach!**

---

### Phase 5 — Bot scripts + control flow `[dream]`

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

## Rust basics learned so far

| Concept | Why we need it | Phase |
|---|---|---|
| `enum` | Define token types | 1 |
| `match` | Handle every token case | 1 |
| `Result<T, E>` | Handle errors safely | 1 |
| `peek()` | Read without consuming | 1 |
| `struct` | Represent fields, variables | 2 |
| `Vec<T>` | Lists of tokens, fields | 2 |
| `Option<T>` | Safe list access | 2 |
| `impl` | Add methods to structs | 4a |
| `File` + `Seek` + `Read` | Read process memory | 4a |
| `from_le_bytes` | Convert bytes to numbers | 4a |
| `env::args()` | Read terminal arguments | 4a |

---

## Project structure (Rust)

```
revlang/
├── Cargo.toml
├── player.rev             ← example RevLang file
└── src/
    ├── main.rs            ← entry point, reads .rev file + PID
    ├── lexer.rs           ← Phase 1: tokenizer
    ├── parser.rs          ← Phase 2–3: grammar + structs
    └── memory.rs          ← Phase 4a: direct memory reading
```

---

## Notes

- RevLang files use the `.rev` extension
- Hex literals are first-class: `0xFF` is a valid number
- The `@` symbol means "at memory offset" — unique to RevLang
- No external tools required — RevLang reads memory directly!
- Linux and Android share the same memory reading code
- Windows support comes in Phase 4b

---

*Document created alongside the RevLang compiler project.*  
*Built step by step, phase by phase.*
