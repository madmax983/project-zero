# 047: Pop Relationships

## Overview

Relationships add social depth to the colony. Pops will form opinions of each other based on interactions. These relationships influence morale when pops are in proximity.

Key features:
- **Relationships Component**: Stores affinity values for other pops.
- **Affinity**: A float value (-100.0 to +100.0) representing love/hate.
- **Proximity Effects**: Being near friends boosts morale; enemies lower it.
- **Interaction Hooks**: Systems like `Socialize` (Tavern) can now modify affinity.

## Dependencies

- `004` — Pop Entity
- `031` — Pop Morale (Needs/Morale calculation)
- `028` — Social Tavern (Socialize action)

## RED Phase: Tests First

Write these tests in `src/layer1/social_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::social::{Relationships, AffinityChange, proximity_social_system, modify_affinity_system, SocialBuff};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_relationships_default() {
        let rel = Relationships::default();
        assert!(rel.affinities.is_empty());
    }

    #[test]
    fn test_affinity_change_event() {
        let mut world = World::new();
        let pop1 = world.spawn((Pop, Relationships::default())).id();
        let pop2 = world.spawn(Pop).id();

        // Trigger event to boost affinity
        world.send_event(AffinityChange {
            source: pop1,
            target: pop2,
            amount: 10.0,
        });

        // Register and run system
        let mut schedule = Schedule::default();
        schedule.add_systems(modify_affinity_system);
        schedule.run(&mut world);

        let rel = world.get::<Relationships>(pop1).unwrap();
        assert_eq!(rel.get_affinity(pop2), 10.0);
    }

    #[test]
    fn test_affinity_clamping() {
        let mut world = World::new();
        let pop1 = world.spawn((Pop, Relationships::default())).id();
        let pop2 = world.spawn(Pop).id();

        world.send_event(AffinityChange { source: pop1, target: pop2, amount: 150.0 });

        let mut schedule = Schedule::default();
        schedule.add_systems(modify_affinity_system);
        schedule.run(&mut world);

        let rel = world.get::<Relationships>(pop1).unwrap();
        assert_eq!(rel.get_affinity(pop2), 100.0); // Clamped at 100
    }

    #[test]
    fn test_proximity_morale_buff() {
        let mut world = World::new();

        // Pop 1 and Pop 2 are friends (affinity 50) and nearby
        let pop1 = world.spawn((
            Pop,
            GridPosition { x: 10, y: 10 },
            Relationships::with_affinity(Entity::PLACEHOLDER, 50.0), // Placeholder until updated
        )).id();

        let pop2 = world.spawn((
            Pop,
            GridPosition { x: 10, y: 11 }, // Adjacent
        )).id();

        // Update relationship with real ID
        world.get_mut::<Relationships>(pop1).unwrap().set_affinity(pop2, 50.0);

        // Run proximity system
        let mut schedule = Schedule::default();
        schedule.add_systems(proximity_social_system);
        schedule.run(&mut world);

        // Check for SocialBuff component
        let buff = world.get::<SocialBuff>(pop1);
        assert!(buff.is_some());
        assert!(buff.unwrap().value > 0.0);
    }

    #[test]
    fn test_proximity_morale_debuff() {
        let mut world = World::new();

        // Pop 1 and Pop 2 are enemies (affinity -50)
        let pop1 = world.spawn((
            Pop,
            GridPosition { x: 10, y: 10 },
            Relationships::default(),
        )).id();

        let pop2 = world.spawn((
            Pop,
            GridPosition { x: 10, y: 11 },
        )).id();

        world.get_mut::<Relationships>(pop1).unwrap().set_affinity(pop2, -50.0);

        let mut schedule = Schedule::default();
        schedule.add_systems(proximity_social_system);
        schedule.run(&mut world);

        let buff = world.get::<SocialBuff>(pop1);
        assert!(buff.is_some());
        assert!(buff.unwrap().value < 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Create `src/layer1/social.rs`

```rust
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use crate::layer1::map::GridPosition;

#[derive(Component, Default)]
pub struct Relationships {
    pub affinities: HashMap<Entity, f32>,
}

