**[Title] Break The ViewMode Knot: Move ViewMode to Shared**
**Tangle:** `ViewMode` is defined in `layer2::system`. It is required by `ui` (`input.rs`, `shell/runtime.rs`, `mod.rs`), `setup.rs`, and `layer2` elements (`render.rs`, `visibility.rs`). This couples `ui` and `setup` to `layer2`.
**Blueprint:**
1. Extract `ViewMode` enum from `layer2::system.rs` into `shared::view_mode.rs`.
2. Update imports in `setup.rs`, `layer2::render.rs`, `layer2::system.rs`, `layer2::visibility.rs`, `ui::input.rs`, `ui::shell::runtime.rs`, and `ui::mod.rs` to use `shared::view_mode::ViewMode`.
3. Add `pub mod view_mode;` to `shared/mod.rs`.
