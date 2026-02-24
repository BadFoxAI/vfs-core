# DRE Roadmap: The Path to the Earned Terminal

## [STABLE] Era 1: The Sovereign Substrate
- [x] Phase 1-4: Deterministic VM & Custom ABI.
- [x] Phase 5-7: Bootstrap Language, VFS, and Self-Hosting Simulation.
- [x] Phase 8: Hardening (Bounds checking & Gas metering).
- [x] Big Bite 4: Sovereignty Achieved (The loop is closed).

## [CURRENT] Era 2: The Industrial Bridge
- [x] Phase 9: TCC Compiler Backend & Self-Hosting.
- [x] Phase 10: The Deception Layer (POSIX Shim).
- [x] Phase 13: Compiler Upgrades (Structs, Arrays, Globals).

## [UPCOMING] Era 3: The Threshold
- [ ] **Phase 14: The Multi-Pass Compiler (Forward Declarations).**
- [ ] **Phase 15: The Preprocessor (Macros/#include).**
- [ ] Re-attempt Interactive Shell with Industrial Compiler.

## Resolved Issues Log
- **WASM Infinite Loop:** Caused by MiniCC lacking scope handling, creating stack leaks in `while` loops. *Fix:* Hoisted variables to function top.
- **Array Garbage Values:** Caused by `arr[i]` treating `arr` as an l-value value instead of a pointer address. *Fix:* Implemented Array Pointer Decay in `gen_expr`.
