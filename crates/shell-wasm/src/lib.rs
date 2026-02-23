use wasm_bindgen::prelude::*;

// We need to verify VFS linking later, for now, let's just get the WASM binary executing.
// use vfs::Vfs; 

#[wasm_bindgen]
pub fn init_shell() -> String {
    "vfs-core: WASM Shell Online. Artifact executable.".to_string()
}
