1. **Understand Goal**: Integrate the three recently completed feature specs (772, 773, 774) with existing systems where their events/outputs are currently isolated. Write integration tests and bridge systems, run pre-commit steps, and verify end-to-end functionality.

2. **INT-772 Integration**: The Biomass Commute -> Colony Resources / Chronicle
   - Find or create a way to feed the `BiomassNetwork`. The spec says it "consumes upkeep" and "digests transit contents". It lacks a clear connection for *feeding* it from colony stockpiles, or reporting digested pops/resources.
   - However, a simpler integration is connecting Digested pops to `AddChronicleEvent` or `PopDied`. Wait, the digestion just despawns the entity: `commands.entity(entity).despawn();`. We need an event for this, but modifying the builder's code should be minimal.
   - If we look at the codebase, we should probably add a `PopDigestedEvent` or similar in `layer1/logistics/biomass_network.rs` when `Pop` component is present.
   - Wait, "Never add gameplay features... If a seam needs a new game mechanic, create a spec." But adding an event is standard integration glue. Wait, maybe the seam is `BiomassNetwork` digestion -> Morale / Needs, or `PopDied` event.
   - Let's modify `digest_transit_contents` to send a `PopDied` event if the digested entity has a `Pop` component.

3. **INT-773 Integration**: Stellar Weather Navigation -> UI / Notifications or Chronicle
   - `FleetDamagedEvent` is fired. It should bridge to `AddChronicleEvent` (Chronicle).
   - Create a `stellar_weather_chronicle_bridge` system in `src/layer2/integration.rs`.

4. **INT-774 Integration**: Architectural Grafting -> Chronicle
   - `GraftBuildingEvent` is fired. It should bridge to `AddChronicleEvent` (Chronicle).
   - Create a `grafting_chronicle_bridge` in `src/layer1/integration.rs`.

Let's refine these plans based on checking exact components.
