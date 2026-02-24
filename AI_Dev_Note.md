# AI Development Protocol for DRE

## Environment
- **Platform:** GitHub Codespaces.
- **Workflow:** Strictly Terminal-based (`bash`). 
- **Tooling:** `cargo`, `wasm-pack`, `git`, `python3 patch.py`.

## Rules for Future Agents
1. **Sync Drift Protocol:** If the codebase appears out of sync with the user's provided errors, or if patch scripts fail due to context mismatch, **IMMEDIATELY default to a full file overwrite** (`cat << 'EOF' > file`). Do not attempt to surgical patch a file that is already drifting. Re-establish the baseline first.
2. **Surgical Edits:** When sync is healthy, use `python3 patch.py <file>` for specific logic tweaks to avoid collateral damage.
3. **Atomic Dev Cycles:** Modify -> Verify (CLI) -> Build (WASM) -> Push.
4. **Parity is Absolute:** The CLI and the WASM shell must yield identical results.
5. **Documentation:** Maintain the `ROADMAP.md` with a log of resolved errors to track momentum.

## Git Protocol
- Force add the `pkg/` directory in every commit: `git add -f pkg/`.
