use wasm_bindgen::prelude::*;
use std::collections::HashMap;

pub const SYSTEM_STATUS: &str = "\
\x1b[36m================================================================================
DRE // DETERMINISTIC RUNTIME ENVIRONMENT
================================================================================\x1b[0m
[ GOLD MASTER STABLE ]
[ ERA 2: THE INDUSTRIAL BRIDGE ]
Status: Preprocessor [MACRO EXPANSION] Active.
";

// --- PREPROCESSOR ---
fn preprocess(src: &str) -> String {
    let mut macros = HashMap::new();
    let mut lines: Vec<String> = src.lines().map(|s| s.to_string()).collect();
    let mut result = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("#define") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                macros.insert(parts[1].to_string(), parts[2..].join(" "));
            }
            continue; // Don't include the #define line in final source
        }
        
        let mut processed_line = line.clone();
        for (name, val) in &macros {
            // Simple replacement (Industrial preprocessors use tokenization, but this works for constants)
            processed_line = processed_line.replace(name, val);
        }
        result.push(processed_line);
    }
    result.join("\n")
}

// --- LEXER ---
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Int, Char, Struct, If, Else, While, Return, Syscall, Sizeof,
    Ident(String), Num(u64), StrLit(String),
    Plus, Minus, Mul, Div, Assign, Lt, Gt, Eq, Arrow, Dot,
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
            '.' => tokens.push(Token::Dot),
            '+' => tokens.push(Token::Plus), 
            '-' => if chars.peek() == Some(&'>') { chars.next(); tokens.push(Token::Arrow); } else { tokens.push(Token::Minus); },
            '*' => tokens.push(Token::Mul), '/' => tokens.push(Token::Div),
            '&' => tokens.push(Token::Ampersand), '<' => tokens.push(Token::Lt), '>' => tokens.push(Token::Gt),
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
                    "struct" => tokens.push(Token::Struct), "sizeof" => tokens.push(Token::Sizeof),
                    "if" => tokens.push(Token::If), "else" => tokens.push(Token::Else),
                    "while" => tokens.push(Token::While), "return" => tokens.push(Token::Return),
                    "syscall" => tokens.push(Token::Syscall),
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
    Call(String, Vec<Expr>), Syscall(Vec<Expr>), 
    Deref(Box<Expr>), AddrOf(String),
    MemberAccess(Box<Expr>, usize),
    ArrayAccess(Box<Expr>, Box<Expr>, usize),
}

#[derive(Clone)]
struct VarInfo { offset: usize, is_array: bool } 

#[derive(Clone)]
struct GlobalInfo { offset: usize, is_array: bool }

#[derive(Clone)]
struct StructField { offset: usize, size: usize }

#[derive(Clone)]
struct StructDef { size: usize, fields: HashMap<String, StructField> }

// --- COMPILER ---
pub struct MiniCC {
    tokens: Vec<Token>, pos: usize,
    locals: HashMap<String, VarInfo>, local_offset: usize, 
    globals: HashMap<String, GlobalInfo>, global_offset: usize,
    structs: HashMap<String, StructDef>,
    label_count: usize,
    data: Vec<u8>, out: String,
}

impl MiniCC {
    pub fn new(source: &str) -> Self { 
        let clean_source = preprocess(source);
        Self { 
            tokens: lex(&clean_source), pos: 0, 
            locals: HashMap::new(), local_offset: 0, 
            globals: HashMap::new(), global_offset: 2048,
            structs: HashMap::new(),
            label_count: 0,
            data: Vec::new(), out: String::new() 
        } 
    }
    fn peek(&self) -> Token { self.tokens[self.pos].clone() }
    fn consume(&mut self) -> Token { let t = self.peek(); if t != Token::EOF { self.pos += 1; } t }
    fn new_label(&mut self) -> String { self.label_count += 1; format!("L{}", self.label_count) }

