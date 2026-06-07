1. **Move task to IN_PROGRESS.md**
   - Claim task 1296 the subconscious grid.
2. **Implement Subconscious Grid (RED phase)**
   - Create `src/layer1/subconscious_grid.rs`.
   - Add RED phase tests as defined in `specs/1296-the-subconscious-grid.md`.
   - Make sure they fail correctly (`cargo test subconscious_grid`).
3. **Implement Subconscious Grid (GREEN phase)**
   - Add minimal implementation to `src/layer1/subconscious_grid.rs`.
   - Register the system in `src/layer1/systems/observation.rs`.
   - Ensure tests pass.
4. **Implement Subconscious Grid (REFACTOR phase)**
   - Implement lockdown logic interacting with SecurityDoor/AccessControl (as suggested in spec).
   - Change efficiency to compose with other systems (if possible or just use a multiplier logic).
   - Ensure tests are still green and coverage is 85%.
5. **Verify and Pre-commit**
   - Run `cargo fmt`, `cargo check`, `cargo clippy -- -D warnings`, `cargo test`.
   - Run `cargo llvm-cov --lib --bins`.
   - Complete pre-commit steps.
6. **Submit**
   - Move task to COMPLETED.md.
   - Run git add/commit with appropriate format.
   - Call `submit`.
