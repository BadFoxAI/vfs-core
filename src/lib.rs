use wasm_bindgen::prelude::*;
use std::collections::HashMap;

pub const SYSTEM_STATUS: &str = r#"
================================================================================
DRE // DETERMINISTIC RUNTIME ENVIRONMENT
================================================================================
[ ARCHITECTURE ]
Host-Independent VFS + Custom ABI + POSIX Shim.

[ BUILD LOG ]
[x] PHASE 1-4: CORE SYSTEM COMPLETE.
[x] PHASE 5: BOOTSTRAP TOOLCHAIN COMPLETE.
[x] PHASE 6: C COMPILER BOOTSTRAP COMPLETE.
[x] PHASE 7: SELF-HOSTING COMPLETE.
[~] PHASE 8: FINAL SYSTEM HARDENING
    [x] 8.1 Memory Safety & Bounds Checking (Segfaults).
    [ ] 8.2 Resource Quotas & Cycle Limits.

UNIT TEST SUITE:
"#;

// --- MINIMAL C COMPILER (MiniCC) ---
pub struct MiniCC {
    locals: HashMap<String, usize>,
    local_offset: usize,
    heap_offset: usize,
}

impl MiniCC {
    pub fn new() -> Self {
        Self { locals: HashMap::new(), local_offset: 0, heap_offset: 2048 }
    }

    pub fn compile(&mut self, source: &str) -> Result<String, String> {
        let mut out = String::new();
        let tokens: Vec<String> = source
            .replace("(", " ( ").replace(")", " ) ")
            .replace("{", " { ").replace("}", " } ")
            .replace(";", " ; ").replace(",", " , ")
            .replace("+", " + ").replace("=", " = ")
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i] == "#include" {
                i += 2; 
            } else if tokens[i] == "int" && i + 1 < tokens.len() {
                if tokens[i+1] == "main" {
                    i += 2; while tokens[i] != "{" { i += 1; }
                } else {
                    let name = &tokens[i+1];
                    let val = &tokens[i+3];
                    self.locals.insert(name.clone(), self.local_offset);
                    out.push_str(&format!("PUSH {} LSTORE {}\n", val, self.local_offset));
                    self.local_offset += 8;
                    while tokens[i] != ";" { i += 1; }
                }
            } else if tokens[i] == "str" {
                let name = &tokens[i+1];
                let val = &tokens[i+3];
                self.locals.insert(name.clone(), self.local_offset);
                out.push_str(&format!("PUSH {} LSTORE {}\n", self.heap_offset, self.local_offset));
                self.local_offset += 8;
                let mut curr_ptr = self.heap_offset;
                for b in val.bytes() {
                    out.push_str(&format!("PUSH {} PUSH {} STOREB\n", b, curr_ptr));
                    curr_ptr += 1;
                }
                out.push_str(&format!("PUSH 0 PUSH {} STOREB\n", curr_ptr));
                self.heap_offset = curr_ptr + 1;
                while tokens[i] != ";" { i += 1; }
            } else if tokens[i] == "poke" {
                i += 2; 
                let mut addr_expr = Vec::new();
                while tokens[i] != "," {
                    addr_expr.push(tokens[i].clone());
                    i += 1;
                }
                i += 1; 
                let val = &tokens[i];
                out.push_str(&self.gen_load(val));
                if addr_expr.len() == 1 {
                    out.push_str(&self.gen_load(&addr_expr[0]));
                } else if addr_expr.len() == 3 && addr_expr[1] == "+" {
                    out.push_str(&self.gen_load(&addr_expr[0]));
                    out.push_str(&self.gen_load(&addr_expr[2]));
                    out.push_str("ADD\n");
                }
                out.push_str("STOREB\n");
                while tokens[i] != ";" { i += 1; }
            } else if tokens[i] == "syscall" {
                i += 2;
                let id = tokens[i].clone(); i += 1;
                let mut args = Vec::new();
                while tokens[i] != ")" {
                    if tokens[i] != "," { args.push(tokens[i].clone()); }
                    i += 1;
                }
                for arg in args.iter().rev() {
                    out.push_str(&self.gen_load(arg));
                }
                out.push_str(&self.gen_load(&id));
                out.push_str("SYSCALL\n");
                while tokens[i] != ";" { i += 1; }
            } else if tokens[i] == "exec" {
                i += 2;
                let fname = &tokens[i];
                if self.locals.contains_key(fname) {
                     out.push_str(&self.gen_load(fname));
                } else {
                    out.push_str(&format!("PUSH {} \n", self.heap_offset));
                    let mut curr_ptr = self.heap_offset;
                    for b in fname.bytes() {
                        out.push_str(&format!("PUSH {} PUSH {} STOREB\n", b, curr_ptr));
                        curr_ptr += 1;
                    }
                    out.push_str(&format!("PUSH 0 PUSH {} STOREB\n", curr_ptr));
                    self.heap_offset = curr_ptr + 1;
                }
                out.push_str("PUSH 5\nSYSCALL\n");
                while tokens[i] != ";" { i += 1; }
            } else if tokens[i] == "putchar" {
                i += 2; let val = &tokens[i];
                out.push_str(&self.gen_load(val));
                out.push_str("PUSH 4\nSYSCALL\n");
                while tokens[i] != ";" { i += 1; }
            } else if tokens[i] == "return" {
                i += 1;
                let mut expr = Vec::new();
                while tokens[i] != ";" { expr.push(tokens[i].clone()); i += 1; }
                if expr.len() == 1 { out.push_str(&self.gen_load(&expr[0])); } 
                else if expr.len() == 3 && expr[1] == "+" {
                    out.push_str(&self.gen_load(&expr[0]));
                    out.push_str(&self.gen_load(&expr[2]));
                    out.push_str("ADD\n");
                }
                out.push_str("PUSH 1\nSYSCALL\nHALT\n");
            }
            i += 1;
        }
        Ok(out)
    }

    fn gen_load(&self, t: &str) -> String {
        if let Ok(n) = t.parse::<u64>() { format!("PUSH {}\n", n) }
        else { format!("LLOAD {}\n", self.locals.get(t).expect(&format!("Undefined Var: {}", t))) }
    }
}

