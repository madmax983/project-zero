# 054: Colony Edicts

## Overview

Introduces a **Colony Policy System** (Edicts) allowing players to enact global rules that trade off resources/stats for other benefits.
- **ColonyPolicies**: A resource tracking active edicts.
- **Policy**: An enum of available edicts (e.g., Rationing, Double Shifts).
- **Effects**: Policies modify global constants or per-pop stats (Need decay rates, Work speed, Morale).

This adds a layer of strategic management, allowing players to respond to crises (e.g., famine) with drastic measures.

## Dependencies

- `005` — Pop Needs (Hunger, Rest)
- `031` — Pop Morale (Morale, Stress)
- `051` — Pop Skills (Work Speed context)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/edicts_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::edicts::{ColonyPolicies, Policy, apply_policy_effects_system};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Speed;
    use crate::layer1::GridPosition;

    #[test]
    fn test_policies_resource_defaults() {
        let policies = ColonyPolicies::default();
        assert!(policies.active_policies.is_empty());
    }

    #[test]
    fn test_toggle_policy() {
        let mut policies = ColonyPolicies::default();

        // Enable Rationing
        policies.toggle(Policy::Rationing);
        assert!(policies.is_active(Policy::Rationing));

        // Disable Rationing
        policies.toggle(Policy::Rationing);
        assert!(!policies.is_active(Policy::Rationing));
    }

    #[test]
    fn test_rationing_reduces_hunger_decay() {
        let mut world = World::new();
        let mut policies = ColonyPolicies::default();
        policies.toggle(Policy::Rationing);
        world.insert_resource(policies);

        // Spawn a pop with standard needs
        let pop = world.spawn((
            Needs { hunger: 1.0, ..Default::default() },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Run system that applies policy effects to needs
        // Note: This assumes `apply_policy_effects_system` modifies needs OR modifies a `NeedRate` component.
        // For MVP, let's assume it modifies the `Needs` directly or sets a flag on the pop.
        // Better design: Needs system checks Policies resource directly.
        // But if we want to test effects in isolation, we need to know how Needs are calculated.

        // Let's assume `apply_policy_effects_system` applies a temporary modifier component or we check logic in `needs_system`.
        // To stick to TDD for *this* feature, let's verify `get_hunger_rate_modifier` function exists.

        let modifier = crate::layer1::edicts::get_hunger_decay_modifier(&world.resource::<ColonyPolicies>());
        assert!(modifier < 1.0); // Should reduce decay (e.g. 0.5)
    }

    #[test]
    fn test_double_shifts_increases_speed_and_reduces_morale() {
        let mut world = World::new();
        let mut policies = ColonyPolicies::default();
        policies.toggle(Policy::DoubleShifts);
        world.insert_resource(policies);

        // Check modifiers
        let speed_mod = crate::layer1::edicts::get_work_speed_modifier(&world.resource::<ColonyPolicies>());
        assert!(speed_mod > 1.0); // e.g. 1.2

        let morale_mod = crate::layer1::edicts::get_morale_modifier(&world.resource::<ColonyPolicies>());
        assert!(morale_mod < 0.0); // e.g. -10.0 (flat penalty) or < 1.0 (multiplier)
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Resource and Enum

```rust
// src/layer1/edicts.rs

use bevy_ecs::prelude::*;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Policy {
    Rationing,      // -50% Hunger decay, -10 Morale
    DoubleShifts,   // +20% Work Speed, -15 Morale, +10% Fatigue
    Propaganda,     // +5 Morale, -5 Knowledge gain (if implemented)
    Conservation,   // +10% Resource Yield (if implemented), -10% Speed
}

#[derive(Resource, Default, Debug)]
pub struct ColonyPolicies {
    pub active_policies: HashSet<Policy>,
}

impl ColonyPolicies {
    pub fn toggle(&mut self, policy: Policy) {
        if self.active_policies.contains(&policy) {
            self.active_policies.remove(&policy);
        } else {
            self.active_policies.insert(policy);
        }
    }

    pub fn is_active(&self, policy: Policy) -> bool {
        self.active_policies.contains(&policy)
    }
}
```

### 2. Implement Modifier Functions

These functions will be used by other systems (`needs_system`, `work_system`, `morale_system`) to fetch current global modifiers.

```rust
// src/layer1/edicts.rs

pub fn get_hunger_decay_modifier(policies: &ColonyPolicies) -> f32 {
    if policies.is_active(Policy::Rationing) {
        0.5
    } else {
        1.0
    }
}

pub fn get_work_speed_modifier(policies: &ColonyPolicies) -> f32 {
    let mut modifier = 1.0;
    if policies.is_active(Policy::DoubleShifts) {
        modifier += 0.2;
    }
    if policies.is_active(Policy::Conservation) {
        modifier -= 0.1;
    }
    modifier
}

pub fn get_morale_modifier(policies: &ColonyPolicies) -> f32 {
    let mut modifier = 0.0; // Flat addition/subtraction to base morale
    if policies.is_active(Policy::Rationing) {
        modifier -= 10.0;
    }
    if policies.is_active(Policy::DoubleShifts) {
        modifier -= 15.0;
    }
    if policies.is_active(Policy::Propaganda) {
        modifier += 5.0;
    }
    modifier
}
```

### 3. Integrate with Systems (Guidance)

The Builder will need to modify existing systems to call these functions.

- **Needs System**: In `src/layer1/needs.rs`, multiply hunger decay by `get_hunger_decay_modifier`.
- **Work System**: In `src/layer1/pop.rs` (or wherever work speed is applied), multiply by `get_work_speed_modifier`.
- **Morale System**: In `src/layer1/morale.rs` (or `memory.rs` depending on where it lives), add `get_morale_modifier` to the final calculation.

## REFACTOR Phase: Quality & Design

- **UI**: Add a simple UI window (keybind `[P]`) to toggle policies.
- **Events**: Log a message when a policy is enacted/revoked ("Rationing Enacted!").
- **Cooldowns**: Prevent spamming toggles (add `last_changed` timestamp per policy).
- **Conflicts**: Prevent conflicting policies (e.g., "Double Rations" vs "Rationing").

## Acceptance Criteria

- [ ] `ColonyPolicies` resource exists.
- [ ] `Policy` enum contains at least `Rationing` and `DoubleShifts`.
- [ ] Toggling works correctly.
- [ ] Modifiers are correctly calculated based on active policies.
- [ ] Tests pass.
- [ ] Builder instructions clarify integration points.

## Technical Guidance

- Use `world.resource::<ColonyPolicies>()` in systems.
- For `apply_policy_effects_system`, since this is a pull-based modifier system (other systems ask "what is the modifier?"), there might not be a dedicated system that runs every frame, unless we want to visualize the active policies or decay cooldowns. The *effects* are applied in the respective systems (Needs, Work, Morale).
