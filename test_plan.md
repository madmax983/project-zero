1. **Understand Goal**: Integrate 1306-architectural-superstition.
   - It needs a way to track negative events and curse buildings.
   - We need to populate `NegativeEventHistory` of buildings near a negative event.
   - The negative events we can use:
     - `PopDied`: If a pop dies near a building, it's a negative event. (We'll use `GridPosition` if the pop still has it, or just use `PopDiedInAccidentEvent` which has `building_entity`). Wait, `PopDiedInAccidentEvent` only has the building entity!
     - `BuildingRemovedEvent` from `src/layer1/core/events.rs`. If a building is destroyed, maybe adjacent buildings get a negative event.

2. Let's create an integration module or add to `src/layer1/core/integration.rs`:
   We will create a system `architectural_superstition_bridge_system` that listens to `PopDied` and `BuildingRemovedEvent` (or maybe `BuildingDamageEvent`?).
   Wait, if we use `PopDiedInAccidentEvent` and `BuildingRemovedEvent`.
   Let's check `BuildingRemovedEvent`. It has `entity`, `position` and `building_type`.
   Let's write `track_negative_events_bridge_system`:
   ```rust
   pub fn track_negative_events_bridge_system(
       mut pop_died_events: EventReader<crate::layer1::pop::PopDied>,
       mut building_removed_events: EventReader<crate::layer1::core::events::BuildingRemovedEvent>,
       pops_query: Query<&crate::layer1::map::GridPosition, With<crate::layer1::pop::Pop>>,
       mut buildings_query: Query<(Entity, &crate::layer1::map::GridPosition, &mut crate::layer1::architecture_superstition::NegativeEventHistory), With<crate::layer1::architecture::Building>>,
   ) { ... }
   ```
