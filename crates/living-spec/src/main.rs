use colored::*;
use abi::isa::Opcode;
use abi::vm::{Machine, VMStatus};

const MANIFESTO: &str = r#"
================================================================================
SOVEREIGN ENGINE SHOP // LIVING SPECIFICATION & ROADMAP
================================================================================

[ MISSION ]
Build a sovereign, deterministic execution substrate (VFS + ABI) that serves as 
the "First Mover" to bootstrap existing ecosystems (C -> LLVM -> Rust) without 
dependency on host OS semantics.

[ ROADMAP ]
--------------------------------------------------------------------------------
PHASE 1: THE SUBSTRATE (Current)
[x] 1.1 Scaffold Workspace (VFS, ABI, Compiler, Shells).
[x] 1.2 Establish CI/CD Pipeline (Netlify WASM Distro).
[x] 1.3 Define Internal ABI / ISA (Opcodes).
[x] 1.4 Implement VM Runtime (The Loop).
[~] 1.5 Verify ABI Parity (CLI vs WASM).
    -> CLI Shell: VERIFIED LOCAL (10+20=30).
    -> WASM Shell: PENDING DEPLOYMENT.

PHASE 2: THE LOADING DOCK
[ ] 2.1 Port TinyCC (tcc) backend to Sovereign ABI.

[ INCIDENT LOG ]
--------------------------------------------------------------------------------
INCIDENT #002: Parity Mismatch
  - Problem: WASM Shell deployed old code. CLI Shell ambiguous run command.
  - Resolution: Hard rewrite of WASM lib.rs. Explicit CLI execution instruction.

================================================================================
UNIT TEST SUITE
================================================================================
"#;

fn main() {
    println!("{}", MANIFESTO);
    let mut passed = 0;
    let mut failed = 0;

    run_test("ABI_OPCODE_MAPPING", test_abi_opcodes, &mut passed, &mut failed);
    run_test("VM_ARITHMETIC_ADD", test_vm_add, &mut passed, &mut failed);

    println!("\n--------------------------------------------------------------------------------");
    if failed == 0 {
        println!("{}", "ALL SYSTEMS NOMINAL.".green().bold());
    } else {
        std::process::exit(1);
    }
}

// --- TEST INFRASTRUCTURE ---

fn run_test<F>(name: &str, test_fn: F, passed: &mut i32, failed: &mut i32)
where F: Fn() -> Result<(), String> {
    print!("TEST: {:<30} ... ", name);
    match test_fn() {
        Ok(_) => { println!("{}", "PASS".green()); *passed += 1; }
        Err(e) => { println!("{}", "FAIL".red()); println!("  -> {}", e); *failed += 1; }
    }
}

fn test_abi_opcodes() -> Result<(), String> {
    if (Opcode::Halt as u8) != 0x00 { return Err("Halt != 0x00".into()); }
    if (Opcode::Add as u8) != 0x20 { return Err("Add != 0x20".into()); }
    Ok(())
}

fn test_vm_add() -> Result<(), String> {
    let program = vec![
        Opcode::Push as u8, 10, 0, 0, 0, 0, 0, 0, 0, 
        Opcode::Push as u8, 20, 0, 0, 0, 0, 0, 0, 0, 
        Opcode::Add as u8, Opcode::Halt as u8
    ];
    let mut vm = Machine::new(program);
    loop {
        match vm.step() {
            Ok(VMStatus::Running) => {}
            Ok(VMStatus::Halted) => break,
            _ => return Err("Unexpected State".into()),
        }
    }
    if *vm.stack.last().unwrap() != 30 { return Err("Math failed".into()); }
    Ok(())
}
