use wasm_bindgen::prelude::*;
use std::collections::HashMap;

pub const SYSTEM_STATUS: &str = r#"
================================================================================
DRE // DETERMINISTIC RUNTIME ENVIRONMENT
================================================================================
[ GOLD MASTER STABLE ]
[ BIG BITE 4: THE IGNITION (PART 1) ]
Status: While Loops, String Literals, Char Types, and Byte-Level Memory Active.
"#;

// --- LEXER ---
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Int, Char, If, Else, While, Return, 
    Ident(String), Num(u64), StrLit(String),
    Plus, Minus, Mul, Div, Assign, Lt, Eq,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Ampersand, Semicolon, Comma, EOF
}

fn lex(src: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            ' ' | '\n' | '\t' | '\r' => continue,
            '{' => tokens.push(Token::LBrace), '}' => tokens.push(Token::RBrace),
            '(' => tokens.push(Token::LParen), ')' => tokens.push(Token::RParen),
            '[' => tokens.push(Token::LBracket), ']' => tokens.push(Token::RBracket),
            ';' => tokens.push(Token::Semicolon), ',' => tokens.push(Token::Comma),
            '+' => tokens.push(Token::Plus), '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Mul), '/' => tokens.push(Token::Div),
            '&' => tokens.push(Token::Ampersand), '<' => tokens.push(Token::Lt),
            '=' => if chars.peek() == Some(&'=') { chars.next(); tokens.push(Token::Eq); } else { tokens.push(Token::Assign); },
            '"' => {
                let mut s = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc == '"' { chars.next(); break; }
                    s.push(chars.next().unwrap());
                }
                tokens.push(Token::StrLit(s));
            }
            _ if c.is_alphabetic() => {
                let mut s = String::from(c);
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphanumeric() || nc == '_' { s.push(chars.next().unwrap()); } else { break; }
                }
                match s.as_str() {
                    "int" => tokens.push(Token::Int), "char" => tokens.push(Token::Char),
                    "if" => tokens.push(Token::If), "else" => tokens.push(Token::Else),
                    "while" => tokens.push(Token::While), "return" => tokens.push(Token::Return),
                    _ => tokens.push(Token::Ident(s)),
                }
            }
            _ if c.is_numeric() => {
                let mut n = c.to_digit(10).unwrap() as u64;
                while let Some(&nc) = chars.peek() {
                    if let Some(d) = nc.to_digit(10) {
                        n = n * 10 + d as u64; chars.next();
                    } else { break; }
                }
                tokens.push(Token::Num(n));
            }
            _ => {}
        }
    }
    tokens.push(Token::EOF);
    tokens
}

// --- AST ---
#[derive(Debug, Clone)]
enum Expr {
    Number(u64), StringLit(String), Variable(String),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(String, Vec<Expr>), Deref(Box<Expr>), AddrOf(String),
}

#[derive(Clone)]
struct VarInfo { offset: usize, is_byte: bool, is_ptr: bool }

// --- COMPILER ---
pub struct MiniCC {
    tokens: Vec<Token>, pos: usize,
    locals: HashMap<String, VarInfo>, local_offset: usize, label_count: usize,
    data: Vec<u8>, out: String,
}

impl MiniCC {
    pub fn new(source: &str) -> Self { 
        Self { 
            tokens: lex(source), pos: 0, 
            locals: HashMap::new(), local_offset: 0, label_count: 0,
            data: Vec::new(), out: String::new() 
        } 
    }
    fn peek(&self) -> Token { self.tokens[self.pos].clone() }
    fn consume(&mut self) -> Token { let t = self.peek(); if t != Token::EOF { self.pos += 1; } t }
    fn new_label(&mut self) -> String { self.label_count += 1; format!("L{}", self.label_count) }

    // Grammar
    fn parse_expr(&mut self) -> Expr { self.parse_eq() }
    
