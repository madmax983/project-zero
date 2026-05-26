Plan to fix the issues:
1.  **Add `temporal_echo_maintenance_bridge_system` implementation**:
    -   Use `run_in_bash_session` to append the system to `src/layer1/core/integration.rs` correctly.

2.  **Register the system in `observation.rs`**:
    -   Use `replace_with_git_merge_diff` to add the registration `crate::layer1::core::integration::temporal_echo_maintenance_bridge_system,` to `src/layer1/systems/observation.rs`.

3.  **Add integration test**:
    -   Use `run_in_bash_session` to create `tests/integration/temporal_echoes_maintenance.rs` with the correct test code.
    -   Use `replace_with_git_merge_diff` to register the module in `tests/integration.rs`.

4.  **Update Markdown files**:
    -   Use `run_in_bash_session` to move `INT-495` from `IN_PROGRESS.md` to `COMPLETED.md` and add it to `SEAM_MAP.md`.

5.  **Run tests**:
    -   Use `run_in_bash_session` to run `cargo test -- test_temporal_echo_rapid_aging` to verify.

6.  **Pre-commit steps**:
    -   Complete pre commit steps to ensure proper testing, verification, review, and reflection are done.

7.  **Submit**:
    -   Submit the pull request.
