1. **Extract Scenario structs from `src/setup.rs` to `src/shared/scenario.rs`**:
   - Use `run_in_bash_session` with `sed` or Python to copy `SetupConfig`, `StartScenarioDifficulty`, `StartScenarioId`, `StartScenarioDefinition`, `ActiveStartScenario`, `AppliedStartScenario`, and `start_scenario_definition` from `src/setup.rs` and write it to `src/shared/scenario.rs`. Make sure `use bevy::prelude::*;` is present.
   - Use `replace_with_git_merge_diff` to add `pub mod scenario;` to `src/shared/mod.rs`.
   - Verify creation of the new file with `list_files` or `read_file`.

2. **Remove definitions from `src/setup.rs` and update imports**:
   - Use `replace_with_git_merge_diff` or a Python script in `run_in_bash_session` to delete the moved structs/enums from `src/setup.rs`.
   - Use `replace_with_git_merge_diff` to add `pub use crate::shared::scenario::*;` in `src/setup.rs` so existing consumers won't break if we miss them. But wait, `StartScenarioId` is used directly in some `setup` consumers, maybe we just do `use crate::shared::scenario::*;` for internal usage, and update the explicit references. Let's just update all explicit references using `sed`.

3. **Update imports in other files**:
   - Use `run_in_bash_session` to run `sed -i 's/crate::setup::StartScenarioId/crate::shared::scenario::StartScenarioId/g' src/ui/input.rs src/ui/menu.rs src/ui/menu_state.rs`
   - Use `run_in_bash_session` to run `sed -i 's/crate::setup::StartScenarioDifficulty/crate::shared::scenario::StartScenarioDifficulty/g'`
   - Use `run_in_bash_session` to run `sed -i 's/crate::setup::{/crate::setup::{/g'` (fix multi-imports in `src/ui/input.rs` manually with `replace_with_git_merge_diff`).

4. **Verify correctness**:
   - Run `cargo check --workspace --all-features`.
   - Run `cargo test --workspace --all-features`.
   - Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

5. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

6. **Submit PR**:
   - Use `run_in_bash_session` to stage changes and commit with "🗺️ Atlas: [architectural change]" and proper description.
   - Call the `submit` tool.
