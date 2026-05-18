1. **Refactor `kinetic_strike_system` for Performance**
   - In `src/layer1/geology/subsurface.rs`, the `kinetic_strike_system` currently iterates over all `subsurface_query` elements for every kinetic strike event and also within a nested loop when applying damage in a radius.
   - Refactor it to early-exit if there are no `KineticStrikeEvent`s (`if events.is_empty() { return; }`).
   - Before processing the events, build a local `HashMap<(i32, i32), (Entity, SubsurfaceResourceKind)>` mapping grid coordinates to the entity and resource kind.
   - Replace the iterations over `subsurface_query` with O(1) lookups in the hash map.
   - This directly addresses the REFACTOR phase requirement: "Performance: Grid lookup using iteration is slow. We should use a spatial hash map... for fast tile lookup."

2. **Run pre-commit instructions**
   - Complete pre-commit steps to make sure proper testing, verifications, reviews, and reflections are done.

3. **Verify tests and coverage**
   - Run `cargo test --lib -- kinetic_strike_system` or similar test.
   - Run `cargo llvm-cov` to verify that `src/layer1/geology/subsurface.rs` has coverage >= 85% and no regressions occurred.

4. **Submit changes**
   - Use the `submit` tool to finalize the code with a descriptive commit message.
