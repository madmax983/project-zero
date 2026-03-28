# 709: The Kessler Cage

## Overview

Reckless orbital expansion has a slow, inevitable consequence: becoming trapped on your own planet. Every destroyed ship, abandoned orbital station, or failed missile launch in Layer 2 adds to a "Debris Cloud" above the planet. As the cloud density increases, there is a rising percentage chance that any ship entering or leaving the atmosphere is critically damaged or destroyed, which in turn adds *more* debris to the cloud.

## Dependencies

- Requires an existing concept of fleets, ships, or nodes representing orbital structures/vessels in Layer 2.

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::debris::DebrisCloud;
    use crate::layer2::movement::{Fleet, Journey};
    use crate::shared::random::SeededRng;
    use rand::SeedableRng;

    #[test]
    fn test_debris_accumulation() {
        let mut app = App::new();
        app.add_systems(Update, accumulate_debris_system);

        let planet = app.world_mut().spawn(DebrisCloud { density: 10.0 }).id();
        app.world_mut().spawn((
            DestroyedVessel,
            OrbitalLocation { node: planet }
        ));

        // Act
        app.update();

        // Assert: Cloud density increased
        let cloud = app.world().get::<DebrisCloud>(planet).unwrap();
        assert!(cloud.density > 10.0);
    }

    #[test]
    fn test_ship_transit_damage_high_density() {
        let mut app = App::new();
        app.insert_resource(SeededRng::from_seed(42));
        app.add_systems(Update, transit_debris_system);

        let planet = app.world_mut().spawn(DebrisCloud { density: 95.0 }).id();
        let ship = app.world_mut().spawn((
            Fleet { hp: 100 },
            TransitAttempt { destination: planet }
        )).id();

        // Act
        app.update();

        // Assert: Ship takes damage due to high debris density
        let fleet = app.world().get::<Fleet>(ship).unwrap();
        assert!(fleet.hp < 100);
    }

    #[test]
    fn test_destroyed_ship_adds_more_debris() {
        let mut app = App::new();
        app.add_event::<ShipDestroyedEvent>();
        app.add_systems(Update, destroyed_ship_cascade_system);

        let planet = app.world_mut().spawn(DebrisCloud { density: 50.0 }).id();

        // Act
        app.world_mut().send_event(ShipDestroyedEvent { location: planet });
        app.update();

        // Assert: Event triggered debris increase
        let cloud = app.world().get::<DebrisCloud>(planet).unwrap();
        assert!(cloud.density > 50.0);
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer2::movement::{Fleet, Journey};
use crate::shared::random::SeededRng;
use rand::Rng;

#[derive(Component)]
pub struct DebrisCloud {
    pub density: f32, // 0.0 to 100.0 (or theoretically higher)
}

#[derive(Component)]
pub struct DestroyedVessel;

#[derive(Component)]
pub struct OrbitalLocation {
    pub node: Entity,
}

#[derive(Component)]
pub struct TransitAttempt {
    pub destination: Entity,
}

#[derive(Event)]
pub struct ShipDestroyedEvent {
    pub location: Entity,
}

pub fn accumulate_debris_system(
    mut commands: Commands,
    mut clouds: Query<&mut DebrisCloud>,
    vessels: Query<(Entity, &OrbitalLocation), With<DestroyedVessel>>,
) {
    for (entity, location) in vessels.iter() {
        if let Ok(mut cloud) = clouds.get_mut(location.node) {
            cloud.density += 5.0; // Flat increase per vessel
        }
        commands.entity(entity).despawn(); // Clean up the destroyed vessel entity
    }
}

pub fn transit_debris_system(
    mut commands: Commands,
    mut clouds: Query<&DebrisCloud>,
    mut ships: Query<(Entity, &mut Fleet, &TransitAttempt)>,
    mut rng: ResMut<SeededRng>,
) {
    for (entity, mut fleet, transit) in ships.iter_mut() {
        if let Ok(cloud) = clouds.get(transit.destination) {
            let hit_chance = cloud.density / 100.0;
            if rng.0.gen::<f32>() < hit_chance {
                fleet.hp -= 20; // Hardcoded damage
                if fleet.hp <= 0 {
                    commands.entity(entity).insert(DestroyedVessel);
                    commands.entity(entity).insert(OrbitalLocation { node: transit.destination });
                }
            }
        }
        commands.entity(entity).remove::<TransitAttempt>(); // Resolve the attempt
    }
}

pub fn destroyed_ship_cascade_system(
    mut events: EventReader<ShipDestroyedEvent>,
    mut clouds: Query<&mut DebrisCloud>,
) {
    for event in events.read() {
        if let Ok(mut cloud) = clouds.get_mut(event.location) {
            cloud.density += 10.0; // Larger chunk for an event-driven destruction
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Code Smell**: Hardcoded `density` increase amounts and `hp` damage. Introduce a `KesslerConfig` resource to hold these constants.
- **Missing Mechanic**: We need a way to clear the debris (e.g., specialized "cleaner ships"). This spec focuses purely on the negative feedback loop.
- **Event Flow**: `ShipDestroyedEvent` is currently redundant with the `DestroyedVessel` component check. Consolidate these into a single event-driven flow or component-driven flow. A component query `With<DestroyedVessel>` is likely better for ECS.

## Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Destroyed ships increment the `DebrisCloud` density of their corresponding node.
- [ ] Ships attempting transit have a percentage chance to take damage based on density.
- [ ] Ships destroyed during transit cascade into further debris.

## Technical Guidance
- Implement in `src/layer2/kessler.rs` or alongside existing debris logic.
- The `TransitAttempt` system needs to run *before* normal `Fleet` movement finishes, acting as an intercept check.
- Keep the `DebrisCloud` component simple. Let the emergent feedback loop do the heavy lifting for tension.

## Questions
*Builder: add questions here if spec is unclear.*
