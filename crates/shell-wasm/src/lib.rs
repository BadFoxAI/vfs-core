use wasm_bindgen::prelude::*;
use vfs::Vfs;

// The WASM Host Shell
// Maps Internal ABI syscalls to Browser APIs.

#[wasm_bindgen]
pub fn init_shell() -> String {
    // This will eventually hook into the VFS and ABI
    "vfs-core: WASM Shell Initialized. Sovereignty established.".to_string()
}
