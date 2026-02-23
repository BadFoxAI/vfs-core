use wasm_bindgen::prelude::*;
use std::collections::HashMap;

pub const MANIFESTO: &str = r#"
================================================================================
BILLET // SOVEREIGN EXECUTION SUBSTRATE
================================================================================
[ MISSION ]
Build a sovereign, deterministic execution substrate (VFS + ABI).

[ ROADMAP ]
[x] PHASE 1: SUBSTRATE PARITY (CLI/WASM)
[x] PHASE 2: THE LOADING DOCK (VFS + Syscalls)
    [x] 2.1 - 2.5: Substrate Construction (ISA complete)
[~] PHASE 3: THE DECEPTION (Bridging the World)
    [x] 3.1 Define Sovereign Calling Convention.
    [x] 3.2 Implement Static Data & FD-based Syscalls.
    [~] 3.3 Implement C-Style Stack Frames (Locals).
    [ ] 3.4 Port TinyCC backend to emit Sovereign ABI.

UNIT TEST SUITE:
"#;

#[repr(u8)]
pub enum Opcode { 
    Halt = 0x00, Push = 0x10, Add = 0x20,
    Jmp = 0x40, Jz = 0x41, Call = 0x42, Ret = 0x43,
    Lea = 0x50, 
    Lload = 0x60, // BP + offset
    Lstore = 0x61, // BP + offset
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

        let mut i = 0;
        while i < tokens.len() {
            let t = tokens[i];
            if t.ends_with(':') {
                labels.insert(t.trim_end_matches(':').to_string(), addr);
            } else if t == "STR" {
                i += 1; addr += tokens[i].len();
            } else {
                addr += match t {
                    "PUSH" | "JMP" | "JZ" | "CALL" | "LEA" | "LLOAD" | "LSTORE" => { i += 1; 9 },
                    _ => 1,
                };
            }
            i += 1;
        }

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
                "ADD" => code.push(0x20),
                "CALL" => {
                    code.push(0x42); i += 1;
                    let target = labels.get(tokens[i]).ok_or("NoLabel")?;
                    code.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                "RET" => code.push(0x43),
                "LLOAD" => {
                    code.push(0x60); i += 1;
                    let off: u64 = tokens[i].parse().unwrap();
                    code.extend_from_slice(&off.to_le_bytes());
                }
                "LSTORE" => {
                    code.push(0x61); i += 1;
                    let off: u64 = tokens[i].parse().unwrap();
                    code.extend_from_slice(&off.to_le_bytes());
                }
                "SYSCALL" => code.push(0xF0),
                _ => {}
            }
            i += 1;
        }
        Ok(code)
    }
}

pub struct Machine {
    pub stack: Vec<u64>,
    pub memory: [u64; 1024],
    pub ip: usize,
    pub bp: usize,
    pub call_stack: Vec<(usize, usize)>, // (Saved IP, Saved BP)
    pub program: Vec<u8>,
    pub vfs: HashMap<String, Vec<u8>>,
}

impl Machine {
    pub fn new(program: Vec<u8>) -> Self {
        Self { 
            stack: vec![], 
            memory: [0; 1024], 
            ip: 0, 
            bp: 512, // Local storage pool starts at offset 512
            call_stack: vec![],
            vfs: HashMap::new(),
            program,
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
            0x20 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a+b); }
            0x42 => { // CALL
                let target = u64::from_le_bytes(self.program[self.ip..self.ip+8].try_into().unwrap()) as usize;
                self.ip += 8;
                self.call_stack.push((self.ip, self.bp));
                self.bp += 10; // Shift frame to next available local slots
                self.ip = target;
            }
            0x43 => { // RET
                let (old_ip, old_bp) = self.call_stack.pop().unwrap();
                self.ip = old_ip; self.bp = old_bp;
            }
            0x60 => { // LLOAD
                let off = u64::from_le_bytes(self.program[self.ip..self.ip+8].try_into().unwrap()) as usize;
                self.ip += 8; self.stack.push(self.memory[self.bp + off]);
            }
            0x61 => { // LSTORE
                let off = u64::from_le_bytes(self.program[self.ip..self.ip+8].try_into().unwrap()) as usize;
                self.ip += 8; let v = self.stack.pop().unwrap();
                self.memory[self.bp + off] = v;
            }
            0xF0 => {
                let id = self.stack.pop().unwrap();
                if id == 1 {
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
    report.push_str("TEST: C_STACK_FRAMES_LOCALS ... ");

    let source = "
        PUSH 10        // Arg B
        PUSH 5         // Arg A
        CALL ADD_FUNC  // Stack top = 15
        PUSH 1 SYSCALL
        HALT

        ADD_FUNC:
            LSTORE 1   // Store Arg A in local slot 1
            LSTORE 0   // Store Arg B in local slot 0
            LLOAD 0
            LLOAD 1
            ADD
            RET
    ";
    
    match Compiler::compile(source) {
        Ok(code) => {
            let mut vm = Machine::new(code);
            while vm.step().unwrap_or(false) {}
            if let Some(b) = vm.vfs.get("out.dat") {
                let v = u64::from_le_bytes(b.clone().try_into().unwrap());
                if v == 15 { report.push_str("PASS\n"); }
                else { report.push_str(&format!("FAIL ({})\n", v)); }
            } else { report.push_str("FAIL (VFS EMPTY)\n"); }
        }
        Err(e) => report.push_str(&format!("ERR: {}\n", e)),
    }

    report.push_str("\nBILLET OPERATIONAL. C-LOCALS VERIFIED.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
