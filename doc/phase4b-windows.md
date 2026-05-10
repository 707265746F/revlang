# Phase 4b — Windows memory reading

> RevLang connects to real Windows processes using ReadProcessMemory.  
> Cross-compiled from Arch Linux — no Windows development environment needed!

---

## The big picture

Phase 4b adds Windows support to RevLang's memory reader.  
The same `.rev` file syntax works on both Linux and Windows — only `memory.rs` changes internally!

```
Linux:   sudo ./revlang player.rev 1234
Windows: revlang.exe notepad.rev 2964
```

Same language. Same syntax. Different platform. ✅

---

## Cross-compilation — Linux → Windows

One of Rust's most powerful features — compile a Windows `.exe` directly from Arch Linux!

```bash
# add the Windows compile target
rustup target add x86_64-pc-windows-gnu

# install the Windows cross-compiler
sudo pacman -S mingw-w64-gcc

# build a Windows .exe from Arch Linux!
cargo build --target x86_64-pc-windows-gnu

# output location
target/x86_64-pc-windows-gnu/debug/revlang.exe
```

No Windows development environment needed — everything built from Arch Linux! 🎉

---

## How Windows memory reading works

On Windows, reading process memory uses the WinAPI:

```
1 → OpenProcess(pid)      → get a handle to the process
2 → ReadProcessMemory()   → read bytes at a given address
3 → CloseHandle()         → release the handle when done
```

In Java terms:

```java
// Java — conceptually similar
ProcessHandle handle = ProcessHandle.of(pid);
// Windows does this at the OS level with ReadProcessMemory
```

---

## Conditional compilation — `#[cfg]`

Rust compiles different code for different platforms using `#[cfg]`:

```rust
#[cfg(target_os = "linux")]   // only compiled on Linux
mod platform {
    // /proc/{pid}/mem implementation
}

#[cfg(target_os = "windows")] // only compiled on Windows
mod platform {
    // ReadProcessMemory implementation
}

// public API — same on all platforms!
pub use platform::Process;
```

This is like `#ifdef` in C — the compiler includes only the relevant code.  
`main.rs` never changes — RevLang automatically uses the right implementation!

---

## Windows implementation — `src/memory.rs`

```rust
#[cfg(target_os = "windows")]
mod platform {
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::memoryapi::ReadProcessMemory;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::winnt::PROCESS_VM_READ;
    use winapi::shared::minwindef::FALSE;

    pub struct Process {
        handle: winapi::um::winnt::HANDLE,
    }

    impl Process {
        pub fn attach(pid: u32) -> Result<Process, String> {
            unsafe {
                let handle = OpenProcess(PROCESS_VM_READ, FALSE, pid);
                if handle.is_null() {
                    Err(format!("Cannot open process {}", pid))
                } else {
                    Ok(Process { handle })
                }
            }
        }

        pub fn read_bytes(&self, address: u64, size: usize) -> Result<Vec<u8>, String> {
            let mut buffer = vec![0u8; size];
            let mut bytes_read: usize = 0;
            unsafe {
                let result = ReadProcessMemory(
                    self.handle,
                    address as *const _,
                    buffer.as_mut_ptr() as *mut _,
                    size,
                    &mut bytes_read,
                );
                if result == FALSE {
                    return Err(format!("Cannot read at 0x{:X}", address));
                }
            }
            Ok(buffer)
        }

        pub fn read_u32(&self, address: u64) -> Result<u32, String> {
            let bytes = self.read_bytes(address, 4)?;
            Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        }

        pub fn read_u64(&self, address: u64) -> Result<u64, String> {
            let bytes = self.read_bytes(address, 8)?;
            Ok(u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        }
    }

    // automatically close the handle when Process is dropped
    impl Drop for Process {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.handle); }
        }
    }
}
```

### `unsafe` — what does it mean?

WinAPI calls require `unsafe` in Rust because Rust cannot guarantee memory safety when calling external C functions. It is like calling JNI in Java — you are stepping outside the safe zone:

```java
// Java JNI — unsafe external call
native void readMemory(long address);
```

```rust
// Rust — unsafe external call
unsafe {
    ReadProcessMemory(handle, address, buffer, size, &mut bytes_read);
}
```

### `Drop` trait — automatic cleanup

The `Drop` trait in Rust is like `AutoCloseable` in Java — it runs automatically when the object goes out of scope:

```java
// Java — manual cleanup or try-with-resources
handle.close();
```

```rust
// Rust — automatic cleanup via Drop trait
impl Drop for Process {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle); }
    }
}
// handle is closed automatically when Process goes out of scope!
```

---

## Finding valid memory addresses on Windows

### Method 1 — PowerShell

```powershell
# find process PID
tasklist | findstr notepad

# get base address of the process modules
(Get-Process -Id 2964).Modules |
    Select-Object BaseAddress, ModuleName |
    Select-Object -First 5

# convert decimal address to hex
"{0:X}" -f 140699569291264
# = 7FF72BD90000
```

### Method 2 — run RevLang as Administrator

Always run `revlang.exe` as Administrator on Windows — same as `sudo` on Linux:

```
Right click cmd.exe → Run as Administrator
```

---

## The MZ magic bytes

When we read `notepad.exe` base address `0x7FF72BD90000`:

```
field1 = 9460301
```

Converting to hex:

```powershell
"{0:X}" -f 9460301
# = 905A4D
```

In little endian bytes: `4D 5A 90` — which is:

```
4D 5A = "MZ" ← Windows PE header magic bytes!
```

Every Windows `.exe` starts with `MZ` — named after Mark Zbikowski,  
one of the original DOS developers. RevLang read the PE signature! 🎉

---

## Famous magic bytes in reversing

```
4D 5A       → "MZ"      → Windows PE executable
7F 45 4C 46 → "ELF"     → Linux executable
CA FE BA BE → CAFEBABE  → Java .class file
DE AD BE EF → DEADBEEF  → common debug marker
50 4B 03 04 → "PK"      → ZIP / APK file
```

---

## Verified output

```
Struct: NotepadMemory
  field1 : u32 @ 0x7FF72BD90000
  field2 : u32 @ 0x7FF72BD90004
  field3 : u32 @ 0x7FF72BD90008

Attaching to PID 2964...
Attached successfully!
Reading fields:
  field1 = 9460301   ← 0x905A4D = MZ PE header!
  field2 = 3
  field3 = 4
```

---

## Platform support summary

| Platform | Method | Status |
|---|---|---|
| Linux | `/proc/{pid}/mem` | ✅ Phase 4a |
| Windows | `ReadProcessMemory` | ✅ Phase 4b |
| Android | `/proc/{pid}/mem` + root | 🔜 Phase 4c |

---

## What comes next — Phase 4c

Android uses the same Linux kernel as Phase 4a!

```
Same /proc/{pid}/mem approach
Same Rust code — almost no changes needed
Need: root access + ADB to find game PID
```

```bash
# find Android game PID via ADB
adb shell pidof com.game.package

# run RevLang on Android (via ADB shell)
adb shell ./revlang game.rev 1234
```

---

*Phase 4b completed and verified.*  
*Next: Phase 4c — Android memory reading.*
