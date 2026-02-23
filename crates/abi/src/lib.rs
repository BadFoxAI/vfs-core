// Internal Application Binary Interface (ABI)
pub mod isa;

// System Call Constants
// These match the 'Syscall' opcode arguments.
pub const SYSCALL_VFS_READ: u64 = 1;
pub const SYSCALL_VFS_WRITE: u64 = 2;
pub const SYSCALL_MEM_ALLOC: u64 = 3;
pub const SYSCALL_DEBUG_OUT: u64 = 4;
pub mod vm;
