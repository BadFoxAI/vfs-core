# DRE Roadmap: The Path to TCC

## [STABLE] Era 1: The Sovereign Substrate
- [x] Phase 1-4: Deterministic VM & Custom ABI.
- [x] Phase 5-7: Bootstrap Language, VFS, and Self-Hosting Simulation.
- [x] Phase 8: Hardening (Bounds checking & Gas metering).

## [CURRENT] Era 2: The Industrial Bridge (Target: Compile TCC)
*Philosophy: We do not want a shell until we can compile a real one. MiniCC exists solely to compile TCC.*
- [ ] **Phase 9: The Preprocessor & Unity Builds.** Implement `#include` so MiniCC can compile the scattered TCC source files as a single massive file, bypassing the need for a linker.
- [ ] **Phase 10: The Deception Layer (Libc Shim).** TCC requires standard C functions. We must write `malloc`, `free`, `fopen`, `fread`, and `printf` in MiniCC using our raw VM syscalls (`sbrk`, `read`, `write`).
- [ ] **Phase 11: Advanced C Features.** Add support for function pointers, `typedef`, and `enum` to MiniCC (these are heavily used in TCC's parser).
- [ ] **Phase 12: The Threshold.** Successfully compile `tcc.c` using MiniCC.

## [UPCOMING] Era 3: The Earned System
*Goal: Use our newly compiled, sovereign TCC to build real software.*
- [ ] Phase 13: Boot the TCC binary inside the VM.
- [ ] Phase 14: Compile a real POSIX shell (e.g., `ash` or `dash`) from source using TCC.
- [ ] Phase 15: Attach the standard I/O of the compiled shell to the host web terminal.

## Resolved Issues Log
- **WASM Infinite Loop:** Caused by MiniCC lacking scope handling. *Fix:* Hoisted variables to function top.
- **Array Garbage Values:** *Fix:* Implemented Array Pointer Decay in `gen_expr`.
- **Dynamic Memory:** Verified `sbrk` syscall (4) works. We now have a heap foundation for Era 2.
