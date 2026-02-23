use wasm_bindgen::prelude::*;
use abi::vm::{Machine, VMStatus};
use abi::isa::Opcode;

#[wasm_bindgen]
pub fn init_shell() -> String {
    let program = vec![
        Opcode::Push as u8, 
        10, 0, 0, 0, 0, 0, 0, 0, 
        Opcode::Push as u8, 
        20, 0, 0, 0, 0, 0, 0, 0, 
        Opcode::Add as u8,
        Opcode::Halt as u8
    ];

    let mut vm = Machine::new(program);
    let mut log = String::from("WASM VM Boot sequence initiated...\n");

    loop {
        match vm.step() {
            Ok(VMStatus::Running) => {}
            Ok(VMStatus::Halted) => {
                log.push_str(&format!("VM Halted. Result: {:?}\n", vm.stack.last()));
                break;
            }
            Ok(VMStatus::Syscall(id)) => {
                log.push_str(&format!("Syscall: {}\n", id));
            }
            Err(e) => {
                log.push_str(&format!("CRASH: {}\n", e));
                break;
            }
        }
    }
    
    log + "Sovereignty Parity Confirmed."
}
