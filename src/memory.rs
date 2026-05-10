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

    // read a u32 from a memory address
    pub fn read_u32(&self, address: u64) -> Result<u32, String> {
        let bytes = self.read_bytes(address, 4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    // read a u64 from a memory address
    pub fn read_u64(&self, address: u64) -> Result<u64, String> {
        let bytes = self.read_bytes(address, 8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}