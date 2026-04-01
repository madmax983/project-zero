# Integration tasks:
1. INT-772: Biomass Commute -> PopDied Event / Chronicle
If `digest_transit_contents` digests an entity that has a `Pop` component, we should probably emit a `PopDied` event so that the rest of the systems (morale, chronicle, etc) can react.
Actually, let's look at `src/layer1/logistics/biomass_network.rs` again.

2. INT-773: Stellar Weather -> Chronicle
When `FleetDamagedEvent` happens, maybe generate an `AddChronicleEvent`.

3. INT-774: Architectural Grafting -> Chronicle
When `GraftBuildingEvent` happens, generate an `AddChronicleEvent`.

Wait! The instructions say: "One seam at a time — Finish wiring one connection before starting another".
Let's just pick one. `INT-774` Architectural Grafting -> Chronicle.
"GraftBuildingEvent is sent but nobody listens" (Actually `process_grafting` listens).
What about: `GraftBuildingEvent` -> `AddChronicleEvent`?
If we look at `src/layer1/integration.rs`, we can add:
```rust
pub fn graft_building_chronicle_bridge(
    mut graft_events: EventReader<GraftBuildingEvent>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    time: Res<SimulationTime>,
) {
    // ...
}
```

Wait, let's look at `design/SEAM_MAP.md` and what makes a good integration. "Who should be reading its output? Who should be feeding it input?"
Grafting creates Frankenstein Architecture. It's a significant event. A chronicle event makes sense.

What about `INT-773` Stellar Weather Navigation? `FleetDamagedEvent` is fired but nothing else happens. It's an event black hole. No one reads `FleetDamagedEvent`! We need a system that reads `FleetDamagedEvent` and applies damage to `FleetHealth` or similar, OR we just record it in Chronicle. Wait, `FleetComposition` has `take_damage`. `FleetHealth` component exists!
In `src/layer2/navigation/stellar_weather.rs`, it sends `FleetDamagedEvent { fleet, amount }`.
In `src/layer2/fleet.rs`, `FleetHealth` is a component.
We should bridge `FleetDamagedEvent` to `FleetHealth`!

Yes! This is a PERFECT integration.

```rust
// In src/layer2/integration.rs:
pub fn fleet_weather_damage_bridge_system(
    mut events: EventReader<FleetDamagedEvent>,
    mut fleets: Query<&mut FleetHealth>,
) {
    for event in events.read() {
        if let Ok(mut health) = fleets.get_mut(event.fleet) {
            health.current -= event.amount;
            if health.current < 0.0 {
                health.current = 0.0;
            }
        }
    }
}
```
Wait, we should also destroy the fleet if health <= 0? Actually, there might already be a system for fleet destruction or we can just let `FleetHealth` hit 0. Let's check `src/layer2/fleet.rs` to see if there is a death system.

Let's check `src/layer1/logistics/biomass_network.rs`.
The `digest_transit_contents` despawns the entity. But if it's a `Pop`, it skips sending `PopDied`, which means "Pop death -> Population counter" UI might get out of sync, or families won't mourn.
We should emit `PopDied` when a Pop is digested!