    fn parse_expr(&mut self) -> Expr { self.parse_eq() }
    fn parse_eq(&mut self) -> Expr {
        let mut left = self.parse_rel();
        if self.peek() == Token::Eq { self.consume(); left = Expr::Binary(Box::new(left), Token::Eq, Box::new(self.parse_rel())); }
        left
    }
    fn parse_rel(&mut self) -> Expr {
        let mut left = self.parse_sum();
        loop { match self.peek() { Token::Lt | Token::Gt => { let op = self.consume(); left = Expr::Binary(Box::new(left), op, Box::new(self.parse_sum())); } _ => break, } }
        left
    }
    fn parse_sum(&mut self) -> Expr {
        let mut left = self.parse_term();
        loop { match self.peek() { Token::Plus | Token::Minus => { let op = self.consume(); left = Expr::Binary(Box::new(left), op, Box::new(self.parse_term())); } _ => break, } }
        left
    }
    fn parse_term(&mut self) -> Expr {
        let mut left = self.parse_unary();
        loop { match self.peek() { Token::Mul | Token::Div => { let op = self.consume(); left = Expr::Binary(Box::new(left), op, Box::new(self.parse_unary())); } _ => break, } }
        left
    }
    fn parse_unary(&mut self) -> Expr {
        match self.peek() {
            Token::Mul => { self.consume(); Expr::Deref(Box::new(self.parse_unary())) }
            Token::Ampersand => { self.consume(); if let Token::Ident(name) = self.consume() { Expr::AddrOf(name) } else { panic!(); } }
            _ => self.parse_postfix(),
        }
    }
    fn parse_postfix(&mut self) -> Expr {
        let mut left = self.parse_primary();
        loop {
            match self.peek() {
                Token::Arrow => {
                    self.consume(); if let Token::Ident(field) = self.consume() {
                        let mut found_offset = None;
                        for (_, def) in &self.structs { if let Some(f) = def.fields.get(&field) { found_offset = Some(f.offset); break; } }
                        if let Some(off) = found_offset { left = Expr::MemberAccess(Box::new(left), off); } else { panic!(); }
                    } else { panic!(); }
                },
                Token::LBracket => { self.consume(); let index = self.parse_expr(); self.consume(); left = Expr::ArrayAccess(Box::new(left), Box::new(index), 8); },
                _ => break,
            }
        }
        left
    }
    fn parse_primary(&mut self) -> Expr {
        match self.consume() {
            Token::Num(n) => Expr::Number(n),
            Token::StrLit(s) => Expr::StringLit(s),
            Token::Sizeof => { self.consume(); self.consume(); let name = if let Token::Ident(s) = self.consume() { s } else { panic!() }; self.consume(); if let Some(def) = self.structs.get(&name) { Expr::Number(def.size as u64) } else { panic!() } }
            Token::Syscall => { self.consume(); let mut args = Vec::new(); if self.peek() != Token::RParen { loop { args.push(self.parse_expr()); if self.peek() == Token::Comma { self.consume(); } else { break; } } } self.consume(); Expr::Syscall(args) }
            Token::Ident(s) => { if self.peek() == Token::LParen { self.consume(); let mut args = Vec::new(); if self.peek() != Token::RParen { loop { args.push(self.parse_expr()); if self.peek() == Token::Comma { self.consume(); } else { break; } } } self.consume(); Expr::Call(s, args) } else { Expr::Variable(s) } }
            Token::LParen => { let e = self.parse_expr(); self.consume(); e }
            _ => panic!("Syntax Error"),
        }
    }

    pub fn compile(&mut self) -> String {
        self.out.push_str("CALL main\nHALT\n");
        let saved_pos = self.pos;
        while self.peek() != Token::EOF { if self.peek() == Token::Struct { self.compile_struct_def(); } else { self.consume(); } }
        self.pos = saved_pos;
        while self.peek() != Token::EOF {
            match self.peek() {
                Token::Struct => { self.consume(); self.consume(); self.consume(); while self.peek() != Token::RBrace && self.peek() != Token::EOF { self.consume(); } self.consume(); self.consume(); },
                Token::Int | Token::Char => {
                    let mut is_func = false;
                    let mut temp_pos = self.pos + 1; 
                    while temp_pos < self.tokens.len() { match &self.tokens[temp_pos] { Token::Mul => temp_pos += 1, Token::Ident(_) => { if temp_pos + 1 < self.tokens.len() && self.tokens[temp_pos+1] == Token::LParen { is_func = true; } break; } _ => break, } }
                    if is_func { self.compile_func(); } else { self.compile_global(); }
                },
                _ => { self.consume(); }
            }
        }
        self.out.clone()
    }

