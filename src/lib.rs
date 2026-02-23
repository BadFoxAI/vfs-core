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
    [~] 2.3 First-Mover Compiler (Assembler).
    [ ] 2.4 Port TinyCC backend to Sovereign ABI.

UNIT TEST SUITE:
"#;

#[repr(u8)]
pub enum Opcode { Halt = 0x00, Push = 0x10, Add = 0x20, Syscall = 0xF0 }

// --- THE FIRST MOVER COMPILER ---
pub struct Compiler;
impl Compiler {
    pub fn compile(source: &str) -> Result<Vec<u8>, String> {
        let mut bytecode = Vec::new();
        let tokens: Vec<&str> = source.split_whitespace().collect();
        let mut i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "HALT" => bytecode.push(0x00),
                "PUSH" => {
                    bytecode.push(0x10);
                    i += 1;
                    let val: u64 = tokens[i].parse().map_err(|_| "Invalid PUSH value")?;
                    bytecode.extend_from_slice(&val.to_le_bytes());
                }
                "ADD" => bytecode.push(0x20),
                "SYSCALL" => bytecode.push(0xF0),
                _ => return Err(format!("Unknown Token: {}", tokens[i])),
            }
            i += 1;
        }
        Ok(bytecode)
    }
}

// --- THE SOVEREIGN VM ---
pub struct Machine {
    pub stack: Vec<u64>,
    pub ip: usize,
    pub program: Vec<u8>,
    pub vfs: HashMap<String, Vec<u8>>,
}

impl Machine {
    pub fn new(program: Vec<u8>) -> Self {
        Self { stack: Vec::new(), ip: 0, program, vfs: HashMap::new() }
    }
    pub fn step(&mut self) -> Result<bool, String> {
        if self.ip >= self.program.len() { return Ok(false); }
        let op = self.program[self.ip];
        self.ip += 1;
        match op {
            0x00 => return Ok(false),
            0x10 => {
                let bytes = self.program[self.ip..self.ip+8].try_into().map_err(|_| "Segfault")?;
                self.ip += 8;
                self.stack.push(u64::from_le_bytes(bytes));
            }
            0x20 => {
                let b = self.stack.pop().ok_or("Stack Underflow")?;
                let a = self.stack.pop().ok_or("Stack Underflow")?;
                self.stack.push(a + b);
            }
            0xF0 => {
                let sys_id = self.stack.pop().ok_or("Syscall ID missing")?;
                match sys_id {
                    1 => {
                        let val = self.stack.pop().ok_or("No value")?;
                        self.vfs.insert("out.dat".to_string(), val.to_le_bytes().to_vec());
                    }
                    _ => return Err("Unknown Syscall".into()),
                }
            }
            _ => return Err(format!("Unknown Opcode: 0x{:02X}", op)),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(MANIFESTO);
    
    report.push_str("TEST: FIRST_MOVER_COMPILER ... ");
    let source = "PUSH 10 PUSH 20 ADD PUSH 1 SYSCALL HALT";
    
    match Compiler::compile(source) {
        Ok(bytecode) => {
            let mut vm = Machine::new(bytecode);
            while let Ok(true) = vm.step() {}
            if let Some(bytes) = vm.vfs.get("out.dat") {
                let val = u64::from_le_bytes(bytes.clone().try_into().unwrap());
                if val == 30 { report.push_str("PASS\n"); }
                else { report.push_str("FAIL (Math)\n"); }
            } else { report.push_str("FAIL (VFS)\n"); }
        }
        Err(e) => report.push_str(&format!("FAIL (Compiler: {})\n", e)),
    }

    report.push_str("\nSOVEREIGN COMPILATION ACTIVE.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
