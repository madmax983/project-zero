# 943: The Martyr's Shrine

## Overview

If Pops die while performing a critical job during an active crisis, the specific tile where they died becomes a temporary "Martyr's Shrine." Pops working near this tile gain a massive, overriding boost to productivity and ignore all negative mood effects, fueled by collective grief and purpose. However, while under this effect, they rapidly drain their own health due to self-neglect, potentially creating more martyrs.

## Dependencies

- None explicitly required, assumes existing Grid, Pop, Death Event, and Jobs systems.

## 1. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::jobs::ProductionModifier;
    use crate::layer1::grid::Position;

    #[test]
    fn test_shrine_creation_on_crisis_death() {
        // Arrange
        let mut app = App::new();
        app.add_event::<PopDiedEvent>();
        app.add_systems(Update, create_martyr_shrine_system);

        // Simulate a crisis death
        app.world_mut().send_event(PopDiedEvent {
            entity: Entity::from_raw(1),
            position: Position { x: 5, y: 5 },
            was_critical_job: true,
            during_crisis: true,
        });

        // Act
        app.update();

        // Assert
        // A shrine should be spawned at the location of death
        let mut query = app.world_mut().query::<(&MartyrsShrine, &Position)>();
        let mut found = false;
        for (_, pos) in query.iter(app.world()) {
            if pos.x == 5 && pos.y == 5 {
                found = true;
                break;
            }
        }
        assert!(found, "Martyr's Shrine should be created on crisis death.");
    }

    #[test]
    fn test_shrine_effect_on_nearby_workers() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, apply_shrine_aura_system);

        // Spawn a shrine
        app.world_mut().spawn((
            MartyrsShrine { radius: 3 },
            Position { x: 5, y: 5 },
        ));

        // Spawn a worker nearby
        let worker_id = app.world_mut().spawn((
            Position { x: 6, y: 5 },
            ProductionModifier { multiplier: 1.0 },
            Needs { health: 100.0, ..Default::default() },
        )).id();

        // Spawn a worker far away
        let far_worker_id = app.world_mut().spawn((
            Position { x: 20, y: 20 },
            ProductionModifier { multiplier: 1.0 },
            Needs { health: 100.0, ..Default::default() },
        )).id();

        // Act
        app.update();

        // Assert
        // Nearby worker should have a production boost and health drain
        let nearby_modifier = app.world().get::<ProductionModifier>(worker_id).unwrap();
        assert!(nearby_modifier.multiplier > 1.0);
        assert!(app.world().get::<ShrineFrenzy>(worker_id).is_some());

        // Far worker should be unaffected
        let far_modifier = app.world().get::<ProductionModifier>(far_worker_id).unwrap();
        assert_eq!(far_modifier.multiplier, 1.0);
        assert!(app.world().get::<ShrineFrenzy>(far_worker_id).is_none());
    }

    #[test]
    fn test_shrine_health_drain() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(Time::default());
        app.add_systems(Update, shrine_frenzy_health_drain_system);

        let worker_id = app.world_mut().spawn((
            ShrineFrenzy,
            Needs { health: 100.0, ..Default::default() },
        )).id();

        // Act
        // Move time forward slightly to apply drain
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(1));
        app.update();

        // Assert
        let needs = app.world().get::<Needs>(worker_id).unwrap();
        assert!(needs.health < 100.0, "Health should drain while in frenzy.");
    }
}
```

## 2. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Stub structures for dependencies
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct Position { pub x: i32, pub y: i32 }

#[derive(Event)]
pub struct PopDiedEvent {
    pub entity: Entity,
    pub position: Position,
    pub was_critical_job: bool,
    pub during_crisis: bool,
}

#[derive(Component)]
pub struct ProductionModifier {
    pub multiplier: f32,
}

#[derive(Component, Default)]
pub struct Needs {
    pub health: f32,
    pub food: f32,
}

// Feature components
#[derive(Component)]
pub struct MartyrsShrine {
    pub radius: i32,
}

#[derive(Component)]
pub struct ShrineFrenzy;

// Systems
pub fn create_martyr_shrine_system(
    mut commands: Commands,
    mut events: EventReader<PopDiedEvent>,
) {
    for event in events.read() {
        if event.was_critical_job && event.during_crisis {
            commands.spawn((
                MartyrsShrine { radius: 3 },
                event.position,
            ));
        }
    }
}

pub fn apply_shrine_aura_system(
    mut commands: Commands,
    shrines: Query<(&MartyrsShrine, &Position)>,
    mut workers: Query<(Entity, &Position, &mut ProductionModifier), Without<ShrineFrenzy>>,
) {
    for (shrine, shrine_pos) in shrines.iter() {
        for (worker_entity, worker_pos, mut modifier) in workers.iter_mut() {
            let dx = (shrine_pos.x - worker_pos.x).abs();
            let dy = (shrine_pos.y - worker_pos.y).abs();

            // Simple Manhattan distance
            if dx + dy <= shrine.radius {
                modifier.multiplier *= 2.0;
                commands.entity(worker_entity).insert(ShrineFrenzy);
            }
        }
    }
}

pub fn shrine_frenzy_health_drain_system(
    mut workers: Query<&mut Needs, With<ShrineFrenzy>>,
    time: Res<Time>,
) {
    for mut needs in workers.iter_mut() {
        needs.health -= 5.0 * time.delta_secs();
    }
}
```

## 3. REFACTOR Phase: Quality & Design

- The distance check in `apply_shrine_aura_system` should ideally use a spatial hash or spatial query if there are many Pops to avoid O(N*M) performance issues.
- Introduce an event or mechanism for shrines to decay over time, preventing permanent overpowered buff zones.
- Add components like `ImmuneToMood` for the `ShrineFrenzy` to enforce the ignoring of negative mood effects.
- Magic numbers for radius and drain should be configurable variables.

## 4. Acceptance Criteria (Testable)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new code.
- [ ] Dying during a crisis creates a Shrine.
- [ ] Shrines boost production of nearby workers but drain their health.

## 5. Technical Guidance

- Place in a new module like `src/layer1/events/shrine.rs` or `src/layer1/culture.rs`.
- Ensure you handle the removal of `ShrineFrenzy` when a Pop moves away from the shrine, which will require an `else` branch or a separate cleanup system based on distance.

## 6. Questions

*Builder: add questions here if spec is unclear.*
