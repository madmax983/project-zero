# 266: Ancestral Graves

## 1. Overview

**Layer:** 1
**Fantasy:** The history of the colony is written on the land itself.
**Mechanic:** Dead pops leave "Grave" tiles. Relatives or friends visit graves for mood buffs. Graves cannot be built over without a severe "Sacrilege" penalty.
**Emergence:** Your efficient city plan is ruined by a poorly placed cemetery from the starving first winter. A "City of the Dead" district naturally forms.
**Tension:** Respect the dead (happiness) vs. expand the factory (efficiency).

## 2. Dependencies

- `004` Pop Entity (for death logic)
- `031` Pop Morale / Mood (for buffs/debuffs)
- `047` Pop Relationships (for relatives/friends)
- `006` Building Placement (for preventing placement on graves without penalty)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, DeathEvent};
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Mood;
    use crate::layer1::graves::{Grave, spawn_grave_on_death_system, visit_grave_system, build_over_grave_system};
    use crate::layer1::building::{BuildingPlacementEvent, DemolishEvent};

    #[test]
    fn test_grave_spawns_on_pop_death() {
        let mut world = World::new();
        let pop_entity = world.spawn((Pop, GridPosition { x: 5, y: 5 })).id();

        let mut events = Events::<DeathEvent>::default();
        events.send(DeathEvent { entity: pop_entity });
        world.insert_resource(events);

        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_grave_on_death_system);
        schedule.run(&mut world);

        let mut query = world.query::<(&Grave, &GridPosition)>();
        let found = query.iter(&world).any(|(_, pos)| pos.x == 5 && pos.y == 5);
        assert!(found, "Grave should be spawned at the location of Pop death");
    }

    #[test]
    fn test_visiting_grave_provides_mood_buff() {
        let mut world = World::new();
        let grave_entity = world.spawn((Grave { deceased_pop: Entity::PLACEHOLDER }, GridPosition { x: 5, y: 5 })).id();
        let pop_entity = world.spawn((Pop, GridPosition { x: 5, y: 5 }, Mood::default())).id();

        // Mock a visit action
        let mut schedule = Schedule::default();
        schedule.add_systems(visit_grave_system);
        schedule.run(&mut world);

        let mood = world.get::<Mood>(pop_entity).unwrap();
        assert!(mood.current_value > 0.0, "Visiting grave should increase mood");
    }

    #[test]
    fn test_building_over_grave_triggers_sacrilege() {
        let mut world = World::new();
        let grave_entity = world.spawn((Grave { deceased_pop: Entity::PLACEHOLDER }, GridPosition { x: 5, y: 5 })).id();
        let builder_pop = world.spawn((Pop, Mood::default())).id();

        let mut events = Events::<BuildingPlacementEvent>::default();
        events.send(BuildingPlacementEvent { builder: builder_pop, position: GridPosition { x: 5, y: 5 } });
        world.insert_resource(events);

        let mut schedule = Schedule::default();
        schedule.add_systems(build_over_grave_system);
        schedule.run(&mut world);

        let mood = world.get::<Mood>(builder_pop).unwrap();
        assert!(mood.current_value < 0.0, "Building over grave should apply Sacrilege mood penalty");

        // Ensure grave is demolished
        let mut query = world.query::<&Grave>();
        assert_eq!(query.iter(&world).count(), 0, "Grave should be removed after building over it");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/graves.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::{Pop, DeathEvent};
use crate::layer1::morale::Mood;
use crate::layer1::building::BuildingPlacementEvent;

#[derive(Component)]
pub struct Grave {
    pub deceased_pop: Entity,
}

pub fn spawn_grave_on_death_system(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    pop_query: Query<&GridPosition, With<Pop>>,
) {
    for event in death_events.iter() {
        if let Ok(pos) = pop_query.get(event.entity) {
            commands.spawn((
                Grave { deceased_pop: event.entity },
                pos.clone(),
            ));
        }
    }
}

pub fn visit_grave_system(
    mut pop_query: Query<(&GridPosition, &mut Mood), With<Pop>>,
    grave_query: Query<&GridPosition, With<Grave>>,
) {
    for (pop_pos, mut mood) in &mut pop_query {
        for grave_pos in &grave_query {
            if pop_pos.x == grave_pos.x && pop_pos.y == grave_pos.y {
                mood.current_value += 5.0; // Minimal buff implementation
            }
        }
    }
}

pub fn build_over_grave_system(
    mut commands: Commands,
    mut build_events: EventReader<BuildingPlacementEvent>,
    grave_query: Query<(Entity, &GridPosition), With<Grave>>,
    mut pop_query: Query<&mut Mood, With<Pop>>,
) {
    for event in build_events.iter() {
        for (grave_entity, grave_pos) in &grave_query {
            if event.position.x == grave_pos.x && event.position.y == grave_pos.y {
                // Apply sacrilege penalty
                if let Ok(mut mood) = pop_query.get_mut(event.builder) {
                    mood.current_value -= 20.0;
                }
                // Despawn grave
                commands.entity(grave_entity).despawn();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Relationships**: The `visit_grave_system` currently buffs any pop on the tile. It should be refactored to check `Relationship` components (Spec 047) to ensure only relatives/friends get the true "Closure" buff, while others might just get a minor "Reflection" buff or none at all.
- **Utility AI Action**: Define a `VisitGrave` Pop Action using `ActionType` and `ScorableCandidate` so that pops intentionally route to graves when their `Social` or `Leisure` needs require fulfillment.
- **UI Marker**: Ensure `Grave` entities have a specific visual representation on the `TerrainGrid` to warn players of the Sacrilege penalty.
- **Global Morale vs Local**: Determine if Sacrilege applies a global `Colony Edict`/Unrest penalty (Spec 050) or just to the builder. Initially applied to builder, but a global notification `NotificationSeverity::Warning` should be sent.

## 6. Acceptance Criteria

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Grave correctly spawns at Pop death location
- [ ] Pop visiting grave receives a Mood buff
- [ ] Building over a grave penalizes Mood and removes the grave

## 7. Technical Guidance

- A Pop death is a critical event. Hook `spawn_grave_on_death_system` closely to the end of the `Execution` phase.
- Be careful with `Entity` ID recycling. `Grave` stores `deceased_pop: Entity`. If querying relationships later, ensure the dead pop's data is either preserved in a `Chronicle/Memory` system (Spec 036) or just store the pop's name/ID in the `Grave` struct instead of the raw ECS `Entity` which might be invalid.
- Recommend storing `deceased_name: String` in `Grave` instead of `Entity` if the Pop entity is fully despawned upon death.

## 8. Questions

*Builder: add questions here if spec is unclear.*
