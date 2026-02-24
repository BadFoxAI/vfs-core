# AI Development Protocol for DRE

## Environment
- **Platform:** GitHub Codespaces.
- **Workflow:** Strictly Terminal-based (`bash`). 
- **Tooling:** `cargo`, `wasm-pack`, `git`.

## Rules for Future Agents
1. **Targeted Updates:** ONLY update the necessary files, not the whole codebase whenever possible. Avoid writing to files that do not require changes.
2. **No Hand-Editing:** Always use `cat << 'EOF' > file` or `sed` for file modifications.
3. **Atomic Dev Cycles:** Every update must include code modifications, local CLI verification, WASM building, and Git pushing in a single chain.
4. **Parity is Absolute:** The CLI and the WASM shell must yield identical results.
5. **No Fluff:** Ux/UI is irrelevant. Aesthetic upgrades are forbidden unless they serve diagnostic clarity.

## Git Protocol
- Force add the `pkg/` directory in every commit: `git add -f pkg/`.
