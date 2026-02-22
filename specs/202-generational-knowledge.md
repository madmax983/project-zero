# 202: Generational Knowledge

## 1. Overview

"The son of the master smith grows up in the forge."

Currently, new Pops (if we had them) start as blank slates. This spec introduces **Generational Knowledge**, where children inherit a portion of their parents' experience. This creates organic specialization: a mining colony will naturally produce better miners over generations.

This feature introduces:
1.  `spawn_child_from_parents`: A specialized spawn function.
2.  Inheritance logic: Children gain 10% of their parents' highest skill XP.
3.  `Lineage` component: Tracks ancestry (for flavor and future mechanics).

## 2. Dependencies

- `051` — Pop Skills & XP (Implemented)
- `062` — Pop Lifecycle (Implemented, aging logic)
- `003` — Pop Entity (Implemented)

## 3. RED Phase: Tests First

```rust
// src/layer1/generational_knowledge_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::skills::{Skills, SkillType};
    use crate::layer1::lifecycle::{Age, LifeStage, spawn_child};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_spawn_child_initial_state() {
        let mut world = World::new();
        // Setup infrastructure if needed (e.g. time)

        // Spawn a child with NO parents
        let child = spawn_child(&mut world, None, None, GridPosition { x: 0, y: 0 });

        let age = world.get::<Age>(child).expect("Child must have Age");
        assert_eq!(age.stage, LifeStage::Child);
        assert_eq!(age.ticks_alive, 0);

        let skills = world.get::<Skills>(child).expect("Child must have Skills");
        assert_eq!(skills.get_xp(SkillType::Mining), 0.0);
    }

    #[test]
    fn test_inherit_xp_from_single_parent() {
        let mut world = World::new();

        // Parent: Master Miner (10,000 XP)
        let parent = world.spawn((
            Pop,
            Skills {
                xp: std::collections::HashMap::from([(SkillType::Mining, 10000.0)]),
            }
        )).id();

        // Spawn Child
        let child = spawn_child(&mut world, Some(parent), None, GridPosition { x: 0, y: 0 });

        let skills = world.get::<Skills>(child).unwrap();
        // Expect 10% inheritance = 1000 XP
        assert_eq!(skills.get_xp(SkillType::Mining), 1000.0);
        // Expect 0 for others
        assert_eq!(skills.get_xp(SkillType::Farming), 0.0);
    }

    #[test]
    fn test_inherit_xp_from_two_parents_max_strategy() {
        let mut world = World::new();

        // Parent 1: Farmer (5000 XP)
        let p1 = world.spawn((
            Pop,
            Skills { xp: std::collections::HashMap::from([(SkillType::Farming, 5000.0)]) }
        )).id();

        // Parent 2: Miner (8000 XP) + Farmer (100 XP)
        let p2 = world.spawn((
            Pop,
            Skills { xp: std::collections::HashMap::from([
                (SkillType::Mining, 8000.0),
                (SkillType::Farming, 100.0)
            ]) }
        )).id();

        let child = spawn_child(&mut world, Some(p1), Some(p2), GridPosition { x: 0, y: 0 });
        let skills = world.get::<Skills>(child).unwrap();

        // Mining: Inherit from P2 (8000 * 0.1 = 800)
        assert_eq!(skills.get_xp(SkillType::Mining), 800.0);

        // Farming: Inherit from P1 (Max of 5000 vs 100 is 5000 * 0.1 = 500)
        assert_eq!(skills.get_xp(SkillType::Farming), 500.0);
    }

    #[test]
    fn test_lineage_component() {
        let mut world = World::new();
        let p1 = world.spawn(Pop).id();

        let child = spawn_child(&mut world, Some(p1), None, GridPosition { x: 0, y: 0 });

        // This test requires defining the Lineage component
        // let lineage = world.get::<crate::layer1::lifecycle::Lineage>(child).unwrap();
        // assert!(lineage.parents.contains(&p1));
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### 1. Define `Lineage` Component

In `src/layer1/lifecycle.rs`:

```rust
#[derive(Component, Debug, Default, Clone)]
pub struct Lineage {
    pub parents: Vec<Entity>,
    pub generation: u32,
}
```

### 2. Implement `spawn_child`

In `src/layer1/lifecycle.rs`:

```rust
use crate::layer1::skills::{Skills, SkillType};
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;

pub fn spawn_child(
    world: &mut World,
    parent1: Option<Entity>,
    parent2: Option<Entity>,
    pos: GridPosition,
) -> Entity {
    // 1. Calculate Initial XP
    let mut initial_xp = Skills::default();

    let parents = [parent1, parent2];
    for p_entity in parents.iter().flatten() {
        if let Some(p_skills) = world.get::<Skills>(*p_entity) {
            for (skill, xp) in &p_skills.xp {
                // Logic: Take 10% of parent XP.
                // If multiple parents have the skill, take the highest result (or accumulate? Spec says inheritance, let's max).
                let inheritance = xp * 0.1;
                let current = initial_xp.get_xp(*skill);
                if inheritance > current {
                    initial_xp.xp.insert(*skill, inheritance);
                }
            }
        }
    }

    // 2. Resolve Lineage
    let mut lineage = Lineage::default();
    let mut max_gen = 0;
    for p in parents.iter().flatten() {
        lineage.parents.push(*p);
        if let Some(l) = world.get::<Lineage>(*p) {
            if l.generation > max_gen { max_gen = l.generation; }
        }
    }
    lineage.generation = max_gen + 1;

    // 3. Spawn
    world.spawn((
        Pop,
        pos,
        Age::new(0), // Newborn
        initial_xp,
        lineage,
        // ... other defaults (Health, Needs, etc)
        crate::layer1::health::Health::default(),
        crate::layer1::needs::Needs::default(),
    )).id()
}
```

## 5. REFACTOR Phase: Quality & Design

- **Trait Inheritance**: Future work. This spec focuses on Knowledge (XP).
- **Capping**: Ensure starting XP doesn't exceed a certain level (e.g. Level 5) to prevent "instant masters" after 10 generations.
- **Log**: Add a log message "A child was born to X and Y."

## 6. Acceptance Criteria

- [ ] `Lineage` component exists.
- [ ] `spawn_child` helper function creates a Pop with `Age::Child`.
- [ ] Child inherits 10% of parents' XP.
- [ ] Tests pass.

## 7. Technical Guidance

- Ensure `spawn_child` adds all necessary components for a valid Pop (`Health`, `Needs`, `Speed`, etc.) so it doesn't crash other systems. Reuse existing `spawn_pop` logic if possible, or refactor `spawn_pop` to call `spawn_child` (or vice versa).
- Be careful with `world.get` inside the function while mutably borrowing `world`. You might need to extract parent data *before* spawning.

## 8. Questions

*Builder: add questions here if spec is unclear.*
