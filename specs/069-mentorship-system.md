# 069: Mentorship System

## Overview

Implement a mentorship mechanic where skilled Pops ("Masters") passively boost the experience gain of nearby less-skilled Pops ("Apprentices") when working on the same task type. This incentivizes grouping workers of different skill levels and adds depth to job assignment.

For the MVP, this system focuses on **ActionType::Work** (Mining, Forestry, Construction). Support for Farming and Refining will be added when those systems are fully integrated into the Utility AI (`066`).

## Dependencies

- `051` — Pop Skills (XP, Skill Levels)
- `021` — Utility AI Work (ActionType::Work)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/mentorship_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::mentorship::{Mentorship, check_mentorship_system, apply_mentorship_xp_system};
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{Skills, SkillType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::execution::{MovementTarget, AtTarget};
    use crate::layer1::utility_ai::ActionType;
    use crate::layer1::designation::{Designation, DesignationType};

    #[test]
    fn test_mentorship_detection_valid_pair() {
        let mut world = World::new();

        // Spawn Master (Mining Lvl 5) working on a Mine designation
        let mut master_skills = Skills::default();
        master_skills.add_xp(SkillType::Mining, 2500.0); // Level 5
        let master_designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 5 }
        )).id();
        let master = world.spawn((
            Pop,
            GridPosition { x: 4, y: 5 }, // Adjacent to designation
            master_skills,
            MovementTarget {
                target_entity: master_designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work
            },
            AtTarget
        )).id();

        // Spawn Apprentice (Mining Lvl 0) working on a Mine designation nearby
        let apprentice_designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 6, y: 5 }
        )).id();
        let apprentice = world.spawn((
            Pop,
            GridPosition { x: 7, y: 5 }, // Within 5 tiles of Master
            Skills::default(), // Level 0
            MovementTarget {
                target_entity: apprentice_designation,
                target_position: GridPosition { x: 6, y: 5 },
                for_action: ActionType::Work
            },
            AtTarget
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(check_mentorship_system);
        schedule.run(&mut world);

        // Assert Apprentice has Mentorship component pointing to Master
        let mentorship = world.get::<Mentorship>(apprentice);
        assert!(mentorship.is_some(), "Apprentice should have Mentorship component");
        let mentorship = mentorship.unwrap();
        assert_eq!(mentorship.master_entity, master);
        assert_eq!(mentorship.skill, SkillType::Mining);
    }

    #[test]
    fn test_mentorship_detection_ignores_too_far() {
        let mut world = World::new();

        // Master at (0,0)
        let mut master_skills = Skills::default();
        master_skills.add_xp(SkillType::Mining, 2500.0);
        let master_des = world.spawn((Designation { designation_type: DesignationType::Mine }, GridPosition { x: 0, y: 0 })).id();
        world.spawn((
            Pop, GridPosition { x: 1, y: 0 }, master_skills,
            MovementTarget { target_entity: master_des, target_position: GridPosition { x: 0, y: 0 }, for_action: ActionType::Work }, AtTarget
        ));

        // Apprentice at (10,0) - Too far
        let app_des = world.spawn((Designation { designation_type: DesignationType::Mine }, GridPosition { x: 10, y: 0 })).id();
        let apprentice = world.spawn((
            Pop, GridPosition { x: 9, y: 0 }, Skills::default(),
            MovementTarget { target_entity: app_des, target_position: GridPosition { x: 10, y: 0 }, for_action: ActionType::Work }, AtTarget
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(check_mentorship_system);
        schedule.run(&mut world);

        assert!(world.get::<Mentorship>(apprentice).is_none(), "Too far for mentorship");
    }

    #[test]
    fn test_mentorship_detection_ignores_small_level_gap() {
        let mut world = World::new();

        // Master Level 1 (100 XP)
        let mut master_skills = Skills::default();
        master_skills.add_xp(SkillType::Mining, 150.0);
        let master_des = world.spawn((Designation { designation_type: DesignationType::Mine }, GridPosition { x: 5, y: 5 })).id();
        world.spawn((
            Pop, GridPosition { x: 4, y: 5 }, master_skills,
            MovementTarget { target_entity: master_des, target_position: GridPosition { x: 5, y: 5 }, for_action: ActionType::Work }, AtTarget
        ));

        // Apprentice Level 0
        let app_des = world.spawn((Designation { designation_type: DesignationType::Mine }, GridPosition { x: 6, y: 5 })).id();
        let apprentice = world.spawn((
            Pop, GridPosition { x: 7, y: 5 }, Skills::default(),
            MovementTarget { target_entity: app_des, target_position: GridPosition { x: 6, y: 5 }, for_action: ActionType::Work }, AtTarget
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(check_mentorship_system);
        schedule.run(&mut world);

        // Gap is only 1 level (needs >= 2)
        assert!(world.get::<Mentorship>(apprentice).is_none(), "Level gap too small");
    }

    #[test]
    fn test_mentorship_apply_xp_bonus() {
        let mut world = World::new();

        // Apprentice with Mentorship active
        let apprentice = world.spawn((
            Pop,
            Skills::default(),
            Mentorship {
                master_entity: Entity::from_raw(999),
                skill: SkillType::Mining,
                multiplier: 1.5,
                expiration: 10,
            },
            // Must be working to gain XP? The spec says "passively boost XP gain".
            // Since work_execution_system adds base XP, this system adds BONUS XP.
            // So we just need to check if XP increases.
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(apply_mentorship_xp_system);
        schedule.run(&mut world);

        let skills = world.get::<Skills>(apprentice).unwrap();
        assert!(skills.get_xp(SkillType::Mining) > 0.0, "Should gain bonus XP from mentorship");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `Mentorship` Component

Create `src/layer1/mentorship.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::skills::SkillType;

#[derive(Component, Debug, Clone)]
pub struct Mentorship {
    pub master_entity: Entity,
    pub skill: SkillType,
    pub multiplier: f32,
    pub expiration: u32,
}
```

### 2. Implement `check_mentorship_system`

This system runs occasionally (e.g., every 50 ticks) to establish mentorship links.

```rust
use crate::layer1::execution::{MovementTarget, AtTarget};
use crate::layer1::skills::Skills;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_ai::ActionType;
use crate::layer1::designation::{Designation, DesignationType};

pub fn check_mentorship_system(
    mut commands: Commands,
    // Query potential apprentices AND masters
    pops: Query<(Entity, &GridPosition, &Skills, &MovementTarget), With<AtTarget>>,
    designations: Query<&Designation>,
) {
    // 1. Collect working pops and their SkillType
    let mut workers = Vec::new();
    for (entity, pos, skills, mt) in &pops {
        if mt.for_action != ActionType::Work { continue; }

        if let Ok(designation) = designations.get(mt.target_entity) {
             let skill_type = match designation.designation_type {
                 DesignationType::Mine => Some(SkillType::Mining),
                 DesignationType::Chop => Some(SkillType::Forestry),
                 DesignationType::Repair | DesignationType::Demolish => Some(SkillType::Construction),
                 _ => None,
             };
             if let Some(st) = skill_type {
                 workers.push((entity, *pos, skills, st));
             }
        }
    }

    // 2. Find pairs (O(N^2) for MVP is fine, N is small)
    for i in 0..workers.len() {
        let (app_entity, app_pos, app_skills, app_skill_type) = workers[i];
        let app_level = app_skills.get_level(app_skill_type);

        // Find best master in range
        let mut best_master = None;

        for j in 0..workers.len() {
            if i == j { continue; }
            let (master_entity, master_pos, master_skills, master_skill_type) = workers[j];

            if app_skill_type != master_skill_type { continue; }

            let dist = (app_pos.x - master_pos.x).abs() + (app_pos.y - master_pos.y).abs();
            if dist > 5 { continue; }

            let master_level = master_skills.get_level(master_skill_type);
            if master_level >= app_level + 2 {
                // Found a valid master
                best_master = Some(master_entity);
                break; // Take first found for MVP
            }
        }

        if let Some(master) = best_master {
            commands.entity(app_entity).insert(Mentorship {
                master_entity: master,
                skill: app_skill_type,
                multiplier: 1.5, // 50% bonus
                expiration: 50, // Re-check every 50 ticks
            });
        }
    }
}
```

### 3. Implement `apply_mentorship_xp_system`

This system runs every tick to apply the XP bonus.

```rust
pub fn apply_mentorship_xp_system(
    mut query: Query<(Entity, &mut Skills, &Mentorship)>,
    mut commands: Commands,
) {
    for (entity, mut skills, mentorship) in &mut query {
        // Add small trickle XP per tick representing "learning by watching"
        // Base work gives 1.0 XP per tick.
        // Multiplier 1.5x means we should add 0.5 extra XP per tick.
        // Or if Mentorship acts as a multiplier on base work, we should handle it there.
        // But to avoid modifying `work_execution_system`, we add the bonus separately here.

        let bonus = 0.5; // Fixed bonus for now
        skills.add_xp(mentorship.skill, bonus);

        // Handle expiration (optional, if using component lifecycle)
        // For MVP, check_mentorship_system overwrites the component, so we rely on that.
        // But we should probably decrement expiration or remove if expired?
        // Let's keep it simple: check_mentorship_system adds/refreshes it.
        // We can check expiration here if we want stricter timing.
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Use `Grid` or spatial hash for finding neighbors instead of O(N^2) loop.
- **Visuals**: Add a "Learning" icon above the apprentice's head.
- **Social**: Add relationship gain between Apprentice and Master (`047 Pop Relationships`).
- **Integration**: Apply bonus to `produce_food_system` (Farming) and `process_refining_system` (Crafting) once they use `ActionType::Work` or similar standardized targeting.

## Acceptance Criteria

- [ ] All tests pass.
- [ ] Apprentices gain significantly more XP when working near a Master.
- [ ] Mentorship only applies to same Skill Type (Mining vs Mining).
- [ ] Mentorship requires Level Gap >= 2.
- [ ] Mentorship requires Range <= 5.

## Technical Guidance

- Ensure `check_mentorship_system` runs less frequently (e.g., in `Observation` schedule or with a timer) to save performance.
- Use `commands.entity(e).insert(...)` to add/update the component. Be careful not to flicker the component every tick if checking frequently.
