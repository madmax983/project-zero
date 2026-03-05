# Specification: The Neural Leech (Layer 1)

## 1. Overview
A piece of forbidden tech that allows the player to designate one Pop as a "Neural Hub." This Pop is permanently confined to a tank; their `Rest` and `Social` needs flatline, and their `Stress` maxes out. In exchange, all Pops within a radius gain a massive cognitive buff, working faster and learning instantly as the Hub processes their mental load. If the Hub dies from stress, all linked Pops suffer an immediate, catastrophic mental breakdown.

## 2. Dependencies
- `005` Pop Needs (Stress, Rest, Social).
- `084` Pop Traits (for tracking linked status).
- `051` Pop Skills/XP (for the learning buff).

## 3. RED Phase: Tests First

```rust
// src/layer1/neural_leech_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Needs, Skills};

    fn setup_world() -> World {
        let mut world = World::new();
        // Base setup...
        world
    }

    #[test]
    fn test_neural_hub_buffs_nearby_pops() {
        let mut world = setup_world();

        let hub = world.spawn((
            Pop,
            NeuralHub,
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let worker = world.spawn((
            Pop,
            Skills::default(),
            Transform::from_xyz(5.0, 0.0, 0.0), // Within radius
        )).id();

        world.run_system_once(apply_neural_link_buffs_system);

        // Worker should now have the Linked buff
        assert!(world.entity(worker).contains::<NeuralLinked>());
    }

    #[test]
    fn test_neural_hub_stress_maxes_out() {
        let mut world = setup_world();

        let hub = world.spawn((
            Pop,
            NeuralHub,
            crate::layer1::needs::StressTracker::default(),
        )).id();

        world.run_system_once(process_neural_hub_decay_system);

        let stress = world.get::<crate::layer1::needs::StressTracker>(hub).unwrap();
        assert!(stress.accumulated_stress >= 99.0); // Should be pegged to max
    }

    #[test]
    fn test_hub_death_causes_cascading_breakdown() {
        let mut world = setup_world();

        let worker = world.spawn((
            Pop,
            NeuralLinked { hub_entity: Entity::PLACEHOLDER }, // Will be updated manually for test
        )).id();

        // Simulate hub death event
        world.send_event(NeuralHubDeathEvent { hub_entity: Entity::PLACEHOLDER });

        world.run_system_once(handle_hub_death_system);

        // Worker should now have a severe breakdown component
        assert!(world.entity(worker).contains::<NeuralShock>());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/tech/neural_leech.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::Pop;

#[derive(Component)]
pub struct NeuralHub;

#[derive(Component)]
pub struct NeuralLinked {
    pub hub_entity: Entity,
}

#[derive(Component)]
pub struct NeuralShock; // Causes mental break

#[derive(Event)]
pub struct NeuralHubDeathEvent {
    pub hub_entity: Entity,
}

pub const LINK_RADIUS: f32 = 20.0;

pub fn apply_neural_link_buffs_system(
    mut commands: Commands,
    hub_query: Query<(Entity, &crate::layer1::grid::Transform), With<NeuralHub>>,
    worker_query: Query<(Entity, &crate::layer1::grid::Transform), (With<Pop>, Without<NeuralHub>, Without<NeuralLinked>)>,
) {
    for (hub_entity, hub_tf) in hub_query.iter() {
        for (worker_entity, worker_tf) in worker_query.iter() {
            // Simplified distance check
            let dist = (hub_tf.translation.x - worker_tf.translation.x).abs()
                     + (hub_tf.translation.y - worker_tf.translation.y).abs();

            if dist <= LINK_RADIUS {
                commands.entity(worker_entity).insert(NeuralLinked { hub_entity });
            }
        }
    }
}

pub fn handle_hub_death_system(
    mut commands: Commands,
    mut events: EventReader<NeuralHubDeathEvent>,
    linked_query: Query<(Entity, &NeuralLinked)>,
) {
    for event in events.read() {
        for (entity, link) in linked_query.iter() {
            if link.hub_entity == event.hub_entity {
                commands.entity(entity).remove::<NeuralLinked>();
                commands.entity(entity).insert(NeuralShock);
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Integration:** Hook `NeuralLinked` into `calculate_work_amount` to actually provide the 2x speed buff.
- **Death Hook:** Ensure `handle_hub_death_system` is triggered whenever a Pop with `NeuralHub` is despawned or dies.
- **Visuals:** Add a visible tether or aura effect connecting the Hub to linked Pops.
- **UI:** Show a clear warning when a Pop is designated as a Hub.

## 6. Acceptance Criteria

- [ ] All tests pass.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Pops near a `NeuralHub` receive the `NeuralLinked` component.
- [ ] Hub Pops have their stress pegged to maximum.
- [ ] Destruction of the Hub applies `NeuralShock` to all linked Pops.

## 7. Technical Guidance
- **System Ordering:** `apply_neural_link_buffs_system` should run in the observation phase, before work amounts are calculated.
- **Cleanup:** Remember to remove `NeuralLinked` if a Pop walks out of the radius.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect: I will answer your questions as they come up.*
