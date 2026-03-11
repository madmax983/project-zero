# 129: The Colony Mascot

## Overview

Introduces a **Colony Mascot**, a unique non-hostile animal that wanders the colony, specifically targeting Social Zones. Pops near the mascot receive a significant **Mood Buff**. If the mascot dies (from starvation, raid, or accident), the entire colony suffers a **Grief** event (negative memory/morale hit).

This adds emotional stakes to the simulation and a reason to protect non-productive entities.

## Dependencies

- `048` — Hostile Fauna (Base `Fauna` entity structure)
- `031` — Pop Morale (Buff system)
- `097` — Social Tavern (Social Zones as targets)
- `057` — Funeral Rites (Grief concept)
- `004` — Pop Entity (Movement/Position)

## RED Phase: Tests First

Write these tests in `src/layer1/mascot_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::fauna::{Fauna, FaunaType, FaunaState};
    use crate::layer1::mascot::{Mascot, MascotBuff, mascot_behavior_system, mascot_buff_system, mascot_death_grief_system};
    use crate::layer1::pop::{Pop, GridPosition};
    use crate::layer1::zone::{Zone, ZoneType};
    use crate::layer1::health::{Health, DeathEvent};
    use crate::layer1::memory::{Memories, MemoryType};
    use crate::layer1::needs::Needs;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<DeathEvent>>();
        world
    }

    #[test]
    fn test_mascot_spawn_defaults() {
        let mascot = Mascot::default();
        // Should have a name or type
        assert!(!mascot.name.is_empty());
    }

    #[test]
    fn test_mascot_seeks_social_zone() {
        let mut world = setup_world();

        // Spawn Mascot at (0,0)
        let mascot = world.spawn((
            Fauna { fauna_type: FaunaType::Mascot, ..Default::default() },
            Mascot::default(),
            GridPosition { x: 0, y: 0 },
        )).id();

        // Spawn Social Zone at (10,10)
        world.spawn((
            Zone { zone_type: ZoneType::Social, ..Default::default() },
            GridPosition { x: 10, y: 10 },
        ));

        // Run behavior
        let mut schedule = Schedule::default();
        schedule.add_systems(mascot_behavior_system);
        schedule.run(&mut world);

        // Mascot should be moving towards (10,10)
        // Check if Fauna state is Move or similar, or check for MovementTarget component
        // Assuming Fauna uses `target_position` in some way or sets a state
        let fauna = world.get::<Fauna>(mascot).unwrap();
        // If reusing Fauna logic, it might set a target.
        // For Mascot specific logic, we might set a custom component or reuse Fauna's target.
        // Let's assume we set the `target` field of Fauna to the zone entity?
        // Or better, check if a MovementTarget was added/updated.

        // For this test, let's assume mascot_behavior_system sets the Fauna target position directly
        // or adds a MovementTarget.
        use crate::layer1::execution::MovementTarget;
        let target = world.get::<MovementTarget>(mascot);
        assert!(target.is_some());
        assert_eq!(target.unwrap().target_position, GridPosition { x: 10, y: 10 });
    }

    #[test]
    fn test_mascot_buff_application() {
        let mut world = setup_world();

        // Mascot at (5,5)
        world.spawn((
            Fauna { fauna_type: FaunaType::Mascot, ..Default::default() },
            Mascot::default(),
            GridPosition { x: 5, y: 5 },
        ));

        // Pop nearby at (5,6)
        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 6 },
            Needs::default(),
        )).id();

        // Pop far away at (0,0)
        let pop_far = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs::default(),
        )).id();

        // Run buff system
        let mut schedule = Schedule::default();
        schedule.add_systems(mascot_buff_system);
        schedule.run(&mut world);

        // Near pop gets buff
        assert!(world.get::<MascotBuff>(pop).is_some());

        // Far pop gets nothing
        assert!(world.get::<MascotBuff>(pop_far).is_none());
    }

    #[test]
    fn test_mascot_death_causes_grief() {
        let mut world = setup_world();

        // Spawn Mascot
        let mascot = world.spawn((
            Fauna { fauna_type: FaunaType::Mascot, ..Default::default() },
            Mascot { name: "Sparky".to_string() },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Spawn Pop
        let pop = world.spawn((
            Pop,
            Memories::default(),
        )).id();

        // Trigger Death Event
        world.send_event(DeathEvent { entity: mascot });

        // Run death system
        let mut schedule = Schedule::default();
        schedule.add_systems(mascot_death_grief_system);
        schedule.run(&mut world);

        // Pop should have Grief memory
        let memories = world.get::<Memories>(pop).unwrap();
        // Assuming MemoryType::MascotDeath exists or generic Grief
        assert!(memories.contains(MemoryType::MascotDeath));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Enums

In `src/layer1/fauna.rs`:
```rust
pub enum FaunaType {
    // ... existing
    Mascot,
}
```

In `src/layer1/memory.rs`:
```rust
pub enum MemoryType {
    // ... existing
    MascotDeath,
}
```

### 2. Create `src/layer1/mascot.rs`

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::fauna::{Fauna, FaunaType, FaunaState};
use crate::layer1::zone::{Zone, ZoneType};
use crate::layer1::pop::Pop;
use crate::layer1::execution::{MovementTarget, ActionType};
use crate::layer1::health::DeathEvent;
use crate::layer1::memory::{Memories, MemoryType};

#[derive(Component, Default)]
pub struct Mascot {
    pub name: String,
}

#[derive(Component)]
pub struct MascotBuff {
    pub amount: f32,
    pub duration: u32,
}

pub fn mascot_behavior_system(
    mut commands: Commands,
    mut query: Query<(Entity, &GridPosition, &mut Fauna), With<Mascot>>,
    zones: Query<(&GridPosition, &Zone)>,
) {
    for (entity, pos, mut fauna) in query.iter_mut() {
        if fauna.state != FaunaState::Wander {
            continue;
        }

        // Find nearest Social/Dining zone
        let mut best_target = None;
        let mut min_dist = i32::MAX;

        for (zone_pos, zone) in zones.iter() {
            if matches!(zone.zone_type, ZoneType::Social | ZoneType::Dining) {
                let dist = pos.distance_manhattan(zone_pos);
                if dist < min_dist {
                    min_dist = dist;
                    best_target = Some(*zone_pos);
                }
            }
        }

        if let Some(target) = best_target {
            // If at target, wander locally or stay?
            // Minimal: move to it.
            if min_dist > 1 {
                commands.entity(entity).insert(MovementTarget {
                    target_entity: Entity::PLACEHOLDER,
                    target_position: target,
                    for_action: ActionType::Idle,
                });
            }
        }
    }
}

pub fn mascot_buff_system(
    mut commands: Commands,
    mascots: Query<&GridPosition, With<Mascot>>,
    pops: Query<(Entity, &GridPosition), With<Pop>>,
) {
    for (pop_entity, pop_pos) in pops.iter() {
        let mut near_mascot = false;
        for mascot_pos in mascots.iter() {
            if pop_pos.distance_chebyshev(mascot_pos) <= 5 {
                near_mascot = true;
                break;
            }
        }

        if near_mascot {
            commands.entity(pop_entity).insert(MascotBuff {
                amount: 0.1, // +10% Mood equivalent
                duration: 1, // Applied every tick while near
            });
        } else {
            commands.entity(pop_entity).remove::<MascotBuff>();
        }
    }
}

pub fn mascot_death_grief_system(
    mut events: EventReader<DeathEvent>,
    mascots: Query<&Mascot>, // To check if dead entity was mascot
    mut pops: Query<&mut Memories, With<Pop>>,
) {
    for event in events.read() {
        // We can't query the component on the dead entity if it's despawned?
        // DeathEvent usually fires BEFORE despawn.
        // If not, we need a separate tracker.
        // Assuming DeathEvent fires before despawn:
        if let Ok(mascot) = mascots.get(event.entity) {
            // It was a mascot!
            for mut memories in pops.iter_mut() {
                memories.add(MemoryType::MascotDeath, 100); // High intensity
            }
        }
    }
}
```

