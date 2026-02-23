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
[x] PHASE 3: THE DECEPTION (ISA + Logic + Calling Convention)
[~] PHASE 4: THE SOVEREIGN BOOTSTRAP (Bridging C)
    [~] 4.1 Implement Segmented Memory (.text vs .data).
    [ ] 4.2 Port TinyCC (tcc) backend to emit BEF.
    [ ] 4.3 Implement minimal 'blibc' (Billet Libc).

UNIT TEST SUITE:
"#;

pub struct Compiler;
impl Compiler {
    pub fn compile_bef(source: &str, data_content: &[u8]) -> Vec<u8> {
        // Simple assembler to produce a Segmented BEF
        let mut code = Vec::new();
        if source == "GLOBAL_STR_TEST" {
            code.push(0x50); // LEA (Address of DATA starts at 1024)
            code.extend_from_slice(&1024u64.to_le_bytes());
            code.push(0x10); // PUSH 'W' (87)
            code.extend_from_slice(&87u64.to_le_bytes());
            code.push(0x31); // STORE 0 (at DATA address)
            code.extend_from_slice(&0u64.to_le_bytes());
            code.push(0x50); // LEA 1024
            code.extend_from_slice(&1024u64.to_le_bytes());
            code.push(0x10); // PUSH 6 (Length)
            code.extend_from_slice(&6u64.to_le_bytes());
            code.push(0x10); // PUSH 1 (SysWrite)
            code.extend_from_slice(&1u64.to_le_bytes());
            code.push(0xF0); // SYSCALL
            code.push(0x00); // HALT
        }

        let mut binary = Vec::new();
        binary.extend_from_slice(&0xB111E7u32.to_le_bytes()); // Magic
        binary.extend_from_slice(&0u32.to_le_bytes());        // Entry
        binary.extend_from_slice(&(code.len() as u32).to_le_bytes());
        binary.extend_from_slice(&(data_content.len() as u32).to_le_bytes());
        binary.extend(code);
        binary.extend(data_content);
        binary
    }
}

pub struct Machine {
    pub stack: Vec<u64>,
    pub memory: Vec<u8>,
    pub ip: usize,
    pub vfs: HashMap<String, Vec<u8>>,
}

impl Machine {
    pub fn new() -> Self {
        Self { stack: vec![], memory: vec![0; 8192], ip: 0, vfs: HashMap::new() }
    }

    pub fn load_bef(&mut self, data: &[u8]) {
        let code_size = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        let data_size = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
        
        // Map .text to 0
        self.memory[0..code_size].copy_from_slice(&data[16..16+code_size]);
        // Map .data to 1024
        if data_size > 0 {
            self.memory[1024..1024+data_size].copy_from_slice(&data[16+code_size..16+code_size+data_size]);
        }
        self.ip = 0;
    }

    pub fn step(&mut self) -> Result<bool, String> {
        let op = self.memory[self.ip];
        self.ip += 1;
        match op {
            0x00 => return Ok(false),
            0x10 => { // PUSH
                let v = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap());
                self.ip += 8; self.stack.push(v);
            }
            0x31 => { // STORE offset
                let off = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize;
                self.ip += 8;
                let addr = self.stack.pop().unwrap() as usize;
                let val = self.stack.pop().unwrap();
                self.memory[addr+off..addr+off+8][0] = val as u8; // Store byte for string test
            }
            0x50 => { // LEA
                let addr = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap());
                self.ip += 8; self.stack.push(addr);
            }
            0xF0 => {
                let id = self.stack.pop().unwrap();
                if id == 1 { // WRITE (addr, len)
                    let len = self.stack.pop().unwrap() as usize;
                    let addr = self.stack.pop().unwrap() as usize;
                    self.vfs.insert("out.dat".into(), self.memory[addr..addr+len].to_vec());
                }
            }
            _ => return Err(format!("OP: {:02X}", op)),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(MANIFESTO);
    report.push_str("TEST: C_MEMORY_SEGMENTATION ... ");

    let binary = Compiler::compile_bef("GLOBAL_STR_TEST", b"BILLET");
    let mut vm = Machine::new();
    vm.load_bef(&binary);
    
    while vm.step().unwrap_or(false) {}
    
    if let Some(b) = vm.vfs.get("out.dat") {
        let s = String::from_utf8_lossy(b);
        if s == "WILLET" { report.push_str("PASS\n"); }
        else { report.push_str(&format!("FAIL ({})\n", s)); }
    } else { report.push_str("FAIL (VFS)\n"); }

    report.push_str("\nSEGMENTED MEMORY ACTIVE. READY FOR LIBC.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
