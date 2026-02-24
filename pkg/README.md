# DRE: The Sovereign Substrate

DRE (Deterministic Runtime Environment) is not a programming project; it is an industrial substrate designed to host stable, legacy software in a completely deterministic, sovereign environment.

## The Philosophy: "Earned, Not Hacked"
We reject the practice of building "toy" versions of system tools. We do not program our own text editors or terminal emulators. Instead, we focus exclusively on building the minimal deterministic substrate required to **host** the original, rock-solid source code of established tools (like TCC, Nano, and Vim).

We only deserve a terminal when we can compile and install one.

## Core Architecture
1. **The VFS (Absolute Truth):** All inputs and artifacts exist here. Ambient access to the host OS is forbidden.
2. **The ABI (The Contract):** A minimal syscall surface for memory, I/O, and execution.
3. **The Host Shell:** A thin driver (WASM/CLI) that maps the ABI to a screen.
4. **The Bootstrap (MiniCC):** A custom, scratch-built C-subset compiler currently residing in `lib.rs`. Its sole purpose is to become capable enough to compile TinyCC.

## The Path to Sovereignty
*   **Era 1: Substrate (Complete):** Establishing the VM, the Bytecode, and the minimal C-subset compiler.
*   **Era 2: TCC Bootstrap (Current):** Upgrading MiniCC with preprocessor `#include` and a fake POSIX `libc` shim to compile the Tiny C Compiler (TCC) from raw source code.
*   **Era 3: The Threshold:** Loading, compiling, and executing a real terminal and editor from pure VFS source code using our newly earned TCC.

Everything else is scaffolding.
