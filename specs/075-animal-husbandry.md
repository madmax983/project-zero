# 075: Animal Husbandry

## Overview

Extends the **Fauna** system (`048`) and **Designated Zones** (`056`) to allow colonists to tame wild animals, keep them in pens (Pastures), and harvest renewable resources (Milk, Wool) or slaughter them for meat.

This introduces a new economic loop: `Tame -> Confine -> Feed -> Harvest`.

## Dependencies

- `048` — Hostile Fauna (Fauna entity, behavior)
- `056` — Designated Zones (Pasture zone)
- `051` — Pop Skills (Husbandry skill)
- `016` — Utility AI (Taming action)

## RED Phase: Tests First

Write these tests in `src/layer1/husbandry_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::fauna::{Fauna, FaunaType, FaunaState};
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{Skills, SkillType};
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::husbandry::{Tame, HusbandryConfig, attempt_tame, pasture_confinement_system};
    use crate::layer1::utility_ai::ActionType;

    // Helper to setup world
    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(ZoneGrid::new(10, 10));
        world
    }

    #[test]
    fn test_tame_success() {
        let mut world = setup_world();

        // High skill pop
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Husbandry, 1000.0); // Level 3+
        let tamer = world.spawn((Pop, skills, GridPosition { x: 0, y: 0 })).id();

        // Wild animal
        let animal = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, state: FaunaState::Wander, ..Default::default() },
            GridPosition { x: 0, y: 1 },
        )).id();

        // Attempt tame
        let success = attempt_tame(&mut world, tamer, animal);

        assert!(success, "High skill should tame successfully");
        assert!(world.get::<Tame>(animal).is_some(), "Animal should have Tame component");

        let fauna = world.get::<Fauna>(animal).unwrap();
        assert_ne!(fauna.state, FaunaState::Attack, "Tamed animal should not attack");
    }

    #[test]
    fn test_tame_failure_aggro() {
        let mut world = setup_world();

        // No skill pop
        let tamer = world.spawn((Pop, Skills::default(), GridPosition { x: 0, y: 0 })).id();

        // Wild animal
        let animal = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, state: FaunaState::Wander, ..Default::default() },
            GridPosition { x: 0, y: 1 },
        )).id();

        // Force failure logic in test or rely on probability (mock RNG if possible)
        // For this test, assume 0 skill = fail
        let success = attempt_tame(&mut world, tamer, animal);

        assert!(!success, "Zero skill should likely fail");
        assert!(world.get::<Tame>(animal).is_none(), "Failed tame should not add component");

        let fauna = world.get::<Fauna>(animal).unwrap();
        assert_eq!(fauna.state, FaunaState::Attack, "Failed tame should trigger Attack");
    }

    #[test]
    fn test_pasture_confinement() {
        let mut world = setup_world();

        // Define Pasture at (0,0) to (2,2)
        let mut zones = world.resource_mut::<ZoneGrid>();
        for y in 0..3 {
            for x in 0..3 {
                zones.set(x, y, ZoneType::Pasture);
            }
        }

        // Tamed animal inside pasture
        let animal = world.spawn((
            Fauna::default(),
            Tame,
            GridPosition { x: 1, y: 1 },
            crate::layer1::execution::MovementTarget {
                target_position: GridPosition { x: 5, y: 5 }, // Try to leave
                ..Default::default()
            },
        )).id();

        // Run confinement system
        pasture_confinement_system(&mut world);

        // Movement target should be clamped to Pasture
        let target = world.get::<crate::layer1::execution::MovementTarget>(animal).unwrap();
        let dest_zone = world.resource::<ZoneGrid>().get(target.target_position.x, target.target_position.y);

        assert_eq!(dest_zone, ZoneType::Pasture, "Tamed animal should be confined to Pasture");
    }

    #[test]
    fn test_resource_production() {
        let mut world = setup_world();

        // Tamed animal (Cow/SpaceRat)
        let animal = world.spawn((
            Fauna { fauna_type: FaunaType::SpaceRat, ..Default::default() },
            Tame { produce_timer: 0, ..Default::default() }, // Ready to produce
            GridPosition { x: 0, y: 0 },
        )).id();

        // Run production system
        crate::layer1::husbandry::husbandry_production_system(&mut world);

        // Should spawn item? Or add to inventory?
        // For MVP, spawn an Item entity at location.
        let items = world.query::<&crate::layer1::item::Item>().iter(&world).count();
        assert!(items > 0, "Should produce resource");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Enums

- **Skills**: Add `SkillType::Husbandry` to `src/layer1/skills.rs`.
- **Zones**: Add `ZoneType::Pasture` to `src/layer1/zone.rs`.
- **ActionType**: Add `ActionType::Tame` and `ActionType::Slaughter` to `src/layer1/utility_ai/types.rs`.
- **DesignationType**: Add `DesignationType::Tame` to `src/layer1/designation.rs`.

### 2. Create `src/layer1/husbandry.rs`

```rust
use bevy_ecs::prelude::*;
use crate::layer1::skills::{Skills, SkillType, get_skill_efficiency};
use crate::layer1::fauna::{Fauna, FaunaState, FaunaType};
use crate::layer1::pop::Pop;
use crate::layer1::map::GridPosition;
use crate::layer1::zone::{ZoneGrid, ZoneType};
use crate::layer1::execution::MovementTarget;

