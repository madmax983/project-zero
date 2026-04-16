1. **Move task to IN_PROGRESS.md**
2. **Add `FaithCurrency` Resource to `src/layer3/diplomacy_reflection.rs`**
   - It should hold a generic Unity/Faith currency value.
3. **Add RED phase tests to `src/layer2/primitives/mod.rs`**
   - Create tests according to the spec (`test_accidental_gods_faith_gain`, `test_accidental_gods_faith_loss_on_neglect`, `test_accidental_gods_primitive_retaliation`). Since `get_faith_currency`, `get_diplomatic_standing`, and `get_building_health` don't exist, we will use proper Bevy ECS query patterns (`app.world().resource::<FaithCurrency>()`, `app.world().get::<DiplomaticRelations>()`, `app.world().get::<Structure>()`).
   - Run `cargo test` to see them fail.
4. **Implement GREEN phase**
   - Add `PrimitiveFollowers` component to `src/layer2/primitives/mod.rs`.
   - Add `PrimitiveRetaliationEvent` event.
   - Add `primitive_faith_system` and `drop_supplies_to_primitives` / helpers.
   - Register the system and event in a plugin or directly in tests.
5. **Pre-commit Steps**
   - Follow instructions from `pre_commit_instructions`
6. **Submit**
   - Update `IN_PROGRESS.md` and `COMPLETED.md`.
   - Commit and submit.