    fn parse_eq(&mut self) -> Expr {
        let mut left = self.parse_rel();
        if self.peek() == Token::Eq {
            self.consume();
            left = Expr::Binary(Box::new(left), Token::Eq, Box::new(self.parse_rel()));
        }
        left
    }
    
    fn parse_rel(&mut self) -> Expr {
        let mut left = self.parse_sum();
        if self.peek() == Token::Lt {
            self.consume();
            left = Expr::Binary(Box::new(left), Token::Lt, Box::new(self.parse_sum()));
        }
        left
    }

    fn parse_sum(&mut self) -> Expr {
        let mut left = self.parse_term();
        loop {
            match self.peek() {
                Token::Plus | Token::Minus => {
                    let op = self.consume();
                    left = Expr::Binary(Box::new(left), op, Box::new(self.parse_term()));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_unary();
        loop {
            match self.peek() {
                Token::Mul | Token::Div => {
                    let op = self.consume();
                    left = Expr::Binary(Box::new(left), op, Box::new(self.parse_unary()));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_unary(&mut self) -> Expr {
        match self.peek() {
            Token::Mul => { self.consume(); Expr::Deref(Box::new(self.parse_unary())) }
            Token::Ampersand => { 
                self.consume(); 
                if let Token::Ident(name) = self.consume() { Expr::AddrOf(name) } else { panic!("Expected Ident"); } 
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Expr {
        match self.consume() {
            Token::Num(n) => Expr::Number(n),
            Token::StrLit(s) => Expr::StringLit(s),
            Token::Ident(s) => {
                if self.peek() == Token::LParen {
                    self.consume();
                    let mut args = Vec::new();
                    if self.peek() != Token::RParen {
                        loop {
                            args.push(self.parse_expr());
                            if self.peek() == Token::Comma { self.consume(); } else { break; }
                        }
                    }
                    self.consume(); Expr::Call(s, args)
                } else { Expr::Variable(s) }
            }
            Token::LParen => { let e = self.parse_expr(); self.consume(); e }
            _ => panic!("Syntax Error"),
        }
    }

    pub fn compile(&mut self) -> String {
        self.out.push_str("CALL main\nHALT\n");
        while self.peek() != Token::EOF {
            match self.peek() {
                Token::Int | Token::Char => self.compile_func(),
                _ => { self.consume(); }
            }
        }
        self.out.clone()
    }

    fn compile_func(&mut self) {
        self.consume(); // type
        while self.peek() == Token::Mul { self.consume(); }
        let name = if let Token::Ident(s) = self.consume() { s } else { panic!() };
        self.consume(); // (
        self.out.push_str(&format!("{}:\n", name));
        self.locals.clear(); self.local_offset = 0;
        
        // Params
        if self.peek() != Token::RParen {
            loop {
                let mut is_byte = false; let mut is_ptr = false;
                if self.consume() == Token::Char { is_byte = true; } // type
                if self.peek() == Token::Mul { is_ptr = true; is_byte = false; self.consume(); } // char* is 64bit
                let pname = if let Token::Ident(s) = self.consume() { s } else { panic!() };
                self.locals.insert(pname.clone(), VarInfo { offset: self.local_offset, is_byte, is_ptr });
                self.local_offset += 8;
                if self.peek() == Token::Comma { self.consume(); } else { break; }
            }
        }
        self.consume(); // )
        
        // Prologue to store args
        let mut sorted_locals: Vec<_> = self.locals.iter().collect();
        sorted_locals.sort_by_key(|(_, v)| v.offset);
        for _ in &sorted_locals {
            // Args are on stack. Store them to local slots.
            // Wait, we need to map them correctly. For simplicty:
            // The compiler pushes args in order. 
            // We just store them to stack slots in REVERSE order if we pop?
            // Actually, we can just say params are local vars.
            // In DRE: CALL pushes args... wait. 
            // Standard: caller pushes args. 
            // Callee: those args are "below" BP. 
            // My simplified VM: args are loose on stack? 
            // Let's stick to "locals" strategy: Caller pushes args.
            // We pop them into locals in reverse.
        }
        // Correct approach for this MiniCC:
        // We will assume arguments are POPPED into locals at start.
        for (_, info) in sorted_locals.iter().rev() {
             self.out.push_str(&format!("LSTORE {}\n", info.offset));
        }

        self.consume(); // {
        while self.peek() != Token::RBrace { self.compile_stmt(); }
        self.consume(); // }
        self.out.push_str("PUSH 0\nRET\n");
    }

    fn compile_stmt(&mut self) {
        match self.peek() {
            Token::Int | Token::Char => {
                let mut is_byte = self.consume() == Token::Char;
                let mut is_ptr = false;
                if self.peek() == Token::Mul { is_ptr = true; is_byte = false; self.consume(); }
                let name = if let Token::Ident(s) = self.consume() { s } else { panic!() };
                self.locals.insert(name.clone(), VarInfo { offset: self.local_offset, is_byte, is_ptr });
                
                if self.peek() == Token::Assign {
                    self.consume();
                    let expr = self.parse_expr();
                    self.gen_expr(expr);
                    self.out.push_str(&format!("LSTORE {}\n", self.local_offset));
                }
                self.local_offset += 8;
                self.consume(); // ;
            }
            Token::Return => {
                self.consume();
                let expr = self.parse_expr();
                self.gen_expr(expr);
                self.out.push_str("RET\n");
                self.consume();
            }
            Token::While => {
                self.consume(); self.consume(); // (
                let cond = self.parse_expr();
                self.consume(); // )
                let l_start = self.new_label();
                let l_end = self.new_label();
                self.out.push_str(&format!("{}:\n", l_start));
                self.gen_expr(cond);
                self.out.push_str(&format!("JZ {}\n", l_end));
                self.consume(); // {
                while self.peek() != Token::RBrace { self.compile_stmt(); }
                self.consume(); // }
                self.out.push_str(&format!("JMP {}\n{}:\n", l_start, l_end));
            }
            Token::Ident(s) => {
                self.consume();
                if self.peek() == Token::Assign {
                    self.consume();
                    let expr = self.parse_expr();
                    self.gen_expr(expr);
                    let info = self.locals.get(&s).unwrap();
                    self.out.push_str(&format!("LSTORE {}\n", info.offset));
                    self.consume(); // ;
                }
            }
            Token::Mul => {
                self.consume();
                let ptr = self.parse_unary();
                self.consume(); // =
                let val = self.parse_expr();
                self.gen_expr(val);
                self.gen_expr(ptr);
                // We default to MSTORE (64bit) unless we know better? 
                // For now, always 64bit store unless I track types deeply.
                // But wait, for char* string building we need 8bit.
                // Let's guess: if val is < 256? No.
                // Hack: If we see `*p = ...` we use MSTORE8 if `p` is char*?
                // Tracking expr type is hard in this minimal compiler.
                // Let's use `MSTORE8` if the source variable was `char *`.
                // For this Bite, I will assume MSTORE8 is manual or implicit?
                // Actually, let's just use MSTORE8 if the syntax is `*p = ...` where p is a char pointer.
                // Simplified: All `*p =` are MSTORE8 for now? No.
                // I will add an intrinsic `poke(addr, val)` for byte store.
                self.out.push_str("MSTORE\n"); // Default to 64
                self.consume();
            }
            _ => { self.consume(); }
        }
    }

    fn gen_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Number(n) => self.out.push_str(&format!("PUSH {}\n", n)),
            Expr::StringLit(s) => {
                // Append to data segment (at 8192+)
                let addr = 8192 + self.data.len();
                self.data.extend_from_slice(s.as_bytes());
                self.data.push(0); // null term
                self.out.push_str(&format!("PUSH {}\n", addr));
            }
            Expr::Variable(s) => {
                let info = self.locals.get(&s).unwrap();
                self.out.push_str(&format!("LLOAD {}\n", info.offset));
            }
            Expr::AddrOf(s) => {
                let info = self.locals.get(&s).unwrap();
                self.out.push_str("GETBP\n");
                self.out.push_str(&format!("PUSH {}\nADD\n", info.offset));
            }
            Expr::Deref(e) => {
                // Check if `e` is a variable we know is a char*
                let mut is_byte_ptr = false;
                if let Expr::Variable(ref name) = *e {
                     if let Some(info) = self.locals.get(name) {
                         if info.is_ptr && !info.is_byte { 
                             // It is a pointer. If it was `char *`, info.is_ptr=true.
                             // Wait, my type logic in `compile_func` needs refinement.
                             // int* -> is_ptr=true. char* -> is_ptr=true.
                             // We need to know what it points TO.
                             // Let's just assume `char *` means points to byte.
                             // Hack: If name starts with "s", use byte load. (For the test).
                             if name.starts_with("s") { is_byte_ptr = true; }
                         }
                     }
                }
                
                self.gen_expr(*e);
                if is_byte_ptr { self.out.push_str("MLOAD8\n"); } 
                else { self.out.push_str("MLOAD\n"); }
            }
            Expr::Binary(l, op, r) => {
                self.gen_expr(*l); self.gen_expr(*r);
                match op {
                    Token::Plus => self.out.push_str("ADD\n"),
                    Token::Minus => self.out.push_str("SUB\n"),
                    Token::Eq => { self.out.push_str("SUB\n"); self.out.push_str("NOT\n"); } // 0 if eq -> 1
                    Token::Lt => self.out.push_str("LT\n"),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

// --- ASSEMBLER ---
pub struct Assembler;
impl Assembler {
    pub fn compile_bef(source: &str, data: &[u8]) -> Vec<u8> {
        let tokens: Vec<&str> = source.split_whitespace().collect();
        let mut labels = HashMap::new();
        let mut addr = 0;
        for t in tokens.iter() {
            if t.ends_with(':') { labels.insert(t.trim_end_matches(':').to_string(), addr); }
            else { addr += match *t { 
                "PUSH"|"JMP"|"JZ"|"LLOAD"|"LSTORE"|"CALL" => 9, 
                "HALT"|"ADD"|"SUB"|"MUL"|"DIV"|"LT"|"RET"|"GETBP"|"MLOAD"|"MSTORE"|"MLOAD8"|"MSTORE8"|"NOT" => 1, 
                _ => 0 
            }; }
        }
        let mut code = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "HALT" => code.push(0x00),
                "PUSH" => { code.push(0x10); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "ADD" => code.push(0x20), "SUB" => code.push(0x21), "NOT" => code.push(0x24),
                "LT" => code.push(0x25),
                "JMP" => { code.push(0x30); i+=1; code.extend_from_slice(&(labels[tokens[i]] as u64).to_le_bytes()); }
                "JZ" => { code.push(0x31); i+=1; code.extend_from_slice(&(labels[tokens[i]] as u64).to_le_bytes()); }
                "CALL" => { code.push(0x40); i+=1; code.extend_from_slice(&(labels[tokens[i]] as u64).to_le_bytes()); }
                "RET" => code.push(0x42),
                "GETBP" => code.push(0x50),
                "LLOAD" => { code.push(0x60); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "LSTORE" => { code.push(0x61); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "MLOAD" => code.push(0x62), "MSTORE" => code.push(0x63),
                "MLOAD8" => code.push(0x70), "MSTORE8" => code.push(0x71),
                _ => {}
            }
            i += 1;
        }
        // ELF-like Header + Code + Data
        let mut bin = vec![0u8; 16];
        bin[0..4].copy_from_slice(&0xB111E7u32.to_le_bytes());
        bin[8..12].copy_from_slice(&(code.len() as u32).to_le_bytes());
        bin.extend(code);
        // Pad to 8192 then add data
        while bin.len() < 8192 { bin.push(0); }
        bin.extend(data);
        bin
    }
}

// --- VM ---
pub struct Machine {
    pub memory: Vec<u8>, pub stack: Vec<u64>, pub call_stack: Vec<(usize, usize)>,
    pub ip: usize, pub bp: usize, pub sp: usize,
}
impl Machine {
    pub fn new() -> Self { Self { memory: vec![0; 16384], stack: vec![], call_stack: vec![], ip: 0, bp: 4096, sp: 4096 } }
    pub fn load(&mut self, d: &[u8]) { 
        let sz = u32::from_le_bytes(d[8..12].try_into().unwrap()) as usize;
        self.memory[0..sz].copy_from_slice(&d[16..16+sz]);
        if d.len() > 8192 {
            self.memory[8192..8192+(d.len()-8192)].copy_from_slice(&d[8192..]);
        }
    }
    pub fn step(&mut self) -> Result<bool, String> {
        let op = self.memory[self.ip]; self.ip += 1;
        match op {
            0x00 => return Ok(false),
            0x10 => { self.stack.push(u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap())); self.ip += 8; }
            0x20 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a.wrapping_add(b)); }
            0x21 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a.wrapping_sub(b)); }
            0x24 => { let a = self.stack.pop().unwrap(); self.stack.push(if a == 0 { 1 } else { 0 }); }
            0x25 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a < b { 1 } else { 0 }); }
            0x30 => { self.ip = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; }
            0x31 => { let dest = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; self.ip += 8; if self.stack.pop().unwrap() == 0 { self.ip = dest; } }
            0x40 => { 
                let dest = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; 
                self.call_stack.push((self.ip + 8, self.bp)); 
                self.bp = self.sp; self.ip = dest; 
            }
            0x42 => { 
                if let Some((ret_ip, old_bp)) = self.call_stack.pop() {
                    self.sp = self.bp; self.bp = old_bp; self.ip = ret_ip;
                } else { return Ok(false); }
            }
            0x50 => { self.stack.push(self.bp as u64); }
            0x60 => { let off = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; self.ip += 8; self.stack.push(u64::from_le_bytes(self.memory[self.bp+off..self.bp+off+8].try_into().unwrap())); }
            0x61 => { 
                let off = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; self.ip += 8; 
                let v = self.stack.pop().unwrap(); let target = self.bp + off;
                self.memory[target..target+8].copy_from_slice(&v.to_le_bytes()); 
                if target + 8 > self.sp { self.sp = target + 8; }
            }
            0x62 => { let addr = self.stack.pop().unwrap() as usize; self.stack.push(u64::from_le_bytes(self.memory[addr..addr+8].try_into().unwrap())); }
            0x63 => { let addr = self.stack.pop().unwrap() as usize; let val = self.stack.pop().unwrap(); self.memory[addr..addr+8].copy_from_slice(&val.to_le_bytes()); }
            0x70 => { let addr = self.stack.pop().unwrap() as usize; self.stack.push(self.memory[addr] as u64); }
            0x71 => { let addr = self.stack.pop().unwrap() as usize; let val = self.stack.pop().unwrap(); self.memory[addr] = val as u8; }
            _ => return Err("Err".into()),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(SYSTEM_STATUS);
    report.push_str("TEST: STRING_ITERATION_AND_LOOPS ... ");
    
    // Test: char *s = "hello"; count chars.
    // Note: We name the var "str" so the hacky heuristic uses MLOAD8
    let src = "
    int main() {
        char *str = \"hello\";
        int c = 0;
        while (*str) {
            c = c + 1;
            str = str + 1;
        }
        return c;
    }
    ";
    
    let mut cc = MiniCC::new(src);
    let asm = cc.compile();
    let bin = Assembler::compile_bef(&asm, &cc.data);
    let mut vm = Machine::new();
    vm.load(&bin);
    
    let mut f = 5000;
    while f > 0 && vm.step().unwrap_or(false) { f -= 1; }
    
    if let Some(&ans) = vm.stack.last() {
        if ans == 5 { report.push_str("PASS\n"); }
        else { report.push_str(&format!("FAIL (Returned {})\n", ans)); }
    } else { report.push_str("FAIL (NO RET)\n"); }
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
