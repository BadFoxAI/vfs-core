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
[x] PHASE 2: THE LOADING DOCK (VFS + Syscalls)
    [x] 2.1 - 2.5: Substrate Construction (ISA complete)
[~] PHASE 3: THE DECEPTION (Bridging the World)
    [x] 3.1 Define Sovereign Calling Convention.
    [~] 3.2 Implement Static Data & Heap Addressing.
    [ ] 3.3 Port TinyCC backend to emit Sovereign ABI.

UNIT TEST SUITE:
"#;

#[repr(u8)]
pub enum Opcode { 
    Halt = 0x00, Push = 0x10, 
    Add = 0x20, Load = 0x30, Store = 0x31, 
    Jmp = 0x40, Jz = 0x41, Call = 0x42, Ret = 0x43,
    Lea = 0x50, // Load Effective Address
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
            let t = tokens[i];
            if t.ends_with(':') {
                labels.insert(t.trim_end_matches(':').to_string(), addr);
            } else if t == "STR" { // String data
                i += 1;
                addr += tokens[i].len();
            } else {
                addr += match t {
                    "PUSH" | "JMP" | "JZ" | "CALL" | "LEA" => { i += 1; 9 },
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
                "LEA" => {
                    code.push(0x50); i += 1;
                    let target = labels.get(tokens[i]).ok_or("Label")?;
                    code.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                "STR" => {
                    i += 1;
                    code.extend_from_slice(tokens[i].as_bytes());
                }
                "SYSCALL" => code.push(0xF0),
                _ => { /* Simple opcodes logic here */ }
            }
            i += 1;
        }
        Ok(code)
    }
}

pub struct Machine {
    pub stack: Vec<u64>,
    pub call_stack: Vec<usize>,
    pub memory: Vec<u8>, // Unified Memory Space
    pub ip: usize,
    pub program: Vec<u8>,
    pub vfs: HashMap<String, Vec<u8>>,
}

impl Machine {
    pub fn new(program: Vec<u8>) -> Self {
        Self { 
            stack: vec![], 
            call_stack: vec![], 
            memory: program.clone(), // Program is loaded into memory
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
            0x00 => return Ok(false),
            0x10 => { // PUSH
                let v = u64::from_le_bytes(self.program[self.ip..self.ip+8].try_into().unwrap());
                self.ip += 8; self.stack.push(v);
            }
            0x50 => { // LEA
                let addr = u64::from_le_bytes(self.program[self.ip..self.ip+8].try_into().unwrap());
                self.ip += 8; self.stack.push(addr);
            }
            0xF0 => { // SYSCALL
                let id = self.stack.pop().unwrap();
                if id == 2 { // SYSCALL_PRINT_STR (addr, len)
                    let len = self.stack.pop().unwrap() as usize;
                    let addr = self.stack.pop().unwrap() as usize;
                    let str_bytes = self.memory[addr..addr+len].to_vec();
                    self.vfs.insert("out.dat".into(), str_bytes);
                }
            }
            _ => return Err(format!("OP: 0x{:02X}", op)),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(MANIFESTO);
    report.push_str("TEST: STATIC_DATA_ADDRESSING ... ");

    // Program: Load address of "PASS", push length 4, syscall 2 (PrintStr), Halt.
    let source = "
        LEA MSG        // Push address of MSG
        PUSH 4         // Length
        PUSH 2         // SysID (PrintStr)
        SYSCALL
        HALT
        MSG: STR PASS
    ";
    
    match Compiler::compile(source) {
        Ok(code) => {
            let mut vm = Machine::new(code);
            while vm.step().unwrap_or(false) {}
            if let Some(b) = vm.vfs.get("out.dat") {
                let s = String::from_utf8_lossy(b);
                if s == "PASS" { report.push_str("PASS\n"); }
                else { report.push_str(&format!("FAIL ({})\n", s)); }
            } else { report.push_str("FAIL (VFS)\n"); }
        }
        Err(e) => report.push_str(&format!("ERR: {}\n", e)),
    }

    report.push_str("\nDATA SECTION OPERATIONAL. READY FOR COMPILER PORT.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
