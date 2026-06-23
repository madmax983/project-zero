1. **Explore the codebase and understand the task**
   - The user asked to implement the `1036` Institutional Memory specification.
   - I explored `design/BACKLOG.md`, `design/IN_PROGRESS.md`, and `specs/1036-institutional-memory.md`.
   - The core components (`src/layer1/institutional_memory.rs` and `src/layer1/institutional_memory_tests.rs`) were already implemented, but they had invalid module paths (`crate::layer1::items`).
   - Replaced `crate::layer1::items` with `crate::layer1::economy::items` in both files.

2. **Verify tests and constraints**
   - Run `cargo fmt` to format code.
   - Run `cargo check` and `cargo test --lib layer1::institutional_memory` to verify correctness.
   - Run `cargo clippy -- -D warnings` to verify there are no warnings.
   - Run `cargo-llvm-cov` to verify that code coverage meets the >=85% threshold.

3. **Update trackers**
   - Claimed the feature in `design/IN_PROGRESS.md` and removed it from `design/BACKLOG.md`.
   - Updated `design/COMPLETED.md` with the completed feature and removed it from `design/IN_PROGRESS.md`.

4. **Pre-commit step**
   - Call `pre_commit_instructions` to ensure proper testing, verification, review, and reflection are done.

5. **Submit the change**
   - Complete the task by invoking `submit`.
