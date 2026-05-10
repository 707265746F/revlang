# Phase 4a — Direct memory reading (Linux)

> RevLang connects to real Linux processes and reads memory directly.  
> No external tools required — pure Rust via `/proc/{pid}/mem`.

---

## The big picture

Before Phase 4a, RevLang could only parse `.rev` files and print struct definitions.  
In Phase 4a, RevLang reads **real memory values** from a running process!

The full pipeline now looks like this:

```
player.rev
    ↓  sudo ./revlang player.rev 1234
lexer.rs      → tokenize
    ↓
parser.rs     → validate structure, build StructDecl
    ↓
memory.rs     → attach to process, read real values
    ↓
terminal      → print field values
```

---

## How Linux memory reading works

On Linux, every running process has a folder in `/proc`:

```
/proc/1234/
    mem    ← the entire memory of the process as a file!
    maps   ← shows all memory regions with their addresses
```

Reading memory is literally opening a file and reading bytes at a position:

```
1 → open  /proc/{pid}/mem  as a file
2 → seek  to the memory address  (like seeking in a video)
3 → read  N bytes from that position
4 → convert bytes to u32/u64 (little endian)
```

In Java terms you already know:

```java
// Java — same idea!
RandomAccessFile mem = new RandomAccessFile("/proc/1234/mem", "r");
mem.seek(0x5571f59c4000L);  // jump to address
byte[] buffer = new byte[4];
mem.read(buffer);            // read 4 bytes
```

---

## Little endian — what is it?

x86 and ARM store numbers in **little endian** order.  
This means the smallest byte comes first in memory.

Example — the number `0x000001A4` in memory:

```
Address:  0x100  0x101  0x102  0x103
Bytes:    0xA4   0x01   0x00   0x00   ← little endian (smallest first)

NOT:      0x00   0x00   0x01   0xA4   ← big endian (largest first)
```

In Rust we convert bytes to numbers with `from_le_bytes` (`le` = little endian):

```rust
let bytes = [0xA4, 0x01, 0x00, 0x00];
let value = u32::from_le_bytes(bytes);  // = 420 = 0x1A4 ✓
```

You already know this from reversing — Cheat Engine uses little endian by default!

---

## Reading the memory map

Before reading memory, always check `/proc/{pid}/maps` to find valid regions:

```bash
cat /proc/22245/maps | head -20
```

Output example:

```
5571ea9ec000-5571ea9fc000 r--p ... /usr/bin/zsh    ← read only
5571ea9fc000-5571eaab5000 r-xp ... /usr/bin/zsh    ← executable
5571f59c4000-5571f5b4e000 rw-p ... [heap]          ← read/write ✓
7fff8a000000-7fff8a021000 rwxp ... [stack]          ← read/write ✓
```

The permissions column (`r--p`, `rw-p`) tells you what you can do:
- `r` → readable
- `w` → writable
- `x` → executable
- `p` → private (copy on write)

Always read from `rw-p` regions — those have real data!

---

## The memory module — `src/memory.rs`

```rust
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub struct Process {
    pub pid: u32,
}

impl Process {
    // attach to a running process by PID
    pub fn attach(pid: u32) -> Result<Process, String> {
        let mem_path = format!("/proc/{}/mem", pid);
        if std::path::Path::new(&mem_path).exists() {
            Ok(Process { pid })
        } else {
            Err(format!("Process {} not found", pid))
        }
    }

    // read N bytes from a memory address
    pub fn read_bytes(&self, address: u64, size: usize) -> Result<Vec<u8>, String> {
        let mem_path = format!("/proc/{}/mem", self.pid);

        let mut file = File::open(&mem_path)
            .map_err(|e| format!("Cannot open mem: {}", e))?;

        file.seek(SeekFrom::Start(address))
            .map_err(|e| format!("Cannot seek to 0x{:X}: {}", address, e))?;

        let mut buffer = vec![0u8; size];
        file.read_exact(&mut buffer)
            .map_err(|e| format!("Cannot read memory at 0x{:X}: {}", address, e))?;

        Ok(buffer)
    }

    // read a u32 from a memory address (4 bytes, little endian)
    pub fn read_u32(&self, address: u64) -> Result<u32, String> {
        let bytes = self.read_bytes(address, 4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    // read a u64 from a memory address (8 bytes, little endian)
    pub fn read_u64(&self, address: u64) -> Result<u64, String> {
        let bytes = self.read_bytes(address, 8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}
```

