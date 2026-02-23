use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Opcode {
    // --- Core Execution ---
    Halt = 0x00,    // Stop execution
    NoOp = 0x01,    // Do nothing

    // --- Stack Manipulation ---
    Push = 0x10,    // Push a 64-bit immediate onto the stack
    Pop  = 0x11,    // Pop value, discard
    Dup  = 0x12,    // Duplicate top of stack

    // --- Arithmetic (Integers) ---
    Add  = 0x20,    // Pop a, Pop b, Push a + b
    Sub  = 0x21,
    Mul  = 0x22,
    Div  = 0x23,

    // --- Memory ---
    Load  = 0x30,   // Pop addr, Push val from heap
    Store = 0x31,   // Pop val, Pop addr, write val to heap

    // --- Control Flow ---
    Jmp   = 0x40,   // Unconditional jump
    Jz    = 0x41,   // Jump if zero (pop condition)
    Call  = 0x42,   // Call function at address
    Ret   = 0x43,   // Return from function

    // --- System Interface ---
    Syscall = 0xF0, // Pop syscall_id, execute system service
}

// The header for our executable format
#[derive(Serialize, Deserialize, Debug)]
pub struct ProgramHeader {
    pub magic: u32,       // 0x5052494D ("PRIM")
    pub entry_point: u64, // Instruction pointer start
    pub code_size: u64,   // Size of code section
    pub data_size: u64,   // Size of static data section
}
