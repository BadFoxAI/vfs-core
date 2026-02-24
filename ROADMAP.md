# DRE Roadmap: The Path to C Bootstrap

## Completed
- [x] Phase 1-4: Deterministic VM Substrate & Custom ABI.
- [x] Phase 5: Bootstrap Language (v0) & File IO.
- [x] Phase 6: POSIX Shim (stdio.h / putchar).
- [x] Phase 7: Self-Hosting (Builder -> Bin -> Exec).
- [x] Phase 8: Hardening (Bounds checking & Gas metering).
- [x] Big Bite 1: AST & Expression Engine.
- [x] Big Bite 2: Stack Frames & Scoping.
- [x] Big Bite 3: Pointers & Memory.

## Current: BIG BITE 4 (THE IGNITION)
### [Bite 4.1] Compiler Substrate (Current)
- [x] Implement `while` loops (Control Flow).
- [x] Implement String Literals & Data Segment.
- [x] Implement Byte-Level Memory Access (`MLOAD8`, `char*`).
- [ ] Verify Self-Hosting Simulation.

### [Bite 4.2] The Self-Host
- [ ] Implement Tokenizer in C-subset.
- [ ] Implement Parser in C-subset.
- [ ] Compile C-Compiler using Rust-MiniCC.
- [ ] DRE becomes sovereign.
