# AI Development Protocol for DRE

## Environment
- **Platform:** GitHub Codespaces.
- **Workflow:** Strictly Terminal-based (`bash`). 
- **Tooling:** `cargo`, `wasm-pack`, `git`, `python3 patch.py`.

## Rules for Future Agents
1. **NO FULL OVERWRITES:** Do not use `cat > file` to overwrite existing source files.
2. **Surgical Edits:** Use `python3 patch.py <file>` to replace specific blocks of code.
   - Create `patch_match.txt` (exact text to find).
   - Create `patch_replace.txt` (text to insert).
   - Run `python3 patch.py target_file`.
3. **Atomic Dev Cycles:** Modify -> Verify (CLI) -> Build (WASM) -> Push.
4. **Parity is Absolute:** The CLI and the WASM shell must yield identical results.

## Git Protocol
- Force add the `pkg/` directory in every commit: `git add -f pkg/`.
