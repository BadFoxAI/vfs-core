use crate::isa::Opcode;

#[derive(Debug)]
pub enum VMStatus {
    Running,
    Halted,
    Syscall(u64), // The VM pauses and asks the Shell for help
}

pub struct Machine {
    pub stack: Vec<u64>,
    pub ip: usize,          // Instruction Pointer
    pub program: Vec<u8>,   // The Bytecode
}

impl Machine {
    pub fn new(program: Vec<u8>) -> Self {
        Self {
            stack: Vec::with_capacity(1024),
            ip: 0,
            program,
        }
    }

    // The Heartbeat: Execute one instruction
    pub fn step(&mut self) -> Result<VMStatus, String> {
        if self.ip >= self.program.len() {
            return Ok(VMStatus::Halted);
        }

        // Fetch
        let op_byte = self.program[self.ip];
        self.ip += 1;

        // Decode & Execute
        // We unsafe transmute for speed, but for now lets match specifically
        match op_byte {
            0x00 => Ok(VMStatus::Halted), // Halt
            0x01 => Ok(VMStatus::Running), // NoOp

            // --- Stack ---
            0x10 => { // Push (read next 8 bytes as u64)
                if self.ip + 8 > self.program.len() { return Err("Segfault: Push".into()); }
                let bytes: [u8; 8] = self.program[self.ip..self.ip+8].try_into().unwrap();
                self.ip += 8;
                let val = u64::from_le_bytes(bytes);
                self.stack.push(val);
                Ok(VMStatus::Running)
            },
            
            // --- Arithmetic ---
            0x20 => { // Add
                let b = self.stack.pop().ok_or("Stack Underflow")?;
                let a = self.stack.pop().ok_or("Stack Underflow")?;
                self.stack.push(a + b);
                Ok(VMStatus::Running)
            },

            // --- System ---
            0xF0 => { // Syscall
                // The argument for the syscall is on the stack
                let sys_id = self.stack.pop().ok_or("Stack Underflow (Syscall)")?;
                Ok(VMStatus::Syscall(sys_id))
            }

            _ => Err(format!("Unknown Opcode: 0x{:02X}", op_byte)),
        }
    }
}
