// Virtual File System (VFS)
// The absolute root of reality for the execution substrate.
// Ambient OS access is strictly forbidden here.

pub trait Vfs {
    fn read(&self, path: &str) -> Result<Vec<u8>, VfsError>;
    fn write(&mut self, path: &str, data: &[u8]) -> Result<(), VfsError>;
}

#[derive(Debug)]
pub enum VfsError {
    NotFound,
    PermissionDenied,
}
