// ─── Linux implementation ────────────────────────────────────────────────────
#[cfg(target_os = "linux")]
mod platform {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};

    pub struct Process {
        pub pid: u32,
    }

    impl Process {
        pub fn attach(pid: u32) -> Result<Process, String> {
            let mem_path = format!("/proc/{}/mem", pid);
            if std::path::Path::new(&mem_path).exists() {
                Ok(Process { pid })
            } else {
                Err(format!("Process {} not found", pid))
            }
        }

        pub fn read_bytes(&self, address: u64, size: usize) -> Result<Vec<u8>, String> {
            let mem_path = format!("/proc/{}/mem", self.pid);
            let mut file = File::open(&mem_path)
                .map_err(|e| format!("Cannot open mem: {}", e))?;
            file.seek(SeekFrom::Start(address))
                .map_err(|e| format!("Cannot seek to 0x{:X}: {}", address, e))?;
            let mut buffer = vec![0u8; size];
            file.read_exact(&mut buffer)
                .map_err(|e| format!("Cannot read at 0x{:X}: {}", address, e))?;
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
}

// ─── Windows implementation ──────────────────────────────────────────────────
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

    // when Process is dropped, close the Windows handle automatically
    impl Drop for Process {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.handle); }
        }
    }
}

// ─── Public API — same on all platforms ──────────────────────────────────────
pub use platform::Process;