    fn compile_struct_def(&mut self) {
        self.consume(); let name = if let Token::Ident(s) = self.consume() { s } else { panic!() }; self.consume();
        let mut current_offset = 0;
        let mut fields = HashMap::new();
        while self.peek() != Token::RBrace {
            let mut _field_size = 8;
            if self.peek() == Token::Struct { self.consume(); let type_name = if let Token::Ident(s) = self.consume() { s } else { panic!() }; if let Some(def) = self.structs.get(&type_name) { _field_size = def.size; } } else { self.consume(); }
            while self.peek() == Token::Mul { self.consume(); _field_size = 8; }
            let fname = if let Token::Ident(s) = self.consume() { s } else { panic!() };
            if self.peek() == Token::LBracket { self.consume(); if let Token::Num(n) = self.consume() { _field_size *= n as usize; } self.consume(); }
            self.consume();
            fields.insert(fname, StructField { offset: current_offset, size: _field_size });
            current_offset += _field_size;
        }
        self.consume(); self.consume();
        self.structs.insert(name, StructDef { size: current_offset, fields });
    }

    fn compile_global(&mut self) {
        self.consume(); while self.peek() == Token::Mul { self.consume(); }
        let name = if let Token::Ident(s) = self.consume() { s } else { panic!() };
        let mut size = 8; let mut is_arr = false;
        if self.peek() == Token::LBracket { self.consume(); if let Token::Num(n) = self.consume() { size = n as usize * 8; } self.consume(); is_arr = true; }
        self.globals.insert(name, GlobalInfo { offset: self.global_offset, is_array: is_arr });
        self.global_offset += size; self.consume();
    }

    fn compile_func(&mut self) {
        if self.consume() == Token::Char { } while self.peek() == Token::Mul { self.consume(); }
        let name = if let Token::Ident(s) = self.consume() { s } else { panic!() };
        self.consume(); self.out.push_str(&format!("{}:\n", name));
        self.locals.clear(); self.local_offset = 0;
        if self.peek() != Token::RParen { loop { self.consume(); while self.peek() == Token::Mul { self.consume(); } let pname = if let Token::Ident(s) = self.consume() { s } else { panic!() }; self.locals.insert(pname.clone(), VarInfo { offset: self.local_offset, is_array: false }); self.local_offset += 8; if self.peek() == Token::Comma { self.consume(); } else { break; } } }
        self.consume(); self.consume();
        while self.peek() != Token::RBrace && self.peek() != Token::EOF { self.compile_stmt(); }
        self.consume(); self.out.push_str("PUSH 0\nRET\n");
    }

