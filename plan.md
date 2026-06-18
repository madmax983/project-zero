1. **Optimize `fauna_behavior_system` in `src/layer1/fauna/mod.rs`.**
   - Replace the `&mut World` argument with idiomatic Bevy queries (`Query`, `Commands`, `Local`).
   - Use `Local<Vec<(Entity, GridPosition)>>` for `pops` and `fauna_updates`, and `Local<Vec<(Entity, Entity, f32)>>` for `attacks`.
   - Update tests in `src/layer1/fauna/mod.rs` to run the system via a `Schedule`.
   - Ensure the system compiles and passes tests.

2. **Run tests & clippy**
   - Run `cargo test --lib layer1::fauna`.
   - Run `cargo clippy --all-targets --all-features -- -D warnings`.
   - Run `cargo fmt --all`.

3. **Check Pre-commit**
   - Ensure pre-commit steps are checked.

4. **Submit PR**
   - Provide summary.
