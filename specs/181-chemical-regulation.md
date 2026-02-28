# 181: Chemical Regulation

## Overview

Pops can consume chemical substances ("Stims" and "Sedatives") to artificially boost their performance or mood. This introduces a risk/reward mechanic: short-term gains (energy, happiness) vs. long-term costs (addiction, health damage, withdrawal).

- **Stims**: Increase Movement Speed and prevent Fatigue (Rest decay), but damage Health over time.
- **Sedatives**: Increase Mood (reduce Stress) and reduce Break risk, but slow Movement Speed and Work Speed.
- **Addiction**: Repeated use builds tolerance and dependency. Withdrawal causes severe debuffs.

## Dependencies

- `016` — Utility AI System (for `ActionType` and evaluation)
- `034` — Pop Health (Health component)
- `031` — Pop Morale (Mood/Stress mechanics)
- `023` — Refining Industry (ItemType and crafting)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/chemical_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::chemical::{ChemicalState, Addiction, ActiveEffect, ChemicalType};
    use crate::layer1::pop::Pop;
    use crate::layer1::health::Health;
    use crate::layer1::needs::Needs;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_chemical_state_default() {
        let state = ChemicalState::default();
        assert!(state.active_effects.is_empty());
        assert!(state.addictions.is_empty());
    }

    #[test]
    fn test_consume_stim_adds_effect() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            ChemicalState::default(),
            Health::default(),
        )).id();

        // Simulate consuming a Stim
        crate::layer1::chemical::consume_chemical(&mut world, pop, ChemicalType::Stim);

        let state = world.get::<ChemicalState>(pop).unwrap();
        assert!(state.active_effects.iter().any(|e| e.chemical == ChemicalType::Stim));
    }

    #[test]
    fn test_stim_effect_modifies_speed() {
        // This test requires integrating with movement system or checking a "speed_modifier" helper
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            ChemicalState {
                active_effects: vec![ActiveEffect {
                    chemical: ChemicalType::Stim,
                    duration: 100,
                    magnitude: 1.5, // +50% speed
                }],
                ..Default::default()
            },
        )).id();

        let speed = crate::layer1::chemical::get_speed_modifier(&world, pop);
        assert!((speed - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_addiction_buildup() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            ChemicalState::default(),
        )).id();

        // Consume multiple times
        for _ in 0..5 {
            crate::layer1::chemical::consume_chemical(&mut world, pop, ChemicalType::Stim);
        }

        let state = world.get::<ChemicalState>(pop).unwrap();
        let addiction = state.get_addiction(ChemicalType::Stim);
        assert!(addiction.is_some());
        assert!(addiction.unwrap().severity > 0.0);
    }

    #[test]
    fn test_withdrawal_triggers() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 1000 });

        let pop = world.spawn((
            Pop,
            ChemicalState {
                addictions: vec![Addiction {
                    chemical: ChemicalType::Stim,
                    severity: 0.8, // High addiction
                    last_consumed_tick: 0, // Long time ago
                    withdrawal_threshold: 100,
                }],
                ..Default::default()
            },
        )).id();

        // Run system
        crate::layer1::chemical::addiction_system(&mut world);

        let state = world.get::<ChemicalState>(pop).unwrap();
        assert!(state.is_in_withdrawal(ChemicalType::Stim));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `ItemType`

Add `Stim` and `Sedative` to `src/layer1/items.rs`.

```rust
pub enum ItemType {
    // ...
    Stim,
    Sedative,
}
```

### 2. Create `ChemicalState` Component

In `src/layer1/chemical.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::items::ItemType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChemicalType {
    Stim,
    Sedative,
}

#[derive(Debug, Clone)]
pub struct ActiveEffect {
    pub chemical: ChemicalType,
    pub duration: u32,
    pub magnitude: f32,
}

#[derive(Debug, Clone)]
pub struct Addiction {
    pub chemical: ChemicalType,
    pub severity: f32, // 0.0 to 1.0
    pub last_consumed_tick: u64,
    pub withdrawal_threshold: u64,
}

#[derive(Component, Default, Debug, Clone)]
pub struct ChemicalState {
    pub active_effects: Vec<ActiveEffect>,
    pub addictions: Vec<Addiction>,
}

impl ChemicalState {
    pub fn get_addiction(&self, chem: ChemicalType) -> Option<&Addiction> {
        self.addictions.iter().find(|a| a.chemical == chem)
    }

    pub fn is_in_withdrawal(&self, chem: ChemicalType) -> bool {
        // Logic to check tick vs last_consumed
        false // Placeholder for GREEN
    }
}
```

### 3. Implement Consumption Logic

```rust
pub fn consume_chemical(world: &mut World, entity: Entity, chem: ChemicalType) {
    if let Some(mut state) = world.get_mut::<ChemicalState>(entity) {
        // 1. Add Effect
        state.active_effects.push(ActiveEffect {
            chemical: chem,
            duration: 500, // Hardcoded for MVP
            magnitude: match chem {
                ChemicalType::Stim => 1.5,
                ChemicalType::Sedative => 0.5,
            },
        });

        // 2. Update Addiction
        // Find existing or add new
        // ...
    }
}
```

### 4. Integrate with Utility AI

Add `ConsumeChemical` to `ActionType` in `src/layer1/utility_types.rs`.
Implement `evaluate_consume_chemical` in `src/layer1/actions/chemical.rs`.

- **Score Calculation**:
    - Base score low (0.1).
    - If `Withdrawal`: Score = 1.0 (Panic!).
    - If `Stim` and `Rest < 0.2`: Score += 0.5.
    - If `Sedative` and `Mood < 0.2`: Score += 0.5.

### 5. Update Movement/Work Systems

Update `get_movement_speed` (wherever it lives, likely needing a refactor or helper) to check `ChemicalState`.

## REFACTOR Phase: Quality & Design

- **Stat Modification**: Instead of hardcoding checks in every system, use a `StatModifier` system where `ChemicalState` writes to a transient `Stats` component every tick?
- **Policy**: Add `ChemicalPolicy` (Allowed, Banned, Mandatory) to `ColonyEdicts`.
- **Overdose**: Logic for consuming too much (Health damage or death).
- **UI**: Display addiction status in Pop Inspector.

## Acceptance Criteria

- [ ] `Stim` and `Sedative` items exist.
- [ ] Pops can consume them.
- [ ] Stims increase speed/energy.
- [ ] Sedatives decrease stress/speed.
- [ ] Addiction builds up over repeated use.
- [ ] Withdrawal triggers if addicted and abstaining.
- [ ] Utility AI prioritizes consumption during withdrawal.
- [ ] Tests pass.

## Technical Guidance

- **ActionType**: Adding `ConsumeChemical` requires updating `src/layer1/utility_types.rs`.
- **GPU Shader**: Remember to update `src/gpu/evaluate.wgsl` to map the new `ActionType` index if strictly required (though mostly for rendering debug or GPU eval optimization).
- **Traits**: Consider `Teetotaler` (refuses chemicals) or `ChemicalFascination` (higher usage) traits in the future.

## Questions

*Builder: Should withdrawal kill the pop or just incapacitate them?*
*Architect: Withdrawal should only incapacitate them (e.g., massive movement/work speed debuffs) and cause extreme stress.*
*Builder: Do we need a dedicated "Chemical Plant" building or just use "Lab"?*
*Architect: Use the existing "Lab" building for now to reduce scope; we can add a Chemical Plant later.*
