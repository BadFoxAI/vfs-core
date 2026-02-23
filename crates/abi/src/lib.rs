// Internal Application Binary Interface (ABI)
// Defines the minimal execution contract and syscall surface.
// This is not WASM. This is not POSIX.

pub const SYSCALL_VFS_READ: u32 = 1;
pub const SYSCALL_VFS_WRITE: u32 = 2;
pub const SYSCALL_MEM_ALLOC: u32 = 3;
pub const SYSCALL_DEBUG_OUT: u32 = 4;