    fn compile_stmt(&mut self) {
        match self.peek() {
            Token::Int | Token::Char | Token::Struct => {
                let mut size = 8; let mut is_arr = false;
                if self.peek() == Token::Struct { self.consume(); self.consume(); } else { self.consume(); }
                while self.peek() == Token::Mul { self.consume(); }
                let name = if let Token::Ident(s) = self.consume() { s } else { panic!() };
                if self.peek() == Token::LBracket { self.consume(); if let Token::Num(n) = self.consume() { size = n as usize * 8; } self.consume(); is_arr = true; }
                self.locals.insert(name.clone(), VarInfo { offset: self.local_offset, is_array: is_arr });
                if self.peek() == Token::Assign { self.consume(); let expr = self.parse_expr(); self.gen_expr(expr); self.out.push_str(&format!("LSTORE {}\n", self.local_offset)); }
                self.local_offset += size; self.consume();
            }
            Token::Return => { self.consume(); let expr = self.parse_expr(); self.gen_expr(expr); self.out.push_str("RET\n"); self.consume(); }
            Token::If => { self.consume(); self.consume(); let cond = self.parse_expr(); self.consume(); let l_false = self.new_label(); self.gen_expr(cond); self.out.push_str(&format!("JZ {}\n", l_false)); self.consume(); while self.peek() != Token::RBrace && self.peek() != Token::EOF { self.compile_stmt(); } self.consume(); if self.peek() == Token::Else { self.consume(); let l_end = self.new_label(); self.out.push_str(&format!("JMP {}\n", l_end)); self.out.push_str(&format!("{}:\n", l_false)); self.consume(); while self.peek() != Token::RBrace && self.peek() != Token::EOF { self.compile_stmt(); } self.consume(); self.out.push_str(&format!("{}:\n", l_end)); } else { self.out.push_str(&format!("{}:\n", l_false)); } }
            Token::While => { self.consume(); self.consume(); let cond = self.parse_expr(); self.consume(); let l_start = self.new_label(); let l_end = self.new_label(); self.out.push_str(&format!("{}:\n", l_start)); self.gen_expr(cond); self.out.push_str(&format!("JZ {}\n", l_end)); self.consume(); while self.peek() != Token::RBrace && self.peek() != Token::EOF { self.compile_stmt(); } self.consume(); self.out.push_str(&format!("JMP {}\n{}:\n", l_start, l_end)); }
            Token::Syscall => { let expr = self.parse_expr(); self.gen_expr(expr); self.out.push_str("POP\n"); self.consume(); }
            Token::Ident(s) => {
                self.consume(); let mut lhs_expr = None;
                if self.peek() == Token::Arrow { self.consume(); let field = if let Token::Ident(f) = self.consume() { f } else { panic!() }; let base = Expr::Variable(s.clone()); let mut found = None; for (_, def) in &self.structs { if let Some(f) = def.fields.get(&field) { found = Some(f.offset); break; } } if let Some(off) = found { lhs_expr = Some(Expr::MemberAccess(Box::new(base), off)); } } 
                else if self.peek() == Token::LBracket { self.consume(); let idx = self.parse_expr(); self.consume(); lhs_expr = Some(Expr::ArrayAccess(Box::new(Expr::Variable(s.clone())), Box::new(idx), 8)); }
                if let Some(lhs) = lhs_expr { self.consume(); let val = self.parse_expr(); self.gen_expr(val); match lhs { Expr::MemberAccess(_, off) => { if let Some(info) = self.locals.get(&s) { self.out.push_str(&format!("LLOAD {}\n", info.offset)); } else if let Some(info) = self.globals.get(&s) { self.out.push_str(&format!("PUSH {}\nMLOAD\n", info.offset)); } self.out.push_str(&format!("PUSH {}\nADD\n", off)); }, Expr::ArrayAccess(_, idx, stride) => { if let Some(info) = self.locals.get(&s) { if info.is_array { self.out.push_str("GETBP\n"); self.out.push_str(&format!("PUSH {}\nADD\n", info.offset)); } else { self.out.push_str(&format!("LLOAD {}\n", info.offset)); } } else if let Some(info) = self.globals.get(&s) { if info.is_array { self.out.push_str(&format!("PUSH {}\n", info.offset)); } else { self.out.push_str(&format!("PUSH {}\nMLOAD\n", info.offset)); } } self.gen_expr(*idx); self.out.push_str(&format!("PUSH {}\nMUL\nADD\n", stride)); }, _ => {} } self.out.push_str("MSTORE\n"); self.consume(); }
                else if self.peek() == Token::Assign { self.consume(); let expr = self.parse_expr(); self.gen_expr(expr); if let Some(info) = self.locals.get(&s) { self.out.push_str(&format!("LSTORE {}\n", info.offset)); } else if let Some(info) = self.globals.get(&s) { self.out.push_str(&format!("PUSH {}\n", info.offset)); self.out.push_str("MSTORE\n"); } self.consume(); } 
                else if self.peek() == Token::LParen { self.consume(); let mut args = Vec::new(); if self.peek() != Token::RParen { loop { args.push(self.parse_expr()); if self.peek() == Token::Comma { self.consume(); } else { break; } } } self.consume(); self.gen_expr(Expr::Call(s, args)); self.out.push_str("POP\n"); self.consume(); }
            }
            Token::Mul => { self.consume(); let ptr = self.parse_unary(); self.consume(); let val = self.parse_expr(); self.gen_expr(val); self.gen_expr(ptr.clone()); self.out.push_str("MSTORE\n"); self.consume(); }
            _ => { self.consume(); }
        }
    }

