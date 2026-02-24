use wasm_bindgen::prelude::*;
use std::collections::HashMap;

pub const SYSTEM_STATUS: &str = r#"
================================================================================
DRE // DETERMINISTIC RUNTIME ENVIRONMENT
================================================================================
[ GOLD MASTER STABLE ]
[ BIG BITE 3: POINTERS & MEMORY ]
Status: Absolute Memory Addressing, Pointer Arithmetic, and Indirect Access Active.
"#;

// --- LEXER ---
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Int, If, Return, Ident(String), Num(u64),
    Plus, Minus, Mul, Div, Assign, Lt,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Ampersand, Semicolon, Comma, EOF
}

fn lex(src: &str) -> Vec<Token> {
    let s = src.replace("(", " ( ").replace(")", " ) ")
               .replace("{", " { ").replace("}", " } ")
               .replace("[", " [ ").replace("]", " ] ")
               .replace(";", " ; ").replace(",", " , ")
               .replace("+", " + ").replace("-", " - ")
               .replace("*", " * ").replace("/", " / ")
               .replace("&", " & ").replace("=", " = ")
               .replace("<", " < ");
    let mut tokens = Vec::new();
    for w in s.split_whitespace() {
        match w {
            "int" => tokens.push(Token::Int),
            "if" => tokens.push(Token::If),
            "return" => tokens.push(Token::Return),
            "+" => tokens.push(Token::Plus),
            "-" => tokens.push(Token::Minus),
            "*" => tokens.push(Token::Mul),
            "/" => tokens.push(Token::Div),
            "&" => tokens.push(Token::Ampersand),
            "=" => tokens.push(Token::Assign),
            "<" => tokens.push(Token::Lt),
            "(" => tokens.push(Token::LParen),
            ")" => tokens.push(Token::RParen),
            "{" => tokens.push(Token::LBrace),
            "}" => tokens.push(Token::RBrace),
            "[" => tokens.push(Token::LBracket),
            "]" => tokens.push(Token::RBracket),
            ";" => tokens.push(Token::Semicolon),
            "," => tokens.push(Token::Comma),
            _ => {
                if let Ok(n) = w.parse::<u64>() { tokens.push(Token::Num(n)); }
                else { tokens.push(Token::Ident(w.to_string())); }
            }
        }
    }
    tokens.push(Token::EOF);
    tokens
}

// --- AST ---
#[derive(Debug)]
enum Expr {
    Number(u64),
    Variable(String),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(String, Vec<Expr>),
    Deref(Box<Expr>),
    AddrOf(String),
}

// --- COMPILER ---
pub struct MiniCC {
    tokens: Vec<Token>, pos: usize,
    locals: HashMap<String, usize>, local_offset: usize, label_count: usize,
    out: String,
}

impl MiniCC {
    pub fn new(source: &str) -> Self { Self { tokens: lex(source), pos: 0, locals: HashMap::new(), local_offset: 0, label_count: 0, out: String::new() } }
    fn peek(&self) -> Token { self.tokens[self.pos].clone() }
    fn consume(&mut self) -> Token { let t = self.peek(); if t != Token::EOF { self.pos += 1; } t }
    fn new_label(&mut self) -> String { self.label_count += 1; format!("L{}", self.label_count) }

    fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_mul();
        loop {
            match self.peek() {
                Token::Plus | Token::Minus | Token::Lt => {
                    let op = self.consume();
                    left = Expr::Binary(Box::new(left), op, Box::new(self.parse_mul()));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_mul(&mut self) -> Expr {
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
            Token::Mul => {
                self.consume(); // *
                Expr::Deref(Box::new(self.parse_unary()))
            }
            Token::Ampersand => {
                self.consume(); // &
                if let Token::Ident(name) = self.consume() {
                    Expr::AddrOf(name)
                } else { panic!("Expected identifier after &"); }
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Expr {
        match self.consume() {
            Token::Num(n) => Expr::Number(n),
            Token::Ident(s) => {
                if self.peek() == Token::LParen {
                    self.consume(); // (
                    let mut args = Vec::new();
                    if self.peek() != Token::RParen {
                        loop {
                            args.push(self.parse_expr());
                            if self.peek() == Token::Comma { self.consume(); } else { break; }
                        }
                    }
                    self.consume(); // )
                    Expr::Call(s, args)
                } else if self.peek() == Token::LBracket {
                    // Array sugar: arr[i] -> *(arr + i)
                    self.consume(); // [
                    let index = self.parse_expr();
                    self.consume(); // ]
                    // Multiply index by 8 (pointer width)
                    let offset = Expr::Binary(Box::new(index), Token::Mul, Box::new(Expr::Number(8)));
                    // We assume 's' is a pointer (address), so we Addr+Offset
                    let addr = Expr::Binary(Box::new(Expr::Variable(s)), Token::Plus, Box::new(offset));
                    Expr::Deref(Box::new(addr))
                } else { Expr::Variable(s) }
            }
            Token::LParen => {
                let e = self.parse_expr();
                self.consume(); // )
                e
            }
            t => panic!("Parser Error: {:?}", t),
        }
    }

    pub fn compile(&mut self) -> String {
        self.out.push_str("CALL main\nHALT\n");
        while self.peek() != Token::EOF {
            if self.peek() == Token::Int {
                self.consume(); // int
                // Handle pointers 'int * p' vs 'int p'
                while self.peek() == Token::Mul { self.consume(); } 
                let name = if let Token::Ident(s) = self.consume() { s } else { panic!() };
                if self.peek() == Token::LParen {
                    self.compile_func(name);
                }
            } else { self.consume(); }
        }
        self.out.clone()
    }

    fn compile_func(&mut self, _name: String) {
        self.consume(); // (
        self.out.push_str(&format!("{}:\n", _name));
        self.locals.clear(); self.local_offset = 0;
        
        let mut params = Vec::new();
        if self.peek() != Token::RParen {
            loop {
                if self.peek() == Token::Int { self.consume(); }
                while self.peek() == Token::Mul { self.consume(); }
                let pname = if let Token::Ident(s) = self.consume() { s } else { panic!() };
                params.push(pname);
                if self.peek() == Token::Comma { self.consume(); } else { break; }
            }
        }
        self.consume(); // )
        self.consume(); // {
        
        for pname in params.into_iter().rev() {
            self.locals.insert(pname, self.local_offset);
            self.out.push_str(&format!("LSTORE {}\n", self.local_offset));
            self.local_offset += 8;
        }
        
        while self.peek() != Token::RBrace && self.peek() != Token::EOF { self.compile_stmt(); }
        self.consume(); // }
        self.out.push_str("PUSH 0\nRET\n");
    }

    fn compile_stmt(&mut self) {
        match self.peek() {
            Token::Int => {
                self.consume();
                while self.peek() == Token::Mul { self.consume(); }
                let name = if let Token::Ident(s) = self.consume() { s } else { panic!() };
                if self.peek() == Token::Assign {
                    self.consume(); // =
                    let expr = self.parse_expr();
                    self.gen_expr(expr);
                    self.locals.insert(name.clone(), self.local_offset);
                    self.out.push_str(&format!("LSTORE {}\n", self.local_offset));
                    self.local_offset += 8;
                    self.consume(); // ;
                } else {
                    // int a; (uninitialized)
                    self.locals.insert(name.clone(), self.local_offset);
                    self.local_offset += 8;
                    self.consume(); // ;
                }
            }
            Token::Return => {
                self.consume();
                let expr = self.parse_expr();
                self.gen_expr(expr);
                self.out.push_str("RET\n");
                self.consume(); // ;
            }
            Token::If => {
                self.consume(); self.consume(); // if (
                let cond = self.parse_expr();
                self.consume(); // )
                self.gen_expr(cond);
                let lbl_end = self.new_label();
                self.out.push_str(&format!("JZ {}\n", lbl_end));
                if self.peek() == Token::LBrace {
                    self.consume();
                    while self.peek() != Token::RBrace { self.compile_stmt(); }
                    self.consume();
                } else { self.compile_stmt(); }
                self.out.push_str(&format!("{}:\n", lbl_end));
            }
            Token::Ident(s) => {
                self.consume();
                if self.peek() == Token::Assign {
                    self.consume(); // =
                    let expr = self.parse_expr();
                    self.gen_expr(expr);
                    let off = self.locals.get(&s).expect("Undefined var");
                    self.out.push_str(&format!("LSTORE {}\n", off));
                    self.consume(); // ;
                }
            }
            Token::Mul => {
                // *ptr = val;
                self.consume(); // *
                let ptr_expr = self.parse_unary(); // We expect an identifier or expression evaluating to addr
                self.consume(); // =
                let val_expr = self.parse_expr();
                
                self.gen_expr(val_expr); // Stack: [val]
                self.gen_expr(ptr_expr); // Stack: [val, addr]
                self.out.push_str("MSTORE\n");
                self.consume(); // ;
            }
            _ => { self.consume(); }
        }
    }

    fn gen_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Number(n) => self.out.push_str(&format!("PUSH {}\n", n)),
            Expr::Variable(s) => {
                let off = self.locals.get(&s).expect("Undefined var");
                self.out.push_str(&format!("LLOAD {}\n", off));
            }
            Expr::AddrOf(s) => {
                let off = self.locals.get(&s).expect("Undefined var");
                self.out.push_str("GETBP\n");
                self.out.push_str(&format!("PUSH {}\nADD\n", off));
            }
            Expr::Deref(e) => {
                self.gen_expr(*e);
                self.out.push_str("MLOAD\n");
            }
            Expr::Binary(l, op, r) => {
                self.gen_expr(*l); self.gen_expr(*r);
                match op {
                    Token::Plus => self.out.push_str("ADD\n"),
                    Token::Minus => self.out.push_str("SUB\n"),
                    Token::Mul => self.out.push_str("MUL\n"),
                    Token::Div => self.out.push_str("DIV\n"),
                    Token::Lt => self.out.push_str("LT\n"),
                    _ => unimplemented!(),
                }
            }
            Expr::Call(name, args) => {
                for arg in args { self.gen_expr(arg); }
                self.out.push_str(&format!("CALL {}\n", name));
            }
        }
    }
}

// --- ASSEMBLER ---
pub struct Assembler;
impl Assembler {
    pub fn compile_bef(source: &str) -> Vec<u8> {
        let tokens: Vec<&str> = source.split_whitespace().collect();
        let mut labels = HashMap::new();
        let mut addr = 0;
        for t in tokens.iter() {
            if t.ends_with(':') { labels.insert(t.trim_end_matches(':').to_string(), addr); }
            else { addr += match *t { 
                "PUSH"|"JMP"|"JZ"|"LLOAD"|"LSTORE"|"CALL" => 9, 
                "HALT"|"ADD"|"SUB"|"MUL"|"DIV"|"LT"|"RET"|"GETBP"|"MLOAD"|"MSTORE" => 1, 
                _ => 0 
            }; }
        }
        let mut code = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "HALT" => code.push(0x00),
                "PUSH" => { code.push(0x10); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "ADD" => code.push(0x20), "SUB" => code.push(0x21),
                "MUL" => code.push(0x22), "DIV" => code.push(0x23),
                "LT" => code.push(0x25),
                "JMP" => { code.push(0x30); i+=1; code.extend_from_slice(&(labels[tokens[i]] as u64).to_le_bytes()); }
                "JZ" => { code.push(0x31); i+=1; code.extend_from_slice(&(labels[tokens[i]] as u64).to_le_bytes()); }
                "CALL" => { code.push(0x40); i+=1; code.extend_from_slice(&(labels[tokens[i]] as u64).to_le_bytes()); }
                "RET" => code.push(0x42),
                "GETBP" => code.push(0x50),
                "LLOAD" => { code.push(0x60); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "LSTORE" => { code.push(0x61); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "MLOAD" => code.push(0x62),
                "MSTORE" => code.push(0x63),
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

// --- VM ---
pub struct Machine {
    pub memory: Vec<u8>, pub stack: Vec<u64>, pub call_stack: Vec<(usize, usize)>,
    pub ip: usize, pub bp: usize, pub sp: usize,
}
impl Machine {
    pub fn new() -> Self { Self { memory: vec![0; 8192], stack: vec![], call_stack: vec![], ip: 0, bp: 4096, sp: 4096 } }
    pub fn load(&mut self, d: &[u8]) { 
        let sz = u32::from_le_bytes(d[8..12].try_into().unwrap()) as usize;
        self.memory[0..sz].copy_from_slice(&d[16..16+sz]);
    }
    pub fn step(&mut self) -> Result<bool, String> {
        let op = self.memory[self.ip]; self.ip += 1;
        match op {
            0x00 => return Ok(false),
            0x10 => { self.stack.push(u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap())); self.ip += 8; }
            0x20 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a.wrapping_add(b)); }
            0x21 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a.wrapping_sub(b)); }
            0x22 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a.wrapping_mul(b)); }
            0x23 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a.wrapping_div(b)); }
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
            _ => return Err("Err".into()),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(SYSTEM_STATUS);
    report.push_str("TEST: POINTERS_AND_MEMORY ... ");
    
    // Test: int a = 10; int *p = &a; *p = 50; return a; -> Should be 50.
    let src = "
    int main() {
        int a = 10;
        int *p = &a;
        *p = 50;
        return a;
    }
    ";
    
    let mut cc = MiniCC::new(src);
    let bin = Assembler::compile_bef(&cc.compile());
    let mut vm = Machine::new();
    vm.load(&bin);
    
    let mut f = 5000;
    while f > 0 && vm.step().unwrap_or(false) { f -= 1; }
    
    if let Some(&ans) = vm.stack.last() {
        if ans == 50 { report.push_str("PASS\n"); }
        else { report.push_str(&format!("FAIL (Returned {})\n", ans)); }
    } else { report.push_str("FAIL (NO RET)\n"); }
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
