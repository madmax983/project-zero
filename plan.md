1. **Fix `HarvestEvent` Panics:**
   - The test panic trace from `cargo test --test integration drone_network` shows: `scale::layer2::mycelial_network::evaluate_ecological_damage_system could not access system parameter Res<'_, Events<HarvestEvent>>`.
   - The issue isn't just in setup.rs and cleanup.rs. Many integration tests define their own local `setup_world()` functions directly in the test file which bypasses the central `src/setup.rs` completely, or they don't use it everywhere.
   - I will use `sed` to insert the required resource in the specific failing tests:
     - `sed -i '/let mut world = scale::setup::setup_world();/a \        world.init_resource::<bevy_ecs::event::Events<scale::layer1::nature::ecology::HarvestEvent>>();' tests/integration/drone_network.rs`
     - `sed -i '/let mut world = scale::setup::setup_world();/a \    world.init_resource::<bevy_ecs::event::Events<scale::layer1::nature::ecology::HarvestEvent>>();' tests/integration/fire_health.rs`
     - `sed -i '/let mut world = scale::setup::setup_world();/a \        world.init_resource::<bevy_ecs::event::Events<scale::layer1::nature::ecology::HarvestEvent>>();' tests/integration/funeral_rites.rs`
     - `sed -i '/let mut world = setup_world();/a \    world.init_resource::<bevy_ecs::event::Events<scale::layer1::nature::ecology::HarvestEvent>>();' tests/integration/seismic_vibration.rs`
     - `sed -i '/let mut world = setup_world();/a \    world.init_resource::<bevy_ecs::event::Events<scale::layer1::nature::ecology::HarvestEvent>>();' tests/integration/social_proximity.rs`

2. **Verify Edits:**
   - I will use `git diff` to verify my edits were successfully applied without duplication.

3. **Verify Tests:**
   - I will run `cargo test --test integration` to verify the integration tests now pass.
   - I will also run the full test suite with `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings` to verify the tests pass and no new warnings are generated.

4. **Pre-commit Steps:**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

5. **Submit the change:**
   - I'll commit the changes with the following message and submit.
   ```
   feat(integration): register HarvestEvent to fix test panics

   Registers HarvestEvent in setup.rs, cleanup.rs, and individual integration tests.
   This fixes panics in test_drone_spawning and other tests caused by a missing Events<HarvestEvent> resource.

   Co-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>
   ```
