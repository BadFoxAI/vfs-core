use wasm_bindgen::prelude::*;

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
                let bytes = self.program[self.ip..self.ip+8].try_into().unwrap();
                self.ip += 8;
                self.stack.push(u64::from_le_bytes(bytes));
            }
            0x20 => {
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();
                self.stack.push(a + b);
            }
            _ => return Err("Invalid Opcode".into()),
        }
        Ok(true)
    }
}

#[wasm_bindgen]
pub fn init_shell() -> String {
    let mut vm = Machine::new(vec![0x10, 10, 0, 0, 0, 0, 0, 0, 0, 0x10, 20, 0, 0, 0, 0, 0, 0, 0, 0x20, 0x00]);
    while let Ok(true) = vm.step() {}
    format!("VFS-CORE: VM Result = {}", vm.stack.last().unwrap_or(&0))
}
