# DRE Roadmap: The Path to C Bootstrap

## Completed
- [x] Phase 1-4: Deterministic VM Substrate & Custom ABI.
- [x] Phase 5: Bootstrap Language (v0) & File IO.
- [x] Phase 6: POSIX Shim (stdio.h / putchar).
- [x] Phase 7: Self-Hosting (Builder -> Bin -> Exec).
- [x] Phase 8: Hardening (Bounds checking & Gas metering).

## Current: BIG BITE Series
### [Bite 1] AST & Expression Engine (Current)
- [ ] Implement Lexer & Recursive Descent Parser.
- [ ] Support complex expressions with precedence: `(a + b) * c`.
- [ ] Replace naive linear compiler with AST-based code generation.

### [Bite 2] Stack Frames & Scoping
- [ ] Implement Base Pointer (`BP`) relative addressing.
- [ ] Support local variables and recursive function calls.
- [ ] Implement function prologues/epilogues in backend.

### [Bite 3] Pointers, Arrays & Heap
- [ ] Support pointer semantics (`*ptr`, `&var`).
- [ ] Implement array indexing and pointer arithmetic.
- [ ] Embed a tiny `malloc` inside the DRE runtime.

### [Bite 4] The Ignition (Bootstrap)
- [ ] Port a minimal standard-compliant C compiler (ChibiCC/TCC) to the DRE.
- [ ] Use DRE-MiniCC to compile the ported C compiler.
- [ ] Achieve complete sovereignty: The DRE compiling its own C-based toolchain.
