use vfs_core::{Machine};

fn main() {
    let mut vm = Machine::new(vec![0x10, 10, 0, 0, 0, 0, 0, 0, 0, 0x10, 20, 0, 0, 0, 0, 0, 0, 0, 0x20, 0x00]);
    while let Ok(true) = vm.step() {}
    println!("CLI Result: {}", vm.stack.last().unwrap_or(&0));
}
