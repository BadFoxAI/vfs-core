# DRE Roadmap: The Path to the Earned Terminal

## [STABLE] Era 1: The Sovereign Substrate
- [x] Phase 1-4: Deterministic VM & Custom ABI.
- [x] Phase 5-7: Bootstrap Language, VFS, and Self-Hosting Simulation.
- [x] Phase 8: Hardening (Bounds checking & Gas metering).
- [x] Big Bite 4: Sovereignty Achieved (The loop is closed).

## [CURRENT] Era 2: The Industrial Bridge
### Phase 9: The Compiler Port (TinyCC)
- [ ] Implement TCC-to-DRE Backend.
- [ ] Compile TCC using the Era 1 Bootstrap Compiler.
- [ ] Achieve full C99 compliance within the VFS.

### Phase 10: The Deception Layer (POSIX Shim)
- [ ] Implement `stdio.h` / `stdlib.h` against DRE Syscalls.
- [ ] Implement File Descriptor mapping (VFS -> POSIX).
- [ ] Enable `malloc/free` heap management within the VM.

## [UPCOMING] Era 3: The Threshold
### Phase 11: The Display Protocol
- [ ] Implement VT100/ANSI escape sequence interpretation in the Host Shell.
- [ ] Map DRE output to a real Terminal Emulator window.

### Phase 12: The Earned Terminal
- [ ] Ingest source code for a stable editor (e.g., `nano`).
- [ ] Compile `nano` using DRE-native TCC.
- [ ] Execute `nano` and edit a file within the VFS.
- [ ] **Sovereignty Threshold Reached.**
