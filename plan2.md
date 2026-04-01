Let's consider all three:

1. **INT-773: Stellar Weather Navigation (FleetDamagedEvent) -> FleetHealth**
   Currently, `apply_stellar_weather_effects` sends a `FleetDamagedEvent`. But no system reads it!
   We should add a bridge system `stellar_weather_damage_bridge_system` in `src/layer2/integration.rs` that reads `FleetDamagedEvent` and applies damage to `FleetHealth`. And if `FleetHealth` <= 0.0, we despawn the fleet (or `commands.entity(fleet).despawn()`). Actually `debris_attrition_system` handles damage similarly.
   Also, we should record a Chronicle event maybe, but the main issue is "damage happens but health doesn't decrease".
   Wait, if we despawn the fleet, we might want to spawn debris, similar to `fleet_combat_system`. Let's just decrease health and let `fleet_combat_system` or similar handle it, or we can despawn it.
   Let's check `layer2/debris.rs` `debris_attrition_system` which damages fleet health.

2. **INT-772: Biomass Commute -> PopDied & ColonyResources**
   In `src/layer1/logistics/biomass_network.rs`, `digest_transit_contents` despawns `entity`. If the entity has `Pop` and `PopName`, it just despawns. It *should* send a `PopDied` event so the rest of the game knows a pop died.
   ```rust
   pub fn digest_transit_contents(
       mut commands: Commands,
       mut networks: Query<&mut BiomassNetwork>,
       transit_query: Query<(Entity, &InTransit, Option<&PopName>)>,
       mut pop_died_events: EventWriter<PopDied>,
   ) {
      //... if network is hungry and it's a pop, send PopDied
   }
   ```
   Wait, if it's a `ResourceYield`, it also just despawns without any UI or log. Let's focus on `PopDied` event.

3. **INT-774: Architectural Grafting -> Chronicle**
   `GraftBuildingEvent` occurs, it modifies tech level, adds `MaintenanceDebt` and `Quirks`. But there is no log or chronicle entry. It makes sense to bridge `GraftBuildingEvent` to `AddChronicleEvent`.

Let's claim one of them: "INT-773: Stellar Weather -> FleetHealth and Chronicle" OR "INT-772: Biomass Commute -> PopDied".

Let's do **INT-773: Stellar Weather -> FleetHealth & Chronicle**

Wait, let's claim `INT-773`. Let's create an entry in `design/IN_PROGRESS.md`.