---

## Key Rust concepts learned

### `impl` — adding methods to a struct

In Java, methods live inside a class:

```java
// Java
class Process {
    public static Process attach(int pid) { ... }
    public int readU32(long address) { ... }
}
```

In Rust, methods are defined in a separate `impl` block:

```rust
// Rust
struct Process { pub pid: u32 }

impl Process {
    pub fn attach(pid: u32) -> Result<Process, String> { ... }
    pub fn read_u32(&self, address: u64) -> Result<u32, String> { ... }
}
```

`&self` means "this method belongs to an instance of Process" —  
like `this` in Java!

### `?` operator — error propagation

Instead of writing `match` for every error, Rust has the `?` shortcut:

```rust
// without ? — verbose
let mut file = match File::open(&mem_path) {
    Ok(f)  => f,
    Err(e) => return Err(format!("Cannot open: {}", e)),
};

// with ? — clean!
let mut file = File::open(&mem_path)
    .map_err(|e| format!("Cannot open: {}", e))?;
```

`?` means: "if this is an error, return it immediately".  
Very similar to throwing an exception in Java!

### `SeekFrom::Start` — jumping to a memory address

```rust
file.seek(SeekFrom::Start(address))?;
```

Like `RandomAccessFile.seek()` in Java — jumps to a specific byte position.

### `vec![0u8; size]` — creating a buffer

```rust
let mut buffer = vec![0u8; size];  // creates a Vec of `size` zeros
```

Like `new byte[size]` in Java — a fixed size buffer of zeros.

---

## Usage

```bash
# build RevLang
cargo build

# find a process PID
echo $$              # your current shell PID
pidof firefox        # PID of Firefox

# check memory map of the process
cat /proc/<PID>/maps | head -20

# run RevLang with sudo (needed for /proc/mem access)
sudo ./target/debug/revlang player.rev <PID>
```

---

## Verified output

```
Struct: ZshMemory
  field1 : u32 @ 0x5571F59C4000
  field2 : u32 @ 0x5571F59C4004
  field3 : u32 @ 0x5571F59C4008

Attaching to PID 22245...
Attached successfully!
Reading fields:
  field1 = 0
  field2 = 0
  field3 = 33
```

Real values read from a real Linux process! 🎉

---

## Platform comparison

| Platform | Method | Status |
|---|---|---|
| Linux | `/proc/{pid}/mem` | ✅ done |
| Android | `/proc/{pid}/mem` | 🔜 almost free — same code! |
| Windows | `ReadProcessMemory` | 🔜 Phase 4b |

---

## What comes next — Phase 4b

Add Windows support using the `winapi` Rust crate.

```rust
// Windows — different API, same RevLang syntax!
unsafe {
    ReadProcessMemory(handle, address, buffer, size, &mut bytes_read);
}
```

The RevLang syntax stays exactly the same — only `memory.rs` changes internally!

---

## Current doc folder

```
doc/
├── revlang.md          ✅ general overview and roadmap
├── phase1-lexer.md     ✅ lexer — character by character analysis
├── phase2-parser.md    ✅ parser — variables and structure validation
├── phase3-structs.md   ✅ structs — memory offset definitions
└── phase4a-memory.md   ✅ memory — direct process memory reading
```

---

*Phase 4a completed and verified.*  
*Next: Phase 4b — Windows memory reading via ReadProcessMemory.*
