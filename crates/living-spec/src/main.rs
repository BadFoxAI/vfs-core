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
[ ] 1.5 Verify ABI Parity (CLI vs WASM).

PHASE 2: THE LOADING DOCK
[ ] 2.1 Port TinyCC (tcc) backend to Sovereign ABI.
[ ] 2.2 Compile tcc using First Mover Compiler.
[ ] 2.3 Store tcc artifact in VFS.

PHASE 3: THE DECEPTION
[ ] 3.1 Implement POSIX Emulation Layer (libc-over-ABI).
[ ] 3.2 Map open/read/write to VFS.

[ INCIDENT LOG & RESOLUTIONS ]
--------------------------------------------------------------------------------
INCIDENT #001: Netlify Build Failure
  - Status: RESOLVED (See: build.sh).

================================================================================
UNIT TEST SUITE // AUTOMATED VERIFICATION
================================================================================
"#;

fn main() {
    println!("{}", MANIFESTO);
    let mut passed = 0;
    let mut failed = 0;

    run_test("ABI_OPCODE_MAPPING", test_abi_opcodes, &mut passed, &mut failed);
    run_test("ABI_STRUCTURE_integrity", test_abi_structure, &mut passed, &mut failed);
    run_test("VM_ARITHMETIC_ADD", test_vm_add, &mut passed, &mut failed);

    println!("\n--------------------------------------------------------------------------------");
    if failed == 0 {
        println!("{}", "ALL SYSTEMS NOMINAL. READY FOR NEXT CYCLE.".green().bold());
    } else {
        println!("{}", format!("SYSTEM FAILURE. {} TESTS FAILED.", failed).red().bold());
        std::process::exit(1);
    }
}

// --- TEST INFRASTRUCTURE ---

fn run_test<F>(name: &str, test_fn: F, passed: &mut i32, failed: &mut i32)
where F: Fn() -> Result<(), String> {
    print!("TEST: {:<30} ... ", name);
    match test_fn() {
        Ok(_) => {
            println!("{}", "PASS".green());
            *passed += 1;
        }
        Err(e) => {
            println!("{}", "FAIL".red());
            println!("  -> REASON: {}", e);
            *failed += 1;
        }
    }
}

// --- UNIT TESTS ---

fn test_abi_opcodes() -> Result<(), String> {
    if (Opcode::Halt as u8) != 0x00 { return Err("Halt != 0x00".into()); }
    if (Opcode::Syscall as u8) != 0xF0 { return Err("Syscall != 0xF0".into()); }
    if (Opcode::Add as u8) != 0x20 { return Err("Add != 0x20".into()); }
    Ok(())
}

fn test_abi_structure() -> Result<(), String> {
    let header = abi::isa::ProgramHeader {
        magic: 0x5052494D,
        entry_point: 0,
        code_size: 1024,
        data_size: 0,
    };
    if header.magic != 0x5052494D { return Err("Magic Number Mismatch".into()); }
    Ok(())
}

fn test_vm_add() -> Result<(), String> {
    // Construct program: PUSH 10, PUSH 20, ADD, HALT
    let program = vec![
        Opcode::Push as u8, 
        10, 0, 0, 0, 0, 0, 0, 0, 
        Opcode::Push as u8, 
        20, 0, 0, 0, 0, 0, 0, 0, 
        Opcode::Add as u8,
        Opcode::Halt as u8
    ];

    let mut vm = Machine::new(program);
    let mut steps = 0;
    loop {
        match vm.step() {
            Ok(VMStatus::Running) => { steps += 1; }
            Ok(VMStatus::Halted) => break,
            Ok(VMStatus::Syscall(_)) => return Err("Unexpected Syscall".into()),
            Err(e) => return Err(e),
        }
        if steps > 100 { return Err("Infinite Loop Detect".into()); }
    }

    let result = *vm.stack.last().ok_or("Stack empty")?;
    if result != 30 {
        return Err(format!("Expected 30, got {}", result));
    }
    Ok(())
}
