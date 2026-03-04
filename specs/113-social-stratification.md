# 113: Social Stratification

## Overview

Introduces **Social Classes** based on job prestige. Pops naturally segregate into strata (Labor, Middle, Elite). Mixing housing or recreation areas between disparate classes causes **Class Friction** (Stress/Morale penalties), encouraging players to build distinct districts or face unrest.

## Dependencies

- `009` — Job System (for `Job` component)
- `031` — Pop Morale (for `Morale` impact)
- `007` — Housing (for `AssignedTo` housing checks)

## RED Phase: Tests First

Write these tests in `src/layer1/social_stratification_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Job, JobType};
    use crate::layer1::social_stratification::{SocialClass, Prestige, calculate_social_class, class_friction_system, ClassFriction};
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;

    #[test]
    fn test_job_prestige_mapping() {
        // Verify jobs map to correct prestige
        assert_eq!(Prestige::from_job(JobType::Miner).value, 1); // Low
        assert_eq!(Prestige::from_job(JobType::Engineer).value, 5); // Medium
        assert_eq!(Prestige::from_job(JobType::Governor).value, 10); // High
    }

    #[test]
    fn test_social_class_calculation() {
        let mut world = World::new();

        // Low Prestige Pop
        let miner = world.spawn((
            Pop,
            Job { job_type: JobType::Miner },
            Prestige { value: 1 },
        )).id();

        // High Prestige Pop
        let governor = world.spawn((
            Pop,
            Job { job_type: JobType::Governor },
            Prestige { value: 10 },
        )).id();

        // Run calculation logic (or system)
        let miner_class = calculate_social_class(&world, miner);
        let gov_class = calculate_social_class(&world, governor);

        assert_eq!(miner_class, SocialClass::Labor);
        assert_eq!(gov_class, SocialClass::Elite);
    }

    #[test]
    fn test_class_friction_housing() {
        let mut world = World::new();

        // Create two pops living next to each other
        let labor_pop = world.spawn((
            Pop,
            SocialClass::Labor,
            GridPosition { x: 10, y: 10 }, // Home position (abstracted)
            Morale::default(),
        )).id();

        let elite_pop = world.spawn((
            Pop,
            SocialClass::Elite,
            GridPosition { x: 10, y: 11 }, // Adjacent home
            Morale::default(),
        )).id();

        // Run friction system
        let mut schedule = Schedule::default();
        schedule.add_systems(class_friction_system);
        schedule.run(&mut world);

        // Check for ClassFriction component or Morale penalty
        let friction = world.get::<ClassFriction>(labor_pop);
        assert!(friction.is_some());
        assert!(friction.unwrap().amount > 0.0);

        let friction_elite = world.get::<ClassFriction>(elite_pop);
        assert!(friction_elite.is_some());
    }

    #[test]
    fn test_no_friction_same_class() {
        let mut world = World::new();

        let pop1 = world.spawn((
            Pop,
            SocialClass::Labor,
            GridPosition { x: 10, y: 10 },
            Morale::default(),
        )).id();

        let pop2 = world.spawn((
            Pop,
            SocialClass::Labor,
            GridPosition { x: 10, y: 11 },
            Morale::default(),
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(class_friction_system);
        schedule.run(&mut world);

        assert!(world.get::<ClassFriction>(pop1).is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/social_stratification.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::pop::JobType;

#[derive(Component, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Prestige {
    pub value: u8,
}

impl Prestige {
    pub fn from_job(job: JobType) -> Self {
        let value = match job {
            JobType::Miner | JobType::Farmer | JobType::Hauler => 1,
            JobType::Builder | JobType::Crafter | JobType::Guard => 3,
            JobType::Engineer | JobType::Doctor | JobType::Merchant => 5,
            JobType::Scientist | JobType::Artist => 7,
            JobType::Governor | JobType::Administrator => 10,
            _ => 1,
        };
        Self { value }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Copy)]
pub enum SocialClass {
    Labor,   // Prestige 0-3
    Middle,  // Prestige 4-7
    Elite,   // Prestige 8+
}

#[derive(Component, Debug, Clone)]
pub struct ClassFriction {
    pub amount: f32, // Accumulated stress from friction
}

pub fn calculate_social_class(world: &World, entity: Entity) -> SocialClass {
    if let Some(prestige) = world.get::<Prestige>(entity) {
        if prestige.value >= 8 {
            return SocialClass::Elite;
        } else if prestige.value >= 4 {
            return SocialClass::Middle;
        }
    }
    SocialClass::Labor
}

pub fn update_social_class_system(
    mut commands: Commands,
    query: Query<(Entity, &Prestige), Changed<Prestige>>,
) {
    for (entity, prestige) in query.iter() {
        let class = if prestige.value >= 8 {
            SocialClass::Elite
        } else if prestige.value >= 4 {
            SocialClass::Middle
        } else {
            SocialClass::Labor
        };
        commands.entity(entity).insert(class);
    }
}

pub fn class_friction_system(
    mut commands: Commands,
    pops: Query<(Entity, &crate::layer1::map::GridPosition, &SocialClass)>,
    other_pops: Query<(Entity, &crate::layer1::map::GridPosition, &SocialClass)>,
) {
    // Naive O(N^2) for Green phase - optimize later
    for (entity, pos, class) in pops.iter() {
        let mut friction_score = 0.0;

        for (other_entity, other_pos, other_class) in other_pops.iter() {
            if entity == other_entity { continue; }

            // Check if neighbors (radius 1 for housing friction)
            // Ideally, check if their *assigned beds* are near, but using position for now (assuming they are home/sleeping)
            let dist = (pos.x - other_pos.x).abs() + (pos.y - other_pos.y).abs();

            if dist <= 2 { // Close proximity
                if class != other_class {
                    // Friction!
                    // Elite <-> Labor = High friction
                    // Elite <-> Middle = Medium
                    // Middle <-> Labor = Low
                    friction_score += 0.05;
                }
            }
        }

        if friction_score > 0.0 {
            commands.entity(entity).insert(ClassFriction { amount: friction_score });
            // Also apply negative Morale modifier here directly or via another system
        } else {
            commands.entity(entity).remove::<ClassFriction>();
        }
    }
}
```

### 2. Integrate with Job System

When a Pop is assigned a job, update their `Prestige` component.

```rust
// src/layer1/pop.rs or similar assignment logic
// commands.entity(pop).insert(Prestige::from_job(new_job));
```

## REFACTOR Phase: Quality & Design

- **Optimization**: The `class_friction_system` is O(N^2). Use a spatial hash or only check neighbors in the `Housing` system (checking assigned beds rather than current position).
- **Visualization**: Show Social Class in the Pop Inspector (Labor: Brown, Middle: Silver, Elite: Gold).
- **Nuance**: Add "Class Traitors" or "Humility" traits that reduce friction.
- **Zoning**: Allow designating "Elite Housing Zones" that strictly forbid Labor classes, enforcing segregation mechanically.

## Acceptance Criteria

- [ ] `Prestige` component exists and calculates correctly from `JobType`.
- [ ] `SocialClass` component is derived from `Prestige`.
- [ ] Pops of different classes living/existing near each other generate `ClassFriction`.
- [ ] `ClassFriction` negatively impacts Morale.
- [ ] Tests pass with >85% coverage.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
