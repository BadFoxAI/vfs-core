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
    [x] 2.3 First-Mover Compiler (Assembler).
    [x] 2.4 Implement Memory + Control Flow (Loops).
    [ ] 2.5 Port TinyCC backend to Sovereign ABI.

UNIT TEST SUITE:
"#;

#[repr(u8)]
pub enum Opcode { 
    Halt = 0x00, Push = 0x10, Add = 0x20, Sub = 0x21,
    Load = 0x30, Store = 0x31, 
    Jmp = 0x40, Jz = 0x41,
    Syscall = 0xF0 
}

pub struct Compiler;
impl Compiler {
    pub fn compile(source: &str) -> Result<Vec<u8>, String> {
        // Pre-process: Strip comments and handle lines
        let clean_source: String = source.lines()
            .map(|line| line.split("//").next().unwrap_or("").trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join(" ");

        let tokens: Vec<&str> = clean_source.split_whitespace().collect();
        let mut labels = HashMap::new();
        let mut addr = 0;

        // Pass 1: Resolve Labels
        let mut i = 0;
        while i < tokens.len() {
            let t = tokens[i];
            if t.ends_with(':') {
                labels.insert(t.trim_end_matches(':').to_string(), addr);
            } else {
                addr += match t {
                    "PUSH" | "JMP" | "JZ" => { i += 1; 9 },
                    _ => 1,
                };
            }
            i += 1;
        }

        // Pass 2: Emit Bytecode
        let mut bytecode = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            let t = tokens[i];
            if t.ends_with(':') { i += 1; continue; }
            match t {
                "HALT" => bytecode.push(0x00),
                "PUSH" => {
                    bytecode.push(0x10); i += 1;
                    let val: u64 = tokens[i].parse().map_err(|_| "Invalid PUSH")?;
                    bytecode.extend_from_slice(&val.to_le_bytes());
                }
                "ADD" => bytecode.push(0x20),
                "SUB" => bytecode.push(0x21),
                "LOAD" => bytecode.push(0x30),
                "STORE" => bytecode.push(0x31),
                "JMP" => {
                    bytecode.push(0x40); i += 1;
                    let target = labels.get(tokens[i]).ok_or("Label Not Found")?;
                    bytecode.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                "JZ" => {
                    bytecode.push(0x41); i += 1;
                    let target = labels.get(tokens[i]).ok_or("Label Not Found")?;
                    bytecode.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                "SYSCALL" => bytecode.push(0xF0),
                _ => return Err(format!("Invalid: {}", t)),
            }
            i += 1;
        }
        Ok(bytecode)
    }
}

pub struct Machine {
    pub stack: Vec<u64>,
    pub memory: [u64; 1024],
    pub ip: usize,
    pub program: Vec<u8>,
    pub vfs: HashMap<String, Vec<u8>>,
}

impl Machine {
    pub fn new(program: Vec<u8>) -> Self {
        Self { stack: Vec::new(), memory: [0; 1024], ip: 0, program, vfs: HashMap::new() }
    }
    pub fn step(&mut self) -> Result<bool, String> {
        if self.ip >= self.program.len() { return Ok(false); }
        let op = self.program[self.ip];
        self.ip += 1;
        match op {
            0x00 => return Ok(false), // Halt
            0x10 => { // Push
                let val = u64::from_le_bytes(self.program[self.ip..self.ip+8].try_into().unwrap());
                self.ip += 8; self.stack.push(val);
            }
            0x20 => { // Add
                let b = self.stack.pop().ok_or("U-Add")?;
                let a = self.stack.pop().ok_or("U-Add")?;
                self.stack.push(a + b);
            }
            0x21 => { // Sub
                let b = self.stack.pop().ok_or("U-Sub")?;
                let a = self.stack.pop().ok_or("U-Sub")?;
                self.stack.push(a.saturating_sub(b));
            }
            0x30 => { // Load
                let addr = self.stack.pop().ok_or("U-Load")? as usize;
                self.stack.push(self.memory[addr]);
            }
            0x31 => { // Store
                let addr = self.stack.pop().ok_or("U-Store")? as usize;
                let val = self.stack.pop().ok_or("U-Store")?;
                self.memory[addr] = val;
            }
            0x40 => { // Jmp
                let target = u64::from_le_bytes(self.program[self.ip..self.ip+8].try_into().unwrap());
                self.ip = target as usize;
            }
            0x41 => { // Jz
                let target = u64::from_le_bytes(self.program[self.ip..self.ip+8].try_into().unwrap());
                let cond = self.stack.pop().ok_or("U-Jz")?;
                if cond == 0 { self.ip = target as usize; } else { self.ip += 8; }
            }
            0xF0 => { // Syscall
                let sys_id = self.stack.pop().ok_or("U-Sys")?;
                if sys_id == 1 {
                    let val = self.stack.pop().ok_or("U-SysVal")?;
                    self.vfs.insert("out.dat".to_string(), val.to_le_bytes().to_vec());
                }
            }
            _ => return Err(format!("0x{:02X}", op)),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(MANIFESTO);
    report.push_str("TEST: LOOP_MULT_5_BY_3 ... ");

    // FIXED ASSEMBLY: Puts Counter on stack then 1 for subtraction (Counter - 1)
    let source = "
        PUSH 0 PUSH 0 STORE    // Result = 0
        PUSH 3 PUSH 1 STORE    // Counter = 3
        LOOP:
            PUSH 1 LOAD        // Get counter
            JZ END             // If counter == 0, jump to END
            PUSH 0 LOAD PUSH 5 ADD PUSH 0 STORE // Result += 5
            PUSH 1 LOAD PUSH 1 SUB PUSH 1 STORE // Counter -= 1
            JMP LOOP
        END:
            PUSH 0 LOAD PUSH 1 SYSCALL HALT
    ";
    
    match Compiler::compile(source) {
        Ok(code) => {
            let mut vm = Machine::new(code);
            let mut fuel = 1000;
            while fuel > 0 && vm.step().unwrap_or(false) { fuel -= 1; }
            
            if fuel == 0 { report.push_str("FAIL (TIMEOUT)\n"); }
            else if let Some(bytes) = vm.vfs.get("out.dat") {
                let val = u64::from_le_bytes(bytes.clone().try_into().unwrap());
                if val == 15 { report.push_str("PASS\n"); }
                else { report.push_str(&format!("FAIL (Result {})\n", val)); }
            } else { report.push_str("FAIL (VFS EMPTY)\n"); }
        }
        Err(e) => report.push_str(&format!("ERR: {}\n", e)),
    }

    report.push_str("\nSOVEREIGN SUBSTRATE OPERATIONAL.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
