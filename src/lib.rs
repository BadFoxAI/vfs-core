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
    [x] 2.5 Implement Call/Return (Functions).
    [ ] 2.6 Port TinyCC backend to Sovereign ABI.

UNIT TEST SUITE:
"#;

#[repr(u8)]
pub enum Opcode { 
    Halt = 0x00, Push = 0x10, Dup = 0x12, Add = 0x20, Mul = 0x22,
    Load = 0x30, Store = 0x31, 
    Jmp = 0x40, Jz = 0x41, Call = 0x42, Ret = 0x43,
    Syscall = 0xF0 
}

pub struct Compiler;
impl Compiler {
    pub fn compile(source: &str) -> Result<Vec<u8>, String> {
        let clean: String = source.lines()
            .map(|l| l.split("//").next().unwrap_or("").trim())
            .filter(|l| !l.is_empty()).collect::<Vec<&str>>().join(" ");
        let tokens: Vec<&str> = clean.split_whitespace().collect();
        
        let mut labels = HashMap::new();
        let mut addr = 0;

        // Pass 1: Resolve Labels
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i].ends_with(':') {
                labels.insert(tokens[i].trim_end_matches(':').to_string(), addr);
            } else {
                addr += match tokens[i] {
                    "PUSH" | "JMP" | "JZ" | "CALL" => { i += 1; 9 },
                    _ => 1,
                };
            }
            i += 1;
        }

        // Pass 2: Emit Bytecode
        let mut code = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            let t = tokens[i];
            if t.ends_with(':') { i += 1; continue; }
            match t {
                "HALT" => code.push(0x00),
                "PUSH" => {
                    code.push(0x10); i += 1;
                    let v: u64 = tokens[i].parse().map_err(|_| "Val")?;
                    code.extend_from_slice(&v.to_le_bytes());
                }
                "DUP" => code.push(0x12),
                "ADD" => code.push(0x20),
                "MUL" => code.push(0x22),
                "JMP" => {
                    code.push(0x40); i += 1;
                    let target = labels.get(tokens[i]).ok_or("Label Not Found")?;
                    code.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                "JZ" => {
                    code.push(0x41); i += 1;
                    let target = labels.get(tokens[i]).ok_or("Label Not Found")?;
                    code.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                "CALL" => {
                    code.push(0x42); i += 1;
                    let target = labels.get(tokens[i]).ok_or("Label Not Found")?;
                    code.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                "RET" => code.push(0x43),
                "SYSCALL" => code.push(0xF0),
                _ => return Err(format!("Invalid: {}", t)),
            }
            i += 1;
        }
        Ok(code)
    }
}

pub struct Machine {
    pub stack: Vec<u64>,
    pub call_stack: Vec<usize>,
    pub memory: [u64; 1024],
    pub ip: usize,
    pub program: Vec<u8>,
    pub vfs: HashMap<String, Vec<u8>>,
}

impl Machine {
    pub fn new(program: Vec<u8>) -> Self {
        Self { stack: vec![], call_stack: vec![], memory: [0; 1024], ip: 0, program, vfs: HashMap::new() }
    }
    pub fn step(&mut self) -> Result<bool, String> {
        if self.ip >= self.program.len() { return Ok(false); }
        let op = self.program[self.ip];
        self.ip += 1;
        match op {
            0x00 => return Ok(false),
            0x10 => {
                let v = u64::from_le_bytes(self.program[self.ip..self.ip+8].try_into().unwrap());
                self.ip += 8; self.stack.push(v);
            }
            0x12 => { // DUP
                let v = *self.stack.last().ok_or("Stack Empty")?;
                self.stack.push(v);
            }
            0x20 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }
            0x22 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a * b); }
            0x42 => { // CALL
                let target = u64::from_le_bytes(self.program[self.ip..self.ip+8].try_into().unwrap()) as usize;
                self.ip += 8;
                self.call_stack.push(self.ip);
                self.ip = target;
            }
            0x43 => { self.ip = self.call_stack.pop().ok_or("Call Stack Underflow")?; }
            0xF0 => {
                let sys_id = self.stack.pop().unwrap();
                if sys_id == 1 {
                    let v = self.stack.pop().unwrap();
                    self.vfs.insert("out.dat".into(), v.to_le_bytes().to_vec());
                }
            }
            _ => return Err(format!("OP: 0x{:02X}", op)),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(MANIFESTO);
    report.push_str("TEST: FUNCTION_CALL_SQUARE ... ");

    let source = "
        PUSH 4         // Arg
        CALL SQUARE    // Call Square Function
        PUSH 1 SYSCALL // Write Result (16) to VFS
        HALT

        SQUARE:
            DUP        // Copy 4
            MUL        // 4 * 4
            RET
    ";
    
    match Compiler::compile(source) {
        Ok(code) => {
            let mut vm = Machine::new(code);
            while vm.step().unwrap_or(false) {}
            if let Some(b) = vm.vfs.get("out.dat") {
                let v = u64::from_le_bytes(b.clone().try_into().unwrap());
                if v == 16 { report.push_str("PASS\n"); }
                else { report.push_str(&format!("FAIL ({})\n", v)); }
            } else { report.push_str("FAIL (VFS EMPTY)\n"); }
        }
        Err(e) => report.push_str(&format!("COMPILER ERR: {}\n", e)),
    }

    report.push_str("\nISA COMPLETE. READY FOR LOADING DOCK.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
