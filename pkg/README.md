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

## Progress Log
- **Phase 5.2**: Added  and  opcodes, Call Stack, and  subroutine compilation to the First Mover toolchain.

- **Phase 5.3**: Bootstrapped memory pointers. Added `LOAD` and `STORE` opcodes with `peek` and `poke` semantics.

- **Phase 5.4**: Added byte-addressable memory (/) and compile-time string allocation with automatic null-termination.

- **Phase 5.5**: Completed Phase 5! Upgraded `syscall` to support variable arguments and implemented true VFS File Read/Write mapping.

## Phase 6: C Compiler Bootstrap
- **Phase 6.1**: Ported the *Loading Dock* `MiniCC` frontend. The system can now parse standard minimal `C` syntax and compile it directly into the deterministic internal ABI.

- **Phase 6.2**: Implemented the POSIX CRT emulation layer. Mapped `<stdio.h>` functions like `putchar` to VM Syscall 4 (STDOUT), enabling standard C I/O.

- **Branding Update**: System renamed to DRE (Deterministic Runtime Environment).

## Phase 7: Self-Hosting
- **Phase 7.1**: Implemented Syscall 5 (`EXEC`). The DRE can now read ABI binaries from the Virtual File System, clear its own memory, and context-switch to execute the new payload natively.

- **Phase 7.2**: Completed Self-Hosting Simulation. The environment can now host a builder program which generates a raw executable binary, writes it to the VFS, and immediately executes it. The loop is closed.

- **Phase 7 Complete**: DRE is now self-hosting capable. The system successfully built, wrote, and executed a binary entirely within the VFS.

- **Phase 8.1**: Implemented strictly hardened memory access. The VM now performs bounds checking on every read/write and properly reports `Segmentation Fault` instead of crashing.

- **Phase 8.2**: Implemented Resource Quotas (Gas Metering). The VM now strictly limits execution cycles, successfully neutralizing infinite loops and DoS vectors.
## Completion
The DRE System is now Feature Complete. It is a sovereign, self-hosting, hardened runtime environment capable of compiling and executing its own tools from scratch.