    fn gen_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Number(n) => self.out.push_str(&format!("PUSH {}\n", n)),
            Expr::StringLit(s) => { let addr = 8192 + self.data.len(); self.data.extend_from_slice(s.as_bytes()); self.data.push(0); self.out.push_str(&format!("PUSH {}\n", addr)); }
            Expr::Variable(s) => { if let Some(info) = self.locals.get(&s) { if info.is_array { self.out.push_str("GETBP\n"); self.out.push_str(&format!("PUSH {}\nADD\n", info.offset)); } else { self.out.push_str(&format!("LLOAD {}\n", info.offset)); } } else if let Some(info) = self.globals.get(&s) { if info.is_array { self.out.push_str(&format!("PUSH {}\n", info.offset)); } else { self.out.push_str(&format!("PUSH {}\nMLOAD\n", info.offset)); } } }
            Expr::MemberAccess(base, offset) => { self.gen_expr(*base); self.out.push_str(&format!("PUSH {}\nADD\nMLOAD\n", offset)); }
            Expr::ArrayAccess(base, idx, stride) => { self.gen_expr(*base); self.gen_expr(*idx); self.out.push_str(&format!("PUSH {}\nMUL\nADD\nMLOAD\n", stride)); }
            Expr::AddrOf(s) => { if let Some(info) = self.locals.get(&s) { self.out.push_str("GETBP\n"); self.out.push_str(&format!("PUSH {}\nADD\n", info.offset)); } else if let Some(info) = self.globals.get(&s) { self.out.push_str(&format!("PUSH {}\n", info.offset)); } }
            Expr::Deref(e) => { self.gen_expr(*e); self.out.push_str("MLOAD\n"); }
            Expr::Call(name, args) => { for arg in args { self.gen_expr(arg); } self.out.push_str(&format!("CALL {}\n", name)); }
            Expr::Syscall(args) => { for arg in args.into_iter().rev() { self.gen_expr(arg); } self.out.push_str("SYSCALL\n"); }
            Expr::Binary(l, op, r) => { self.gen_expr(*l); self.gen_expr(*r); match op { Token::Plus => self.out.push_str("ADD\n"), Token::Minus => self.out.push_str("SUB\n"), Token::Eq => { self.out.push_str("SUB\n"); self.out.push_str("NOT\n"); } Token::Lt => self.out.push_str("LT\n"), Token::Gt => self.out.push_str("GT\n"), _ => {} } }
        }
    }
}

