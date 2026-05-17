# 067: Militia System

## Overview

Introduces a **Draft** mechanic to mobilize colonists for combat.
When a Pop is **Drafted**, they ignore normal work/needs and prioritize **Combat Actions** (attacking hostile entities).
This system also formalizes **Weapons** as equippable items (using the existing `Equipment` component from Spec 058) and adds a turn-based combat loop.

This allows players to defend the colony against **Hostile Fauna** (Spec 048) and future threats.

## Dependencies

- `058` — Personal Tools (Equipment component)
- `048` — Hostile Fauna (Target entities)
- `034` — Pop Health (Damage application)
- `016` — Utility AI (Action evaluation override)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/combat_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Equipment, Drafted};
    use crate::layer1::combat::{CombatState, Weapon, AttackProperties, combat_evaluation_system};
    use crate::layer1::health::Health;
    use crate::layer1::utility_ai::{ActionType, PopAction};
    use crate::layer1::map::GridPosition;
    use crate::layer1::fauna::{Fauna, FaunaType}; // Spec 048

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup standard resources (Time, etc)
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::utility_ai::types::UtilityConfig::default());
        world
    }

    // 1. Drafting Logic
    #[test]
    fn test_draft_toggle_overrides_behavior() {
        let mut world = setup_world();
        let pop = world.spawn((
            Pop,
            Drafted, // The new component
            PopAction { current: ActionType::Idle, ..Default::default() },
            Equipment::default(),
            GridPosition { x: 0, y: 0 },
            // Needs would normally drive behavior, but Drafted suppresses them
        )).id();

        // Run evaluation
        // We expect normal Utility AI to run, but Drafted should force Combat logic
        // or return a specific ActionType::Fight score.

        // For this test, we check if evaluate_actions_system prioritizes Fight
        // when an enemy is present.

        // Spawn Enemy
        let enemy = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, ..Default::default() },
            GridPosition { x: 1, y: 0 },
            Health::default(),
        )).id();

        // Evaluate
        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Fight);
    }

    #[test]
    fn test_undrafted_pop_flees_or_ignores() {
        let mut world = setup_world();
        let pop = world.spawn((
            Pop,
            // Not Drafted
            PopAction::default(),
            Equipment::default(),
            GridPosition { x: 0, y: 0 },
        )).id();

        let enemy = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, ..Default::default() },
            GridPosition { x: 1, y: 0 },
            Health::default(),
        )).id();

        crate::layer1::utility_ai::evaluate_actions_system(&mut world);

        let action = world.get::<PopAction>(pop).unwrap();
        // Should NOT be Fight. Could be Flee (future) or Idle/Work.
        assert_ne!(action.current, ActionType::Fight);
    }

    // 2. Weapon & Equipment
    #[test]
    fn test_weapon_properties() {
        let sword = Weapon {
            properties: AttackProperties {
                damage: 10.0,
                range: 1.0,
                cooldown: 10,
                accuracy: 0.9,
            },
        };
        assert_eq!(sword.properties.damage, 10.0);
    }

    #[test]
    fn test_pop_uses_equipped_weapon() {
        let mut world = setup_world();

        // Create Weapon Entity
        let sword = world.spawn(Weapon {
            properties: AttackProperties { damage: 20.0, range: 1.0, cooldown: 10, accuracy: 1.0 },
        }).id();

        // Create Pop with Sword
        let pop = world.spawn((
            Pop,
            Drafted,
            Equipment { weapon: Some(sword), ..Default::default() }, // Updated Equipment struct
            GridPosition { x: 0, y: 0 },
            CombatState::default(),
        )).id();

        // Create Enemy
        let enemy = world.spawn((
            Fauna::default(),
            GridPosition { x: 1, y: 0 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        // Manually trigger attack (simulate execution system)
        crate::layer1::combat::execute_attack(&mut world, pop, enemy);

        // Check Enemy Health
        let health = world.get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 80.0); // 100 - 20
    }

    #[test]
    fn test_attack_cooldown() {
        let mut world = setup_world();
        let pop = world.spawn((
            Pop,
            Drafted,
            CombatState { cooldown: 5, ..Default::default() }, // On cooldown
            Equipment::default(),
        )).id();

        let enemy = world.spawn((Fauna::default(), Health::default())).id();

        // Try attack
        crate::layer1::combat::execute_attack(&mut world, pop, enemy);

        // Should fail/no damage
        let health = world.get::<Health>(enemy).unwrap();
        assert_eq!(health.current, health.max);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `Equipment` Component (`src/layer1/items.rs`)

Add `weapon` field.

```rust
#[derive(Component, Debug, Default)]
pub struct Equipment {
    pub tool: Option<Entity>,   // For Work
    pub weapon: Option<Entity>, // For Combat
}
```

### 2. Define `Combat` Types (`src/layer1/combat.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug)]
pub struct Drafted;

#[derive(Component, Default, Debug)]
pub struct CombatState {
    pub cooldown: u32,
    pub last_target: Option<Entity>,
}

#[derive(Clone, Copy, Debug)]
pub struct AttackProperties {
    pub damage: f32,
    pub range: f32,
    pub cooldown: u32,
    pub accuracy: f32,
}

#[derive(Component, Debug)]
pub struct Weapon {
    pub properties: AttackProperties,
}
```

### 3. Update `ActionType` (`src/layer1/utility_ai/types.rs`)

Add `Fight`. Update `COUNT` to **16**.

```rust
pub enum ActionType {
    // ... existing ...
    Fight, // 15
}

impl ActionType {
    pub const COUNT: usize = 16;

    pub const fn as_index(self) -> usize {
        match self {
            // ...
            Self::Fight => 15,
        }
    }
}
```

**CRITICAL**: You MUST update `src/gpu/buffers.rs` `GpuPopInput` struct to size 16 arrays and padding `[u32; 3]` (12 bytes) to maintain 16-byte alignment (Total 176 bytes).
Also update `src/gpu/shaders/evaluate.wgsl`.

### 4. Implement Evaluation Logic (`src/layer1/combat.rs`)

```rust
use crate::layer1::utility_ai::{ActionType, UtilityConfig};

pub fn evaluate_fight_action(
    drafted: bool,
    pop_pos: &GridPosition,
    enemies: &Query<(Entity, &GridPosition), With<Fauna>>,
) -> Option<(f32, Entity)> {
    if !drafted { return None; }

    // Find nearest enemy
    let mut best_target = None;
    let mut min_dist = f32::MAX;

    for (entity, pos) in enemies.iter() {
        let dist = pop_pos.distance_chebyshev(pos) as f32;
        if dist < min_dist {
            min_dist = dist;
            best_target = Some(entity);
        }
    }

    if let Some(target) = best_target {
        // High score for combat when drafted
        // Distance penalty applies, but base score is high (e.g., 0.9)
        return Some((0.95 - (min_dist * 0.01).min(0.5), target));
    }

    None
}
```

### 5. Combat Execution System (`src/layer1/execution.rs`)

In `execution_system` match loop:

```rust
ActionType::Fight => {
    // Check range
    // If in range:
    //   Check cooldown
    //   If ready: Attack (Apply damage, Start cooldown)
    // Else:
    //   Move towards target
}
```

## REFACTOR Phase: Quality & Design

- **Visuals**: Render weapons on Pops.
- **Squads**: Group drafted pops to move together.
- **Targeting**: Manual targeting override (Right-click attack).
- **Ammo**: Ranged weapons need ammo (Quivers/Batteries).
- **Fleeing**: Undrafted pops should auto-flee `Fight` actions.

## Acceptance Criteria

- [ ] `Drafted` component toggles combat behavior.
- [ ] `ActionType::Fight` implemented and GPU arrays resized.
- [ ] Pops equip `Weapon` items.
- [ ] Combat system handles cooldowns and damage.
- [ ] Drafted pops auto-attack nearby enemies.
- [ ] Tests pass.

## Technical Guidance

- **GPU Alignment**:
  - `success_count: [u32; 16]` (64 bytes)
  - `attempt_count: [u32; 16]` (64 bytes)
  - Previous fields: 32 bytes + 4 bytes.
  - Total payload: ~164 bytes.
  - Next alignment: 176 bytes.
  - Padding needed: 12 bytes (`[u32; 3]`).
- **Input Handling**: Add `Draft` toggle to UI (Spec `015` Selection might need update to show "Draft" button).

## Questions
*Builder: add questions here if spec is unclear.*
- **Architectural Contradictions:** ActionType inside Utility AI system isn't matching up properly with how action types and evaluations are designed and implemented currently. Also the evaluation interval calculation and structures might differ significantly.
