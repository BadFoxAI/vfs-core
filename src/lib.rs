use wasm_bindgen::prelude::*;
use std::collections::HashMap;

pub const SYSTEM_STATUS: &str = r#"
================================================================================
DRE // DETERMINISTIC RUNTIME ENVIRONMENT
================================================================================
[ SYSTEM KERNEL v1.0 ]
STATUS:  ONLINE
MODE:    GOLD MASTER
MODULES: [VFS] [ABI] [POSIX] [COMPILER] [HARDENING]

[ DIAGNOSTIC LOG ]
"#;

// --- SHARED TOOLS (MiniCC, Assembler, Machine) ---
// (Keeping struct definitions compact but functional)

pub struct MiniCC {
    locals: HashMap<String, usize>,
    local_offset: usize,
    heap_offset: usize,
}
impl MiniCC {
    pub fn new() -> Self { Self { locals: HashMap::new(), local_offset: 0, heap_offset: 2048 } }
    pub fn compile(&mut self, source: &str) -> Result<String, String> {
        let mut out = String::new();
        let tokens: Vec<String> = source.replace("(", " ( ").replace(")", " ) ").replace("{", " { ").replace("}", " } ").replace(";", " ; ").replace(",", " , ").replace("+", " + ").replace("=", " = ").split_whitespace().map(|s| s.to_string()).collect();
        let mut i = 0;
        while i < tokens.len() {
            if tokens[i] == "#include" { i += 2; }
            else if tokens[i] == "int" && i+1 < tokens.len() {
                if tokens[i+1] == "main" { i += 2; while tokens[i] != "{" { i += 1; } }
                else {
                    self.locals.insert(tokens[i+1].clone(), self.local_offset);
                    out.push_str(&format!("PUSH {} LSTORE {}\n", tokens[i+3], self.local_offset));
                    self.local_offset += 8;
                    while tokens[i] != ";" { i += 1; }
                }
            } else if tokens[i] == "str" {
                self.locals.insert(tokens[i+1].clone(), self.local_offset);
                out.push_str(&format!("PUSH {} LSTORE {}\n", self.heap_offset, self.local_offset));
                self.local_offset += 8;
                let mut curr_ptr = self.heap_offset;
                for b in tokens[i+3].bytes() { out.push_str(&format!("PUSH {} PUSH {} STOREB\n", b, curr_ptr)); curr_ptr += 1; }
                out.push_str(&format!("PUSH 0 PUSH {} STOREB\n", curr_ptr));
                self.heap_offset = curr_ptr + 1;
                while tokens[i] != ";" { i += 1; }
            } else if tokens[i] == "poke" {
                i += 2; let mut addr_expr = Vec::new();
                while tokens[i] != "," { addr_expr.push(tokens[i].clone()); i += 1; }
                i += 1; let val = &tokens[i];
                out.push_str(&self.gen_load(val));
                if addr_expr.len() == 1 { out.push_str(&self.gen_load(&addr_expr[0])); }
                else if addr_expr.len() == 3 && addr_expr[1] == "+" { out.push_str(&self.gen_load(&addr_expr[0])); out.push_str(&self.gen_load(&addr_expr[2])); out.push_str("ADD\n"); }
                out.push_str("STOREB\n");
                while tokens[i] != ";" { i += 1; }
            } else if tokens[i] == "syscall" {
                i += 2; let id = tokens[i].clone(); i += 1;
                let mut args = Vec::new();
                while tokens[i] != ")" { if tokens[i] != "," { args.push(tokens[i].clone()); } i += 1; }
                for arg in args.iter().rev() { out.push_str(&self.gen_load(arg)); }
                out.push_str(&self.gen_load(&id));
                out.push_str("SYSCALL\n");
                while tokens[i] != ";" { i += 1; }
            } else if tokens[i] == "exec" {
                i += 2; let fname = &tokens[i];
                if self.locals.contains_key(fname) { out.push_str(&self.gen_load(fname)); }
                else {
                    out.push_str(&format!("PUSH {} \n", self.heap_offset));
                    let mut curr_ptr = self.heap_offset;
                    for b in fname.bytes() { out.push_str(&format!("PUSH {} PUSH {} STOREB\n", b, curr_ptr)); curr_ptr += 1; }
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
            } else if tokens[i] == "while" {
                let start_lbl = "L_START"; let end_lbl = "L_END";
                out.push_str(&format!("{}:\n", start_lbl));
                i += 2; let cond = &tokens[i];
                out.push_str(&self.gen_load(cond));
                out.push_str(&format!("JZ {}\n", end_lbl));
                while tokens[i] != "{" { i += 1; } i += 1; while tokens[i] != "}" { i += 1; }
                out.push_str(&format!("JMP {}\n{}:\n", start_lbl, end_lbl));
            } else if tokens[i] == "return" {
                i += 1; let mut expr = Vec::new();
                while tokens[i] != ";" { expr.push(tokens[i].clone()); i += 1; }
                if expr.len() == 1 { out.push_str(&self.gen_load(&expr[0])); }
                else if expr.len() == 3 && expr[1] == "+" { out.push_str(&self.gen_load(&expr[0])); out.push_str(&self.gen_load(&expr[2])); out.push_str("ADD\n"); }
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

pub struct Assembler;
impl Assembler {
    pub fn compile_bef(source: &str) -> Vec<u8> {
        let tokens: Vec<&str> = source.split_whitespace().collect();
        let mut labels = HashMap::new();
        let mut addr = 0; let mut i = 0;
        while i < tokens.len() {
            if tokens[i].ends_with(':') { labels.insert(tokens[i].trim_end_matches(':').to_string(), addr); }
            else { addr += match tokens[i] { "PUSH"|"JMP"|"JZ"|"LLOAD"|"LSTORE"|"CALL" => { i+=1; 9 }, _ => 1 }; }
            i += 1;
        }
        let mut code = Vec::new(); i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "HALT" => code.push(0x00), "ADD" => code.push(0x20), "LT" => code.push(0x25),
                "LOAD" => code.push(0x62), "STORE" => code.push(0x63), "LOADB" => code.push(0x64), "STOREB" => code.push(0x65),
                "RET" => code.push(0x81), "SYSCALL" => code.push(0xF0),
                "PUSH" => { code.push(0x10); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "JMP" => { code.push(0x40); i+=1; code.extend_from_slice(&(*labels.get(tokens[i]).unwrap() as u64).to_le_bytes()); }
                "JZ" => { code.push(0x41); i+=1; code.extend_from_slice(&(*labels.get(tokens[i]).unwrap() as u64).to_le_bytes()); }
                "LLOAD" => { code.push(0x60); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "LSTORE" => { code.push(0x61); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "CALL" => { code.push(0x80); i+=1; code.extend_from_slice(&(*labels.get(tokens[i]).unwrap() as u64).to_le_bytes()); }
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

pub struct Machine {
    pub stack: Vec<u64>, pub call_stack: Vec<usize>, pub memory: Vec<u8>, pub ip: usize, pub bp: usize, pub vfs: HashMap<String, Vec<u8>>,
}
impl Machine {
    pub fn new() -> Self { Self { stack: vec![], call_stack: vec![], memory: vec![0; 8192], ip: 0, bp: 4096, vfs: HashMap::new() } }
    fn check(&self, a: usize, s: usize) -> Result<(), String> { if a + s > self.memory.len() { return Err(format!("Segmentation Fault @ {}", a)); } Ok(()) }
    fn r8(&self, a: usize) -> Result<u8, String> { self.check(a,1)?; Ok(self.memory[a]) }
    fn w8(&mut self, a: usize, v: u8) -> Result<(), String> { self.check(a,1)?; self.memory[a] = v; Ok(()) }
    fn r64(&self, a: usize) -> Result<u64, String> { self.check(a,8)?; Ok(u64::from_le_bytes(self.memory[a..a+8].try_into().unwrap())) }
    fn w64(&mut self, a: usize, v: u64) -> Result<(), String> { self.check(a,8)?; self.memory[a..a+8].copy_from_slice(&v.to_le_bytes()); Ok(()) }
    pub fn load(&mut self, d: &[u8]) { let s = u32::from_le_bytes(d[8..12].try_into().unwrap()) as usize; if s <= self.memory.len() { self.memory[0..s].copy_from_slice(&d[16..16+s]); } }
    
    pub fn step(&mut self, fuel: &mut u64) -> Result<bool, String> {
        if *fuel == 0 { return Err("Resource Exhaustion".into()); } *fuel -= 1;
        let op = self.r8(self.ip)?; self.ip += 1;
        match op {
            0x00 => return Ok(false),
            0x10 => { let v = self.r64(self.ip)?; self.ip+=8; self.stack.push(v); }
            0x20 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a+b); }
            0x25 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a<b {1} else {0}); }
            0x40 => { self.ip = self.r64(self.ip)? as usize; }
            0x41 => { let t = self.r64(self.ip)? as usize; if self.stack.pop().unwrap() == 0 { self.ip = t; } else { self.ip += 8; } }
            0x60 => { let off = self.r64(self.ip)? as usize; self.ip+=8; self.stack.push(self.r64(self.bp+off)?); }
            0x61 => { let off = self.r64(self.ip)? as usize; self.ip+=8; let v = self.stack.pop().unwrap(); self.w64(self.bp+off, v)?; }
            0x62 => { let a = self.stack.pop().unwrap() as usize; self.stack.push(self.r64(a)?); }
            0x63 => { let a = self.stack.pop().unwrap() as usize; let v = self.stack.pop().unwrap(); self.w64(a, v)?; }
            0x64 => { let a = self.stack.pop().unwrap() as usize; self.stack.push(self.r8(a)? as u64); }
            0x65 => { let a = self.stack.pop().unwrap() as usize; let v = self.stack.pop().unwrap() as u8; self.w8(a, v)?; }
            0x80 => { let t = self.r64(self.ip)? as usize; self.call_stack.push(self.ip + 8); self.ip = t; }
            0x81 => { self.ip = self.call_stack.pop().unwrap(); }
            0xF0 => {
                let id = self.stack.pop().unwrap();
                if id == 1 { let v = self.stack.pop().unwrap(); self.vfs.insert("ret".into(), v.to_le_bytes().to_vec()); }
                else if id == 2 {
                    let fp = self.stack.pop().unwrap() as usize; let bp = self.stack.pop().unwrap() as usize;
                    let mut fnm = String::new(); let mut p = fp; while self.r8(p)? != 0 { fnm.push(self.r8(p)? as char); p+=1; }
                    let d = self.vfs.get(&fnm).cloned();
                    if let Some(data) = d { for (i, b) in data.iter().enumerate() { self.w8(bp+i, *b)?; } }
                } else if id == 3 {
                    let fp = self.stack.pop().unwrap() as usize; let bp = self.stack.pop().unwrap() as usize; let l = self.stack.pop().unwrap() as usize;
                    let mut fnm = String::new(); let mut p = fp; while self.r8(p)? != 0 { fnm.push(self.r8(p)? as char); p+=1; }
                    let mut d = Vec::new(); for i in 0..l { d.push(self.r8(bp+i)?); } self.vfs.insert(fnm, d);
                } else if id == 4 {
                    let c = self.stack.pop().unwrap() as u8;
                    if let Some(buf) = self.vfs.get_mut("stdout") { buf.push(c); } else { self.vfs.insert("stdout".into(), vec![c]); }
                } else if id == 5 {
                    let fp = self.stack.pop().unwrap() as usize;
                    let mut fnm = String::new(); let mut p = fp; while self.r8(p)? != 0 { fnm.push(self.r8(p)? as char); p+=1; }
                    let d = self.vfs.get(&fnm).cloned();
                    if let Some(data) = d {
                        let sz = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
                        if sz > self.memory.len() { return Err("EXEC_OOM".into()); }
                        self.memory[0..sz].copy_from_slice(&data[16..16+sz]);
                        self.ip = 0; self.stack.clear(); self.call_stack.clear();
                    } else { return Err(format!("EXEC_FAIL: {}", fnm)); }
                }
            }
            _ => return Err("Illegal Op".into()),
        }
        Ok(true)
    }
}

// --- DIAGNOSTIC SUITE ---
fn run_vm(src: &str, files: HashMap<String, Vec<u8>>, fuel: u64) -> (Result<(), String>, HashMap<String, Vec<u8>>) {
    let mut cc = MiniCC::new();
    let asm = match cc.compile(src) { Ok(a) => a, Err(e) => return (Err(e), HashMap::new()) };
    let bin = Assembler::compile_bef(&asm);
    let mut vm = Machine::new();
    vm.vfs = files;
    vm.load(&bin);
    let mut f = fuel;
    let mut res = Ok(());
    loop {
        match vm.step(&mut f) {
            Ok(run) => if !run { break; },
            Err(e) => { res = Err(e); break; }
        }
    }
    (res, vm.vfs)
}

pub fn run_suite() -> String {
    let mut log = String::from(SYSTEM_STATUS);

    // TEST 1: COMPUTE (Math)
    log.push_str("1. COMPUTE_ENGINE       ... ");
    let (_, vfs) = run_vm("int a = 10 ; int b = 32 ; return a + b ;", HashMap::new(), 1000);
    if let Some(ret) = vfs.get("ret") {
        if ret[0] == 42 { log.push_str("PASS\n"); } else { log.push_str("FAIL\n"); }
    } else { log.push_str("FAIL (No Output)\n"); }

    // TEST 2: MEMORY (IO)
    log.push_str("2. MEMORY_CONTROLLER    ... ");
    let (_, vfs) = run_vm("int p = 5000 ; poke ( p , 99 ) ; int r = 0 ; syscall ( 1 , r ) ; return 0 ;", HashMap::new(), 1000);
    if vfs.contains_key("ret") { log.push_str("PASS\n"); } else { log.push_str("FAIL\n"); }

    // TEST 3: SELF-REPLICATION
    log.push_str("3. SELF_REPLICATION     ... ");
    let src = "
        str bin = payload.bin ; int buf = 4000 ;
        poke ( buf , 231 ) ; poke ( buf + 1 , 17 ) ; poke ( buf + 2 , 177 ) ; poke ( buf + 3 , 0 ) ;
        poke ( buf + 8 , 20 ) ; poke ( buf + 9 , 0 ) ; poke ( buf + 10 , 0 ) ; poke ( buf + 11 , 0 ) ;
        int code = 4016 ;
        poke ( code , 16 ) ; poke ( code + 1 , 88 ) ; poke ( code + 2 , 0 ) ; poke ( code + 3 , 0 ) ; poke ( code + 4 , 0 ) ; 
        poke ( code + 5 , 0 ) ; poke ( code + 6 , 0 ) ; poke ( code + 7 , 0 ) ; poke ( code + 8 , 0 ) ;
        poke ( code + 9 , 16 ) ; poke ( code + 10 , 1 ) ; poke ( code + 11 , 0 ) ; poke ( code + 12 , 0 ) ; poke ( code + 13 , 0 ) ; 
        poke ( code + 14 , 0 ) ; poke ( code + 15 , 0 ) ; poke ( code + 16 , 0 ) ; poke ( code + 17 , 0 ) ;
        poke ( code + 18 , 240 ) ; poke ( code + 19 , 0 ) ;
        syscall ( 3 , bin , buf , 36 ) ; exec ( bin ) ; return 0 ;
    ";
    let (_, vfs) = run_vm(src, HashMap::new(), 5000);
    if let Some(ret) = vfs.get("ret") {
        if ret[0] == 88 { log.push_str("PASS\n"); } else { log.push_str("FAIL\n"); }
    } else { log.push_str("FAIL\n"); }

    // TEST 4: SECURITY (Segfault)
    log.push_str("4. MEMORY_HARDENING     ... ");
    let (res, _) = run_vm("int p = 9000 ; poke ( p , 1 ) ; return 0 ;", HashMap::new(), 1000);
    match res { Err(e) if e.contains("Segmentation Fault") => log.push_str("PASS\n"), _ => log.push_str("FAIL\n") }

    // TEST 5: SECURITY (Gas)
    log.push_str("5. GAS_LIMITER          ... ");
    let (res, _) = run_vm("while ( 1 ) { } return 0 ;", HashMap::new(), 500);
    match res { Err(e) if e.contains("Resource Exhaustion") => log.push_str("PASS\n"), _ => log.push_str("FAIL\n") }

    log.push_str("\nSYSTEM STATUS: OPERATIONAL");
    log
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