// --- ASSEMBLER & VM ---
pub struct Assembler;
impl Assembler {
    pub fn compile_bef(source: &str, data: &[u8]) -> Vec<u8> {
        let tokens: Vec<&str> = source.split_whitespace().collect();
        let mut labels = HashMap::new();
        let mut addr = 0;
        for t in tokens.iter() {
            if t.ends_with(':') { labels.insert(t.trim_end_matches(':').to_string(), addr); }
            else { addr += match *t { "PUSH"|"JMP"|"JZ"|"LLOAD"|"LSTORE"|"CALL" => 9, "HALT"|"ADD"|"SUB"|"MUL"|"DIV"|"LT"|"GT"|"RET"|"GETBP"|"MLOAD"|"MSTORE"|"MLOAD8"|"MSTORE8"|"NOT"|"SYSCALL"|"POP" => 1, _ => 0 }; }
        }
        let mut code = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "HALT" => code.push(0x00),
                "PUSH" => { code.push(0x10); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "POP" => code.push(0x11),
                "ADD" => code.push(0x20), "SUB" => code.push(0x21), "MUL" => code.push(0x22), "NOT" => code.push(0x24),
                "LT" => code.push(0x25), "GT" => code.push(0x26),
                "JMP" => { code.push(0x30); i+=1; code.extend_from_slice(&(labels[tokens[i]] as u64).to_le_bytes()); }
                "JZ" => { code.push(0x31); i+=1; code.extend_from_slice(&(labels[tokens[i]] as u64).to_le_bytes()); }
                "CALL" => { code.push(0x40); i+=1; code.extend_from_slice(&(labels[tokens[i]] as u64).to_le_bytes()); }
                "RET" => code.push(0x42),
                "GETBP" => code.push(0x50),
                "LLOAD" => { code.push(0x60); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "LSTORE" => { code.push(0x61); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "MLOAD" => code.push(0x62), "MSTORE" => code.push(0x63),
                "MLOAD8" => code.push(0x70), "MSTORE8" => code.push(0x71),
                "SYSCALL" => code.push(0x80),
                _ => {}
            }
            i += 1;
        }
        let mut bin = vec![0u8; 16];
        bin[0..4].copy_from_slice(&0xB111E7u32.to_le_bytes());
        bin[8..12].copy_from_slice(&(code.len() as u32).to_le_bytes());
        bin.extend(code);
        while bin.len() < 8192 { bin.push(0); }
        bin.extend(data);
        bin
    }
}

pub struct Machine {
    pub memory: Vec<u8>, pub stack: Vec<u64>, pub call_stack: Vec<(usize, usize)>,
    pub ip: usize, pub bp: usize, pub sp: usize,
    pub vfs: HashMap<String, Vec<u8>>, pub fds: HashMap<u64, (String, usize)>, pub next_fd: u64,
    pub brk: usize,
}
impl Machine {
    pub fn new() -> Self { 
        let mut vfs = HashMap::new();
        vfs.insert("/dev/stdin".to_string(), Vec::new());
        vfs.insert("/dev/stdout".to_string(), Vec::new());
        let mut fds = HashMap::new();
        fds.insert(0, ("/dev/stdin".to_string(), 0)); 
        fds.insert(1, ("/dev/stdout".to_string(), 0));
        Self { memory: vec![0; 1024 * 1024], stack: vec![], call_stack: vec![], ip: 0, bp: 4096, sp: 4096, vfs, fds, next_fd: 3, brk: 512 * 1024 } 
    }
    pub fn load(&mut self, d: &[u8]) { 
        let sz = u32::from_le_bytes(d[8..12].try_into().unwrap()) as usize;
        self.memory[0..sz].copy_from_slice(&d[16..16+sz]);
        if d.len() > 8192 { self.memory[8192..8192+(d.len()-8192)].copy_from_slice(&d[8192..]); }
    }
    pub fn step(&mut self) -> Result<bool, String> {
        let op = self.memory[self.ip]; self.ip += 1;
        match op {
            0x00 => return Ok(false),
            0x10 => { self.stack.push(u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap())); self.ip += 8; }
            0x11 => { self.stack.pop(); }
            0x20 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a.wrapping_add(b)); }
            0x21 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a.wrapping_sub(b)); }
            0x22 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a.wrapping_mul(b)); } 
            0x24 => { let a = self.stack.pop().unwrap(); self.stack.push(if a == 0 { 1 } else { 0 }); }
            0x25 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a < b { 1 } else { 0 }); }
            0x26 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(if a > b { 1 } else { 0 }); }
            0x30 => { self.ip = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; }
            0x31 => { let dest = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; self.ip += 8; if self.stack.pop().unwrap() == 0 { self.ip = dest; } }
            0x40 => { let dest = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; self.call_stack.push((self.ip + 8, self.bp)); self.bp = self.sp; self.ip = dest; }
            0x42 => { if let Some((ret_ip, old_bp)) = self.call_stack.pop() { self.sp = self.bp; self.bp = old_bp; self.ip = ret_ip; } else { return Ok(false); } }
            0x50 => { self.stack.push(self.bp as u64); }
            0x60 => { let off = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; self.ip += 8; self.stack.push(u64::from_le_bytes(self.memory[self.bp+off..self.bp+off+8].try_into().unwrap())); }
            0x61 => { let off = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; self.ip += 8; let v = self.stack.pop().unwrap(); let target = self.bp + off; self.memory[target..target+8].copy_from_slice(&v.to_le_bytes()); if target + 8 > self.sp { self.sp = target + 8; } }
            0x62 => { let addr = self.stack.pop().unwrap() as usize; self.stack.push(u64::from_le_bytes(self.memory[addr..addr+8].try_into().unwrap())); }
            0x63 => { let addr = self.stack.pop().unwrap() as usize; let val = self.stack.pop().unwrap(); self.memory[addr..addr+8].copy_from_slice(&val.to_le_bytes()); }
            0x70 => { let addr = self.stack.pop().unwrap() as usize; self.stack.push(self.memory[addr] as u64); }
            0x71 => { let addr = self.stack.pop().unwrap() as usize; let val = self.stack.pop().unwrap(); self.memory[addr] = val as u8; }
            0x80 => { 
                let sys_num = self.stack.pop().unwrap();
                match sys_num {
                    1 => { let addr = self.stack.pop().unwrap() as usize; let mut name = String::new(); let mut i = addr; while i < self.memory.len() && self.memory[i] != 0 { name.push(self.memory[i] as char); i += 1; } let fd = self.next_fd; self.next_fd += 1; if !self.vfs.contains_key(&name) { self.vfs.insert(name.clone(), Vec::new()); } self.fds.insert(fd, (name, 0)); self.stack.push(fd); }
                    2 => { let fd = self.stack.pop().unwrap(); let buf = self.stack.pop().unwrap() as usize; let len = self.stack.pop().unwrap() as usize; if let Some((name, pos)) = self.fds.get_mut(&fd) { let file = self.vfs.get(name).unwrap(); let mut read_bytes = 0; for i in 0..len { if *pos + i < file.len() && buf + i < self.memory.len() { self.memory[buf + i] = file[*pos + i]; read_bytes += 1; } else { break; } } *pos += read_bytes; self.stack.push(read_bytes as u64); } else { self.stack.push(0); } }
                    3 => { let fd = self.stack.pop().unwrap(); let buf = self.stack.pop().unwrap() as usize; let len = self.stack.pop().unwrap() as usize; if let Some((name, pos)) = self.fds.get_mut(&fd) { let file = self.vfs.get_mut(name).unwrap(); for i in 0..len { if buf + i < self.memory.len() { if name == "/dev/stdout" { file.push(self.memory[buf + i]); } else { if *pos + i < file.len() { file[*pos + i] = self.memory[buf + i]; } else { file.push(self.memory[buf + i]); } } } } if name != "/dev/stdout" { *pos += len; } self.stack.push(len as u64); } else { self.stack.push(0); } }
                    4 => { let inc = self.stack.pop().unwrap() as i64; let old_brk = self.brk; if inc > 0 { self.brk += inc as usize; } else if inc < 0 { self.brk -= (-inc) as usize; } self.stack.push(old_brk as u64); }
                    _ => self.stack.push(0),
                }
            }
            _ => return Err("Err".into()),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(SYSTEM_STATUS);
    let pass_msg = "\x1b[32mPASS\x1b[0m\n";
    
    // Test 1: Stack Vars
    report.push_str("TEST: COMPILER_STACK_VARS ......... ");
    let mut cc1 = MiniCC::new("int main() { return 118; }");
    let mut vm1 = Machine::new();
    vm1.load(&Assembler::compile_bef(&cc1.compile(), &cc1.data));
    while vm1.step().unwrap_or(false) {}
    if vm1.stack.last() == Some(&118) { report.push_str(pass_msg); } else { report.push_str("\x1b[31mFAIL\x1b[0m\n"); }

    // Test 2: Arrays
    report.push_str("TEST: COMPILER_ARRAYS_NESTED ...... ");
    let src_a = "int arr[10]; int main() { arr[0] = 100; arr[1] = 50; return arr[0] + arr[1]; }";
    let mut cc_a = MiniCC::new(src_a);
    let mut vm_a = Machine::new();
    vm_a.load(&Assembler::compile_bef(&cc_a.compile(), &cc_a.data));
    while vm_a.step().unwrap_or(false) {}
    if vm_a.stack.last() == Some(&150) { report.push_str(pass_msg); } else { report.push_str("FAIL\n"); }

    // Test 3: Multi-Pass
    report.push_str("TEST: MULTIPASS_FORWARD_DECLS ..... ");
    let src_f = "int main() { return foo(); } int foo() { return 99; }";
    let mut cc_f = MiniCC::new(src_f);
    let mut vm_f = Machine::new();
    vm_f.load(&Assembler::compile_bef(&cc_f.compile(), &cc_f.data));
    while vm_f.step().unwrap_or(false) {}
    if vm_f.stack.last() == Some(&99) { report.push_str(pass_msg); } else { report.push_str("FAIL\n"); }

    // Test 4: Preprocessor #define
    report.push_str("TEST: PREPROCESSOR_DEFINES ........ ");
    let src_p = "
    #define MAGIC_NUM 42
    #define STATUS_OK 1
    int main() { 
        if (MAGIC_NUM == 42) { return STATUS_OK; }
        return 0;
    }
    ";
    let mut cc_p = MiniCC::new(src_p);
    let mut vm_p = Machine::new();
    vm_p.load(&Assembler::compile_bef(&cc_p.compile(), &cc_p.data));
    while vm_p.step().unwrap_or(false) {}
    if vm_p.stack.last() == Some(&1) { report.push_str(pass_msg); } 
    else { report.push_str(&format!("\x1b[31mFAIL (Got {:?})\x1b[0m\n", vm_p.stack.last())); }

    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
