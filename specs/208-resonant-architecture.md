# 208: Resonant Architecture

## Overview

Currently, a room's quality is a generic score (Space + Beauty). **Resonant Architecture** adds a layer of "Vibe" or "Resonance" to rooms based on their materials and contents. A room built of Red Granite with a War Table has a **Martial Resonance**. A room filled with books and blue light has an **Intellectual Resonance**.

Pops with matching **Traits** (e.g., Aggressive, Intellectual) gain mood buffs when using Resonant rooms. Pops with opposing traits gain debuffs. This encourages specialized room design rather than generic "high quality" boxes.

## Dependencies

- `064` — Room Quality (Core logic)
- `084` — Pop Traits (Traits to resonate with)
- `089` — Material Provenance (Optional, but enhances the feature via Material Types)

## RED Phase: Tests First

Write these tests in `src/layer1/resonant_architecture_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::room_quality::calculate_room_resonance;
    use crate::layer1::resonant_architecture::{ResonanceType, ResonanceSource, ResonanceAffinity};
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::pop::Pop;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::morale::{Thought, ThoughtType};
    use std::collections::HashSet;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup necessary resources (ZoneGrid, etc.) similar to Room Quality tests
        world.insert_resource(crate::layer1::zone::ZoneGrid::new(10, 10));
        world
    }

    #[test]
    fn test_calculate_room_resonance_martial() {
        let mut world = setup_world();

        // Spawn a "War Room" (Walls + Weapon Rack)
        // Assume WeaponRack has ResonanceSource(Martial, 10.0)
        world.spawn((
            Building { building_type: BuildingType::Statue }, // Placeholder for Weapon Rack
            GridPosition { x: 1, y: 1 },
            ResonanceSource {
                resonance_type: ResonanceType::Martial,
                strength: 10.0,
            },
        ));

        let (resonance_type, strength) = calculate_room_resonance(&world, GridPosition { x: 1, y: 1 });

        assert_eq!(resonance_type, Some(ResonanceType::Martial));
        assert!(strength >= 10.0);
    }

    #[test]
    fn test_mixed_resonance_dominance() {
        let mut world = setup_world();

        // 10 Martial vs 5 Nature
        world.spawn((
            Building { building_type: BuildingType::Statue },
            GridPosition { x: 1, y: 1 },
            ResonanceSource { resonance_type: ResonanceType::Martial, strength: 10.0 },
        ));
        world.spawn((
            Building { building_type: BuildingType::FlowerBed },
            GridPosition { x: 1, y: 2 },
            ResonanceSource { resonance_type: ResonanceType::Nature, strength: 5.0 },
        ));

        let (resonance_type, strength) = calculate_room_resonance(&world, GridPosition { x: 1, y: 1 });

        // Martial should dominate
        assert_eq!(resonance_type, Some(ResonanceType::Martial));
        assert_eq!(strength, 10.0); // Or 10 - 5? Usually dominant wins.
    }

    #[test]
    fn test_trait_affinity_mood_impact() {
        // Trait: Aggressive -> Loves Martial
        let affinity = ResonanceAffinity::get_affinity(Trait::Aggressive, ResonanceType::Martial);
        assert!(affinity > 0.0);

        // Trait: Pacifist -> Hates Martial
        let affinity = ResonanceAffinity::get_affinity(Trait::Pacifist, ResonanceType::Martial);
        assert!(affinity < 0.0);
    }

    #[test]
    fn test_apply_resonance_thought() {
        let mut world = setup_world();
        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Traits(HashSet::from([Trait::Aggressive])),
            crate::layer1::morale::Morale::default(),
        )).id();

        // Mock a Martial room
        // We need to inject the resonance calculation or mock the room
        // For unit test, we can call the application logic directly with a known resonance.

        crate::layer1::resonant_architecture::apply_resonance_thought(
            &mut world,
            pop,
            Some(ResonanceType::Martial),
            10.0
        );

        let morale = world.get::<crate::layer1::morale::Morale>(pop).unwrap();
        // Should have positive thought
        assert!(morale.thoughts.iter().any(|t| t.mood_modifier > 0.0));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Resonance Types (`src/layer1/resonant_architecture.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::traits::Trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResonanceType {
    Martial,      // Weapons, Red, Metal
    Nature,       // Plants, Green, Wood
    Intellectual, // Books, Blue, Glass
    Industrial,   // Machines, Grey, Steel
    Sacred,       // Altars, Gold, Relics
    Void,         // Dark, Purple, Vacuum
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ResonanceSource {
    pub resonance_type: ResonanceType,
    pub strength: f32,
}
```

### 2. Define Affinity Logic

```rust
pub struct ResonanceAffinity;

impl ResonanceAffinity {
    pub fn get_affinity(trait_type: Trait, resonance: ResonanceType) -> f32 {
        match (trait_type, resonance) {
            (Trait::Aggressive, ResonanceType::Martial) => 1.0,
            (Trait::Pacifist, ResonanceType::Martial) => -1.0,

            (Trait::NatureLover, ResonanceType::Nature) => 1.0,
            (Trait::Industrialist, ResonanceType::Nature) => -0.5,

            (Trait::Intellectual, ResonanceType::Intellectual) => 1.0,

            (Trait::HardWorker, ResonanceType::Industrial) => 0.5,
            (Trait::Lazy, ResonanceType::Industrial) => -0.5,

            (Trait::Spiritual, ResonanceType::Sacred) => 1.0,

            (Trait::VoidTouched, ResonanceType::Void) => 1.0,
            (Trait::Traditionalist, ResonanceType::Void) => -1.0,

            _ => 0.0,
        }
    }
}
```

### 3. Implement `calculate_room_resonance`

Extend `calculate_room_quality` logic to scan for `ResonanceSource` components in the room tiles.

```rust
use crate::layer1::map::GridPosition;
use crate::layer1::room_quality::calculate_room_tiles; // Refactor room_quality to expose tile finding

pub fn calculate_room_resonance(world: &World, pos: GridPosition) -> (Option<ResonanceType>, f32) {
    let tiles = calculate_room_tiles(world, pos); // Needs refactor of 064

    let mut scores = std::collections::HashMap::new();

    // Query all ResonanceSources
    // For MVP Green Phase, we can iterate all ResonanceSources and check if they are in the room.
    // This avoids dependency on BuildingMap being up-to-date in tests.
    // Optimization (Refactor Phase): Use BuildingMap or spatial query.
    let mut query = world.query::<(&GridPosition, &ResonanceSource)>();

    // Convert tiles to HashSet for fast lookup
    let room_tile_set: std::collections::HashSet<GridPosition> = tiles.into_iter().collect();

    for (pos, source) in query.iter(world) {
        if room_tile_set.contains(pos) {
            *scores.entry(source.resonance_type).or_insert(0.0) += source.strength;
        }
    }

    // Find dominant
    let mut best_type = None;
    let mut best_score = 0.0;

    for (res_type, score) in scores {
        if score > best_score {
            best_score = score;
            best_type = Some(res_type);
        }
    }

    // Threshold
    if best_score < 5.0 {
        return (None, 0.0);
    }

    (best_type, best_score)
}
```

### 4. Apply Thoughts

```rust
pub fn apply_resonance_thought(world: &mut World, pop: Entity, resonance: Option<ResonanceType>, strength: f32) {
    let Some(res_type) = resonance else { return; };

    let traits = if let Ok(t) = world.query::<&crate::layer1::traits::Traits>().get(world, pop) {
        t
    } else {
        return; // No traits
    };

    let mut total_affinity = 0.0;
    for t in &traits.0 {
        total_affinity += ResonanceAffinity::get_affinity(*t, res_type);
    }

    if total_affinity.abs() < 0.1 { return; }

    let mood_impact = total_affinity * (strength / 10.0).min(5.0); // Cap multiplier

    // Apply thought
    // We need new ThoughtTypes in Morale system (to be added in Green Phase 2)
    // For MVP, we can reuse a generic thought or add specific ones.
    // Let's assume we add `ResonantAtmosphere` and `DissonantAtmosphere`.

    let thought_type = if mood_impact > 0.0 {
        crate::layer1::morale::ThoughtType::ResonantAtmosphere
    } else {
        crate::layer1::morale::ThoughtType::DissonantAtmosphere
    };

    if let Some(mut morale) = world.get_mut::<crate::layer1::morale::Morale>(pop) {
        // Impact scaled by affinity
        // Note: Thought constructor usually takes an integer duration
        let mut thought = crate::layer1::morale::Thought::new(thought_type, 1);
        thought.mood_modifier = mood_impact; // Override mood modifier if supported, or use different types
        morale.add_thought(thought);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Refactor Room Logic**: `calculate_room_quality` and `calculate_room_resonance` share the "flood fill" logic. Extract `get_room_tiles(pos)` into a shared helper in `room_quality.rs`.
- **Material Integration**: If `MaterialType` exists, map materials to default resonance (Wood -> Nature, Metal -> Industrial, Stone -> Martial/Sacred?).
- **UI**: Display "Room Resonance: Martial (Strong)" in the Room Inspector.
- **Caching**: Resonance shouldn't change unless furniture/walls change. Cache it on the Room/Zone.

## Acceptance Criteria

- [ ] `ResonanceSource` component exists.
- [ ] Buildings can be tagged with Resonance.
- [ ] `calculate_room_resonance` correctly identifies the dominant vibe.
- [ ] Pops with matching traits get mood buffs.
- [ ] Pops with opposing traits get mood debuffs.
- [ ] Tests pass.

## Technical Guidance

- Modify `src/layer1/room_quality.rs` to expose `flood_fill_room` or similar public helper.
- Ensure `ResonanceSource` is added to relevant `BuildingType`s in `spawn_building`.
