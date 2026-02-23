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

pub struct Compiler;
impl Compiler {
    pub fn compile(source: &str) -> Result<Vec<u8>, String> {
        let clean: String = source.lines()
            .map(|l| l.split("//").next().unwrap_or(""))
            .collect::<Vec<&str>>().join(" ");
        let tokens: Vec<&str> = clean.split_whitespace().collect();

        let mut labels = HashMap::new();
        let mut addr = 0;

        // Pass 1: Address calculation
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

        // Pass 2: Bytecode generation
        let mut code = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            let t = tokens[i];
            if t.ends_with(':') { i += 1; continue; }
            match t {
                "HALT" => code.push(0x00),
                "PUSH" => {
                    code.push(0x10); i += 1;
                    let v: u64 = tokens[i].parse().map_err(|_| "PushVal")?;
                    code.extend_from_slice(&v.to_le_bytes());
                }
                "ADD" => code.push(0x20),
                "SUB" => code.push(0x21),
                "LOAD" => code.push(0x30),
                "STORE" => code.push(0x31),
                "JMP" => {
                    code.push(0x40); i += 1;
                    let target = labels.get(tokens[i]).ok_or("NoLabel")?;
                    code.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                "JZ" => {
                    code.push(0x41); i += 1;
                    let target = labels.get(tokens[i]).ok_or("NoLabel")?;
                    code.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                "SYSCALL" => code.push(0xF0),
                _ => return Err(format!("Token: {}", t)),
            }
            i += 1;
        }
        Ok(code)
    }
}

pub struct Machine {
    pub stack: Vec<u64>,
    pub mem: [u64; 1024],
    pub ip: usize,
    pub prog: Vec<u8>,
    pub vfs: HashMap<String, Vec<u8>>,
}

impl Machine {
    pub fn new(prog: Vec<u8>) -> Self {
        Self { stack: vec![], mem: [0; 1024], ip: 0, prog, vfs: HashMap::new() }
    }
    pub fn step(&mut self) -> Result<bool, String> {
        if self.ip >= self.prog.len() { return Ok(false); }
        let op = self.prog[self.ip];
        self.ip += 1;
        match op {
            0x00 => return Ok(false),
            0x10 => { // PUSH
                let v = u64::from_le_bytes(self.prog[self.ip..self.ip+8].try_into().unwrap());
                self.ip += 8; self.stack.push(v);
            }
            0x20 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a + b); }
            0x21 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a.saturating_sub(b)); }
            0x30 => { let a = self.stack.pop().unwrap() as usize; self.stack.push(self.mem[a]); }
            0x31 => { let a = self.stack.pop().unwrap() as usize; let v = self.stack.pop().unwrap(); self.mem[a] = v; }
            0x40 => { // JMP
                self.ip = u64::from_le_bytes(self.prog[self.ip..self.ip+8].try_into().unwrap()) as usize;
            }
            0x41 => { // JZ
                let target = u64::from_le_bytes(self.prog[self.ip..self.ip+8].try_into().unwrap()) as usize;
                if self.stack.pop().unwrap() == 0 { self.ip = target; } else { self.ip += 8; }
            }
            0xF0 => {
                if self.stack.pop().unwrap() == 1 {
                    let v = self.stack.pop().unwrap();
                    self.vfs.insert("out.dat".into(), v.to_le_bytes().to_vec());
                }
            }
            _ => return Err(format!("OP: {:02X}", op)),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(MANIFESTO);
    report.push_str("TEST: LOOP_MULT_5_BY_3 ... ");

    let src = "
        PUSH 0 PUSH 0 STORE
        PUSH 3 PUSH 1 STORE
        LOOP:
            PUSH 1 LOAD JZ END
            PUSH 5 PUSH 0 LOAD ADD PUSH 0 STORE
            PUSH 1 PUSH 1 LOAD SUB PUSH 1 STORE
            JMP LOOP
        END:
            PUSH 0 LOAD PUSH 1 SYSCALL HALT
    ";

    match Compiler::compile(src) {
        Ok(code) => {
            let mut vm = Machine::new(code);
            let mut fuel = 1000;
            while fuel > 0 && vm.step().unwrap_or(false) { fuel -= 1; }
            
            if fuel == 0 { report.push_str("TIMEOUT\n"); }
            else if let Some(b) = vm.vfs.get("out.dat") {
                let v = u64::from_le_bytes(b.clone().try_into().unwrap());
                if v == 15 { report.push_str("PASS\n"); }
                else { report.push_str(&format!("FAIL ({})\n", v)); }
            } else { report.push_str("FAIL (VFS)\n"); }
        }
        Err(e) => report.push_str(&format!("ERR: {}\n", e)),
    }
    report.push_str("\nSOVEREIGN SUBSTRATE OPERATIONAL.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
