use wasm_bindgen::prelude::*;
use std::collections::HashMap;

pub const MANIFESTO: &str = r#"
================================================================================
SOVEREIGN ENGINE SHOP // LIVING SPECIFICATION & ROADMAP
================================================================================
[ MISSION ]
Build a sovereign, deterministic execution substrate (VFS + ABI).

[ ROADMAP ]
[x] PHASE 1: SUBSTRATE PARITY (CLI/WASM)
[~] PHASE 2: THE LOADING DOCK (VFS + Syscalls)
    [x] 2.1 Implement VFS logic in VM.
    [x] 2.2 Implement Syscall Opcode (0xF0).
    [ ] 2.3 Port TinyCC backend to Sovereign ABI.

UNIT TEST SUITE:
"#;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Opcode { 
    Halt = 0x00, 
    Push = 0x10, 
    Add = 0x20, 
    Syscall = 0xF0 
}

pub struct Machine {
    pub stack: Vec<u64>,
    pub ip: usize,
    pub program: Vec<u8>,
    pub vfs: HashMap<String, Vec<u8>>, // The Root of Authority
}

impl Machine {
    pub fn new(program: Vec<u8>) -> Self {
        Self { 
            stack: Vec::new(), 
            ip: 0, 
            program, 
            vfs: HashMap::new() 
        }
    }

    pub fn step(&mut self) -> Result<bool, String> {
        if self.ip >= self.program.len() { return Ok(false); }
        let op = self.program[self.ip];
        self.ip += 1;

        match op {
            0x00 => return Ok(false), // Halt
            0x10 => { // Push u64
                let bytes = self.program[self.ip..self.ip+8].try_into().map_err(|_| "Segfault")?;
                self.ip += 8;
                self.stack.push(u64::from_le_bytes(bytes));
            }
            0x20 => { // Add
                let b = self.stack.pop().ok_or("Stack Underflow")?;
                let a = self.stack.pop().ok_or("Stack Underflow")?;
                self.stack.push(a + b);
            }
            0xF0 => { // Syscall
                let sys_id = self.stack.pop().ok_or("Syscall ID missing")?;
                match sys_id {
                    1 => { // SYSCALL_VFS_WRITE (addr: dummy, val: top of stack)
                        let val = self.stack.pop().ok_or("No value to write")?;
                        self.vfs.insert("out.dat".to_string(), val.to_le_bytes().to_vec());
                    }
                    _ => return Err(format!("Unknown Syscall: {}", sys_id)),
                }
            }
            _ => return Err(format!("Unknown Opcode: 0x{:02X}", op)),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(MANIFESTO);
    
    // TEST: VFS_SYSCALL
    // Program: PUSH 10, PUSH 20, ADD, PUSH 1 (SysID), SYSCALL, HALT
    report.push_str("TEST: VFS_SYSCALL_WRITE ... ");
    let program = vec![
        0x10, 10, 0, 0, 0, 0, 0, 0, 0, 
        0x10, 20, 0, 0, 0, 0, 0, 0, 0, 
        0x20, 
        0x10, 1, 0, 0, 0, 0, 0, 0, 0, // Push Syscall ID 1
        0xF0, 
        0x00
    ];
    
    let mut vm = Machine::new(program);
    while let Ok(true) = vm.step() {}
    
    // Verify that 'out.dat' in VFS contains 30
    if let Some(bytes) = vm.vfs.get("out.dat") {
        let val = u64::from_le_bytes(bytes.clone().try_into().unwrap());
        if val == 30 {
            report.push_str("PASS\n");
        } else {
            report.push_str(&format!("FAIL (Got {})\n", val));
        }
    } else {
        report.push_str("FAIL (VFS Empty)\n");
    }

    report.push_str("\nSOVEREIGN SUBSTRATE OPERATIONAL.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String {
    run_suite()
}
