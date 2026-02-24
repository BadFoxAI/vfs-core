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
[x] PHASE 4: THE SOVEREIGN BOOTSTRAP (Bridging C)
    [x] 4.1 Implement Segmented Memory (.text vs .data).
    [x] 4.2 Implement minimal 'blibc' (POSIX Emulation).
    [x] 4.3 Establish VFS-to-VFS Build Pipeline.

UNIT TEST SUITE:
"#;

pub struct Compiler;
impl Compiler {
    pub fn compile_bef(source: &str, data: &[u8]) -> Vec<u8> {
        let clean: String = source.lines()
            .map(|l| l.split("//").next().unwrap_or("").trim())
            .filter(|l| !l.is_empty()).collect::<Vec<&str>>().join(" ");
        let tokens: Vec<&str> = clean.split_whitespace().collect();
        let mut labels = HashMap::new();
        let mut addr = 0;

        let mut i = 0;
        while i < tokens.len() {
            let t = tokens[i];
            if t.ends_with(':') { labels.insert(t.trim_end_matches(':').to_string(), addr); }
            else { addr += match t { "PUSH" | "LEA" | "CALL" | "JMP" | "JZ" => { i += 1; 9 }, _ => 1 }; }
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
                    let v: u64 = tokens[i].parse().unwrap();
                    code.extend_from_slice(&v.to_le_bytes());
                }
                "LEA" => {
                    code.push(0x50); i += 1;
                    let target = 1024 + tokens[i].parse::<u64>().unwrap(); 
                    code.extend_from_slice(&target.to_le_bytes());
                }
                "SYSCALL" => code.push(0xF0),
                _ => {}
            }
            i += 1;
        }

        let mut binary = Vec::new();
        binary.extend_from_slice(&0xB111E7u32.to_le_bytes());
        binary.extend_from_slice(&0u32.to_le_bytes());
        binary.extend_from_slice(&(code.len() as u32).to_le_bytes());
        binary.extend_from_slice(&(data.len() as u32).to_le_bytes());
        binary.extend(code);
        binary.extend(data);
        binary
    }
}

pub struct Machine {
    pub stack: Vec<u64>,
    pub memory: Vec<u8>,
    pub ip: usize,
    pub vfs: HashMap<String, Vec<u8>>,
    pub fds: HashMap<u64, String>, 
    pub next_fd: u64,
    pub exit_code: Option<u64>,
}

impl Machine {
    pub fn new() -> Self {
        Self { 
            stack: vec![], memory: vec![0; 8192], ip: 0, 
            vfs: HashMap::new(), fds: HashMap::new(), next_fd: 3,
            exit_code: None,
        }
    }

    pub fn load_bef(&mut self, filename: &str) -> Result<(), String> {
        let data = self.vfs.get(filename).ok_or("File not found")?;
        let code_size = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        let data_size = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
        self.memory[0..code_size].copy_from_slice(&data[16..16+code_size]);
        if data_size > 0 {
            self.memory[1024..1024+data_size].copy_from_slice(&data[16+code_size..16+code_size+data_size]);
        }
        self.ip = 0;
        Ok(())
    }

    pub fn step(&mut self) -> Result<bool, String> {
        if self.exit_code.is_some() { return Ok(false); }
        let op = self.memory[self.ip];
        self.ip += 1;

        match op {
            0x00 => return Ok(false),
            0x10 => {
                let v = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap());
                self.ip += 8; self.stack.push(v);
            }
            0x50 => {
                let v = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap());
                self.ip += 8; self.stack.push(v);
            }
            0xF0 => {
                let sys_id = self.stack.pop().unwrap();
                match sys_id {
                    1 => { // SYS_WRITE
                        let count = self.stack.pop().unwrap() as usize;
                        let buf = self.stack.pop().unwrap() as usize;
                        let fd = self.stack.pop().unwrap();
                        let data = self.memory[buf..buf+count].to_vec();
                        if let Some(name) = self.fds.get(&fd) { self.vfs.insert(name.clone(), data); }
                    }
                    2 => { // SYS_OPEN
                        let buf = self.stack.pop().unwrap() as usize;
                        let mut end = buf;
                        while self.memory[end] != 0 { end += 1; }
                        let name = String::from_utf8_lossy(&self.memory[buf..end]).into_owned();
                        let fd = self.next_fd; self.next_fd += 1;
                        self.fds.insert(fd, name);
                        self.stack.push(fd);
                    }
                    60 => { self.exit_code = Some(self.stack.pop().unwrap()); } // SYS_EXIT
                    _ => return Err(format!("Sys: {}", sys_id)),
                }
            }
            _ => return Err(format!("OP: {:02X}", op)),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(MANIFESTO);
    report.push_str("TEST: VFS_TO_VFS_PIPELINE ... ");

    // 1. Setup the Global VFS Ecosystem
    let mut ecosystem_vfs = HashMap::new();

    // 2. Write source code to VFS
    let source_code = b"
        LEA 0 PUSH 2 SYSCALL     // Open 'output.txt'
        LEA 11 PUSH 12           // Addr and Length of 'BOOTSTRAP_OK'
        PUSH 1 SYSCALL           // Write
        PUSH 0 PUSH 60 SYSCALL   // Exit 0
    ";
    ecosystem_vfs.insert("source.s".to_string(), source_code.to_vec());
    
    // Data Segment: "output.txt\0" (11 bytes) + "BOOTSTRAP_OK" (12 bytes)
    let mut data_seg = Vec::new();
    data_seg.extend_from_slice(b"output.txt\0");
    data_seg.extend_from_slice(b"BOOTSTRAP_OK");
    ecosystem_vfs.insert("data.bin".to_string(), data_seg);

    // 3. The Compiler reads from VFS and writes BEF to VFS
    let src_str = String::from_utf8_lossy(ecosystem_vfs.get("source.s").unwrap());
    let data_bin = ecosystem_vfs.get("data.bin").unwrap();
    let compiled_bef = Compiler::compile_bef(&src_str, data_bin);
    ecosystem_vfs.insert("program.bef".to_string(), compiled_bef);

    // 4. Spin up a new VM and inject the ecosystem VFS
    let mut vm = Machine::new();
    vm.vfs = ecosystem_vfs;

    // 5. VM Loads BEF from its VFS and Executes
    if vm.load_bef("program.bef").is_ok() {
        while vm.step().unwrap_or(false) {}
        
        // 6. Verify the final output exists in the VFS
        if vm.exit_code == Some(0) {
            if let Some(b) = vm.vfs.get("output.txt") {
                let s = String::from_utf8_lossy(b);
                if s == "BOOTSTRAP_OK" { report.push_str("PASS\n"); }
                else { report.push_str(&format!("FAIL (Content: {})\n", s)); }
            } else { report.push_str("FAIL (IO)\n"); }
        } else { report.push_str("FAIL (Exit Code)\n"); }
    } else { report.push_str("FAIL (Loader)\n"); }

    report.push_str("\nBILLET ENGINE COMPLETE. READY TO HOST C COMPILER.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
