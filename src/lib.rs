use wasm_bindgen::prelude::*;
use std::collections::HashMap;

pub const SYSTEM_STATUS: &str = r#"
================================================================================
BILLET // DETERMINISTIC EXECUTION ENVIRONMENT
================================================================================
[ ARCHITECTURE ]
Host-Independent VFS + Custom ABI + POSIX Shim.

[ BUILD LOG ]
[x] PHASE 1-4: CORE SYSTEM COMPLETE.
[~] PHASE 5: BOOTSTRAP TOOLCHAIN (Language v0)
    [~] 5.1 Implement v0 Compiler (Loop/Var Support).
    [ ] 5.2 Implement Function support.

UNIT TEST SUITE:
"#;

pub struct V0Compiler {
    locals: HashMap<String, usize>,
    local_offset: usize,
    label_count: usize,
}

impl V0Compiler {
    pub fn new() -> Self {
        Self { locals: HashMap::new(), local_offset: 0, label_count: 0 }
    }

    pub fn compile(&mut self, source: &str) -> Result<String, String> {
        let mut out = String::new();
        let tokens: Vec<&str> = source.split_whitespace().collect();
        let mut i = 0;

        while i < tokens.len() {
            match tokens[i] {
                "var" => {
                    let name = tokens[i+1];
                    let val = tokens[i+3];
                    self.locals.insert(name.to_string(), self.local_offset);
                    out.push_str(&format!("PUSH {} LSTORE {}\n", val, self.local_offset));
                    self.local_offset += 8;
                    i += 5; // var x = n ;
                }
                "while" => {
                    let start_lbl = format!("L_START_{}", self.label_count);
                    let end_lbl = format!("L_END_{}", self.label_count);
                    self.label_count += 1;

                    let lhs = tokens[i+2];
                    let rhs = tokens[i+4];
                    out.push_str(&format!("{}:\n", start_lbl));
                    out.push_str(&self.gen_load(lhs));
                    out.push_str(&self.gen_load(rhs));
                    out.push_str("LT\n");
                    out.push_str(&format!("JZ {}\n", end_lbl));
                    i += 7; // while ( a < b ) {

                    // Internal Body
                    while tokens[i] != "}" {
                        if self.locals.contains_key(tokens[i]) {
                            let name = tokens[i];
                            let off = *self.locals.get(name).unwrap();
                            out.push_str(&self.gen_load(tokens[i+2]));
                            out.push_str(&self.gen_load(tokens[i+4]));
                            out.push_str("ADD\n");
                            out.push_str(&format!("LSTORE {}\n", off));
                            i += 6; // x = x + 1 ;
                        } else if tokens[i] == "syscall" {
                            out.push_str(&self.gen_load(tokens[i+4]));
                            out.push_str(&self.gen_load(tokens[i+2]));
                            out.push_str("SYSCALL\n");
                            i += 7; // syscall ( id , val ) ;
                        } else { i += 1; }
                    }
                    out.push_str(&format!("JMP {}\n{}:\n", start_lbl, end_lbl));
                    i += 1; // }
                }
                "HALT" => { out.push_str("HALT\n"); i += 1; }
                _ => i += 1,
            }
        }
        Ok(out)
    }

    fn gen_load(&self, t: &str) -> String {
        if let Ok(n) = t.parse::<u64>() { format!("PUSH {}\n", n) }
        else { format!("LLOAD {}\n", self.locals.get(t).unwrap()) }
    }
}

