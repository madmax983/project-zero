# 073: Vermin Infestation

## Overview

The colony is not sterile. Large stockpiles of food and waste attract **Vermin** (rats, roaches, etc.). Vermin are an abstract "Infestation Level" that increases food spoilage rates and can trigger disease outbreaks (future). Players must manage waste and enact policies to keep the vermin population under control.

## Dependencies

- `032` — Entropy and Spoilage (Vermin accelerate this)
- `049` — Industrial Waste (Waste attracts vermin)
- `054` — Colony Edicts (Counter-measure)

## RED Phase: Tests First

Write these tests in `src/layer1/vermin_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::vermin::{VerminState, vermin_growth_system, calculate_spoilage_modifier};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::edicts::{ColonyPolicies, Policy};
    use crate::layer1::spoilage::GLOBAL_SPOILAGE_RATE;
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(VerminState::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(ColonyPolicies::default());
        world
    }

    #[test]
    fn test_initialization() {
        let world = setup_world();
        let vermin = world.resource::<VerminState>();
        assert_eq!(vermin.severity, 0.0);
        assert_eq!(vermin.max_severity, 100.0);
    }

    #[test]
    fn test_vermin_growth_from_food() {
        let mut world = setup_world();

        // Add Food
        {
            let mut res = world.resource_mut::<ColonyResources>();
            res.food = 1000.0;
        }

        // Run system
        world.run_system_once(vermin_growth_system).unwrap();

        let vermin = world.resource::<VerminState>();
        assert!(vermin.severity > 0.0, "Vermin should grow when food is present");
    }

    #[test]
    fn test_vermin_growth_from_waste() {
        let mut world = setup_world();

        // Add Waste
        {
            let mut res = world.resource_mut::<ColonyResources>();
            res.waste = 500.0;
        }

        // Run system
        world.run_system_once(vermin_growth_system).unwrap();

        let vermin = world.resource::<VerminState>();
        assert!(vermin.severity > 0.0, "Vermin should grow when waste is present");
    }

    #[test]
    fn test_pest_control_policy_reduces_growth() {
        let mut world_normal = setup_world();
        let mut world_policy = setup_world();

        // Setup identical conditions
        {
            let mut res = world_normal.resource_mut::<ColonyResources>();
            res.food = 1000.0;
        }
        {
            let mut res = world_policy.resource_mut::<ColonyResources>();
            res.food = 1000.0;
            world_policy.resource_mut::<ColonyPolicies>().toggle(Policy::PestControl);
        }

        // Run systems
        world_normal.run_system_once(vermin_growth_system).unwrap();
        world_policy.run_system_once(vermin_growth_system).unwrap();

        let severity_normal = world_normal.resource::<VerminState>().severity;
        let severity_policy = world_policy.resource::<VerminState>().severity;

        assert!(severity_policy < severity_normal, "Pest Control should reduce growth");
    }

    #[test]
    fn test_vermin_decay_when_clean() {
        let mut world = setup_world();

        // Set initial infestation
        {
            let mut vermin = world.resource_mut::<VerminState>();
            vermin.severity = 50.0;
        }

        // No food/waste
        // Run system
        world.run_system_once(vermin_growth_system).unwrap();

        let vermin = world.resource::<VerminState>();
        assert!(vermin.severity < 50.0, "Vermin should die off without food/waste");
    }

    #[test]
    fn test_spoilage_modifier() {
        let mut vermin = VerminState::default();

        // No vermin
        vermin.severity = 0.0;
        let mod_zero = calculate_spoilage_modifier(&vermin);
        assert!((mod_zero - 1.0).abs() < f32::EPSILON);

        // Max vermin
        vermin.severity = 100.0;
        let mod_max = calculate_spoilage_modifier(&vermin);
        assert!(mod_max > 1.0, "Spoilage should increase with vermin");
        assert!(mod_max >= 5.0, "Spoilage should be significant at max severity");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `VerminState` (`src/layer1/vermin.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;
use crate::layer1::edicts::{ColonyPolicies, Policy};