impl Relationships {
    pub fn get_affinity(&self, target: Entity) -> f32 {
        *self.affinities.get(&target).unwrap_or(&0.0)
    }

    pub fn set_affinity(&mut self, target: Entity, value: f32) {
        self.affinities.insert(target, value.clamp(-100.0, 100.0));
    }

    // Test helper
    pub fn with_affinity(target: Entity, value: f32) -> Self {
        let mut r = Self::default();
        r.set_affinity(target, value);
        r
    }
}

#[derive(Event)]
pub struct AffinityChange {
    pub source: Entity,
    pub target: Entity,
    pub amount: f32,
}

pub fn modify_affinity_system(
    mut events: EventReader<AffinityChange>,
    mut query: Query<&mut Relationships>,
) {
    for event in events.read() {
        if let Ok(mut rel) = query.get_mut(event.source) {
            let current = rel.get_affinity(event.target);
            rel.set_affinity(event.target, current + event.amount);
        }
    }
}

#[derive(Component)]
pub struct SocialBuff {
    pub value: f32,
}

pub fn proximity_social_system(
    mut commands: Commands,
    pops: Query<(Entity, &GridPosition, &Relationships)>,
    other_pops: Query<(Entity, &GridPosition)>,
) {
    // O(N^2) naive implementation for Green phase
    for (entity, pos, rel) in pops.iter() {
        let mut total_buff = 0.0;

        for (other_entity, other_pos) in other_pops.iter() {
            if entity == other_entity { continue; }

            // Naive distance check
            let dx = (pos.x - other_pos.x).abs();
            let dy = (pos.y - other_pos.y).abs();
            let distance = dx.max(dy); // Chebyshev

            if distance <= 5 { // 5 tile radius
                let affinity = rel.get_affinity(other_entity);
                if affinity > 20.0 {
                    total_buff += 0.1; // Small boost per friend
                } else if affinity < -20.0 {
                    total_buff -= 0.1; // Small penalty per enemy
                }
            }
        }

        if total_buff.abs() > f32::EPSILON {
            commands.entity(entity).insert(SocialBuff {
                value: total_buff,
            });
        } else {
            commands.entity(entity).remove::<SocialBuff>();
        }
    }
}
```

### 2. Integrate with Execution System

Update `src/layer1/execution.rs` (work_execution_system) to query `Option<&SocialBuff>`.

```rust
// In work_execution_system query:
// Query<..., Option<&Needs>, Option<&Memories>, Option<&SocialBuff>>

// Inside morale calculation:
let morale = needs.map_or(0.5, |n| {
    let base = n.morale();

    // Apply memories
    let memory_mod = memories.map_or(0.0, |m| {
        m.items.iter().map(|i| i.memory_type.base_mood_impact() * i.intensity).sum::<f32>()
    });

    // Apply social buff
    let social_mod = social_buff.map_or(0.0, |s| s.value);

    (base + memory_mod + social_mod).clamp(0.0, 1.0)
});
```

## REFACTOR Phase: Quality & Design

- **Performance**: The O(N^2) proximity check is inefficient for >100 pops. Use a spatial hash or restrict to interacting pops (e.g., same room).
- **Events**: Add specific `SocialInteraction` events (Chat, Argument) that trigger `AffinityChange` with specific amounts.
- **UI**: Display "Friend of X" in the inspector UI.
- **Decay**: Relationships should drift towards 0 over time (forgotten).

## Acceptance Criteria

- [ ] `Relationships` component stores affinity correctly.
- [ ] `AffinityChange` event updates affinity and clamps values.
- [ ] Pops near friends get a positive `SocialBuff`.
- [ ] Pops near enemies get a negative `SocialBuff`.
- [ ] Morale calculation includes `SocialBuff`.
- [ ] Tests pass.

## Technical Guidance

- Use `GridPosition` coordinates directly for distance until `distance_chebyshev` helper is available.
- Keep affinity updates decoupled via events.
- Ensure `SocialBuff` is removed when no friends are nearby.

## Questions

*Builder: add questions here if spec is unclear.*
