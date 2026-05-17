# Execution Plan

## Claiming the Task
1. Use `run_in_bash_session` to execute `sed -i '/1068/d' design/BACKLOG.md` to remove the task from the backlog.
2. Use `run_in_bash_session` to execute `echo "- [ ] \`1068\` Crustal Tides — \`specs/1068-crustal-tides.md\` — claimed $(date +%Y-%m-%d)" >> design/IN_PROGRESS.md` to add the task to in-progress.
3. Use `run_in_bash_session` to execute `git add design/ && git commit -m "claim: 1068 crustal tides"` to commit the claim.

## Implementation Steps
4. **Update `TerrainType` enum:** Use `replace_with_git_merge_diff` to modify `src/layer1/nature/terrain.rs`. Add `FaultLine(bool)` to the `TerrainType` enum (where `true` means open magma exposed, `false` means closed). Update `name` to return `"FaultLine"`, `is_walkable` to return false when `FaultLine(true)` and true when `FaultLine(false)`, and `heat_retention` to return `0.5` for `FaultLine(true)` and `0.5` for `FaultLine(false)`. Also, handle it in other `TerrainType` match blocks as needed. Verify with `cargo check`.
5. **Create `src/layer1/geology/crustal_tides.rs`:** Use `write_file` to create the file and implement `process_crustal_tides`. Track fault line tile closures and apply damage. Implement `process_crustal_tides` to collect closed `FaultLine` coordinates into a `HashSet`, then iterate over `Structure` components with a `GridPosition` and apply 50 damage to their `current_hp` if their position exists in the set. Add the RED phase tests from the spec at the bottom of the file inside `#[cfg(test)]`. Use `cargo check` to verify syntax.
6. **Register module:** Use `replace_with_git_merge_diff` to add `pub mod crustal_tides;` to `src/layer1/geology/mod.rs`. Verify by ensuring `cargo check` passes.
7. **Register system:** Use `replace_with_git_merge_diff` on `src/layer1/systems/environment.rs` to register `crate::layer1::geology::crustal_tides::process_crustal_tides` in the system schedule tuple near `check_seismic_events`. Verify with `cargo check`.
8. **Run tests:** Use `run_in_bash_session` to execute `cargo test --lib layer1` and `cargo clippy --all-targets --all-features -- -D warnings` to verify the implementation.
9. **Pre-commit steps:** Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done. Call `pre_commit_instructions` and follow them.
10. **Commit & mark completed:** Use `run_in_bash_session` to execute `sed -i '/1068/d' design/IN_PROGRESS.md` and `echo "- [x] \`1068\` Crustal Tides — \`specs/1068-crustal-tides.md\` — completed $(date +%Y-%m-%d)" >> design/COMPLETED.md`, then `git add .` and `git commit` to finalize.
