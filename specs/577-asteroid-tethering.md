# 577: Asteroid Tethering

## 1. Overview
Instead of destroying incoming asteroids on Layer 2, players can deploy "Tether-Ships" to arrest their momentum and drag them into a low, unstable Layer 1 orbit. This allows building high-efficiency zero-G extraction facilities directly on the tethered rock. The tether requires constant, massive power from the Layer 1 colony to maintain orbit. If the power plant fails, the asteroid de-orbits and obliterates a sector of the planet.

## 2. Dependencies
- Core Layer 1 ECS (Entities, Components, Systems)
- Power Grid system (Power Plant, Power Consumption, Localized Power Failure)
- Layer 2 event interaction (Incoming Asteroid)
- Localized destruction (Sector Obliteration)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::power::{PowerGrid, PowerConsumer, PowerPlant};
    use crate::layer1::grid::GridPosition;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<AsteroidDeorbitEvent>();
        app.add_systems(Update, process_tethered_asteroids_system);
        app
    }

    #[test]
    fn test_tethered_asteroid_maintains_orbit() {
        let mut app = setup_app();

        // Arrange: A fully functioning power grid maintaining the tether
        app.world.spawn((
            TetheredAsteroid { energy_required: 100 },
        ));
        app.world.spawn((
            PowerGrid { available_energy: 200, ..Default::default() },
        ));

        // Act
        app.update();

        // Assert: The asteroid should maintain orbit, no de-orbit event
        let events = app.world.resource::<Events<AsteroidDeorbitEvent>>();
        let mut reader = events.get_reader();
        let iter: Vec<_> = reader.read(events).collect();
        assert_eq!(iter.len(), 0);
    }

    #[test]
    fn test_tethered_asteroid_deorbits_on_power_failure() {
        let mut app = setup_app();

        // Arrange: A failing power grid
        let asteroid = app.world.spawn((
            TetheredAsteroid { energy_required: 100 },
        )).id();
        app.world.spawn((
            PowerGrid { available_energy: 50, ..Default::default() },
        ));

        // Act
        app.update();

        // Assert: The asteroid de-orbits
        let events = app.world.resource::<Events<AsteroidDeorbitEvent>>();
        let mut reader = events.get_reader();
        let iter: Vec<_> = reader.read(events).collect();
        assert_eq!(iter.len(), 1);
        assert_eq!(iter[0].asteroid, asteroid);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::power::PowerGrid;

#[derive(Component)]
pub struct TetheredAsteroid {
    pub energy_required: i32,
}

#[derive(Event)]
pub struct AsteroidDeorbitEvent {
    pub asteroid: Entity,
}

pub fn process_tethered_asteroids_system(
    query: Query<(Entity, &TetheredAsteroid)>,
    mut power_query: Query<&mut PowerGrid>,
    mut event_writer: EventWriter<AsteroidDeorbitEvent>,
) {
    for mut grid in power_query.iter_mut() {
        for (entity, asteroid) in query.iter() {
            if grid.available_energy >= asteroid.energy_required {
                grid.available_energy -= asteroid.energy_required;
            } else {
                event_writer.send(AsteroidDeorbitEvent { asteroid: entity });
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Mutating `PowerGrid` directly in the system might cause order-dependent bugs if other systems also consume power. Consider standardizing power consumption using the existing `PowerConsumer` components.
- **Performance**: Simple iteration is fast, but we need to ensure priority consumption (e.g., Tethers consume first, or Tethers fail if total consumption exceeds generation).
- **API Improvements**: Map the `AsteroidDeorbitEvent` to target specific `GridPosition`s to handle the localized devastation correctly.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] A `TetheredAsteroid` consumes power from the grid.
- [ ] If power drops below the required amount, an `AsteroidDeorbitEvent` is sent out.

## 7. Technical Guidance
- Integrate with Layer 1's grid destruction tools. The `AsteroidDeorbitEvent` should trigger a massive explosion or obliteration function on a sector.
- Establish the logic for *Tether-Ships* bringing it in from Layer 2.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
