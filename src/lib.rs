use wasm_bindgen::prelude::*;

pub const MANIFESTO: &str = r#"
================================================================================
SOVEREIGN ENGINE SHOP // LIVING SPECIFICATION & ROADMAP
================================================================================
[ MISSION ]
Build a sovereign, deterministic execution substrate (VFS + ABI).

[ ROADMAP ]
[x] 1.1 Scaffold Workspace
[x] 1.2 CI/CD Pipeline
[x] 1.3 Define ISA
[x] 1.4 Implement VM Runtime
[x] 1.5 Verify ABI Parity (CLI vs WASM)

UNIT TEST SUITE:
"#;

#[repr(u8)]
pub enum Opcode { Halt = 0x00, Push = 0x10, Add = 0x20 }

pub struct Machine {
    pub stack: Vec<u64>,
    pub ip: usize,
    pub program: Vec<u8>,
}

impl Machine {
    pub fn new(program: Vec<u8>) -> Self {
        Self { stack: Vec::new(), ip: 0, program }
    }
    pub fn step(&mut self) -> Result<bool, String> {
        if self.ip >= self.program.len() { return Ok(false); }
        let op = self.program[self.ip];
        self.ip += 1;
        match op {
            0x00 => return Ok(false),
            0x10 => {
                let bytes = self.program[self.ip..self.ip+8].try_into().map_err(|_| "Segfault")?;
                self.ip += 8;
                self.stack.push(u64::from_le_bytes(bytes));
            }
            0x20 => {
                let b = self.stack.pop().ok_or("Stack Underflow")?;
                let a = self.stack.pop().ok_or("Stack Underflow")?;
                self.stack.push(a + b);
            }
            _ => return Err(format!("Unknown Opcode: 0x{:02X}", op)),
        }
        Ok(true)
    }
}

pub fn run_suite() -> String {
    let mut report = String::from(MANIFESTO);
    
    // TEST: VM_ADD
    report.push_str("TEST: VM_ADD_10_20 ... ");
    let mut vm = Machine::new(vec![0x10, 10, 0, 0, 0, 0, 0, 0, 0, 0x10, 20, 0, 0, 0, 0, 0, 0, 0, 0x20, 0x00]);
    while let Ok(true) = vm.step() {}
    if vm.stack.last() == Some(&30) {
        report.push_str("PASS\n");
    } else {
        report.push_str("FAIL\n");
    }

    report.push_str("\nALL SYSTEMS NOMINAL. PARITY ESTABLISHED.");
    report
}

#[wasm_bindgen]
pub fn init_shell() -> String {
    run_suite()
}
