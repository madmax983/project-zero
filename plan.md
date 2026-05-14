1.  **Implement `src/layer3/bureaucracy_of_truth.rs`**
    *   Create a new module file `src/layer3/bureaucracy_of_truth.rs`.
    *   **RED Phase:** Add tests `test_corrupt_governor_falsifies_colony_report` and `test_loyal_governor_reports_truth` as specified in `specs/1071-the-bureaucracy-of-truth.md`. Ensure these fail initially.
    *   **GREEN Phase:**
        *   Define `ColonyState`, `ColonyReport` components.
        *   We need to handle `Governor` component. `Governor` exists in `src/layer2/governance.rs`, but the spec uses a slightly different version (`loyalty` and `corruption`). Let's use `GovernorStats` from `src/layer2/governance.rs` for corruption and loyalty. Wait, the spec has `Governor` with `loyalty` and `corruption`. Since `Governor` and `GovernorStats` are already in `layer2`, we should adapt the tests and implementation to use the existing `GovernorStats` which has `corruption`. It lacks `loyalty`. The existing codebase architecture is the source of truth. We will add `loyalty` to `GovernorStats` in `src/layer2/governance.rs` if needed, or stick to just using corruption. Actually, adding `loyalty: f32` to `GovernorStats` is fine, or we can just redefine `Governor` as specified if it's meant to be a new component for this spec. However, since `Governor` is in `layer2`, we should probably extend `GovernorStats` with `loyalty`. Let's check `GovernorStats`.
        *   Implement `generate_colony_reports_system`.
    *   **REFACTOR Phase:** Implement progressive falsification.
2.  **Integrate `bureaucracy_of_truth`**
    *   Add `pub mod bureaucracy_of_truth;` to `src/layer3/mod.rs`.
    *   Register the new system `generate_colony_reports_system` in `src/simulation.rs`.
3.  **Ensure Testing and Verification**
    *   Run `cargo test --lib layer3::bureaucracy_of_truth` to verify.
    *   Check test coverage with `cargo llvm-cov`.
    *   Run `cargo clippy -- -D warnings`.
4.  **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
5.  **Submit the changes**
