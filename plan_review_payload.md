1. **Initialize Resource**: Use `replace_with_git_merge_diff` to modify `src/setup.rs` to insert `world.insert_resource(crate::layer1::economy::existential_audit::PrecursorAI { next_audit_tick: 1000 });` inside `setup_world_with_config`.
2. **Register Systems**: Use `replace_with_git_merge_diff` to modify `src/layer1/systems/economy.rs` to add `crate::layer1::economy::existential_audit::existential_audit_system` and `crate::layer1::economy::existential_audit::existential_crisis_decay_system` to the schedule in the `register` function.
3. **Verify Modification**: Use `run_in_bash_session` to run `git diff` and ensure the changes to `src/setup.rs` and `src/layer1/systems/economy.rs` were applied successfully.
4. **Write Integration Test**: Use `run_in_bash_session` with `cat << 'EOF' > tests/integration/existential_audit_bridge.rs` containing integration tests that check if `ExistentialCrisis` applies and decays appropriately:
```rust
use bevy::prelude::*;
use scale::layer1::economy::existential_audit::{existential_audit_system, existential_crisis_decay_system, PrecursorAI, IndustrialBuilding, ExistentialCrisis};
use scale::layer1::pop::Pop;
use scale::shared::time::SimulationTime;

#[test]
fn test_existential_audit_integration() {
    let mut app = App::new();
    app.init_resource::<SimulationTime>();
    app.insert_resource(PrecursorAI { next_audit_tick: 100 });

    app.add_systems(Update, (existential_audit_system, existential_crisis_decay_system).chain());

    let pop_entity = app.world_mut().spawn(Pop).id();
    app.world_mut().spawn(IndustrialBuilding { efficiency: 100.0, cultural_value: 0.0 });

    // Tick to 100 to trigger audit
    let mut time = app.world_mut().resource_mut::<SimulationTime>();
    time.tick = 100;
    app.update();

    let pop = app.world().entity(pop_entity);
    assert!(pop.contains::<ExistentialCrisis>(), "Pop should have crisis after failed audit");

    // Advance 500 ticks to clear duration
    for _ in 0..500 {
        app.update();
    }

    let pop = app.world().entity(pop_entity);
    assert!(!pop.contains::<ExistentialCrisis>(), "Crisis should decay after 500 updates");
}
```
And use `replace_with_git_merge_diff` to add `#[path = "integration/existential_audit_bridge.rs"] mod existential_audit_bridge;` to `tests/integration.rs`.
5. **Verify Tests Setup**: Use `run_in_bash_session` with `cat` and `git diff` to verify the creation and linking of the test file.
6. **Run Tests**: Use `run_in_bash_session` to execute `cargo test --lib && cargo test --test integration` and `cargo clippy -- -D warnings` to guarantee correctness and no linting issues.
7. **Update Documentation**: Use `replace_with_git_merge_diff` to remove task `1069` from `design/IN_PROGRESS.md` and use `echo "- [x] \`1069\` The Existential Audit — \`specs/1069-the-existential-audit.md\` — completed 2026-05-23" >> design/COMPLETED.md`. Then run `git diff design/` to verify.
8. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
9. **Submit**: Use `run_in_bash_session` to run `git add .`, `git commit -m "feat(integration): connect existential audit to schedules and resources\n\nCo-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>"`, and `git push` to finish the implementation.