pub struct Assembler;
impl Assembler {
    pub fn compile_bef(source: &str) -> Vec<u8> {
        let clean: String = source.lines().map(|l| l.split("//").next().unwrap_or("").trim()).filter(|l| !l.is_empty()).collect::<Vec<&str>>().join(" ");
        let tokens: Vec<&str> = clean.split_whitespace().collect();
        let mut labels = HashMap::new();
        let mut addr = 0;
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i].ends_with(':') { labels.insert(tokens[i].trim_end_matches(':').to_string(), addr); }
            else { addr += match tokens[i] { "PUSH"|"JMP"|"JZ"|"LLOAD"|"LSTORE" => { i+=1; 9 }, _ => 1 }; }
            i += 1;
        }
        let mut code = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            let t = tokens[i];
            if t.ends_with(':') { i += 1; continue; }
            match t {
                "HALT" => code.push(0x00),
                "PUSH" => { code.push(0x10); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "ADD" => code.push(0x20),
                "LT" => code.push(0x25),
                "JMP" => { code.push(0x40); i+=1; code.extend_from_slice(&(*labels.get(tokens[i]).unwrap() as u64).to_le_bytes()); }
                "JZ" => { code.push(0x41); i+=1; code.extend_from_slice(&(*labels.get(tokens[i]).unwrap() as u64).to_le_bytes()); }
                "LLOAD" => { code.push(0x60); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "LSTORE" => { code.push(0x61); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "SYSCALL" => code.push(0xF0),
                _ => {}
            }
            i += 1;
        }
        let mut bin = vec![];
        bin.extend_from_slice(&0xB111E7u32.to_le_bytes());
        bin.extend_from_slice(&[0u8; 12]); // Entry/Size/Data simplified
        bin[8..12].copy_from_slice(&(code.len() as u32).to_le_bytes());
        bin.extend(code);
        bin
    }
}

pub struct Machine {
    pub stack: Vec<u64>,
    pub memory: Vec<u8>,
    pub ip: usize,
    pub bp: usize,
    pub vfs: HashMap<String, Vec<u8>>,
}

impl Machine {
    pub fn new() -> Self { Self { stack: vec![], memory: vec![0; 8192], ip: 0, bp: 4096, vfs: HashMap::new() } }
    pub fn load(&mut self, data: &[u8]) {
        let size = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        self.memory[0..size].copy_from_slice(&data[16..16+size]);
        self.ip = 0;
    }
    pub fn step(&mut self) -> Result<bool, String> {
        let op = self.memory[self.ip];
        self.ip += 1;
        match op {
            0x00 => return Ok(false),
            0x10 => { let v = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()); self.ip+=8; self.stack.push(v); }
            0x20 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a+b); }
            0x25 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a<b {1} else {0}); }
            0x40 => { self.ip = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; }
            0x41 => { let t = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; if self.stack.pop().unwrap()==0 {self.ip=t} else {self.ip+=8}; }
            0x60 => { let off = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; self.ip+=8; self.stack.push(u64::from_le_bytes(self.memory[self.bp+off..self.bp+off+8].try_into().unwrap())); }
            0x61 => { let off = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; self.ip+=8; let v = self.stack.pop().unwrap(); self.memory[self.bp+off..self.bp+off+8].copy_from_slice(&v.to_le_bytes()); }
            0xF0 => { let id = self.stack.pop().unwrap(); if id == 1 { let v = self.stack.pop().unwrap(); self.vfs.insert("out.dat".into(), v.to_le_bytes().to_vec()); } }
            _ => return Err("Err".into()),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(SYSTEM_STATUS);
    report.push_str("TEST: V0_LOOP_COMPILATION ... ");

    let src = "var i = 0 ; while ( i < 5 ) { i = i + 1 ; } syscall ( 1 , i ) ; HALT";
    let mut v0 = V0Compiler::new();
    let asm = v0.compile(src).unwrap();
    let bin = Assembler::compile_bef(&asm);

    let mut vm = Machine::new();
    vm.load(&bin);
    let mut fuel = 500;
    while fuel > 0 && vm.step().unwrap_or(false) { fuel -= 1; }

    if let Some(b) = vm.vfs.get("out.dat") {
        let v = u64::from_le_bytes(b.clone().try_into().unwrap());
        if v == 5 { report.push_str("PASS\n"); }
        else { report.push_str(&format!("FAIL ({})\n", v)); }
    } else { report.push_str("FAIL (IO)\n"); }
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
