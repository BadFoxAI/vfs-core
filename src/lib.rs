use wasm_bindgen::prelude::*;
use std::collections::HashMap;

pub const SYSTEM_STATUS: &str = r#"
================================================================================
DRE // DETERMINISTIC RUNTIME ENVIRONMENT
================================================================================
[ GOLD MASTER STABLE ]
[ BIG BITE 1: AST & EXPRESSION ENGINE ]
Status: Lexer + Recursive Descent Parser Active.
"#;

// --- LEXER ---
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Int, Str, If, Else, While, Return, Exec, Putchar, Syscall, Poke,
    Ident(String), Num(u64), Lit(String),
    Plus, Minus, Mul, Div, Assign, Eq, Lt,
    LParen, RParen, LBrace, RBrace, Semicolon, Comma, EOF
}

fn lex(src: &str) -> Vec<Token> {
    let s = src.replace("(", " ( ").replace(")", " ) ")
               .replace("{", " { ").replace("}", " } ")
               .replace(";", " ; ").replace(",", " , ")
               .replace("+", " + ").replace("-", " - ")
               .replace("*", " * ").replace("/", " / ")
               .replace("=", " = ").replace("<", " < ");
    let mut tokens = Vec::new();
    let words: Vec<&str> = s.split_whitespace().collect();
    for w in words {
        match w {
            "int" => tokens.push(Token::Int),
            "str" => tokens.push(Token::Str),
            "return" => tokens.push(Token::Return),
            "exec" => tokens.push(Token::Exec),
            "putchar" => tokens.push(Token::Putchar),
            "syscall" => tokens.push(Token::Syscall),
            "poke" => tokens.push(Token::Poke),
            "while" => tokens.push(Token::While),
            "+" => tokens.push(Token::Plus),
            "-" => tokens.push(Token::Minus),
            "*" => tokens.push(Token::Mul),
            "/" => tokens.push(Token::Div),
            "=" => tokens.push(Token::Assign),
            "<" => tokens.push(Token::Lt),
            "(" => tokens.push(Token::LParen),
            ")" => tokens.push(Token::RParen),
            "{" => tokens.push(Token::LBrace),
            "}" => tokens.push(Token::RBrace),
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
    Syscall(u64, Vec<Expr>),
}

// --- COMPILER ---
pub struct MiniCC {
    tokens: Vec<Token>,
    pos: usize,
    locals: HashMap<String, usize>,
    local_offset: usize,
    out: String,
}

impl MiniCC {
    pub fn new(source: &str) -> Self {
        Self { tokens: lex(source), pos: 0, locals: HashMap::new(), local_offset: 0, out: String::new() }
    }

    fn peek(&self) -> Token { self.tokens[self.pos].clone() }
    fn consume(&mut self) -> Token {
        let t = self.peek();
        if t != Token::EOF { self.pos += 1; }
        t
    }

    // Grammar: Expr -> Mul ( (Plus|Minus) Mul )*
    fn parse_expr(&mut self) -> Expr {
        let mut left = self.parse_mul();
        loop {
            match self.peek() {
                Token::Plus | Token::Minus | Token::Lt => {
                    let op = self.consume();
                    let right = self.parse_mul();
                    left = Expr::Binary(Box::new(left), op, Box::new(right));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_mul(&mut self) -> Expr {
        let mut left = self.parse_primary();
        loop {
            match self.peek() {
                Token::Mul | Token::Div => {
                    let op = self.consume();
                    let right = self.parse_primary();
                    left = Expr::Binary(Box::new(left), op, Box::new(right));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_primary(&mut self) -> Expr {
        match self.consume() {
            Token::Num(n) => Expr::Number(n),
            Token::Ident(s) => Expr::Variable(s),
            Token::LParen => {
                let e = self.parse_expr();
                self.consume(); // RParen
                e
            }
            _ => panic!("Parser Error"),
        }
    }

    pub fn compile(&mut self) -> String {
        while self.peek() != Token::EOF {
            match self.peek() {
                Token::Int => {
                    self.consume();
                    let name = if let Token::Ident(s) = self.consume() { s } else { panic!() };
                    if self.peek() == Token::LParen {
                        // skip function main() for now
                        while self.consume() != Token::LBrace {}
                    } else {
                        self.consume(); // =
                        let expr = self.parse_expr();
                        self.gen_expr(expr);
                        self.locals.insert(name.clone(), self.local_offset);
                        self.out.push_str(&format!("LSTORE {}\n", self.local_offset));
                        self.local_offset += 8;
                        self.consume(); // ;
                    }
                }
                Token::Return => {
                    self.consume();
                    let expr = self.parse_expr();
                    self.gen_expr(expr);
                    self.out.push_str("PUSH 1 SYSCALL HALT\n");
                    self.consume(); // ;
                }
                Token::RBrace => { self.consume(); }
                _ => { self.consume(); }
            }
        }
        self.out.clone()
    }

    fn gen_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Number(n) => self.out.push_str(&format!("PUSH {}\n", n)),
            Expr::Variable(s) => {
                let off = self.locals.get(&s).expect("Undefined var");
                self.out.push_str(&format!("LLOAD {}\n", off));
            }
            Expr::Binary(l, op, r) => {
                self.gen_expr(*l);
                self.gen_expr(*r);
                match op {
                    Token::Plus => self.out.push_str("ADD\n"),
                    Token::Lt => self.out.push_str("LT\n"),
                    _ => unimplemented!("Op Not Implemented"),
                }
            }
            _ => {}
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
        for (i, &t) in tokens.iter().enumerate() {
            if t.ends_with(':') { labels.insert(t.trim_end_matches(':').to_string(), addr); }
            else { addr += match t { "PUSH"|"JMP"|"JZ"|"LLOAD"|"LSTORE"|"CALL" => 9, "HALT"|"ADD"|"LT"|"SYSCALL"|"RET" => 1, _ => 0 }; }
        }
        let mut code = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "HALT" => code.push(0x00),
                "PUSH" => { code.push(0x10); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "ADD" => code.push(0x20),
                "LT" => code.push(0x25),
                "LLOAD" => { code.push(0x60); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "LSTORE" => { code.push(0x61); i+=1; code.extend_from_slice(&tokens[i].parse::<u64>().unwrap().to_le_bytes()); }
                "SYSCALL" => code.push(0xF0),
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
    pub memory: Vec<u8>, pub stack: Vec<u64>, pub ip: usize, pub bp: usize, pub vfs: HashMap<String, Vec<u8>>,
}
impl Machine {
    pub fn new() -> Self { Self { memory: vec![0; 8192], stack: vec![], ip: 0, bp: 4096, vfs: HashMap::new() } }
    pub fn load(&mut self, d: &[u8]) { 
        let sz = u32::from_le_bytes(d[8..12].try_into().unwrap()) as usize;
        self.memory[0..sz].copy_from_slice(&d[16..16+sz]);
    }
    pub fn step(&mut self) -> Result<bool, String> {
        let op = self.memory[self.ip]; self.ip += 1;
        match op {
            0x00 => return Ok(false),
            0x10 => { self.stack.push(u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap())); self.ip += 8; }
            0x20 => { let b = self.stack.pop().unwrap(); let a = self.stack.pop().unwrap(); self.stack.push(a+b); }
            0x60 => { let off = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; self.ip += 8; self.stack.push(u64::from_le_bytes(self.memory[self.bp+off..self.bp+off+8].try_into().unwrap())); }
            0x61 => { let off = u64::from_le_bytes(self.memory[self.ip..self.ip+8].try_into().unwrap()) as usize; self.ip += 8; let v = self.stack.pop().unwrap(); self.memory[self.bp+off..self.bp+off+8].copy_from_slice(&v.to_le_bytes()); }
            0xF0 => { let id = self.stack.pop().unwrap(); if id == 1 { let v = self.stack.pop().unwrap(); self.vfs.insert("ret".into(), v.to_le_bytes().to_vec()); } }
            _ => return Err("Err".into()),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(SYSTEM_STATUS);
    report.push_str("TEST: AST_EXPRESSION_PARSING ... ");
    
    // We test nested addition: (10 + 20) + 12 = 42
    let src = "int main() { int a = 10 + 20 ; return a + 12 ; }";
    let mut cc = MiniCC::new(src);
    let asm = cc.compile();
    let bin = Assembler::compile_bef(&asm);
    let mut vm = Machine::new();
    vm.load(&bin);
    let mut f = 1000;
    while f > 0 && vm.step().unwrap_or(false) { f -= 1; }
    
    if let Some(r) = vm.vfs.get("ret") {
        if r[0] == 42 { report.push_str("PASS\n"); }
        else { report.push_str(&format!("FAIL ({})\n", r[0])); }
    } else { report.push_str("FAIL (IO)\n"); }
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String { run_suite() }
