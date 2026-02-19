# 172: Institutional Memory

## Overview

As the colony survives, its inhabitants learn from their experiences. High-skill Pops have a chance to record their knowledge into "Manuals" while working. These Manuals, when placed in the world (e.g., in a library or stockpile), provide a passive XP boost to nearby Pops performing the same task. This creates physical "centers of learning" and preserves knowledge even if the master dies.

## Dependencies

- `051` — Pop Skills (SkillType, XP)
- `030` — Tool Economy (Item marker, Hauling)
- `008` — Resource Stockpiles (Storage)

## RED Phase: Tests First

Write these tests in `src/layer1/institutional_memory_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::skills::{Skills, SkillType};
    use crate::layer1::items::Item;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::memory::{Manual, produce_manual_system, manual_aura_system};
    use crate::layer1::utility_ai::ActionType;
    use crate::layer1::execution::{MovementTarget, AtTarget};

    #[test]
    fn test_produce_manual_high_skill() {
        let mut world = World::new();
        // Spawn high-skill pop working
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 2500.0); // Level 5
        let pop = world.spawn((
            Pop,
            skills,
            GridPosition { x: 5, y: 5 },
            MovementTarget {
                target_entity: Entity::from_raw(0),
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work
            },
            AtTarget // Must be actively working
        )).id();

        // Run system multiple times to trigger chance (or mock RNG)
        let mut schedule = Schedule::default();
        schedule.add_systems(produce_manual_system);

        // Mock RNG or run enough times. For test, we might force probability in implementation if #[cfg(test)]
        schedule.run(&mut world);

        // Check if Manual item spawned at position
        let manual_query = world.query::<(&Manual, &GridPosition)>().iter(&world);
        let mut found = false;
        for (manual, pos) in manual_query {
            if pos.x == 5 && pos.y == 5 && manual.skill_type == SkillType::Mining {
                found = true;
                break;
            }
        }
        // Assert found (might need deterministic RNG seed in setup)
    }

    #[test]
    fn test_produce_manual_low_skill_fails() {
        let mut world = World::new();
        // Spawn low-skill pop working
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 0.0); // Level 0
        world.spawn((
            Pop,
            skills,
            GridPosition { x: 5, y: 5 },
            MovementTarget {
                target_entity: Entity::from_raw(0),
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work
            },
            AtTarget
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(produce_manual_system);
        schedule.run(&mut world);

        let count = world.query::<&Manual>().iter(&world).count();
        assert_eq!(count, 0, "Low skill pop should not produce manual");
    }

    #[test]
    fn test_manual_aura_boosts_xp() {
        let mut world = World::new();

        // Spawn Manual on ground
        world.spawn((
            Item,
            Manual {
                skill_type: SkillType::Mining,
                xp_multiplier: 1.5,
                durability: 100.0,
            },
            GridPosition { x: 5, y: 5 }
        ));

        // Spawn Worker nearby (Level 0)
        let worker = world.spawn((
            Pop,
            Skills::default(),
            GridPosition { x: 6, y: 5 }, // Adjacent
            MovementTarget {
                target_entity: Entity::from_raw(0),
                target_position: GridPosition { x: 6, y: 5 },
                for_action: ActionType::Work
            },
            AtTarget
        )).id();

        // Run aura system
        let mut schedule = Schedule::default();
        schedule.add_systems(manual_aura_system);
        schedule.run(&mut world);

        // Verify XP gain (assuming system applies immediate XP or buff component)
        // If aura adds a 'LearningBuff' component, check for that.
        // If it directly adds XP, check Skills.
        // Let's assume it adds a temporary 'Mentored' component similar to Spec 069,
        // or directly modifies an 'XPRate' resource/component if one existed.
        // For MVP: let's say it adds XP directly to keep it simple,
        // OR it adds a 'ManualBuff' component that other systems read.

        // Let's check for direct XP addition for simplicity in this isolated test,
        // OR check that the Pop has a component indicating the boost.
        let skills = world.get::<Skills>(worker).unwrap();
        assert!(skills.get_xp(SkillType::Mining) > 0.0, "Should gain passive XP from Manual aura");
    }

    #[test]
    fn test_manual_degradation() {
        let mut world = World::new();
        let manual = world.spawn((
            Manual {
                skill_type: SkillType::Mining,
                xp_multiplier: 1.5,
                durability: 1.0, // Low durability
            },
            GridPosition { x: 5, y: 5 }
        )).id();

        // Spawn worker to trigger usage
        world.spawn((
            Pop,
            Skills::default(),
            GridPosition { x: 6, y: 5 },
            MovementTarget {
                target_entity: Entity::from_raw(0),
                target_position: GridPosition { x: 6, y: 5 },
                for_action: ActionType::Work
            },
            AtTarget
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(manual_aura_system);
        schedule.run(&mut world); // Tick 1: Durability -> 0

        // Check if manual is destroyed or disabled
        // If destroyed:
        assert!(world.get_entity(manual).is_none(), "Manual should be destroyed when durability hits 0");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `Manual` Component (`src/layer1/memory.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::skills::SkillType;