#[derive(Component, Default)]
pub struct Tame {
    pub produce_timer: u32, // Ticks until next resource
    pub hunger: f32,
}

#[derive(Resource)]
pub struct HusbandryConfig {
    pub tame_difficulty: f32,
}

/// Attempts to tame an animal. Returns true on success.
pub fn attempt_tame(world: &mut World, tamer: Entity, animal: Entity) -> bool {
    // 1. Get Skill
    let skill_level = if let Ok(skills) = world.get::<Skills>(tamer) {
        skills.get_efficiency(SkillType::Husbandry)
    } else {
        1.0
    };

    // 2. Roll vs Difficulty (Simple check for MVP)
    // Base chance 30% * skill
    let chance = 0.3 * skill_level;
    let roll = rand::random::<f32>(); // Or deterministic hash

    if roll < chance {
        // Success
        world.entity_mut(animal).insert(Tame::default());
        if let Some(mut fauna) = world.get_mut::<Fauna>(animal) {
            fauna.state = FaunaState::Wander; // Reset aggro
            // Optional: Set specific "Tame" state if needed, but Tame component handles logic
        }
        true
    } else {
        // Failure: Aggro
        if let Some(mut fauna) = world.get_mut::<Fauna>(animal) {
            fauna.state = FaunaState::Attack;
            fauna.target = Some(tamer);
        }
        false
    }
}

/// System to confine tamed animals to Pastures.
pub fn pasture_confinement_system(world: &mut World) {
    let zone_grid = world.resource::<ZoneGrid>();
    let mut adjustments = Vec::new();

    let query = world.query::<(Entity, &GridPosition, &MovementTarget, With<Tame>)>();

    for (entity, pos, target, _) in query.iter(world) {
        let current_zone = zone_grid.get(pos.x, pos.y);
        let target_zone = zone_grid.get(target.target_position.x, target.target_position.y);

        // If currently in Pasture, MUST stay in Pasture
        if current_zone == ZoneType::Pasture && target_zone != ZoneType::Pasture {
            // Cancel movement or redirect to random tile in Pasture
            // For MVP: Just remove MovementTarget (Idle)
            adjustments.push(entity);
        }
    }

    for e in adjustments {
        world.entity_mut(e).remove::<MovementTarget>();
    }
}

/// System for resource production (Milk/Wool).
pub fn husbandry_production_system(world: &mut World) {
    let mut produced = Vec::new();

    let mut query = world.query::<(Entity, &mut Tame, &GridPosition, &Fauna)>();
    for (entity, mut tame, pos, fauna) in query.iter_mut(world) {
        if tame.produce_timer > 0 {
            tame.produce_timer -= 1;
        } else {
            // Produce!
            tame.produce_timer = 1000; // Reset
            let item_type = match fauna.fauna_type {
                FaunaType::SpaceRat => None, // Rats don't produce milk? Maybe "Rat Fur"?
                FaunaType::Wolf => None,
                // Add Cow/Sheep later
                _ => None,
            };

            if let Some(itype) = item_type {
                produced.push((*pos, itype));
            }
        }
    }

    // Spawn items
    // ...
}
```

### 3. Integrate with Utility AI

- Add `evaluate_tame` action evaluator.
- Checks `DesignationType::Tame`.
- High priority for Pops with `Husbandry` skill.

## REFACTOR Phase: Quality & Design

- **Optimization**: `pasture_confinement_system` checks zones every tick. Can be optimized to only check when `MovementTarget` is added/changed.
- **UI**: Add "Zone: Pasture" overlay visualization.
- **Feeding**: Tamed animals currently don't eat. Add `Hunger` logic where they eat Grass tiles in Pasture (turning them to Dirt).
- **Breeding**: Tamed animals should reproduce if fed.

## Acceptance Criteria

- [ ] `Husbandry` skill exists and gains XP from taming.
- [ ] `ZoneType::Pasture` restricts tamed animal movement.
- [ ] Colonists can be assigned to Tame wild animals designated by player.
- [ ] Failed taming triggers attack.
- [ ] Tamed animals produce resources or can be slaughtered.
- [ ] Tests pass.

## Technical Guidance

- Use `rand` crate or a deterministic RNG resource for taming rolls to keep simulation deterministic if needed.
- Ensure `Designation` is removed after successful taming.
- `Tame` component should likely override `Fauna` AI behavior (disable `fauna_behavior_system` or modifying it to ignore Tame entities).

## Questions

- Should wolves be tameable? (Yes, becoming dogs/defense).
- Do we need a "Lasso" tool? (No, abstract it as "Taming Kit" or just hands for MVP).
  - *Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
