use abi::vm::{Machine, VMStatus};
use abi::isa::Opcode;

fn main() {
    println!("vfs-core: CLI Shell // Booting VM...");

    // Hand-assembled program: 10 + 20
    let program = vec![
        Opcode::Push as u8, 
        10, 0, 0, 0, 0, 0, 0, 0, // 64-bit '10'
        Opcode::Push as u8, 
        20, 0, 0, 0, 0, 0, 0, 0, // 64-bit '20'
        Opcode::Add as u8,
        Opcode::Halt as u8
    ];

    let mut vm = Machine::new(program);
    let mut cycles = 0;

    loop {
        match vm.step() {
            Ok(VMStatus::Running) => { cycles += 1; }
            Ok(VMStatus::Halted) => {
                println!("VM Halted normally after {} cycles.", cycles);
                println!("Stack Top: {:?}", vm.stack.last());
                break;
            }
            Ok(VMStatus::Syscall(id)) => {
                println!("Syscall Requested: {}", id);
                // In future: handle syscall and resume
            }
            Err(e) => {
                println!("VM CRASH: {}", e);
                break;
            }
        }
    }
}
