# 1144: Brain Drain Migration

## 1. Overview
The "Brain Drain Migration" feature allows highly skilled Pops (with maximum intelligence stats) to monitor the "freedom" and "living standards" of nearby factions. If a neighboring empire offers a significantly better life, these Pops may emigrate, immediately boosting the rival's tech. This adds tension between providing expensive luxuries/freedoms and locking down borders.

## 2. Dependencies
- `bevy_ecs` setup for Pop entities with intelligence stats.
- Pop components for `Intelligence`, `Freedom`, and `LivingStandard`.
- System to monitor neighboring factions' stats (Layer 3 knowledge).
- Event system for Pop migration/emigration.

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    fn assert_emigrated(mut reader: EventReader<EmigrationEvent>) {
        assert_eq!(reader.read().count(), 1);
    }

    fn assert_not_emigrated(mut reader: EventReader<EmigrationEvent>) {
        assert_eq!(reader.read().count(), 0);
    }

    #[test]
    fn test_pop_emigrates_when_neighbor_has_better_standards() {
        // Arrange: Setup world, Pop with max intelligence, low freedom/standards
        let mut world = World::new();
        let entity = world.spawn((
            Intelligence { value: 100 },
            Freedom { value: 20 },
            LivingStandard { value: 30 },
            Pop,
        )).id();

        // Simulate neighbor with better stats
        world.insert_resource(NeighborStats {
            freedom: 80,
            living_standard: 90,
        });

        world.init_resource::<Events<EmigrationEvent>>();

        // Act: Run the migration check system
        world.run_system_once(check_brain_drain_migration);

        // Assert: Pop should have emigrated (EmigrationEvent fired, entity removed)
        world.run_system_once(assert_emigrated);
        assert!(world.get_entity(entity).is_none());
    }

    #[test]
    fn test_pop_stays_when_neighbor_has_lower_standards() {
        // Arrange: Setup world, Pop with max intelligence, high freedom/standards
        let mut world = World::new();
        let entity = world.spawn((
            Intelligence { value: 100 },
            Freedom { value: 90 },
            LivingStandard { value: 90 },
            Pop,
        )).id();

        // Simulate neighbor with worse stats
        world.insert_resource(NeighborStats {
            freedom: 40,
            living_standard: 50,
        });

        world.init_resource::<Events<EmigrationEvent>>();

        // Act: Run the migration check system
        world.run_system_once(check_brain_drain_migration);

        // Assert: Pop should not have emigrated (No EmigrationEvent, entity still exists)
        world.run_system_once(assert_not_emigrated);
        assert!(world.get_entity(entity).is_some());
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Intelligence {
    pub value: u32,
}

#[derive(Component)]
pub struct Freedom {
    pub value: u32,
}

#[derive(Component)]
pub struct LivingStandard {
    pub value: u32,
}

#[derive(Component)]
pub struct Pop;

#[derive(Resource)]
pub struct NeighborStats {
    pub freedom: u32,
    pub living_standard: u32,
}

#[derive(Event)]
pub struct EmigrationEvent {
    pub pop_entity: Entity,
}

pub fn check_brain_drain_migration(
    mut commands: Commands,
    query: Query<(Entity, &Intelligence, &Freedom, &LivingStandard), With<Pop>>,
    neighbor_stats: Option<Res<NeighborStats>>,
    mut emigration_events: EventWriter<EmigrationEvent>,
) {
    if let Some(neighbor) = neighbor_stats {
        for (entity, intelligence, freedom, living_standard) in query.iter() {
            if intelligence.value >= 100 {
                // Emigrate if neighbor is significantly better
                if neighbor.freedom > freedom.value + 20 && neighbor.living_standard > living_standard.value + 20 {
                    emigration_events.send(EmigrationEvent { pop_entity: entity });
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Code Smells**: Hardcoded thresholds (`>= 100`, `+ 20`). Move these to a configurable resource or component.
- **Performance**: Querying all Pops might be expensive if population is huge. Consider adding a marker component like `HighlySkilled` to only query relevant Pops.
- **API Improvements**: Ensure `NeighborStats` can handle multiple neighbors, perhaps using an entity or ID to specify which neighbor the Pop is emigrating to.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: Pops with max intelligence migrate when neighbor stats are significantly higher.

## 7. Technical Guidance
- **Gotchas**: Remember to register the `EmigrationEvent` in the app setup so it can be used by other systems (e.g., to adjust tech points for the receiving faction).
- **Integration**: The migration check should probably run periodically rather than every tick to save performance.

## 8. Questions
*Builder: add questions here if spec is unclear.*
