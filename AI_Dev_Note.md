# AI Development Protocol for DRE

## Environment
- **Platform:** GitHub Codespaces.
- **Workflow:** Strictly Terminal-based (`bash`). 
- **Tooling:** `cargo`, `wasm-pack`, `git`.

## Rules for Future Agents
1. **No Hand-Editing:** Always use `cat << 'EOF' > file` or `sed` for file modifications.
2. **Atomic Dev Cycles:** Every update must include:
    - Code logic update (`src/lib.rs`).
    - Local CLI verification (`cargo run`).
    - WebAssembly build (`wasm-pack build --target web`).
    - Staging, Committing, and Pushing to Netlify.
3. **Parity is Absolute:** The CLI and the WASM shell must yield identical results. This is a Deterministic Runtime Environment; drift is failure.
4. **No Fluff:** Ux/UI is irrelevant. Aesthetic upgrades are forbidden unless they serve diagnostic clarity.
5. **Packaged Context:** The user uses `Repomix` to bundle the repository. When providing code, provide the full file content to ensure consistency.

## Git Protocol
- Force add the `pkg/` directory in every commit: `git add -f pkg/`.
- Netlify relies on the pre-built `pkg/` folder to serve the WASM.