### 3. Integrate Buff

Update `src/layer1/needs.rs` (or wherever mood is calculated) to check for `MascotBuff`.

```rust
// Inside morale calculation
if let Some(buff) = world.get::<MascotBuff>(entity) {
    morale += buff.amount;
}
```

## REFACTOR Phase: Quality & Design

- **Interaction**: Pops should stop and "Pet" the mascot (animation/state).
- **Naming**: Allow player to rename the mascot.
- **Variety**: Different mascots (Cat, Dog, Capybara) with different buffs/behaviors.
- **Needs**: Mascots need food too. If they starve, it's tragic.
- **Zone Wandering**: Use a random point *within* the zone, not just the center, to look natural.

## Acceptance Criteria

- [ ] `Mascot` component exists.
- [ ] `MascotDeath` memory type exists.
- [ ] Mascot automatically moves towards Social/Dining zones.
- [ ] Pops within 5 tiles of a mascot get `MascotBuff`.
- [ ] `MascotBuff` increases Morale.
- [ ] When a mascot dies, all pops receive `MascotDeath` memory.
- [ ] Tests pass.

## Technical Guidance

- Ensure `DeathEvent` is ordered before entity despawn in `death_system`.
- Use `distance_chebyshev` for consistency with other proximity checks.
- Keep `MascotBuff` ephemeral (duration 1 tick) if applied every frame, or persistent with a timer if applied on interaction.

## Questions

- Can we have multiple mascots? (Yes, buff should probably not stack infinitely, maybe cap at 1 instance).
- Do mascots trigger "Hostile Fauna" traps? (They should trigger traps but traps should ideally filter... or not. Tragedy is emergent).
  - *Architect:* Yes, mascots will trigger traps. Emergent tragedy is intended.

*Architect:* Limit the colony to one active mascot at a time to prevent buff stacking.
