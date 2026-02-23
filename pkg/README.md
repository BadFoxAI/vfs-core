# vfs-core: Sovereign Substrate

This repository defines the architecture of a completely sovereign, deterministic compilation and execution environment. It is built on an "engine shop" ethos: we do not rewrite existing software ecosystems; we build the minimal deterministic substrate required to host them.

## Core Architecture

The system is defined by three strict layers. External operating systems and environments are treated strictly as hardware drivers.

1. **The Virtual File System (VFS):** The absolute root of reality. All compilation inputs, module resolution, standard library access, and build artifacts exist exclusively within the VFS. Ambient access to the host OS is forbidden.
2. **The Runtime Contract (Internal ABI):** A proprietary, minimal execution format designed for systems-level semantics. It defines a strictly bounded syscall surface for memory, VFS I/O, and execution. It explicitly excludes host-dependent concepts like OS threading or process spawning.
3. **The Host Shells:** Thin drivers (e.g., native CLI, WebAssembly) that ingest ABI artifacts and execute them. They do not alter semantics; they solely map the ABI syscall surface to the underlying host.

## The Bootstrap Trajectory

We reject the compulsion to rewrite C compilers, LLVM, or Rust. The bootstrap sequence leverages existing infrastructure to pull the legacy world into our sovereign environment:

*   **Phase 1 (First Mover):** A minimal compiler (written in Rust) targeting a basic systems language, operating strictly against the VFS and emitting the Internal ABI.
*   **Phase 2 (Loading Dock):** Porting the backend of a tiny C compiler (e.g., `tcc`) to emit the Internal ABI, and compiling it via the First Mover.
*   **Phase 3 (Deception Layer):** Implementing a strict POSIX emulation layer on top of the Internal ABI's minimal syscall surface.
*   **Phase 4 (Ignition):** Using the ported C compiler and POSIX emulation to compile legacy GCC/Clang, modern LLVM, and modern Rustc entirely within the VFS.

The platform is the VFS, the ABI, and the runtime contract. Everything else is scaffolding.

## Deployment
**WASM Shell:** [vfs-core.netlify.app](https://vfs-core.netlify.app)
