1. **Refactor Pyramids of Doom & Duplication in `src/layer1/core/integration.rs`**:
   - Extract memory insertion into a helper function `grant_inspector_memory` inside `src/layer1/core/integration.rs` to flatten `inspector_outcome_bridge_system`. This will replace the duplicated `.par_iter_mut().for_each(...)` loops for adding memories to pops.
   - Refactor `public_grievance_grudge_bridge` by extracting the `add_or_update_grudge` logic into a helper function to avoid duplicating the code between entities that already have the component and those getting a new one.

2. **Verify tests and format**:
   - Run `cargo fmt --all`
   - Run `cargo clippy --all-targets --all-features -- -D warnings`
   - Run `cargo test --lib layer1` or similar to make sure tests pass.

3. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

4. Submit change using `default_api:submit` with the appropriate branch, title and message.
