1. **Explore & Verify Context**
   - Read the relevant parts of the `src/layer1/core/integration.rs` to verify imports and structure.
   - We need to implement `ghost_code_chronicle_bridge` function which bridges from `BuildingCompletedEvent` where a `GhostCode` is added to a building, to an `AddChronicleEvent`. Wait, the test uses `Added<GhostCode>`?
   - Wait, `BuildingCompletedEvent` triggers `ghost_infection_system` which inserts `GhostCode` to the building entity.
   - So `ghost_code_chronicle_bridge` can either query `Query<Entity, Added<GhostCode>>` or listen to something else.
   - Let's look at `tests/integration/ghost_code_chronicle_bridge.rs`. It just creates a building WITH `GhostCode` and then sends `BuildingCompletedEvent`. So the bridge might actually listen to `BuildingCompletedEvent`, then check if the entity in the event has `GhostCode` component. If so, it emits an `AddChronicleEvent`.

2. **Implement `ghost_code_chronicle_bridge`**
   - In `src/layer1/core/integration.rs`:
     ```rust
     use crate::layer1::tech::ghost_code::GhostCode;

     /// INT-1277: Bridges GhostCode infection to AddChronicleEvent (Chronicle).
     pub fn ghost_code_chronicle_bridge(
         mut events: bevy_ecs::event::EventReader<crate::layer1::events::BuildingCompletedEvent>,
         query: bevy_ecs::system::Query<&GhostCode>,
         mut chronicle_events: bevy_ecs::event::EventWriter<crate::layer1::core::chronicle::AddChronicleEvent>,
     ) {
         for event in events.read() {
             if query.get(event.entity).is_ok() {
                 chronicle_events.send(crate::layer1::core::chronicle::AddChronicleEvent {
                     text: "A newly constructed building inherited phantom behaviors from the site's previous structure.".to_string(),
                     importance: crate::layer1::core::chronicle::EventImportance::Minor,
                 });
             }
         }
     }
     ```

3. **Register the Bridge**
   - Open `src/layer1/systems/observation.rs` (or wherever appropriate) and register `crate::layer1::core::integration::ghost_code_chronicle_bridge`.

4. **Verify Tests**
   - Run `cargo test --test integration ghost_code_chronicle_bridge`.

5. **Update SEAM_MAP.md**
   - Add `INT-1277` to `design/SEAM_MAP.md`.
   - Update `design/IN_PROGRESS.md` and `design/COMPLETED.md`.
