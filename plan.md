1. **Decouple Setup from UI**:
   - `src/setup.rs` currently imports types from `crate::ui::*` (e.g. `InputContextStack`, `RenderCache`, `Selection`, `UiState`, `generate_world_history`, `TechUiState`, `ShellConfig`, `MenuState`).
   - Create a new module `src/shared/ui_state.rs` or move these state initializations to where the UI is actually initialized (e.g. `src/main.rs`, `src/bin/wasm_app.rs` or `src/ui/mod.rs`), or move the default types out of `ui` into `shared`.
   - Update `src/setup.rs` to no longer reference `crate::ui::`. It should be purely concerned with simulation setup.
2. **Review Tangle**:
   - `ui::state::UiState`, `ui::selection::Selection`, `ui::input::InputContextStack`, `ui::map::RenderCache`, `ui::tech::TechUiState`, `ui::shell::ShellConfig`, `ui::menu_state::MenuState`.
   - I will extract these structs from their respective `ui::` modules into a `shared::ui_state` or `shared::input` module so that `setup.rs` can initialize them without depending on the `ui` module, OR I will modify `src/main.rs` and `src/bin/wasm_app.rs` to insert these resources *after* `setup_world()` is called.
   - Actually, moving the UI resource insertions to a new function `setup_ui_resources(world: &mut World)` in `src/ui/mod.rs` makes the most architectural sense. `setup.rs` will only handle simulation resources. `main.rs` and `wasm_app.rs` will call both.

3. **Pre-commit step**:
   - Run `pre_commit_instructions` tool to ensure proper testing, verification, review, and reflection are done.
4. **Submit change**:
   - Submit the change with a commit message describing the fix for "The Leak" / "The Tangle" where core simulation setup depended on the UI rendering layer.
