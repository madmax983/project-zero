# 1047: Asteroid Tethering

## 1. Overview
The colony can capture small asteroids and tether them in low orbit for extreme resource extraction ("Sky-Mines"). The tether requires constant power maintenance. If the power grid fails, the tether's orbit decays, crashing the asteroid into the planet and causing catastrophic damage while dropping massive resources.

## 2. Dependencies
- Layer 1 `energy` system (PowerGrid / PowerConsumer).
- Layer 2 `events` (for orbital tracking/crash events).

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::energy::{PowerConsumer, PowerSource};

    #[test]
    fn test_tether_consumes_power_and_crashes_on_failure() {
        let mut app = App::new();
        app.add_plugins(AsteroidTetheringPlugin);

        // Note: For a complete integration test, the Energy plugin that toggles
        // consumer active states based on grid capacity would also be added here.
        // For this unit test, we manually simulate the grid failing the power by mutating it.
        app.add_event::<AsteroidCrashEvent>();

        // Spawn a power source that produces 10 power
        let source = app.world_mut().spawn(PowerSource { output: 10.0, active: true }).id();

        // Spawn an asteroid tether demanding 20 power (more than produced)
        let tether = app.world_mut().spawn((
            AsteroidTether { stability: 100.0 },
            PowerConsumer { demand: 20.0, active: true }
        )).id();

        // Simulate tick where demand > supply
        if let Some(mut consumer) = app.world_mut().get_mut::<PowerConsumer>(tether) {
            consumer.active = false;
        }
        app.update();

        // Asteroid stability should decrease due to power failure
        let tether_state = app.world().get::<AsteroidTether>(tether).unwrap();
        assert!(tether_state.stability < 100.0, "Tether stability should decrease when unpowered.");

        // Simulate enough ticks for stability to reach 0
        for _ in 0..10 {
            app.update();
        }

        // Verify crash event was emitted
        let events = app.world().resource::<Events<AsteroidCrashEvent>>();
        let mut reader = events.get_reader();
        assert!(reader.read(events).next().is_some(), "Crash event should be emitted when stability hits 0.");

        // Verify entity is destroyed
        assert!(app.world().get::<AsteroidTether>(tether).is_none(), "Tether entity should be destroyed after crashing.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// src/layer2/orbit/tether.rs
use bevy::prelude::*;
use crate::layer1::energy::PowerConsumer;

#[derive(Component)]
pub struct AsteroidTether {
    pub stability: f32,
}

#[derive(Event, Debug)]
pub struct AsteroidCrashEvent {
    pub tether_entity: Entity,
}

pub struct AsteroidTetheringPlugin;

impl Plugin for AsteroidTetheringPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AsteroidCrashEvent>()
           .add_systems(Update, tether_decay_system);
    }
}

fn tether_decay_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AsteroidTether, &PowerConsumer)>,
    mut crash_events: EventWriter<AsteroidCrashEvent>,
) {
    for (entity, mut tether, consumer) in query.iter_mut() {
        if !consumer.active {
            tether.stability -= 10.0;
            if tether.stability <= 0.0 {
                crash_events.send(AsteroidCrashEvent { tether_entity: entity });
                commands.entity(entity).despawn();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Integrate resource drops from the `AsteroidCrashEvent` (e.g. populating the grid with ore).
- Link the crash event to the colony's Health/Damage system to destroy buildings on impact.
- Extract the stability decay rate to a configurable constant or resource.

## 6. Acceptance Criteria (Testable!)
- [ ] Test `test_tether_consumes_power_and_crashes_on_failure` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.

## 7. Technical Guidance
- Place the core logic in `src/layer2/orbit/tether.rs` and the tests in the same file or a dedicated test module.
- `PowerConsumer.active` will be determined by the existing `energy` module's grid overload rules. Ensure you are tapping into the same components.
## 8. Questions
*Builder: add questions here if spec is unclear.*
