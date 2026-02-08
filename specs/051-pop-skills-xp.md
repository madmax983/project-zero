# 051: Pop Skills and Experience

## Overview

Implement a skill system where Pops gain experience (XP) by performing tasks. As they level up, they become more efficient at those tasks. This adds progression to individual Pops and incentives specialization.

Skills cover major activities:
- **Mining**: Breaking rocks.
- **Forestry**: Chopping trees.
- **Farming**: Producing food/fiber.
- **Construction**: Building structures.
- **Crafting**: Refining resources.

## Dependencies

- `021` — Utility AI Work (for `work_execution_system`)
- `023` — Refining Industry (for `process_refining_system`)
- `008` — Farm Building (for `produce_food_system`)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/skills_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::skills::{Skills, SkillType, get_skill_efficiency};
    use crate::layer1::pop::Pop;

    #[test]
    fn test_skills_default_empty() {
        let skills = Skills::default();
        assert_eq!(skills.get_xp(SkillType::Mining), 0.0);
        assert_eq!(skills.get_level(SkillType::Mining), 0);
    }

    #[test]
    fn test_gain_xp_increases_level() {
        let mut skills = Skills::default();

        // Level 0 -> 1 requires 100 XP
        skills.add_xp(SkillType::Mining, 50.0);
        assert_eq!(skills.get_level(SkillType::Mining), 0);

        skills.add_xp(SkillType::Mining, 50.0);
        assert_eq!(skills.get_level(SkillType::Mining), 1);
    }

    #[test]
    fn test_efficiency_scaling() {
        let mut skills = Skills::default();

        // Level 0 efficiency = 1.0
        assert!((skills.get_efficiency(SkillType::Mining) - 1.0).abs() < f32::EPSILON);

        // Level 1 efficiency = 1.1 (10% bonus)
        skills.add_xp(SkillType::Mining, 100.0);
        assert!((skills.get_efficiency(SkillType::Mining) - 1.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_skill_types_exist() {
        let _ = SkillType::Mining;
        let _ = SkillType::Forestry;
        let _ = SkillType::Farming;
        let _ = SkillType::Construction;
        let _ = SkillType::Crafting;
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `Skills` and `SkillType`

Create `src/layer1/skills.rs`:

```rust
use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillType {
    Mining,
    Forestry,
    Farming,
    Construction,
    Crafting,
    // Future: Medicine, Research, Social
}

#[derive(Component, Default, Clone, Debug)]
pub struct Skills {
    pub xp: HashMap<SkillType, f32>,
}

impl Skills {
    pub fn add_xp(&mut self, skill: SkillType, amount: f32) {
        let current = self.xp.entry(skill).or_insert(0.0);
        *current += amount;
    }

    pub fn get_xp(&self, skill: SkillType) -> f32 {
        *self.xp.get(&skill).unwrap_or(&0.0)
    }

    pub fn get_level(&self, skill: SkillType) -> u32 {
        let xp = self.get_xp(skill);
        // Formula: Level = floor(sqrt(XP / 100))
        // 0-99 XP = Lvl 0
        // 100-399 XP = Lvl 1
        // 400-899 XP = Lvl 2
        (xp / 100.0).sqrt().floor() as u32
    }

    pub fn get_efficiency(&self, skill: SkillType) -> f32 {
        let level = self.get_level(skill);
        // Base 1.0 + 0.1 per level
        1.0 + (level as f32 * 0.1)
    }
}

pub fn get_skill_efficiency(skills: Option<&Skills>, skill: SkillType) -> f32 {
    skills.map_or(1.0, |s| s.get_efficiency(skill))
}
```

### 2. Update `work_execution_system`

In `src/layer1/execution.rs`:
- Query `Option<&mut Skills>` (mutable because we add XP).
- Determine `SkillType` from `DesignationType`.
- Apply efficiency multiplier.
- Add XP (e.g., 1.0 per tick).

```rust
// Inside work_execution_system loop:
let (pop_entity, designation_entity, morale, action_type, mut skills) = ...; // Update query

// Determine skill
let skill_type = match designation_type {
    DesignationType::Mine => Some(SkillType::Mining),
    DesignationType::Chop => Some(SkillType::Forestry),
    DesignationType::Construct => Some(SkillType::Construction), // If implemented
    _ => None,
};

let skill_eff = if let Some(st) = skill_type {
    get_skill_efficiency(skills.as_deref(), st)
} else {
    1.0
};

// Update work amount
let work_amount = WORK_PER_TICK * tool_efficiency * morale_efficiency * skill_eff;

// Add XP
if let Some(st) = skill_type {
    if let Some(s) = skills.as_mut() {
        s.add_xp(st, 1.0); // Constant XP per tick
    }
}
```

### 3. Update `produce_food_system`

In `src/layer1/farm.rs`:
- Query `&Skills` of workers.
- Sum up efficiency.

```rust
// Inside produce_food_system:
let mut total_efficiency = 0.0;
for &worker_entity in &farm.workers {
    if let Ok((_, skills)) = pop_query.get(worker_entity) {
         total_efficiency += get_skill_efficiency(skills, SkillType::Farming);
         // Note: We can't easily add XP here without mutable query of all pops every tick.
         // For MVP, maybe only add XP when a harvest completes?
         // Or change produce_food_system to iterate workers mutably?
    }
}
let production = total_efficiency * FOOD_PER_WORKER_PER_TICK * modifier;
```

*Note on XP for Farming*: Since `produce_food_system` iterates farms, updating random workers' XP component is efficient enough (Entity lookup). We should add `1.0` XP per tick to all workers in the farm.

### 4. Update `process_refining_system`

In `src/layer1/refining.rs`:
- Identify workers in range.
- Use best worker's skill or average?
- Simplest: Use the single nearest worker found, apply their efficiency, add XP to them.

```rust
// Inside process_refining_system loop:
// Find nearest worker
let best_worker = worker_positions.iter()
    .min_by_key(|(e, p)| distance(p, pos));

if let Some((worker_entity, _)) = best_worker {
    // Get worker skills (requires query lookup)
    // Apply efficiency
    // Add XP to worker
}
```

## REFACTOR Phase: Quality & Design

- **UI**: Display skills in Inspector (`src/ui/inspector.rs`).
- **XP Bar**: Visual feedback for leveling up? (Log message for now).
- **Decay**: Skills decay if unused? (Not for MVP).
- **Caps**: Max level cap? (Level 10 = 10000 XP).

## Acceptance Criteria

- [ ] `Skills` component exists and tracks XP per type.
- [ ] Leveling formula works (100 XP = Lvl 1).
- [ ] Efficiency formula works (Lvl 1 = +10%).
- [ ] Mining/Forestry/Farming/Crafting speeds are boosted by skills.
- [ ] Pops gain XP when performing these tasks.
- [ ] Tests pass.

## Technical Guidance

- Be careful with `Query` conflicts. If `produce_food_system` needs to write to `Skills` (for XP), it needs `&mut Skills`.
- `produce_food_system` iterates `Farm` (which has `Entity` IDs of workers). It then needs to get `&mut Skills` for those entities. `pop_query.get_component_mut::<Skills>(entity)` is the way.
