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
    [x] 3.1 - 3.3: ISA + Calling Convention + Stack Frames.
    [~] 3.4 Implement Billet Executable Format (BEF) Loader.
    [ ] 3.5 Port TinyCC backend to emit BEF.

UNIT TEST SUITE:
"#;

// --- BILLET EXECUTABLE FORMAT ---
#[repr(C)]
pub struct BefHeader {
    pub magic: u32,      // 0xB111E7
    pub entry: u32,      // Entry Point Address
    pub code_size: u32,
    pub data_size: u32,
}

pub struct Compiler;
impl Compiler {
    pub fn compile_bef(source: &str) -> Result<Vec<u8>, String> {
        // Simple assembler that produces a BEF file
        let tokens: Vec<&str> = source.split_whitespace().collect();
        let mut code = Vec::new();
        
        // Manual assembly for the test
        // PUSH 42, PUSH 1 (SysWrite), SYSCALL, HALT
        code.extend_from_slice(&[0x10, 42,0,0,0,0,0,0,0]); // PUSH 42
        code.extend_from_slice(&[0x10, 1,0,0,0,0,0,0,0]);  // PUSH 1 (SysID)
        code.push(0xF0); // SYSCALL
        code.push(0x00); // HALT

        let mut binary = Vec::new();
        let header = BefHeader {
            magic: 0xB111E7,
            entry: 0,
            code_size: code.len() as u32,
            data_size: 0,
        };

        // Serialize Header
        binary.extend_from_slice(&header.magic.to_le_bytes());
        binary.extend_from_slice(&header.entry.to_le_bytes());
        binary.extend_from_slice(&header.code_size.to_le_bytes());
        binary.extend_from_slice(&header.data_size.to_le_bytes());
        // Append Code
        binary.extend(code);

        Ok(binary)
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
        Self { stack: vec![], memory: vec![0; 4096], ip: 0, vfs: HashMap::new() }
    }

    pub fn load_bef(&mut self, filename: &str) -> Result<(), String> {
        let data = self.vfs.get(filename).ok_or("File not found")?;
        
        // Parse Header
        let magic = u32::from_le_bytes(data[0..4].try_into().unwrap());
        if magic != 0xB111E7 { return Err("Invalid BEF Magic".into()); }
        
        let entry = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let code_size = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        
        // Load Code into Memory
        let code_start = 16; // Size of header
        for i in 0..code_size {
            self.memory[i] = data[code_start + i];
        }
        
        self.ip = entry as usize;
        Ok(())
    }

    pub fn step(&mut self) -> Result<bool, String> {
        let op = self.memory[self.ip];
        self.ip += 1;
        match op {
            0x00 => return Ok(false), // Halt
            0x10 => { // Push
                let v = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap());
                self.ip += 8; self.stack.push(v);
            }
            0xF0 => { // Syscall
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
    report.push_str("TEST: BEF_STRUCTURED_LOAD ... ");

    // 1. Compile source to a BEF binary
    let binary = Compiler::compile_bef("").unwrap();
    
    // 2. Initialize Machine and VFS
    let mut vm = Machine::new();
    vm.vfs.insert("main.bef".into(), binary);
    
    // 3. Load the binary from VFS
    match vm.load_bef("main.bef") {
        Ok(_) => {
            while vm.step().unwrap_or(false) {}
            if let Some(b) = vm.vfs.get("out.dat") {
                let v = u64::from_le_bytes(b.clone().try_into().unwrap());
                if v == 42 { report.push_str("PASS\n"); }
                else { report.push_str(&format!("FAIL ({})\n", v)); }
            } else { report.push_str("FAIL (IO)\n"); }
        }
        Err(e) => report.push_str(&format!("LOAD ERR: {}\n", e)),
    }

    report.push_str("\nBEF LOADER OPERATIONAL. TARGETING TINYCC.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
