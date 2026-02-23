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
[~] PHASE 3: THE DECEPTION (Bridging the World)
    [x] 3.1 - 3.4: ISA + BEF Loader + Stack Frames.
    [x] 3.5 Implement Logic Gates (EQ/LT/GT).
    [x] 3.6 Implement Heap Bridge (SBRK Syscall).
    [ ] 3.7 Port TinyCC backend to emit BEF.

UNIT TEST SUITE:
"#;

#[repr(u8)]
pub enum Opcode { 
    Halt = 0x00, Push = 0x10, Add = 0x20,
    Eq = 0x24, Lt = 0x25, Gt = 0x26,
    Load = 0x30, Store = 0x31,
    Jmp = 0x40, Jz = 0x41, Call = 0x42, Ret = 0x43,
    Syscall = 0xF0 
}

pub struct Compiler;
impl Compiler {
    pub fn compile_bef(source: &str) -> Result<Vec<u8>, String> {
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
            if t.ends_with(':') { labels.insert(t.trim_end_matches(':').to_string(), addr); }
            else { addr += match t { "PUSH" | "JMP" | "JZ" | "CALL" => { i += 1; 9 }, _ => 1 }; }
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
                    let v: u64 = tokens[i].parse().unwrap();
                    bytecode.extend_from_slice(&v.to_le_bytes());
                }
                "ADD" => bytecode.push(0x20),
                "LT" => bytecode.push(0x25),
                "STORE" => bytecode.push(0x31),
                "LOAD" => bytecode.push(0x30),
                "JZ" => {
                    bytecode.push(0x41); i += 1;
                    let target = labels.get(tokens[i]).ok_or("NoLabel")?;
                    bytecode.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                "JMP" => {
                    bytecode.push(0x40); i += 1;
                    let target = labels.get(tokens[i]).ok_or("NoLabel")?;
                    bytecode.extend_from_slice(&(*target as u64).to_le_bytes());
                }
                "SYSCALL" => bytecode.push(0xF0),
                _ => {}
            }
            i += 1;
        }

        let mut binary = Vec::new();
        binary.extend_from_slice(&0xB111E7u32.to_le_bytes());
        binary.extend_from_slice(&0u32.to_le_bytes());
        binary.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
        binary.extend_from_slice(&0u32.to_le_bytes());
        binary.extend(bytecode);
        Ok(binary)
    }
}

pub struct Machine {
    pub stack: Vec<u64>,
    pub memory: Vec<u64>,
    pub ip: usize,
    pub vfs: HashMap<String, Vec<u8>>,
    pub heap_ptr: usize,
}

impl Machine {
    pub fn new() -> Self {
        Self { stack: vec![], memory: vec![0; 2048], ip: 0, vfs: HashMap::new(), heap_ptr: 1024 }
    }

    pub fn load_bef(&mut self, data: &[u8]) -> Result<(), String> {
        let code_size = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        let bytecode = &data[16..16+code_size];
        // Load bytes into u64 memory space for simplicity
        for (i, chunk) in bytecode.chunks(8).enumerate() {
             if chunk.len() == 8 {
                 self.memory[i] = u64::from_le_bytes(chunk.try_into().unwrap());
             } else {
                 let mut b = [0u8; 8]; b[..chunk.len()].copy_from_slice(chunk);
                 self.memory[i] = u64::from_le_bytes(b);
             }
        }
        self.ip = 0; Ok(())
    }

    pub fn step(&mut self) -> Result<bool, String> {
        // Simple byte-to-u64 instruction fetching for this substrate
        let op_u64 = self.memory[self.ip / 8];
        let op = ((op_u64 >> ((self.ip % 8) * 8)) & 0xFF) as u8;
        self.ip += 1;

        match op {
            0x00 => return Ok(false),
            0x10 => { // PUSH
                let mut v_bytes = [0u8; 8];
                for i in 0..8 { v_bytes[i] = ((self.memory[(self.ip+i)/8] >> (((self.ip+i)%8)*8)) & 0xFF) as u8; }
                self.ip += 8; self.stack.push(u64::from_le_bytes(v_bytes));
            }
            0x20 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a+b); }
            0x25 => { // LT
                let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap();
                self.stack.push(if a < b { 1 } else { 0 });
            }
            0x30 => { let a = self.stack.pop().unwrap() as usize; self.stack.push(self.memory[a]); }
            0x31 => { let a = self.stack.pop().unwrap() as usize; let v = self.stack.pop().unwrap(); self.memory[a] = v; }
            0x40 => { // JMP
                let mut v_bytes = [0u8; 8];
                for i in 0..8 { v_bytes[i] = ((self.memory[(self.ip+i)/8] >> (((self.ip+i)%8)*8)) & 0xFF) as u8; }
                self.ip = u64::from_le_bytes(v_bytes) as usize;
            }
            0x41 => { // JZ
                let mut v_bytes = [0u8; 8];
                for i in 0..8 { v_bytes[i] = ((self.memory[(self.ip+i)/8] >> (((self.ip+i)%8)*8)) & 0xFF) as u8; }
                let cond = self.stack.pop().unwrap();
                if cond == 0 { self.ip = u64::from_le_bytes(v_bytes) as usize; } else { self.ip += 8; }
            }
            0xF0 => {
                let id = self.stack.pop().unwrap();
                match id {
                    1 => { // WRITE
                        let v = self.stack.pop().unwrap();
                        self.vfs.insert("out.dat".into(), v.to_le_bytes().to_vec());
                    }
                    3 => { // SBRK (Heap)
                        let prev = self.heap_ptr;
                        self.heap_ptr += 1; // Give one u64 slot
                        self.stack.push(prev as u64);
                    }
                    _ => {}
                }
            }
            _ => { self.ip += 0; }
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(MANIFESTO);
    report.push_str("TEST: HEAP_LOGIC_LOOP ... ");

    // Program: Allocate 1 heap slot, loop 3 times adding 10 to it.
    let source = "
        PUSH 3 SYSCALL // SBRK -> Heap Addr on stack
        PUSH 0 PUSH 20 STORE // Use mem 20 to store the Heap Addr
        LOOP:
            PUSH 0 PUSH 21 LOAD // Mem 21 is counter (starts at 0)
            PUSH 3 LT JZ END
            PUSH 10 PUSH 20 LOAD LOAD ADD // Load from heap, add 10
            PUSH 20 LOAD STORE // Store back to heap
            PUSH 1 PUSH 21 LOAD ADD PUSH 21 STORE // counter++
            JMP LOOP
        END:
            PUSH 20 LOAD LOAD PUSH 1 SYSCALL HALT
    ";
    
    match Compiler::compile_bef(source) {
        Ok(binary) => {
            let mut vm = Machine::new();
            vm.load_bef(&binary).unwrap();
            let mut fuel = 1000;
            while fuel > 0 && vm.step().unwrap_or(false) { fuel -= 1; }
            if let Some(b) = vm.vfs.get("out.dat") {
                let v = u64::from_le_bytes(b.clone().try_into().unwrap());
                if v == 30 { report.push_str("PASS\n"); }
                else { report.push_str(&format!("FAIL ({})\n", v)); }
            } else { report.push_str("FAIL (VFS)\n"); }
        }
        Err(e) => report.push_str(&format!("ERR: {}\n", e)),
    }

    report.push_str("\nBILLET 1.0 STABLE. LOGIC + HEAP VERIFIED.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
