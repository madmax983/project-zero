1. We are dealing with `1306-architectural-superstition.md`. The systems `evaluate_architectural_superstition` and `apply_cursed_penalties` exist in `src/layer1/architecture_superstition.rs` and are registered in `src/layer1/systems/observation.rs`.
2. The missing piece is bridging negative events (`PopDied`, `BuildingRemovedEvent`) to add `NegativeEvent` structs to nearby buildings' `NegativeEventHistory`.
3. We will write `architectural_superstition_bridge.rs` (or add to `src/layer1/core/integration.rs`) a system `track_negative_events_bridge_system` that listens for `PopDied` and `BuildingRemovedEvent`, checks their locations, and pushes `NegativeEvent` to all `NegativeEventHistory` components on buildings within a certain radius (e.g. distance <= 5).
4. `PopDied` events don't have position. Wait, does `PopDied` have position? No. So we must query `GridPosition` of the dead pop *before* it's despawned, or just use what we can. If the pop is already despawned, maybe we can't get the position. Let's see how other systems handle `PopDied` position.
   - Some systems query `Query<&GridPosition, With<Pop>>` using `pop_died.entity`. If it's already despawned, we miss it.
   - Wait, `BuildingRemovedEvent` has `position`. We can definitely use that!
   - What about `PopDiedInAccidentEvent`? It's in `src/layer1/haunted_assembly_lines.rs` and has `building_entity`.

Let's write a bridge system that listens to:
  - `PopDied` (try to get position from `Query<&GridPosition>`)
  - `BuildingRemovedEvent`
  - `PopDiedInAccidentEvent` (has `building_entity`)

We will add this bridge system to `src/layer1/core/integration.rs` and register it in `src/simulation.rs` or `src/layer1/systems/observation.rs`.

Wait, the prompt says we should do integration work (INT-1306).
Let's create the bridge test first in `tests/integration/architectural_superstition_bridge.rs`.