#[derive(Resource)]
pub struct VerminState {
    /// 0.0 to 100.0
    pub severity: f32,
    pub max_severity: f32,
}

impl Default for VerminState {
    fn default() -> Self {
        Self {
            severity: 0.0,
            max_severity: 100.0,
        }
    }
}
```

### 2. Implement Growth Logic

```rust
// Growth Constants
const VERMIN_GROWTH_PER_FOOD: f32 = 0.001; // 1000 food -> +1.0 severity/tick? No, too fast.
// Let's say we want 1000 food to generate 10.0 severity over a season (250 ticks).
// 10 / 250 = 0.04 per tick.
// 0.04 / 1000 = 0.00004
const GROWTH_FACTOR_FOOD: f32 = 0.00005;
const GROWTH_FACTOR_WASTE: f32 = 0.0001; // Waste attracts more vermin
const NATURAL_DECAY: f32 = 0.05; // Dies off slowly if no food

pub fn vermin_growth_system(
    mut vermin: ResMut<VerminState>,
    resources: Res<ColonyResources>,
    policies: Res<ColonyPolicies>,
) {
    let food_impact = resources.food * GROWTH_FACTOR_FOOD;
    let waste_impact = resources.waste * GROWTH_FACTOR_WASTE;

    let mut growth = food_impact + waste_impact;

    // Apply Pest Control Policy
    if policies.is_active(Policy::PestControl) {
        growth *= 0.5; // Halves growth rate
        // Also increases decay?
    }

    if growth > 0.0 {
        vermin.severity += growth;
    } else {
        vermin.severity -= NATURAL_DECAY;
    }

    // Clamp
    vermin.severity = vermin.severity.clamp(0.0, vermin.max_severity);
}
```

### 3. Implement Spoilage Modifier

```rust
pub fn calculate_spoilage_modifier(vermin: &VerminState) -> f32 {
    // 0 severity -> 1.0x
    // 100 severity -> 10.0x (Massive spoilage)
    1.0 + (vermin.severity / 100.0) * 9.0
}
```

### 4. Integration with Spoilage System (`src/layer1/spoilage.rs`)

*Builder Note: Modify `spoilage_system` to use the modifier.*

```rust
// In src/layer1/spoilage.rs
// Add VerminState to system arguments
pub fn spoilage_system(world: &mut World) {
    // ...
    let modifier = if let Some(vermin) = world.get_resource::<VerminState>() {
        crate::layer1::vermin::calculate_spoilage_modifier(vermin)
    } else {
        1.0
    };

    let decay = resources.food * GLOBAL_SPOILAGE_RATE * modifier;
    // ...
}
```

### 5. Add Policy (`src/layer1/edicts.rs`)

*Builder Note: Add `PestControl` to `Policy` enum.*

```rust
pub enum Policy {
    Rationing,
    DoubleShifts,
    PestControl, // New
}
```

## REFACTOR Phase: Quality & Design

- **Notifications**: Trigger a notification when `severity > 50.0` ("Vermin infestation detected!").
- **Visuals**: Render small "rat" sprites on food stockpiles if severity is high.
- **Cats**: Add a `Pet` component later that actively reduces `severity`.
- **Edict Cost**: Implement the "Work Speed Penalty" for `PestControl` in `edicts.rs` (people spending time setting traps).
    - Update `get_work_speed_modifier`: if `PestControl`, modifier -= 0.05.

## Acceptance Criteria

- [ ] `VerminState` resource exists and initializes correctly.
- [ ] Vermin severity increases with Food and Waste.
- [ ] Vermin severity decreases when resources are low.
- [ ] `Policy::PestControl` significantly reduces vermin growth.
- [ ] Food spoilage rate is multiplied by infestation level (up to 10x).
- [ ] Tests pass.

## Technical Guidance

- Ensure `VerminState` is inserted in `main.rs`.
- Ensure `vermin_growth_system` runs in the schedule (before `spoilage_system`).
- Don't forget to update `edicts.rs` and `spoilage.rs` as they are integration points.

## Questions

*Builder: add questions here if spec is unclear.*
