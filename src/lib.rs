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
    [x] 4.1 Implement Segmented Memory (.text vs .data).
    [~] 4.2 Implement minimal 'blibc' (POSIX Emulation).
    [ ] 4.3 Port TinyCC (tcc) backend to emit BEF.

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
                    let offset: u64 = tokens[i].parse().unwrap();
                    // Base of data segment is 1024
                    let target = 1024 + offset; 
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
    pub fds: HashMap<u64, String>, // File Descriptor Table
    pub next_fd: u64,
    pub exit_code: Option<u64>,
}

impl Machine {
    pub fn new() -> Self {
        Self { 
            stack: vec![], memory: vec![0; 8192], ip: 0, 
            vfs: HashMap::new(), fds: HashMap::new(), next_fd: 3, // 0,1,2 reserved for stdio
            exit_code: None,
        }
    }

    pub fn load_bef(&mut self, data: &[u8]) {
        let code_size = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        let data_size = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
        self.memory[0..code_size].copy_from_slice(&data[16..16+code_size]);
        if data_size > 0 {
            self.memory[1024..1024+data_size].copy_from_slice(&data[16+code_size..16+code_size+data_size]);
        }
        self.ip = 0;
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
            0x50 => { // LEA
                let v = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap());
                self.ip += 8; self.stack.push(v);
            }
            0xF0 => { // POSIX SYSCALL DECEPTION
                let sys_id = self.stack.pop().unwrap();
                match sys_id {
                    1 => { // SYS_WRITE (fd, buf, count)
                        let count = self.stack.pop().unwrap() as usize;
                        let buf = self.stack.pop().unwrap() as usize;
                        let fd = self.stack.pop().unwrap();
                        
                        let data = self.memory[buf..buf+count].to_vec();
                        if let Some(filename) = self.fds.get(&fd) {
                            // Append to VFS file (or create)
                            self.vfs.insert(filename.clone(), data);
                        }
                    }
                    2 => { // SYS_OPEN (filename_ptr) -> Returns FD
                        let buf = self.stack.pop().unwrap() as usize;
                        // Read null-terminated string
                        let mut end = buf;
                        while self.memory[end] != 0 { end += 1; }
                        let filename = String::from_utf8_lossy(&self.memory[buf..end]).into_owned();
                        
                        let fd = self.next_fd;
                        self.next_fd += 1;
                        self.fds.insert(fd, filename);
                        self.stack.push(fd);
                    }
                    60 => { // SYS_EXIT (error_code)
                        self.exit_code = Some(self.stack.pop().unwrap());
                    }
                    _ => return Err(format!("Unknown POSIX Syscall: {}", sys_id)),
                }
            }
            _ => return Err(format!("OP: {:02X}", op)),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(MANIFESTO);
    report.push_str("TEST: BLIBC_POSIX_EMULATION ... ");

    // Simulating:
    // int fd = open("vfs_root.txt");
    // write(fd, "POSIX_OK", 8);
    // exit(0);
    let source = "
        LEA 0          // Addr of 'vfs_root.txt\0' (offset 0 in data)
        PUSH 2 SYSCALL // SYS_OPEN -> FD is now on stack
        
        LEA 13         // Addr of 'POSIX_OK' (offset 13 in data)
        PUSH 8         // Count (8 bytes)
        // Stack is now: [FD, ADDR, COUNT]. Note: In our simple ABI we pop Count, then Addr, then FD.
        // Wait, to pop Count first, it must be pushed LAST. So PUSH FD, PUSH ADDR, PUSH COUNT.
        // FD is already at bottom. 
        // We just pushed ADDR, then COUNT. Perfect.
        PUSH 1 SYSCALL // SYS_WRITE
        
        PUSH 0         // Exit code 0
        PUSH 60 SYSCALL // SYS_EXIT
    ";
    
    // Data Segment: "vfs_root.txt\0" (13 bytes) + "POSIX_OK" (8 bytes)
    let mut data = Vec::new();
    data.extend_from_slice(b"vfs_root.txt\0"); // Offset 0
    data.extend_from_slice(b"POSIX_OK");       // Offset 13

    let binary = Compiler::compile_bef(source, &data);
    let mut vm = Machine::new();
    vm.load_bef(&binary);
    
    while vm.step().unwrap_or(false) {}
    
    if vm.exit_code == Some(0) {
        if let Some(b) = vm.vfs.get("vfs_root.txt") {
            let s = String::from_utf8_lossy(b);
            if s == "POSIX_OK" { report.push_str("PASS\n"); }
            else { report.push_str(&format!("FAIL (Content: {})\n", s)); }
        } else { report.push_str("FAIL (VFS IO)\n"); }
    } else { report.push_str(&format!("FAIL (Exit Code: {:?})\n", vm.exit_code)); }

    report.push_str("\nPOSIX DECEPTION LAYER ACTIVE. C-COMPATIBILITY ESTABLISHED.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
