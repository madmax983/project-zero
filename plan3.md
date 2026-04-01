Wait, if I do all three integrations, I will complete a major block. The instructions say "One seam at a time — Finish wiring one connection before starting another".
Let's choose `INT-773` and wire it completely.
The task: `FleetDamagedEvent` -> `FleetHealth` + `AddChronicleEvent`.

Let's look at `FleetDamagedEvent` in `src/layer2/navigation/stellar_weather.rs`:
```rust
#[derive(Event)]
pub struct FleetDamagedEvent {
    pub fleet: Entity,
    pub amount: f32,
}
```

Wait, `FleetHealth` has `current` and `max`. If it hits 0, the fleet is destroyed.

In `src/layer2/integration.rs`, we can add:
```rust
use crate::layer2::navigation::stellar_weather::FleetDamagedEvent;
use crate::layer2::fleet::{FleetHealth, FleetComposition};

/// Bridges `FleetDamagedEvent` (Stellar Weather) to `FleetHealth` and `AddChronicleEvent` (Chronicle).
pub fn stellar_weather_damage_bridge_system(
    mut commands: Commands,
    mut events: EventReader<FleetDamagedEvent>,
    mut fleets: Query<(&mut FleetHealth, Option<&mut FleetComposition>)>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
    time: Res<SimulationTime>,
) {
    for event in events.read() {
        if let Ok((mut health, mut maybe_comp)) = fleets.get_mut(event.fleet) {
            // Apply damage
            health.current -= event.amount;

            // If fleet has composition, apply damage to ships
            if let Some(mut comp) = maybe_comp {
                comp.take_damage(event.amount);
            }

            chronicle_events.send(AddChronicleEvent {
                importance: EventImportance::Major,
                message: "A fleet was heavily damaged by a sudden solar flare.".to_string(),
                tick: time.0,
                ..default()
            });

            if health.current <= 0.0 {
                commands.entity(event.fleet).despawn();
                chronicle_events.send(AddChronicleEvent {
                    importance: EventImportance::Legendary,
                    message: "A fleet was entirely consumed by a solar flare.".to_string(),
                    tick: time.0,
                    ..default()
                });
            }
        }
    }
}
```

Wait, let's also register this in `src/simulation.rs`.
And create `tests/integration/stellar_weather_bridge.rs` testing this.
