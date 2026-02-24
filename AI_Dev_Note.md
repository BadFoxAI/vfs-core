# AI Development Protocol for DRE

## Environment
- **Platform:** GitHub Codespaces.
- **Workflow:** Strictly Terminal-based (`bash`). 
- **Tooling:** `cargo`, `wasm-pack`, `git`, `python3 patch.py`.

## Rules for Future Agents
1. **Sync Drift Protocol:** If the codebase appears out of sync with the user's provided errors, or if patch scripts fail due to context mismatch, **IMMEDIATELY default to a full file overwrite** (`cat << 'EOF' > file`). Re-establish the baseline first.
2. **Surgical Edits:** When sync is healthy, use `python3 patch.py <file>` for specific logic tweaks to avoid collateral damage.
3. **Atomic Dev Cycles:** Modify -> Verify (CLI) -> Build (WASM) -> Push.
4. **Parity is Absolute:** The CLI and the WASM shell must yield identical results.
5. **The Prime Directive:** The immediate goal is to compile Tiny C Compiler (TCC). Scaffold only what is necessary to achieve this. Do not build UI or shells until TCC can compile them.
6. **The Monolith (Refactoring):** `lib.rs` is currently a monolithic file (God Object) containing the VM, Lexer, Parser, and Compiler. *Refactoring is approved but secondary.* If refactoring into modules, do it in a way that strictly preserves the 100% pass rate of `run_suite()`.

## Git Protocol
- Force add the `pkg/` directory in every commit: `git add -f pkg/`.
