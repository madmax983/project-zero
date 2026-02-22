# 200: Superstitious Totems

## Overview

*"It's not just a rock. It's the rock that stopped the airlock from closing on my foot."*

In the face of the void and mounting stress, Pops cling to small comforts. This feature introduces **Totems**: personal trinkets (Lucky Coins, Rabbit Feet, Strange Geodes) that Pops spontaneously create or find when highly stressed. Equipped Totems reduce stress accumulation. Losing a Totem (unequipping or destruction) triggers a **Bad Omen**, causing a massive stress spike.

This adds a layer of psychological resilience and vulnerability, making inventory management a narrative choice (do you keep the useless rock or carry more ammo?).

## Dependencies

- `058` — Personal Tools (Equipment slots)
- `127` — Stress Breakdowns (Stress mechanics)
- `031` — Pop Morale (Mood modifiers)

## RED Phase: Tests First

Write these tests in `src/layer1/totem_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Equipment};
    use crate::layer1::stress::{StressTracker, check_stress_breakdown_system};
    use crate::layer1::totems::{Totem, check_spontaneous_totem_creation, unequip_totem_system, BadOmen};
    use crate::layer1::items::Item;
    use crate::layer1::needs::Needs;

    #[test]
    fn test_equipment_has_totem_slot() {
        let mut world = World::new();
        let entity = world.spawn((Pop, Equipment::default())).id();
        let eq = world.get::<Equipment>(entity).unwrap();
        // This field does not exist yet
        assert!(eq.totem.is_none());
    }

    #[test]
    fn test_totem_creation_at_high_stress() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_spontaneous_totem_creation);

        // Pop with high stress (but not broken) and empty totem slot
        let pop = world.spawn((
            Pop,
            Equipment::default(),
            StressTracker { accumulated_stress: 80.0 }, // Near breakdown (100)
            Needs::default(),
        )).id();

        // Run system (might need to run multiple times or force probability in test)
        // For test purposes, assume system creates if stress > 75 and slot empty
        schedule.run(&mut world);

        let eq = world.get::<Equipment>(pop).unwrap();
        assert!(eq.totem.is_some());

        let totem_entity = eq.totem.unwrap();
        let totem = world.get::<Totem>(totem_entity).unwrap();
        assert!(totem.stress_relief > 0.0);
    }

    #[test]
    fn test_totem_reduces_stress_accumulation() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        // Spawn Totem
        let totem = world.spawn((
            Item,
            Totem { stress_relief: 0.5 },
        )).id();

        // Pop with low morale (gains +1.0 stress normally)
        let pop = world.spawn((
            Pop,
            Equipment { totem: Some(totem), ..Default::default() },
            Needs { hunger: 0.1, rest: 0.1, leisure: 0.1 }, // Morale < 0.15
            StressTracker::default(),
        )).id();

        schedule.run(&mut world);

        let tracker = world.get::<StressTracker>(pop).unwrap();
        // Base change +1.0, Totem -0.5 => Net +0.5
        assert_eq!(tracker.accumulated_stress, 0.5);
    }

    #[test]
    fn test_unequipping_triggers_bad_omen() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(unequip_totem_system);

        let totem = world.spawn(Totem { stress_relief: 0.5 }).id();
        let pop = world.spawn((
            Pop,
            Equipment { totem: Some(totem), ..Default::default() }, // Starts equipped
            StressTracker::default(),
        )).id();

        // Force unequip (simulate UI or logic removal)
        // We might need a command or event to trigger this system,
        // or the system watches for Component Removal/Change.
        // For this test, we manually remove the item from Equipment component
        // and expect the system to detect the change?
        // Actually, detecting "Removal" from a component field is hard in ECS.
        // Better: trigger a "UnequipEvent".

        world.entity_mut(pop).insert(Equipment { totem: None, ..Default::default() });

        // This is tricky to test with just systems unless we use "Changed<Equipment>".
        // Let's assume `unequip_totem_system` checks Changed<Equipment> and sees None where there was Some.
        // But Changed doesn't give previous value.
        // Alternative: Use an event `EventWriter<UnequipItem>`.

        // Let's assume we fire an event
        let mut events = world.resource_mut::<Events<crate::layer1::items::UnequipEvent>>();
        events.send(crate::layer1::items::UnequipEvent {
            actor: pop,
            item: totem,
            slot: "totem".to_string()
        });

        schedule.run(&mut world);

        // Should have BadOmen
        assert!(world.get::<BadOmen>(pop).is_some());

        // Should have high stress spike
        let tracker = world.get::<StressTracker>(pop).unwrap();
        assert!(tracker.accumulated_stress >= 50.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Equipment (`src/layer1/items.rs`)

```rust
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Equipment {
    pub tool: Option<Entity>,
    pub weapon: Option<Entity>,
    pub body: Option<Entity>,
    pub head: Option<Entity>,
    pub totem: Option<Entity>, // Add this
}
```

### 2. Define Totem (`src/layer1/totems.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Totem {
    pub stress_relief: f32, // e.g. 0.5
    pub description: String,
}

