# 090: Emotional Contagion

## Overview

Pops are social creatures. Extreme emotions (Terror, Joy, Rage) are infectious. When a Pop reaches a critical morale threshold (Very High or Very Low), they emit an "emotional aura". Other Pops within a short radius have a chance to "catch" this mood, receiving a temporary mood modifier.

This system creates emergent feedback loops:
- **Tantrum Spirals:** One stressed pop causes nearby pops to become stressed, leading to a colony-wide breakdown.
- **Festivals:** One happy pop (e.g., from a great meal or party) lifts the spirits of those around them.

## Dependencies

- `031` — Pop Morale (Core mood system, `Morale` component)
- `004` — Pop Entity (Position/Transform)
- `047` — Pop Relationships (Optional integration for stronger effects on friends)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/morale_tests.rs (or new file src/layer1/contagion_tests.rs)

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::morale::{Morale, MoodModifier, MoodModifierType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::contagion::{EmotionalContagionSystem, ContagionCooldown}; // New module

    #[test]
    fn test_contagion_spreads_negative_mood() {
        let mut world = World::new();
        // Register necessary components/resources
        // world.init_resource::<Time>(); // if needed

        // 1. Create Source Pop (Low Morale)
        let source = world.spawn((
            GridPosition { x: 10, y: 10 },
            Morale { value: 10.0, ..Default::default() }, // Very Low
            ContagionCooldown::default(), // New component to prevent spam
        )).id();

        // 2. Create Target Pop (Neutral Morale, nearby)
        let target = world.spawn((
            GridPosition { x: 11, y: 10 }, // Adjacent
            Morale { value: 50.0, ..Default::default() }, // Neutral
            ContagionCooldown::default(),
        )).id();

        // 3. Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(EmotionalContagionSystem);
        schedule.run(&mut world);

        // 4. Assert Target received negative modifier
        let target_morale = world.get::<Morale>(target).unwrap();
        assert!(target_morale.modifiers.iter().any(|m| m.label == "Witnessed Breakdown"),
            "Target should have 'Witnessed Breakdown' modifier");
    }

    #[test]
    fn test_contagion_spreads_positive_mood() {
        let mut world = World::new();

        // 1. Create Source Pop (High Morale)
        let source = world.spawn((
            GridPosition { x: 10, y: 10 },
            Morale { value: 95.0, ..Default::default() }, // Very High
            ContagionCooldown::default(),
        )).id();

        // 2. Create Target Pop (Neutral Morale, nearby)
        let target = world.spawn((
            GridPosition { x: 11, y: 11 }, // Diagonal
            Morale { value: 50.0, ..Default::default() },
            ContagionCooldown::default(),
        )).id();

        // 3. Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(EmotionalContagionSystem);
        schedule.run(&mut world);

        // 4. Assert Target received positive modifier
        let target_morale = world.get::<Morale>(target).unwrap();
        assert!(target_morale.modifiers.iter().any(|m| m.label == "Witnessed Joy"),
            "Target should have 'Witnessed Joy' modifier");
    }

    #[test]
    fn test_contagion_range_limit() {
        let mut world = World::new();

        // Source
        let source = world.spawn((
            GridPosition { x: 10, y: 10 },
            Morale { value: 5.0, ..Default::default() },
            ContagionCooldown::default(),
        )).id();

        // Distant Target (Outside range, e.g., range is 5)
        let distant_target = world.spawn((
            GridPosition { x: 20, y: 20 },
            Morale { value: 50.0, ..Default::default() },
            ContagionCooldown::default(),
        )).id();

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(EmotionalContagionSystem);
        schedule.run(&mut world);

        // Assert NO modifier
        let target_morale = world.get::<Morale>(distant_target).unwrap();
        assert!(target_morale.modifiers.is_empty(), "Distant target should not be affected");
    }

    #[test]
    fn test_contagion_cooldown() {
        let mut world = World::new();

        // Source
        let source = world.spawn((
            GridPosition { x: 10, y: 10 },
            Morale { value: 5.0, ..Default::default() },
            ContagionCooldown { timer: 100 }, // Recently triggered
        )).id();

        // Target
        let target = world.spawn((
            GridPosition { x: 11, y: 10 },
            Morale { value: 50.0, ..Default::default() },
            ContagionCooldown::default(),
        )).id();

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(EmotionalContagionSystem);
        schedule.run(&mut world);

        // Assert NO modifier (Source on cooldown)
        let target_morale = world.get::<Morale>(target).unwrap();
        assert!(target_morale.modifiers.is_empty(), "Source on cooldown should not spread emotion");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. New Module `src/layer1/contagion.rs`

```rust
use bevy::prelude::*;
use crate::layer1::morale::{Morale, MoodModifier};
use crate::layer1::map::GridPosition;

// Constants
const CONTAGION_RANGE: f32 = 5.0; // Tiles
const CONTAGION_COOLDOWN: u32 = 200; // Ticks
const HIGH_MORALE_THRESHOLD: f32 = 90.0;
const LOW_MORALE_THRESHOLD: f32 = 20.0;

#[derive(Component, Default)]
pub struct ContagionCooldown {
    pub timer: u32,
}

pub fn emotional_contagion_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &GridPosition, &Morale, &mut ContagionCooldown)>,
) {
    // 1. Separate Sources and Targets to avoid borrow checker issues
    // Alternatively, iterate combinations. Since N is small (<100), O(N^2) is acceptable for MVP.
    // Optimally: Collect sources first.

    let mut sources = Vec::new();

    for (entity, pos, morale, mut cooldown) in pops.iter_mut() {
        if cooldown.timer > 0 {
            cooldown.timer -= 1;
            continue;
        }

        if morale.value <= LOW_MORALE_THRESHOLD {
            sources.push((entity, *pos, "Witnessed Breakdown", -5.0));
            cooldown.timer = CONTAGION_COOLDOWN;
        } else if morale.value >= HIGH_MORALE_THRESHOLD {
            sources.push((entity, *pos, "Witnessed Joy", 5.0));
            cooldown.timer = CONTAGION_COOLDOWN;
        }
    }

    // 2. Apply effects to nearby pops
    // We need a second query or just use the first one but we can't iterate mutable inside mutable.
    // Solution: We collected sources. Now iterate all pops as targets (read-only for pos, mutable for morale).
    // Bevy requires separate queries for this to be clean, or use `iter_combinations`.

    // For simplicity in Green phase:
    // We can't mutate the source's cooldown AND query all pops in the same loop easily without disjoint queries.
    // Better approach:
    // - Query 1: (Entity, &GridPosition, &Morale) -> Identify Sources
    // - Query 2: (Entity, &GridPosition, &mut Morale) -> Apply Effects
    // - Query 3: (Entity, &mut ContagionCooldown) -> Update Cooldowns

    // BUT: To keep it single-pass simple:
    // Use `iter_combinations`!

    // Actually, `iter_combinations` is perfect for pairwise interactions.
    // Only one partner needs to be the "source".

    // Let's stick to the simplest valid Rust code:
    // Collect effects to apply, then apply them.

    // (Implementation details left to Builder, but logic is defined)
}
```

## REFACTOR Phase: Quality & Design

- **Spatial Hashing**: If Pop count grows > 100, `O(N^2)` checks will be slow. Use a spatial grid or KD-tree to find neighbors efficiently.
- **Trait Integration**:
  - `Empath` trait: Higher chance/magnitude to receive contagion.
  - `Stoic` trait: Immune or reduced effect.
  - `Psychic` trait: Larger broadcast radius.
- **Event Log**: Should generate a log message if a contagion event affects > 3 people ("A panic is spreading!").
- **Visuals**: Spawn a temporary particle/icon above the source pop (e.g., a storm cloud or a sun).

## Acceptance Criteria

- [ ] All RED phase tests pass.
- [ ] Pops with Morale < 20 spread negative mood.
- [ ] Pops with Morale > 90 spread positive mood.
- [ ] Cooldown prevents infinite stacking of modifiers every tick.
- [ ] Range check works (distant pops unaffected).

## Technical Guidance

- Use `iter_combinations_mut` from Bevy if possible, but be careful with double-application (A affects B, B affects A).
- Safest pattern:
    1. Collect `(Entity, Position, MoodType)` of all active sources.
    2. Collect `(Entity, Position)` of all potential targets.
    3. Calculate distances and create a list of `Command`s or `MoodModifier` applications.
    4. Apply them.
- Ensure `ContagionCooldown` is added to the Pop Bundle in `src/layer1/pop.rs`.
