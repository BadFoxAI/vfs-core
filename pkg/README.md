# DRE: The Sovereign Substrate

DRE (Deterministic Runtime Environment) is not a programming project; it is an industrial substrate designed to host stable, legacy software in a completely deterministic, sovereign environment.

## The Philosophy: "Earned, Not Hacked"
We reject the practice of building "toy" versions of system tools. We do not program our own text editors or terminal emulators. Instead, we focus exclusively on building the minimal deterministic substrate required to **host** the original, rock-solid source code of established tools (like TCC, Nano, and Vim).

We only deserve a terminal when we can compile and install one.

## Core Architecture
1. **The VFS (Absolute Truth):** All inputs and artifacts exist here. Ambient access to the host OS is forbidden.
2. **The ABI (The Contract):** A minimal syscall surface for memory, I/O, and execution.
3. **The Host Shell:** A thin driver (WASM/CLI) that maps the ABI to a screen or terminal emulator.

## The Path to Sovereignty
*   **Era 1: Substrate (Complete):** Establishing the VM, the Bytecode, and the minimal C-subset compiler.
*   **Era 2: The Industrial Bridge (Current):** Porting TinyCC (TCC) to the DRE ABI to gain full C99 compliance.
*   **Era 3: The Deception Layer:** Implementing a POSIX-compliant `libc` shim to trick legacy software into running natively.
*   **Era 4: The Threshold:** Loading, compiling, and executing a real terminal and editor from pure VFS source code.

Everything else is scaffolding.