#[derive(Component, Debug)]
pub struct BadOmen {
    pub duration: u32,
}

pub fn check_spontaneous_totem_creation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut crate::layer1::items::Equipment, &crate::layer1::stress::StressTracker)>,
) {
    for (entity, mut eq, stress) in query.iter_mut() {
        if eq.totem.is_none() && stress.accumulated_stress > 75.0 {
            // 1% chance per tick (simplified for Green phase)
            if rand::random::<f32>() < 0.01 {
                let totem = commands.spawn((
                    crate::layer1::items::Item,
                    Totem {
                        stress_relief: 0.5,
                        description: "Lucky Rock".to_string()
                    }
                )).id();
                eq.totem = Some(totem);

                // Immediate relief
                // Note: StressTracker is read-only in query, need mut if we want to reduce it here.
                // For MVP, just creating it is enough.
            }
        }
    }
}
```

### 3. Update Stress Logic (`src/layer1/stress.rs`)

Modify `check_stress_breakdown_system`:

```rust
// Add Equipment query
// ...
if let Some(eq) = equipment {
    if let Some(totem_entity) = eq.totem {
        if let Ok(totem) = totem_query.get(totem_entity) {
            total_change -= totem.stress_relief;
        }
    }
}
// ...
```

### 4. Unequip Logic (`src/layer1/totems.rs`)

```rust
pub fn unequip_totem_system(
    mut commands: Commands,
    mut events: EventReader<crate::layer1::items::UnequipEvent>,
    query_totem: Query<&Totem>,
    mut query_stress: Query<&mut crate::layer1::stress::StressTracker>,
) {
    for event in events.read() {
        if let Ok(_) = query_totem.get(event.item) {
             // It was a totem
             commands.entity(event.actor).insert(BadOmen { duration: 1000 });
             if let Ok(mut stress) = query_stress.get_mut(event.actor) {
                 stress.accumulated_stress += 50.0;
             }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Totem Types**: Implement specific types with different bonuses (Rabbit Foot = Luck, Geode = Stress, Photo = Morale).
- **Narrative Log**: "Pop X found a Lucky Rock and feels safer." / "Pop X lost their Lucky Rock and is terrified!"
- **Integration**:
  - `UnequipEvent` needs to be standardized if not already.
  - Totems should be droppable/tradable.
  - Bad Omen should apply a visual effect (purple cloud?).

## Acceptance Criteria

- [ ] `Equipment` struct includes `totem`.
- [ ] High stress triggers random Totem creation.
- [ ] Equipped Totem reduces stress accumulation rate.
- [ ] Unequipping/Losing Totem triggers `BadOmen` (Stress spike).
- [ ] Tests pass.

## Technical Guidance

- Ensure `Totem` entities are despawned if the Pop dies, or dropped to the ground (Layer 1 item drop logic).
- Avoid circular dependencies between `totems.rs` and `stress.rs` if possible; strictly speaking `stress` depends on `totems` (for relief) and `totems` depends on `stress` (for creation). This is circular.
  - **Solution**: Keep `Totem` definition in `items.rs` or `totems.rs`.
  - Pass `Totem` query to `check_stress_breakdown_system` in `stress.rs`.
  - Pass `StressTracker` to `check_spontaneous_totem_creation` in `totems.rs`.
  - This is fine as long as module imports don't cycle.