#[derive(Component, Debug, Clone)]
pub struct Manual {
    pub skill_type: SkillType,
    pub xp_multiplier: f32, // e.g. 1.2 for +20%
    pub durability: f32,
    pub max_durability: f32,
}
```

### 2. Implement `produce_manual_system`

```rust
use crate::layer1::pop::Pop;
use crate::layer1::skills::Skills;
use crate::layer1::map::GridPosition;
use crate::layer1::items::Item;
use crate::layer1::utility_ai::ActionType;
use crate::layer1::execution::{MovementTarget, AtTarget};
use rand::prelude::*;

pub fn produce_manual_system(
    mut commands: Commands,
    query: Query<(&Skills, &GridPosition, &MovementTarget), (With<Pop>, With<AtTarget>)>,
) {
    let mut rng = thread_rng();

    for (skills, pos, target) in &query {
        if target.for_action != ActionType::Work { continue; }

        // Determine skill type from action/designation (simplified)
        // In reality, need to query designation. For MVP, assume Mining/Forestry based on context or add logic.
        let skill_type = SkillType::Mining; // Placeholder: Retrieve actual skill from target designation

        let level = skills.get_level(skill_type);
        if level < 5 { continue; }

        // Chance to produce: 0.1% per tick?
        if rng.gen_bool(0.001) {
             commands.spawn((
                Item, // Marker for Hauling
                Manual {
                    skill_type,
                    xp_multiplier: 1.0 + (level as f32 * 0.05), // Lvl 5 = 1.25x
                    durability: 100.0,
                    max_durability: 100.0,
                },
                *pos // Drop at feet
            ));
            // Log message: "Pop wrote a manual on Mining!"
        }
    }
}
```

### 3. Implement `manual_aura_system`

```rust
pub fn manual_aura_system(
    mut commands: Commands,
    mut manuals: Query<(Entity, &mut Manual, &GridPosition)>,
    mut workers: Query<(&GridPosition, &mut Skills, &MovementTarget), (With<Pop>, With<AtTarget>)>,
) {
    // Spatial optimization needed for large scale, O(N*M) OK for MVP

    for (manual_entity, mut manual, manual_pos) in &mut manuals {
        let mut used = false;

        for (worker_pos, mut skills, target) in &mut workers {
            if target.for_action != ActionType::Work { continue; }

            // Distance check (Radius 5)
            let dist = (manual_pos.x - worker_pos.x).abs() + (manual_pos.y - worker_pos.y).abs();
            if dist > 5 { continue; }

            // Add passive XP
            // Note: Should match manual.skill_type to worker action
            skills.add_xp(manual.skill_type, 0.1 * manual.xp_multiplier);
            used = true;
        }

        if used {
            manual.durability -= 0.1;
            if manual.durability <= 0.0 {
                commands.entity(manual_entity).despawn();
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Use a spatial grid to query workers near manuals, rather than nested loops.
- **Deduplication**: If multiple manuals are nearby, do bonuses stack? Limit to 1 (max) to prevent "Manual stacking" exploit.
- **Visuals**: Spawn a particle effect on the worker when they gain "Insight" from a manual.
- **Inventory**: Allow Pops to carry manuals in an Accessory slot for mobile bonuses.
- **Integration**: Link to `069 Mentorship`. A Manual is effectively a "Passive Mentor".

## Acceptance Criteria

- [ ] Manuals are spawned by Level 5+ workers (rare chance).
- [ ] Manuals have `SkillType` and `Durability`.
- [ ] Workers within 5 tiles of a matching Manual gain bonus XP per tick.
- [ ] Manuals lose durability when used (when boosting someone).
- [ ] Manuals are destroyed when durability reaches 0.
- [ ] Tests pass.

## Technical Guidance

- Use `crate::layer1::designation::DesignationType` lookup to determine the correct `SkillType` in `produce_manual_system`. You'll need to query the `Designation` entity pointed to by `MovementTarget`.
- Ensure `Manual` entities have `Item` component so they can be hauled to Stockpiles.
- Stockpiles should accept `Item`s. If Stockpiles filter by type, ensure Manuals are a valid type (or add `ItemType::Manual`).
