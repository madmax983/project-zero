1. **Move task to IN_PROGRESS.md**
2. **Create `src/layer1/infrastructure/spatial_compression.rs` with RED Phase tests**
   - Adapt the spec's RED phase to match actual types: `GridPosition` has no `z`, use `ZLevel` alongside it.
   - We must define `PowerNode` locally or use the one from `layer1::energy` (but the energy module has `PowerSource`/`PowerConsumer`, wait... `layer1::physics::gravity_plating.rs` defines a `PowerNode` but it's specific. We can define a generic `PowerNode` in `spatial_compression.rs` or use `PowerConsumer` if appropriate. The spec specifically requests `crate::layer1::power::{PowerGrid, PowerNode}`, but since `power` doesn't exist (it's `energy`), we can just define `PowerNode` inside `spatial_compression.rs` to keep it simple and fulfill the spec). Actually, the spec uses `PowerNode` from `crate::layer1::power::PowerNode`. We will just define it locally since `power` module is not present (the closest is `energy::PowerConsumer`, but `PowerNode` makes the test compile without changing other modules). Or better, define it locally so the test works verbatim with minor path adjustments.
   - For `DamageEvent`, we'll define a local `PocketDamageEvent` or use `crate::layer1::shields::DamageEvent`. Let's just define a local `DimensionalDamageEvent` or use `Health::take_damage` in the system, and emit `DamageEvent` locally if needed for the test to read.
3. **Commit RED Phase tests**
4. **Implement GREEN Phase**
   - Make the tests pass. Ensure ejection sets `GridPosition` and `ZLevel` matching external. Apply damage using `Health::take_damage` on the ejected entities. Remove `InsidePocket` using `commands.entity(ent).remove::<InsidePocket>()`.
5. **Commit GREEN Phase**
6. **Register the module**
   - In `src/layer1/infrastructure/mod.rs`, add `pub mod spatial_compression; pub use spatial_compression::*;`.
   - In `src/layer1/systems/environment.rs`, register the systems `monitor_pocket_power_system` and `process_pocket_collapse_system`.
7. **Verify Coverage and run checks**
   - Run `cargo test` and `cargo clippy` and `cargo llvm-cov` to ensure >= 85% coverage.
8. **Pre-commit step**
   - Call `pre_commit_instructions`
9. **Finalize and Submit**
   - Move task to `COMPLETED.md`
   - Commit and push