// --- ALIGNED ASSEMBLER ---
pub struct Assembler;
impl Assembler {
    pub fn compile_bef(source: &str) -> Vec<u8> {
        let tokens: Vec<&str> = source.split_whitespace().collect();
        let mut labels = HashMap::new();
        let mut addr = 0;
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i].ends_with(':') {
                labels.insert(tokens[i].trim_end_matches(':').to_string(), addr);
            } else {
                addr += match tokens[i] {
                    "PUSH" | "JMP" | "JZ" | "LLOAD" | "LSTORE" | "CALL" => { i += 1; 9 },
                    _ => 1,
                };
            }
            i += 1;
        }
        let mut code = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "HALT" => code.push(0x00),
                "PUSH" => { code.push(0x10); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "ADD" => code.push(0x20),
                "LT" => code.push(0x25),
                "JMP" => { code.push(0x40); i+=1; code.extend_from_slice(&(*labels.get(tokens[i]).unwrap() as u64).to_le_bytes()); }
                "JZ" => { code.push(0x41); i+=1; code.extend_from_slice(&(*labels.get(tokens[i]).unwrap() as u64).to_le_bytes()); }
                "LLOAD" => { code.push(0x60); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "LSTORE" => { code.push(0x61); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "LOAD" => code.push(0x62),
                "STORE" => code.push(0x63),
                "LOADB" => code.push(0x64),
                "STOREB" => code.push(0x65),
                "CALL" => { code.push(0x80); i+=1; code.extend_from_slice(&(*labels.get(tokens[i]).unwrap() as u64).to_le_bytes()); }
                "RET" => code.push(0x81),
                "SYSCALL" => code.push(0xF0),
                t if t.ends_with(':') => {}
                _ => {}
            }
            i += 1;
        }
        let mut bin = vec![0u8; 16];
        bin[0..4].copy_from_slice(&0xB111E7u32.to_le_bytes());
        bin[8..12].copy_from_slice(&(code.len() as u32).to_le_bytes());
        bin.extend(code);
        bin
    }
}

// --- VM (HARDENED) ---
pub struct Machine {
    pub stack: Vec<u64>,
    pub call_stack: Vec<usize>,
    pub memory: Vec<u8>,
    pub ip: usize,
    pub bp: usize,
    pub vfs: HashMap<String, Vec<u8>>,
}

impl Machine {
    pub fn new() -> Self { Self { stack: vec![], call_stack: vec![], memory: vec![0; 8192], ip: 0, bp: 4096, vfs: HashMap::new() } }
    
    // Bounds Checking Helpers
    fn check_bounds(&self, addr: usize, size: usize) -> Result<(), String> {
        if addr + size > self.memory.len() {
            return Err(format!("Segmentation Fault: Access at {}, Limit {}", addr, self.memory.len()));
        }
        Ok(())
    }

    fn read_u8(&self, addr: usize) -> Result<u8, String> {
        self.check_bounds(addr, 1)?;
        Ok(self.memory[addr])
    }

    fn write_u8(&mut self, addr: usize, val: u8) -> Result<(), String> {
        self.check_bounds(addr, 1)?;
        self.memory[addr] = val;
        Ok(())
    }

    fn read_u64(&self, addr: usize) -> Result<u64, String> {
        self.check_bounds(addr, 8)?;
        Ok(u64::from_le_bytes(self.memory[addr..addr+8].try_into().unwrap()))
    }

    fn write_u64(&mut self, addr: usize, val: u64) -> Result<(), String> {
        self.check_bounds(addr, 8)?;
        self.memory[addr..addr+8].copy_from_slice(&val.to_le_bytes());
        Ok(())
    }

    pub fn load(&mut self, data: &[u8]) {
        let size = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        if size <= self.memory.len() {
            self.memory[0..size].copy_from_slice(&data[16..16+size]);
        }
    }

    pub fn step(&mut self) -> Result<bool, String> {
        let op = self.read_u8(self.ip)?;
        self.ip += 1;
        match op {
            0x00 => return Ok(false),
            0x10 => { let v = self.read_u64(self.ip)?; self.ip+=8; self.stack.push(v); }
            0x20 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a+b); }
            0x25 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a<b {1} else {0}); }
            0x40 => { self.ip = self.read_u64(self.ip)? as usize; }
            0x41 => { 
                let t = self.read_u64(self.ip)? as usize; 
                let cond = self.stack.pop().unwrap();
                if cond == 0 { self.ip = t; } else { self.ip += 8; }
            }
            0x60 => { 
                let off = self.read_u64(self.ip)? as usize; 
                self.ip += 8; 
                let val = self.read_u64(self.bp + off)?;
                self.stack.push(val);
            }
            0x61 => { 
                let off = self.read_u64(self.ip)? as usize; 
                self.ip += 8; 
                let v = self.stack.pop().unwrap();
                self.write_u64(self.bp + off, v)?;
            }
            0x62 => { let addr = self.stack.pop().unwrap() as usize; let val = self.read_u64(addr)?; self.stack.push(val); }
            0x63 => { let addr = self.stack.pop().unwrap() as usize; let val = self.stack.pop().unwrap(); self.write_u64(addr, val)?; }
            0x64 => { let addr = self.stack.pop().unwrap() as usize; let val = self.read_u8(addr)? as u64; self.stack.push(val); }
            0x65 => { let addr = self.stack.pop().unwrap() as usize; let val = self.stack.pop().unwrap() as u8; self.write_u8(addr, val)?; }
            0x80 => { let t = self.read_u64(self.ip)? as usize; self.call_stack.push(self.ip + 8); self.ip = t; }
            0x81 => { self.ip = self.call_stack.pop().unwrap(); }
            0xF0 => { 
                let id = self.stack.pop().unwrap(); 
                if id == 1 { 
                    let v = self.stack.pop().unwrap(); self.vfs.insert("out.dat".into(), v.to_le_bytes().to_vec()); 
                } else if id == 2 {
                    let fname_ptr = self.stack.pop().unwrap() as usize;
                    let buf_ptr = self.stack.pop().unwrap() as usize;
                    let mut fname = String::new(); let mut p = fname_ptr;
                    while self.read_u8(p)? != 0 { fname.push(self.read_u8(p)? as char); p += 1; }
                    if let Some(data) = self.vfs.get(&fname) {
                        for (idx, &b) in data.iter().enumerate() { self.write_u8(buf_ptr + idx, b)?; }
                    }
                } else if id == 3 {
                    let fname_ptr = self.stack.pop().unwrap() as usize;
                    let buf_ptr = self.stack.pop().unwrap() as usize;
                    let len = self.stack.pop().unwrap() as usize;
                    let mut fname = String::new(); let mut p = fname_ptr;
                    while self.read_u8(p)? != 0 { fname.push(self.read_u8(p)? as char); p += 1; }
                    let mut data = Vec::with_capacity(len);
                    for i in 0..len { data.push(self.read_u8(buf_ptr + i)?); }
                    self.vfs.insert(fname, data);
                } else if id == 4 {
                    let c = self.stack.pop().unwrap() as u8;
                    if let Some(buf) = self.vfs.get_mut("stdout.txt") { buf.push(c); } 
                    else { self.vfs.insert("stdout.txt".into(), vec![c]); }
                } else if id == 5 {
                    let fname_ptr = self.stack.pop().unwrap() as usize;
                    let mut fname = String::new(); let mut p = fname_ptr;
                    while self.read_u8(p)? != 0 { fname.push(self.read_u8(p)? as char); p += 1; }
                    if let Some(data) = self.vfs.get(&fname) {
                        let size = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
                        if size > self.memory.len() { return Err("EXEC Fail: Binary too large".into()); }
                        self.memory[0..size].copy_from_slice(&data[16..16+size]);
                        self.ip = 0; self.stack.clear(); self.call_stack.clear();
                    } else { return Err(format!("EXEC FAIL: {}", fname)); }
                }
            }
            _ => return Err("Err".into()),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(SYSTEM_STATUS);
    report.push_str("TEST: SEGFAULT_HANDLING ... ");

    // Program: poke ( 9000 , 1 ) ; -> Should trigger Segfault (Limit is 8192)
    let c_src = "
        int bad_ptr = 9000 ;
        poke ( bad_ptr , 1 ) ;
        return 0 ;
    ";

    let mut cc = MiniCC::new();
    let asm = cc.compile(c_src).unwrap();
    let bin = Assembler::compile_bef(&asm);

    let mut vm = Machine::new();
    vm.load(&bin);
    
    let mut result = Ok(false);
    let mut fuel = 1000;
    while fuel > 0 {
        result = vm.step();
        if result.is_err() || !result.clone().unwrap() { break; }
        fuel -= 1;
    }

    match result {
        Err(e) => {
            if e.contains("Segmentation Fault") { report.push_str("PASS\n"); }
            else { report.push_str(&format!("FAIL (Wrong Error: {})\n", e)); }
        },
        Ok(_) => report.push_str("FAIL (No Error)\n"),
    }
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
